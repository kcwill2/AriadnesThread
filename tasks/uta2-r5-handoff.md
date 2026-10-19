# UTA2-R5 Handoff - sweep-A remainder: graph consumer map + lang-cpp/scala branch pins + cli arm verification

Binding inputs:
1. This file. 2. docs/specs/unit-test-audit.md "Round 2" (R5 bullet). 3. crates/graph/src/graph.rs:321 (map_aggregation_consumers) + planner consumer call site crates/cluster/src/planner.rs:171 (read-only evidence); crates/lang-cpp/src/tokenizer.rs:310-400 + plugin.rs:77; crates/lang-scala/src/boundary.rs; crates/cli/src/commands/transform.rs (action-arm survey). 4. Model: round-1 pin files per crate.

## Touch-set (hard)
- tasks/uta2-r5-journal.md
- crates/graph/src/**, crates/lang-cpp/src/**, crates/lang-scala/src/**, crates/cli/src/** (test-module additions + tests dir only; NO src logic changes this hand - if one seems needed, journal STOP)
You MUST NOT touch: other crates. No stash/branch change.

## What to build (pins + one verification)
1. graph map_aggregation_consumers (ZERO-coverage function): 3-file chain fixture tu->hdr->mid: transitive consumer listing, FileId-sorted output, tu_id-missing-from-closure skipped, empty-in -> empty-out.
2. lang-cpp raw_string_open multibyte: byte-escape specimens `"...é...R\"( { } )"` and leading-guard `raw_string_open("\"abc", 0) == false`; assert no brace leak + exact line/col. scan_raw_string_with_fallback header guard: REPORT-only (owner call: debug_assert vs delete) unless directly testable.
3. lang-cpp probe_compiler_status Available arm: probe the test binary itself (libtest --version banner) -> Available(banner-first-line); empty-stdout-success -> Available(binary-name) fallback pinned honestly; keep the Missing-arm lock intact.
4. lang-scala boundary.rs: EXTERNAL_PATH_MARKERS frozen-4 pin + near-miss false table (`targets`, `LIB`).
5. cli transform.rs action-arm verification: check whether H7a's seam tests already cover per-action arms; if yes journal-cite and skip; if no, add table-driven arm test asserting honest Err per stage (test-module only).

## RGB, hygiene, locks
All pins; a red that exposes unlisted wrong behavior = pin-exact + report. Existing locks byte-untouched.
Hygiene/encoding exit 0.

## Gates (timeout 600000 ms each)
1. `python tools/testrun.py --distill -- -p ariadne-graph`
2. `python tools/testrun.py --distill -- -p ariadne-lang-cpp`
3. `python tools/testrun.py --distill -- -p ariadne-lang-scala`
4. `python tools/testrun.py --distill -- -p ariadne-cli` (skip with journal cite if verification finds arms already covered)
5. hygiene + encoding witnesses
6. Commit: `python tools/commit_scoped.py --scope "crates/graph/**" --scope "crates/lang-cpp/**" --scope "crates/lang-scala/**" --scope "crates/cli/**" --scope "tasks/uta2-r5-journal.md" -m "test(graph,lang-cpp,lang-scala,cli): uta2-r5 consumer map + lexer branch + boundary marker pins"` (truth-adjust).

## Discipline
Write/Edit only, pure ASCII (byte escapes for multibyte); distill entry points; journal per green crate; STOP on context wall (order: graph, lang-cpp, lang-scala, cli).

## Report back
Commit hash + stat; per-crate canonical lines; row-5 verification verdict with evidence; out-of-scope observations.
