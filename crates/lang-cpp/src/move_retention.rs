//! Marked comment-out retention for the C++ move hook (shape C2,
//! extraction-symbol-move SM-2).
//!
//! The retention form is the `ARIADNE-EXTRACT-OUT` line-prefix region: the
//! locator-certified declaration span stays in the file with every line
//! prefixed by the two bytes `//` between a BEGIN marker (bound to the plan
//! id and the requested names) and an END marker. Block comments are never
//! used - a `*/` inside moved code would terminate a region comment
//! mid-symbol. Every refusal answers `None` so the caller proceeds WITHOUT
//! the move (copy fallback); each refusal is one named branch (R-A..R-G in
//! the SM-2 handoff), mirroring the honest fallback of
//! `consumer_rewrite_spec`. The hook never re-slices text the locator did
//! not certify.

use ariadne_lang::SymbolKind;

/// Kind preference order for the verbatim locator; the first located body
/// is the certified span (`Other` is never tried).
const KIND_PREFERENCE: [SymbolKind; 6] = [
    SymbolKind::Class,
    SymbolKind::Struct,
    SymbolKind::Union,
    SymbolKind::Enum,
    SymbolKind::Function,
    SymbolKind::TypeAlias,
];

/// Recognition grammar (kickoff section 6): lines matching exactly
/// `^// ARIADNE-EXTRACT-OUT BEGIN plan=(\S+) symbols=(\S+)$` and
/// `^// ARIADNE-EXTRACT-OUT END plan=(\S+)$`.
const BEGIN_HEAD: &str = "// ARIADNE-EXTRACT-OUT BEGIN plan=";
const END_HEAD: &str = "// ARIADNE-EXTRACT-OUT END plan=";
const SYMBOLS_SEPARATOR: &str = " symbols=";

/// A contiguous whole-line span to retire, with the indices of the
/// requested names it answers.
struct Region {
    first_line: usize,
    last_line: usize,
    name_indices: Vec<usize>,
}

/// Comment out the named top-level symbols of `source` under
/// `ARIADNE-EXTRACT-OUT` markers bound to `plan_id`, or `None` when any
/// name refuses (the caller then proceeds without the move).
pub(crate) fn move_out_text(
    source: &str,
    qualified_names: &[String],
    plan_id: &str,
) -> Option<String> {
    if qualified_names.is_empty() {
        return None;
    }
    let lines = SourceLines::new(source);
    let interiors = lines.marked_interiors();
    let mut regions: Vec<Region> = Vec::new();
    for (name_index, name) in qualified_names.iter().enumerate() {
        let body = certified_body(source, name)?;
        let occurrence = unique_unmarked_occurrence(source, &body, &interiors)?;
        let first_line = lines.line_containing(occurrence);
        let last_line = lines.line_containing(occurrence + body.len() - 1);
        if refuses_region_boundary(source, &lines, first_line, last_line) {
            return None;
        }
        if !lines.is_line_aligned(occurrence, body.len(), first_line, last_line) {
            return None;
        }
        regions.push(Region {
            first_line,
            last_line,
            name_indices: vec![name_index],
        });
    }
    Some(render_marked_source(&lines, merge_regions(regions), qualified_names, plan_id))
}

/// The locator's certified verbatim body for `name` under the kind
/// preference order, or `None` (R-A, and - after a previous marking - R-B:
/// the tokenizer cannot see comment contents, so text retired inside a
/// marked region is simply no longer locatable).
fn certified_body(source: &str, name: &str) -> Option<String> {
    let wanted: Vec<(String, SymbolKind)> = KIND_PREFERENCE
        .iter()
        .map(|kind| (name.to_string(), *kind))
        .collect();
    crate::symbol_locator::locate(source, &wanted)
        .into_iter()
        .next()
        .map(|located| located.body)
}

/// R-C (byte uniqueness): the body must occur exactly ONCE outside existing
/// marked regions - several byte-identical copies mean we cannot know
/// which declaration to retire, so the candidate refuses whole.
fn unique_unmarked_occurrence(
    source: &str,
    body: &str,
    interiors: &[(usize, usize)],
) -> Option<usize> {
    let mut found: Option<usize> = None;
    for (offset, _) in source.match_indices(body) {
        let inside_marked = interiors
            .iter()
            .any(|(start, end)| offset >= *start && offset < *end);
        if inside_marked {
            continue;
        }
        if found.is_some() {
            return None;
        }
        found = Some(offset);
    }
    found
}

/// The refusal preflight (shape R2, the M25 span-safety hazard class):
/// R-E an unterminated block comment before the region, R-F a backslash
/// continuation on the line before the region, R-G a backslash continuation
/// on the region's last line. Each branch refuses the whole candidate.
fn refuses_region_boundary(
    source: &str,
    lines: &SourceLines<'_>,
    first_line: usize,
    last_line: usize,
) -> bool {
    if unterminated_block_comment_before(&source[..lines.line_starts[first_line]]) {
        return true;
    }
    if first_line > 0 && lines.contents[first_line - 1].ends_with('\\') {
        return true;
    }
    lines.contents[last_line].ends_with('\\')
}

/// R-E: conservative byte scan of the pre-region text - a `/*` with no
/// following `*/` (the scan does not know about line comments, which is
/// the conservative direction: it refuses more, never less).
fn unterminated_block_comment_before(preceding_text: &str) -> bool {
    let mut rest = preceding_text;
    while let Some(open) = rest.find("/*") {
        rest = &rest[open + 2..];
        match rest.find("*/") {
            Some(close) => rest = &rest[close + 2..],
            None => return true,
        }
    }
    false
}

/// Adjacent or overlapping spans merge into one region (one BEGIN/END pair
/// per retired run of declarations).
fn merge_regions(mut regions: Vec<Region>) -> Vec<Region> {
    regions.sort_by_key(|region| region.first_line);
    let mut merged: Vec<Region> = Vec::new();
    for region in regions {
        match merged.last_mut() {
            Some(previous) if region.first_line <= previous.last_line + 1 => {
                previous.last_line = previous.last_line.max(region.last_line);
                previous.name_indices.extend(region.name_indices);
            }
            _ => merged.push(region),
        }
    }
    merged
}

/// Render the marked file: regions apply BOTTOM-UP (descending line
/// positions) so earlier line indices stay valid for later splices.
fn render_marked_source(
    lines: &SourceLines<'_>,
    mut regions: Vec<Region>,
    qualified_names: &[String],
    plan_id: &str,
) -> String {
    for region in &mut regions {
        region.name_indices.sort_unstable();
    }
    let plan_field = plan_field_ascii(plan_id);
    let mut blocks: Vec<String> = (0..lines.contents.len())
        .map(|index| format!("{}{}", lines.contents[index], lines.terminators[index]))
        .collect();
    for region in regions.iter().rev() {
        let names: Vec<&String> = region
            .name_indices
            .iter()
            .map(|&name_index| &qualified_names[name_index])
            .collect();
        let mut marked_block =
            vec![region_marker_open(&plan_field, &names, lines.dominant_terminator)];
        for line_index in region.first_line..=region.last_line {
            let terminator = if lines.terminators[line_index].is_empty() {
                lines.dominant_terminator
            } else {
                lines.terminators[line_index]
            };
            marked_block.push(format!(
                "{}{}",
                prefix_out_region(lines.contents[line_index]),
                terminator
            ));
        }
        marked_block.push(region_marker_close(&plan_field, lines.dominant_terminator));
        blocks.splice(region.first_line..=region.last_line, marked_block);
    }
    blocks.concat()
}

/// The BEGIN marker line: provenance is the plan id; the symbols field is
/// the requested names, normalized one-way for the grammar.
fn region_marker_open(plan_field: &str, names: &[&String], terminator: &str) -> String {
    let names_field = names
        .iter()
        .map(|name| normalize_marker_name(name))
        .collect::<Vec<String>>()
        .join(";");
    format!(
        "{BEGIN_HEAD}{plan_field}{SYMBOLS_SEPARATOR}{names_field}{terminator}"
    )
}

/// The END marker line, bound to the same plan field.
fn region_marker_close(plan_field: &str, terminator: &str) -> String {
    format!("{END_HEAD}{plan_field}{terminator}")
}

/// The line-prefix retention form: the two bytes `//` prepended to the
/// ORIGINAL line bytes (no added space; stripping restores exactly; an
/// empty original line becomes bare `//`).
fn prefix_out_region(line_content: &str) -> String {
    format!("//{line_content}")
}

/// The plan id as the grammar demands it: printable-ASCII without space
/// (`0x21..=0x7E`), every other character `?` - the A9 ASCII channel plus
/// the `\S+` recognition fields' no-space demand.
fn plan_field_ascii(plan_id: &str) -> String {
    plan_id
        .chars()
        .map(|character| {
            if (0x21..=0x7E).contains(&(character as u32)) {
                character
            } else {
                '?'
            }
        })
        .collect()
}

/// One requested name for the symbols field: characters outside
/// `[A-Za-z0-9_:]` map to `_` (documented one-way; names are human
/// provenance, the byte truth lives in the region lines).
fn normalize_marker_name(name: &str) -> String {
    name.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' || character == ':' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

/// The source split into lines with terminators and byte offsets preserved.
struct SourceLines<'a> {
    contents: Vec<&'a str>,
    terminators: Vec<&'a str>,
    line_starts: Vec<usize>,
    dominant_terminator: &'static str,
}

impl<'a> SourceLines<'a> {
    fn new(source: &'a str) -> Self {
        let mut contents = Vec::new();
        let mut terminators = Vec::new();
        let mut line_starts = Vec::new();
        let mut byte_offset = 0usize;
        let mut crlf_count = 0usize;
        for chunk in source.split_inclusive('\n') {
            let terminator = if chunk.ends_with("\r\n") {
                crlf_count += 1;
                "\r\n"
            } else if chunk.ends_with('\n') {
                "\n"
            } else {
                ""
            };
            contents.push(&chunk[..chunk.len() - terminator.len()]);
            terminators.push(terminator);
            line_starts.push(byte_offset);
            byte_offset += chunk.len();
        }
        let line_count = contents.len();
        let dominant_terminator =
            if line_count > 0 && crlf_count * 2 >= line_count { "\r\n" } else { "\n" };
        Self {
            contents,
            terminators,
            line_starts,
            dominant_terminator,
        }
    }

    /// Index of the line whose bytes contain `byte`.
    fn line_containing(&self, byte: usize) -> usize {
        match self.line_starts.binary_search(&byte) {
            Ok(exact) => exact,
            Err(next_start) => next_start - 1,
        }
    }

    /// Byte ranges of already-marked region interiors (between a BEGIN
    /// marker line and its END marker line, markers included); the
    /// recognition grammar above defines the boundaries. An unterminated
    /// BEGIN conservatively marks to end of file.
    fn marked_interiors(&self) -> Vec<(usize, usize)> {
        let mut interiors = Vec::new();
        let mut open: Option<usize> = None;
        for (index, content) in self.contents.iter().enumerate() {
            if open.is_none() && begin_marker_plan(content).is_some() {
                open = Some(self.line_starts[index]);
            } else if end_marker_plan(content).is_some() {
                if let Some(start) = open.take() {
                    interiors.push((start, self.line_starts[index]));
                }
            }
        }
        if let Some(start) = open {
            interiors.push((start, usize::MAX / 2));
        }
        interiors
    }

    /// R-D (line alignment): before the body on its first line and after
    /// the body on its last line only whitespace may live - that is what
    /// makes "region lines minus the prefix equal the original WHOLE line
    /// bytes" literally true.
    fn is_line_aligned(
        &self,
        occurrence: usize,
        body_len: usize,
        first_line: usize,
        last_line: usize,
    ) -> bool {
        let before_span = &self.contents[first_line][..occurrence - self.line_starts[first_line]];
        let after_span = &self.contents[last_line][occurrence + body_len - self.line_starts[last_line]..];
        before_span.trim().is_empty() && after_span.trim().is_empty()
    }
}

/// Parsed plan field of a BEGIN marker line, per the recognition grammar.
fn begin_marker_plan(content: &str) -> Option<&str> {
    let rest = content.strip_prefix(BEGIN_HEAD)?;
    let (plan, names) = rest.split_once(SYMBOLS_SEPARATOR)?;
    (is_closed_field(plan) && is_closed_field(names)).then_some(plan)
}

/// Parsed plan field of an END marker line, per the recognition grammar.
fn end_marker_plan(content: &str) -> Option<&str> {
    let plan = content.strip_prefix(END_HEAD)?;
    is_closed_field(plan).then_some(plan)
}

/// A recognition-grammar field: non-empty and whitespace-free.
fn is_closed_field(field: &str) -> bool {
    !field.is_empty() && !field.chars().any(char::is_whitespace)
}

#[cfg(test)]
mod tests {
    use crate::dependency_rewriter::CppDependencyRewriter;
    use ariadne_lang::DependencyRewriter;

    const FIXTURE_THING_H: &str = include_str!("../../../tests/fixtures/basic/include/thing.h");

    const BEGIN_THING: &str = "// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-TEST symbols=Thing";
    const END_PLAN_TEST: &str = "// ARIADNE-EXTRACT-OUT END plan=PLAN-TEST";
    const OPEN_HEAD: &str = "// ARIADNE-EXTRACT-OUT BEGIN ";
    const CLOSE_HEAD: &str = "// ARIADNE-EXTRACT-OUT END ";

    fn move_out(source: &str, names: &[&str], plan_id: &str) -> Option<String> {
        let qualified: Vec<String> = names.iter().map(|name| name.to_string()).collect();
        CppDependencyRewriter.move_out_top_level_symbols(source, &qualified, plan_id)
    }

    /// Lines of a text, each WITH its terminator (the last may have none).
    fn split_lines_keep_terminators(text: &str) -> Vec<&str> {
        text.split_inclusive('\n').collect()
    }

    /// A split line minus its trailing terminator (precise strip, no trimming
    /// of meaningful whitespace).
    fn line_content(line_with_terminator: &str) -> &str {
        let without_lf = line_with_terminator.strip_suffix('\n').unwrap_or(line_with_terminator);
        without_lf.strip_suffix('\r').unwrap_or(without_lf)
    }

    fn count_lines_with_head(text: &str, head: &str) -> usize {
        split_lines_keep_terminators(text)
            .iter()
            .filter(|line| line_content(line).starts_with(head))
            .count()
    }

    /// Byte-exact manual matcher for `^// ARIADNE-EXTRACT-OUT BEGIN
    /// plan=(\S+) symbols=(\S+)$` (std only; no regex dependency).
    fn begin_marker_fields(content: &str) -> Option<(&str, &str)> {
        let rest = content.strip_prefix("// ARIADNE-EXTRACT-OUT BEGIN plan=")?;
        let (plan, names) = rest.split_once(" symbols=")?;
        let fields_are_closed_runs = !plan.is_empty()
            && !names.is_empty()
            && !plan.chars().any(char::is_whitespace)
            && !names.chars().any(char::is_whitespace);
        fields_are_closed_runs.then_some((plan, names))
    }

    /// Byte-exact manual matcher for `^// ARIADNE-EXTRACT-OUT END plan=(\S+)$`.
    fn end_marker_plan(content: &str) -> Option<&str> {
        let plan = content.strip_prefix("// ARIADNE-EXTRACT-OUT END plan=")?;
        (!plan.is_empty() && !plan.chars().any(char::is_whitespace)).then_some(plan)
    }

    #[test]
    fn move_out_marks_line_aligned_class_region_verbatim_under_prefix() {
        // Fixture walkthrough (tests/fixtures/basic/include/thing.h, 13
        // lines): `class Thing {` at line 6 through `};` at line 13 (the
        // ctor declaration :8 and the method declaration :9 sit INSIDE the
        // record span). The region is those 8 whole lines; every byte of the
        // other 5 lines stays untouched.
        let marked = move_out(FIXTURE_THING_H, &["Thing"], "PLAN-TEST")
            .expect("extraction-symbol-move SM-2: class Thing must be retired under marked comment-out");
        let source_lines = split_lines_keep_terminators(FIXTURE_THING_H);
        let marked_lines = split_lines_keep_terminators(&marked);

        let begin_positions: Vec<usize> = marked_lines
            .iter()
            .enumerate()
            .filter(|(_, line)| begin_marker_fields(line_content(line)).is_some())
            .map(|(index, _)| index)
            .collect();
        let end_positions: Vec<usize> = marked_lines
            .iter()
            .enumerate()
            .filter(|(_, line)| end_marker_plan(line_content(line)).is_some())
            .map(|(index, _)| index)
            .collect();
        assert_eq!(begin_positions.len(), 1, "exactly one BEGIN marker line");
        assert_eq!(end_positions.len(), 1, "exactly one END marker line");
        let begin = begin_positions[0];
        let end = end_positions[0];

        // Exact marker bytes.
        assert_eq!(line_content(marked_lines[begin]), BEGIN_THING);
        assert_eq!(line_content(marked_lines[end]), END_PLAN_TEST);
        assert_eq!(end, begin + 9, "eight region lines between the markers");

        // Region lines: prefix-strip equals the original 8 lines byte-exactly.
        for offset in 0..8 {
            let expected = format!("//{}", source_lines[5 + offset]);
            assert_eq!(marked_lines[begin + 1 + offset], expected, "region line {offset} must be the original bytes under the two-byte prefix");
        }

        // Bytes outside the region unchanged.
        assert_eq!(&marked_lines[..begin], &source_lines[..5], "bytes before the region unchanged");
        assert_eq!(&marked_lines[end + 1..], &source_lines[13..], "bytes after the region unchanged");
    }

    #[test]
    fn move_out_marks_multiple_spans_bottom_up_and_merges_adjacent_regions() {
        // SM-I1 hazard corpus: namespaced class, template, multi-line
        // signature. Gear (lines 1-3) and identity (lines 4-7) are adjacent
        // and merge; spaced_sum (lines 9-13) keeps its own pair.
        let source = "namespace widgets {\nclass Gear {\n    int ratio;\n};\ntemplate <typename T>\nT identity(T value) {\n    return value;\n}\n\nint spaced_sum(\n    int left,\n    int right) {\n    return left + right;\n}\n";
        let marked = move_out(source, &["widgets::Gear", "identity", "spaced_sum"], "PLAN-TEST")
            .expect("extraction-symbol-move SM-2: all three spans must be retired under marked comment-out");
        assert_eq!(count_lines_with_head(&marked, OPEN_HEAD), 2, "adjacent spans merge; the non-adjacent span keeps its own pair");
        assert_eq!(count_lines_with_head(&marked, CLOSE_HEAD), 2);
        assert!(marked.contains("// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-TEST symbols=widgets::Gear;identity"));
        assert!(marked.contains("// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-TEST symbols=spaced_sum"));

        let marked_lines = split_lines_keep_terminators(&marked);
        let mut reconstruction = String::new();
        for line in &marked_lines {
            let content = line_content(line);
            let is_marker =
                begin_marker_fields(content).is_some() || end_marker_plan(content).is_some();
            if is_marker {
                continue;
            }
            let terminator = &line[content.len()..];
            reconstruction.push_str(content.strip_prefix("//").unwrap_or(content));
            reconstruction.push_str(terminator);
        }
        assert_eq!(reconstruction, source, "removing marker lines and the two-byte prefixes must restore the source byte-for-byte (bottom-up application, earlier line indices valid)");
    }

    #[test]
    fn move_out_preserves_crlf_terminators_in_regions_and_markers() {
        let source = "int compute() {\r\n    return 42;\r\n}\r\n";
        let marked = move_out(source, &["compute"], "PLAN-TEST")
            .expect("extraction-symbol-move SM-2: the CRLF body must be retired under marked comment-out");
        let marked_lines = split_lines_keep_terminators(&marked);
        assert_eq!(marked_lines.len(), 5, "BEGIN + 3 region lines + END");
        assert_eq!(marked_lines[0], "// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-TEST symbols=compute\r\n", "marker lines use the file's dominant terminator");
        assert_eq!(marked_lines[4], "// ARIADNE-EXTRACT-OUT END plan=PLAN-TEST\r\n");
        let source_lines = split_lines_keep_terminators(source);
        for offset in 0..3 {
            let expected = format!("//{}", source_lines[offset]);
            assert_eq!(marked_lines[1 + offset], expected, "region lines keep their \\r\\n verbatim, including the \\r under the prefix");
        }
    }

    #[test]
    fn move_out_binds_marker_provenance_to_plan_id_and_normalizes_names() {
        // A qualifying prefix segment may carry characters outside
        // [A-Za-z0-9_:]; the locator's suffix match still answers the
        // record. The symbols field maps them to '_' (documented one-way).
        let record_source = "class Thing {\n    int id;\n};\n";
        let marked = move_out(record_source, &["release:v2-beta::Thing"], "PLAN-TEST")
            .expect("extraction-symbol-move SM-2: the suffixed qualified name must retire the record");
        assert!(marked.contains("// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-TEST symbols=release:v2_beta::Thing"));

        // A plan id carrying a non-ASCII byte renders '?' per character
        // (byte escape per AGENTS.md fixture rule, never a literal).
        let struct_source = "struct Part {\n    int count;\n};\n";
        let marked_foreign = move_out(struct_source, &["Part"], "PLAN-\u{e9}")
            .expect("extraction-symbol-move SM-2: a non-ASCII plan id must still retire the struct with ? rendering");
        assert!(marked_foreign.contains("// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-? symbols=Part"));

        // Recognition contract: every produced marker line matches its
        // regex-equivalent; no region content line matches either.
        let marked_lines = split_lines_keep_terminators(&marked_foreign);
        assert!(begin_marker_fields(line_content(marked_lines[0])).is_some());
        assert!(end_marker_plan(line_content(marked_lines[4])).is_some());
        for content_line in &marked_lines[1..4] {
            let content = line_content(content_line);
            assert!(begin_marker_fields(content).is_none() && end_marker_plan(content).is_none(), "region content line must not masquerade as a marker: {content}");
        }
    }

    #[test]
    fn move_out_refuses_unlocatable_name() {
        // Pair (R-A): the clean variant marks first (Red non-vacuity).
        let marked = move_out(FIXTURE_THING_H, &["Thing"], "PLAN-TEST")
            .expect("extraction-symbol-move SM-2: the paired clean variant must mark");
        assert!(marked.contains(BEGIN_THING));
        assert!(
            move_out(FIXTURE_THING_H, &["ThingConfig"], "PLAN-TEST").is_none(),
            "R-A: no kind in the preference order locates ThingConfig in this 13-line header"
        );
    }

    #[test]
    fn move_out_refuses_name_already_locatable_only_inside_marked_region() {
        // Pair (R-B, idempotence): Alpha marks, re-offering Alpha answers
        // None (the tokenizer cannot see comment contents, so the marked
        // declaration is simply no longer locatable - the OUTCOME SM-I1
        // pins), while a different un-marked name in the same file still
        // marks. Never nested marking.
        let source = "class Alpha {\n    int a;\n};\n\nclass Beta {\n    int b;\n};\n";
        let once = move_out(source, &["Alpha"], "PLAN-TEST")
            .expect("extraction-symbol-move SM-2: the paired clean variant must mark");
        assert!(once.contains("// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-TEST symbols=Alpha"));
        assert!(
            move_out(&once, &["Alpha"], "PLAN-TEST").is_none(),
            "R-B: re-offering a name locatable only inside the marked region answers None"
        );
        let twice = move_out(&once, &["Beta"], "PLAN-TEST")
            .expect("extraction-symbol-move SM-2: a different un-marked name in the same file still marks");
        assert_eq!(count_lines_with_head(&twice, OPEN_HEAD), 2, "the second pair is added; the first region stays intact (no nesting)");
        assert!(twice.contains("// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-TEST symbols=Alpha"));
        assert!(twice.contains("// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-TEST symbols=Beta"));
    }

    #[test]
    fn move_out_refuses_body_ambiguous_or_not_line_aligned() {
        // Pair (R-C, byte uniqueness): a body occurring twice outside marked
        // regions cannot safely retire; the unique variant marks.
        let unique_source = "void helper();\nvoid other();\n";
        let marked_unique = move_out(unique_source, &["helper"], "PLAN-TEST")
            .expect("extraction-symbol-move SM-2: the paired unique variant must mark");
        assert!(marked_unique.contains("// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-TEST symbols=helper"));
        assert!(
            move_out("void helper();\nvoid helper();\n", &["helper"], "PLAN-TEST").is_none(),
            "R-C: multiple byte-identical occurrences outside marked regions = cannot safely retire"
        );

        // Pair (R-D, line alignment): bytes before the body on the first
        // line (or after it on the last) beyond whitespace refuse.
        let aligned_source = "int counter = 0;\nint compute() { return counter; }\n";
        let marked_aligned = move_out(aligned_source, &["compute"], "PLAN-TEST")
            .expect("extraction-symbol-move SM-2: the paired aligned variant must mark");
        assert!(marked_aligned.contains("// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-TEST symbols=compute"));
        assert!(
            move_out("int counter = 0; int compute() { return counter; }\n", &["compute"], "PLAN-TEST").is_none(),
            "R-D: leading text before the body on its first line refuses (whole-line retention)"
        );
    }

    #[test]
    fn move_out_refuses_region_start_inside_unterminated_block_comment() {
        // Pair (R-E, conservative pre-region scan): a `/*` in the pre-region
        // bytes with no `*/` before the region start refuses, even when the
        // tokenizer never opened a block comment (the `/*` sits in a line
        // comment); closing it on the same line marks.
        let closed = "// see /* design */\nclass Marker { int x; };\n";
        let marked = move_out(closed, &["Marker"], "PLAN-TEST")
            .expect("extraction-symbol-move SM-2: the paired closed-comment variant must mark");
        assert!(marked.contains("// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-TEST symbols=Marker"));
        assert!(
            move_out("// see /* design\nclass Marker { int x; };\n", &["Marker"], "PLAN-TEST").is_none(),
            "R-E: an unterminated block comment before the region start refuses (conservative scan)"
        );
    }

    #[test]
    fn move_out_refuses_backslash_continuation_on_the_line_before_the_region() {
        // Pair (R-F, macro-with-trailing-backslash hazard, SM-I1 corpus):
        // the preceding line ending in a backslash would pull the BEGIN
        // marker into that line's logical continuation.
        let clean = "#define TRACE(msg) fprintf(msg)\nclass Marker { int x; };\n";
        let marked = move_out(clean, &["Marker"], "PLAN-TEST")
            .expect("extraction-symbol-move SM-2: the paired clean variant must mark");
        assert!(marked.contains("// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-TEST symbols=Marker"));
        assert!(
            move_out("#define TRACE(msg) fprintf(msg) \\\nclass Marker { int x; };\n", &["Marker"], "PLAN-TEST").is_none(),
            "R-F: a backslash continuation on the line before the region refuses"
        );
    }

    #[test]
    fn move_out_refuses_backslash_continuation_at_the_region_last_line() {
        // Pair (R-G): a region whose LAST line ends in a backslash would
        // splice the END marker into the comment chain and break the
        // balanced-pair grammar. The refusal contract pins the OUTCOME
        // (None); under planner refinement R-D this same input is also
        // not line-aligned, so both named refusals answer it - the
        // R-G predicate is checked independently in the preflight.
        let clean = "class Marker { int x; };\n";
        let marked = move_out(clean, &["Marker"], "PLAN-TEST")
            .expect("extraction-symbol-move SM-2: the paired clean variant must mark");
        assert!(marked.contains("// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-TEST symbols=Marker"));
        assert!(
            move_out("class Marker { int x; }; \\\n", &["Marker"], "PLAN-TEST").is_none(),
            "R-G: a backslash continuation at the region's last line refuses"
        );
    }
}
