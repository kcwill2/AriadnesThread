//! AR-08 characterization locks: the stage-5 SYMBOL metric pass as observed
//! through the PUBLIC pipeline API. The pass is being switched from the
//! name-based `LoadAnalyzer::analyze_symbol_load` (which resolves each name
//! with an O(S) scan, so the whole pass is O(S^2) string comparisons) to the
//! AR-07 by-id seam `analyze_symbol_load_by_id`, and the locks below are the
//! byte-equality leash for that refactor: same elements, same order, same
//! values.
//!
//! Two fixtures, one rendering:
//! - the C++ lane (the shape `load_stage_lifecycle_test.rs` uses) locks the
//!   ordinary path: ids and names are identical there (`SymbolId::new(name)`
//!   in `lang-cpp/src/parser.rs`), so every name resolves to its own record;
//! - the Scala lane locks the case the C++ lane cannot reach: two symbols in
//!   different packages share the record NAME while their ids differ
//!   (`name = decl.simple_name`, id = package-qualified). There the legacy
//!   pass emits the FIRST key-ascending match twice - captured from the
//!   pre-change tree, see the literal's note - so the refactor may remove the
//!   scan but must keep the resolution semantics.
//!
//! Report paths are relative in these artifacts; the rendering normalizes any
//! absolute spelling against the fixture root so the literals never carry a
//! temp directory, and `two_independent_tempdirs_render_identically` witnesses
//! that the normalization folds nothing but the root spelling.

use std::fs;
use std::path::Path;

use ariadne_graph::ContributingSource;
use ariadne_index::SymbolId;
use ariadne_pipeline::Pipeline;
use ariadne_report::LoadReport;
use tempfile::TempDir;

/// C++ fixture: two headers with one struct each, one shared base header,
/// and one translation unit per header defining a function.
fn setup_cpp_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("include")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("include/common.h"), "struct Common { int id; };").unwrap();
    fs::write(
        root.join("include/header_a.h"),
        concat!("#inc", "lude \"common.h\"\nstruct ThingA { Common c; };"),
    )
    .unwrap();
    fs::write(root.join("include/header_b.h"), "struct ThingB { double x; };").unwrap();
    fs::write(
        root.join("src/tu1.cpp"),
        concat!("#inc", "lude \"header_a.h\"\nvoid use_a(ThingA a) {}"),
    )
    .unwrap();
    fs::write(
        root.join("src/tu2.cpp"),
        concat!("#inc", "lude \"header_b.h\"\nvoid use_b(ThingB b) {}"),
    )
    .unwrap();

    let root_str = root.to_string_lossy().replace('\\', "/");
    let compile_commands = format!(
        r#"[
    {{ "directory": "{root_str}", "command": "cc -Iinclude -c src/tu1.cpp", "file": "src/tu1.cpp" }},
    {{ "directory": "{root_str}", "command": "cc -Iinclude -c src/tu2.cpp", "file": "src/tu2.cpp" }}
]"#,
        root_str = root_str
    );
    fs::write(root.join("compile_commands.json"), compile_commands).unwrap();
    dir
}

/// Scala fixture with the name-collision shape the C++ lane cannot produce:
/// `a.Widget` and `b.Widget` are distinct ids carrying the same record name
/// `Widget`, defined in different files. A by-id pass that skipped the
/// legacy resolution order would render `b/widget.scala` for the second
/// entry instead of the first-match metric the legacy pass emits.
fn setup_scala_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("project")).unwrap();
    fs::write(root.join("build.sbt"), "name := \"ar08-symbol-lock\"\n").unwrap();
    fs::write(root.join("project/build.properties"), "sbt.version=1.9.9\n").unwrap();

    let package_a = root.join("src/main/scala/a");
    let package_b = root.join("src/main/scala/b");
    fs::create_dir_all(&package_a).unwrap();
    fs::create_dir_all(&package_b).unwrap();
    fs::write(
        package_a.join("widget.scala"),
        "package a\n\nclass Widget {\n  def identify(): Int = 1\n}\n",
    )
    .unwrap();
    fs::write(
        package_b.join("widget.scala"),
        "package b\n\nimport a.Widget\n\nclass Widget {\n  def measure(): Int = 2\n}\n",
    )
    .unwrap();
    dir
}

/// Folds a path to one comparable spelling: forward slashes, no Windows
/// verbatim prefix.
fn normalized_spelling(path: &Path) -> String {
    let slashed = path.to_string_lossy().replace('\\', "/");
    match slashed.strip_prefix("//?/") {
        Some(stripped) => stripped.to_string(),
        None => slashed,
    }
}

/// Renders a report path relative to the fixture root. Artifacts carry
/// relative paths today; an absolute one must normalize against the root or
/// show up as `ABSOLUTE-UNNORMALIZED`, never silently as a temp directory.
fn path_label(path: Option<&Path>, root: &Path) -> String {
    let Some(path) = path else {
        return "<none>".to_string();
    };
    let rendered = normalized_spelling(path);
    match rendered.strip_prefix(&normalized_spelling(root)) {
        Some(tail) => format!("<ROOT>{tail}"),
        None if Path::new(&rendered).is_relative() => rendered,
        None => format!("ABSOLUTE-UNNORMALIZED:{rendered}"),
    }
}

fn id_list(symbol_ids: &[SymbolId]) -> String {
    symbol_ids
        .iter()
        .map(|symbol_id| symbol_id.0.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

/// One contributor as `count:path:symbols`, matching the ordering the metric
/// itself carries (`symbol_count desc, path asc`). Any borrow works, so the
/// largest-collateral `Option` renders through the same shape (empty when
/// there is no collateral source at all, exactly when the source list is
/// empty).
fn source_list<'a, Sources>(sources: Sources, root: &Path) -> String
where
    Sources: IntoIterator<Item = &'a ContributingSource>,
{
    sources
        .into_iter()
        .map(|source| {
            format!(
                "{}:{}:{}",
                source.symbol_count,
                path_label(Some(&source.path), root),
                id_list(&source.symbols)
            )
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

/// One line per symbol metric, in report order, carrying every `LoadMetric`
/// field (target identity, the four loads, both symbol sets, both source
/// groupings, and the largest-collateral pick).
fn render_symbol_metrics(report: &LoadReport, root: &Path) -> String {
    report
        .symbol_metrics
        .iter()
        .map(|metric| {
            format!(
                concat!(
                    "name={} file={} sem={} inc={} col={} tot={} ",
                    "sem_syms=[{}] col_syms=[{}] sem_srcs=[{}] col_srcs=[{}] lcs={}"
                ),
                metric.target_name,
                path_label(metric.target_file.as_deref(), root),
                metric.semantic_load,
                metric.include_load,
                metric.collateral_load,
                metric.total_load,
                id_list(&metric.semantic_symbols),
                id_list(&metric.collateral_symbols),
                source_list(&metric.semantic_sources, root),
                source_list(&metric.collateral_sources, root),
                source_list(metric.largest_collateral_source.iter(), root),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Cold-builds stage 5 on a fixture copy and renders the symbol pass.
/// `workers = Some(1)` pins the scoped pool the way the I6 event literal does.
fn render_symbol_pass(fixture_root: &Path) -> String {
    let mut pipeline = Pipeline::new(fixture_root, Some(1)).expect("pipeline boots");
    pipeline.ensure_load().expect("stage 5 builds");
    let report = pipeline.get_or_build_load().expect("load report is served");
    render_symbol_metrics(&report, fixture_root)
}

/// The C++ lane of the stage-5 symbol pass, captured from the pre-change tree
/// (the AR-08 journal records the capture run). Any add, drop, reorder, or
/// value change in the symbol pass fails here.
#[test]
fn cpp_lane_symbol_metrics_match_recorded_literal() {
    let fixture = setup_cpp_repo();
    let rendered = render_symbol_pass(fixture.path());
    let expected: &[&str] = &[
        "name=Common file=include/common.h sem=1 inc=1 col=0 tot=1 sem_syms=[Common] col_syms=[] sem_srcs=[1:include/common.h:Common] col_srcs=[] lcs=",
        "name=ThingA file=include/header_a.h sem=2 inc=2 col=0 tot=2 sem_syms=[Common,ThingA] col_syms=[] sem_srcs=[1:include/common.h:Common + 1:include/header_a.h:ThingA] col_srcs=[] lcs=",
        "name=ThingB file=include/header_b.h sem=1 inc=1 col=0 tot=1 sem_syms=[ThingB] col_syms=[] sem_srcs=[1:include/header_b.h:ThingB] col_srcs=[] lcs=",
        "name=use_a file=src/tu1.cpp sem=3 inc=3 col=0 tot=3 sem_syms=[Common,ThingA,use_a] col_syms=[] sem_srcs=[1:include/common.h:Common + 1:include/header_a.h:ThingA + 1:src/tu1.cpp:use_a] col_srcs=[] lcs=",
        "name=use_b file=src/tu2.cpp sem=2 inc=2 col=0 tot=2 sem_syms=[ThingB,use_b] col_syms=[] sem_srcs=[1:include/header_b.h:ThingB + 1:src/tu2.cpp:use_b] col_srcs=[] lcs=",
    ];
    assert_eq!(
        rendered.lines().collect::<Vec<_>>(),
        expected.to_vec(),
        "the C++ stage-5 symbol pass must stay byte-equal to the recorded literal"
    );
}

/// The Scala lane: `a.Widget` and `b.Widget` share the record name, so the
/// legacy name pass resolves BOTH to the first key-ascending match and emits
/// `a/widget.scala`'s metric twice. Captured from the pre-change tree. The
/// by-id switch may drop the O(S) scan per symbol, never this resolution
/// outcome: line 2 carries the first-match metric, not `b/widget.scala`.
#[test]
fn scala_lane_name_collision_keeps_first_wins_metric() {
    let fixture = setup_scala_repo();
    let rendered = render_symbol_pass(fixture.path());
    let expected: &[&str] = &[
        "name=Widget file=src/main/scala/a/widget.scala sem=1 inc=1 col=0 tot=1 sem_syms=[a.Widget] col_syms=[] sem_srcs=[1:src/main/scala/a/widget.scala:a.Widget] col_srcs=[] lcs=",
        "name=Widget file=src/main/scala/a/widget.scala sem=1 inc=1 col=0 tot=1 sem_syms=[a.Widget] col_syms=[] sem_srcs=[1:src/main/scala/a/widget.scala:a.Widget] col_srcs=[] lcs=",
    ];
    assert_eq!(
        rendered.lines().collect::<Vec<_>>(),
        expected.to_vec(),
        "the Scala stage-5 symbol pass must keep the first-wins metric for the shared record name"
    );
}

/// Normalization witness: two independent fixture copies (different temp
/// roots) render identically, so the literals above pin fixture content and
/// nothing else.
#[test]
fn two_independent_tempdirs_render_identically() {
    let first = setup_scala_repo();
    let second = setup_scala_repo();
    assert_eq!(
        render_symbol_pass(first.path()),
        render_symbol_pass(second.path()),
        "the rendering must not carry the temp root"
    );
}
