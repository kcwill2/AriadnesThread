//! The compact stage-5 file metric equals the full file metric with its
//! collateral lists trimmed: no `collateral_symbols`, and each collateral
//! source keeps its full count but only its smallest symbol ids, and only if
//! a capped justification list could show it. Checked for every file of a
//! generated repo with a layered header graph.

use ariadne_graph::{is_includable_header, CompactShape, LoadAnalyzer, LoadMetric};
use ariadne_repo::FileId;
use std::collections::HashSet;
use ariadne_pipeline::Pipeline;
use std::fs;
use std::path::Path;

const HEADERS: usize = 30;
const UNITS: usize = 10;

fn write(root: &Path, rel: &str, contents: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

/// Header k includes two later headers, defines a few types, and has an
/// inline function calling a dependency's; units include and call a few.
fn layered_repo(root: &Path) {
    write(
        root,
        "app/app.vcxproj",
        "<Project><ItemDefinitionGroup><ClCompile>\
         <AdditionalIncludeDirectories>..\\lib\\include</AdditionalIncludeDirectories>\
         </ClCompile></ItemDefinitionGroup></Project>",
    );
    for header in 0..HEADERS {
        let deps: Vec<usize> = [header + 1, header + 3].into_iter().filter(|d| *d < HEADERS).collect();
        let mut text = String::from("#pragma once\n");
        for dep in &deps {
            text.push_str(&format!("#include \"lib/h{dep}.h\"\n"));
        }
        // Header sizes differ, so source counts rank instead of all tying.
        for kind in 0..(2 + header % 5) {
            text.push_str(&format!("struct Type{header}x{kind} {{ int value{kind}; }};\n"));
        }
        let call = deps.first().map_or(String::from("0"), |dep| format!("helper{dep}(value)"));
        text.push_str(&format!("inline int helper{header}(int value) {{ Type{header}x0 t{{}}; return t.value0 + {call}; }}\n"));
        write(root, &format!("lib/include/lib/h{header}.h"), &text);
    }
    for unit in 0..UNITS {
        let first = unit % HEADERS;
        let second = (unit * 7 + 2) % HEADERS;
        write(
            root,
            &format!("app/src/unit{unit}.cpp"),
            &format!(
                "#include \"lib/h{first}.h\"\n#include \"lib/h{second}.h\"\n\
                 int unit{unit}() {{ Type{second}x1 t{{}}; return helper{first}({unit}) + t.value1; }}\n"
            ),
        );
    }
}

/// Whether the UI's capped justification ranking shows this collateral
/// source: fewer than `named` header sources rank before it by combined
/// (used + collateral) count, ties broken by file id.
fn could_be_shown(full: &LoadMetric, index: usize, named: usize) -> bool {
    let source = &full.collateral_sources[index];
    if !is_includable_header(&source.path) {
        return false;
    }
    let combined = |id: FileId| -> usize {
        full.semantic_sources
            .iter()
            .chain(&full.collateral_sources)
            .filter(|s| s.file_id == id)
            .map(|s| s.symbol_count)
            .sum()
    };
    let mine = combined(source.file_id);
    let header_ids: HashSet<FileId> = full
        .semantic_sources
        .iter()
        .chain(&full.collateral_sources)
        .filter(|s| s.symbol_count > 0 && is_includable_header(&s.path))
        .map(|s| s.file_id)
        .collect();
    let ahead = header_ids
        .into_iter()
        .filter(|id| {
            let theirs = combined(*id);
            theirs > mine || (theirs == mine && id.0 < source.file_id.0)
        })
        .count();
    ahead < named
}

/// The full metric as the compact entry promises to return it.
fn trimmed(mut full: LoadMetric, shape: CompactShape) -> LoadMetric {
    full.collateral_symbols.clear();
    let shown: Vec<bool> = (0..full.collateral_sources.len())
        .map(|index| could_be_shown(&full, index, shape.named_sources))
        .collect();
    for (source, keep) in full.collateral_sources.iter_mut().zip(shown) {
        source.symbols.sort();
        source.symbols.truncate(if keep { shape.names_per_source } else { 0 });
    }
    full.largest_collateral_source = full.collateral_sources.first().cloned();
    full
}

#[test]
fn compact_file_metric_is_the_trimmed_full_metric_for_every_file() {
    let dir = tempfile::tempdir().unwrap();
    layered_repo(dir.path());
    let mut pipeline = Pipeline::new(dir.path(), None).unwrap();
    let (graph, _closures) = pipeline.get_or_build_include_graph().unwrap();
    let reduced = pipeline.get_or_build_reduced().unwrap();
    let analyzer = LoadAnalyzer::new(&graph, &reduced);

    let mut compared = 0usize;
    let mut trimmed_sources = 0usize;
    let mut unnamed_sources = 0usize;
    for file_id in reduced.file_symbols.keys() {
        if !graph.file_paths.contains_key(file_id) {
            continue;
        }
        let full = analyzer.analyze_file_load_by_id(*file_id).unwrap();
        for (names_per_source, named_sources) in [(2, 1), (2, 3), (1000, 1000)] {
            let shape = CompactShape { names_per_source, named_sources };
            trimmed_sources += full
                .collateral_sources
                .iter()
                .filter(|source| source.symbol_count > names_per_source)
                .count();
            let compact = analyzer.analyze_file_load_compact_by_id(*file_id, shape);
            unnamed_sources += compact
                .collateral_sources
                .iter()
                .filter(|source| source.symbols.is_empty())
                .count();
            assert_eq!(compact, trimmed(full.clone(), shape), "file {file_id:?}");
        }
        compared += 1;
    }
    assert!(compared >= HEADERS, "every header compared ({compared})");
    assert!(trimmed_sources > 0, "the small sample size must actually trim");
    assert!(unnamed_sources > 0, "the small named-source cap must actually drop names");
}
