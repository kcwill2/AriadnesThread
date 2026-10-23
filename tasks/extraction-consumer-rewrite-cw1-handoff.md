# CW-1 Handoff - extraction-consumer-rewrite: C++ implements the reserved consumer_rewrite_spec hook (m20/m19 amendments as declared riders)

Milestone: `extraction-consumer-rewrite` (UNREGISTERED post-M28 track, no
M-number). Hand 1 of 3, strictly serial: CW-1 -> CW-2 -> CW-3. CW-2's cli
faithfulness oracle asserts THIS hand's behavior; CW-3's copy claims it.
Author rev: `a38c7aaa` on `main` (planner-verified, 2026-10-23). Start rev:
your dispatch head - journal it verbatim; every reviewer diff is against it.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/extraction-consumer-rewrite-kickoff.md` section 6 (frozen-lock
   amendment policy - the sharpest edge of this hand) and section 4 (inherited
   locks; the UTA intake row).
3. `tasks/extraction-consumer-rewrite-shape.md` section 0 (the defect and the
   adjudicated seam), C1 + C2 + C3 (your contracts), section 2 (invariants
   X1-X8), section 7 risks R1/R3/R4/R5.
4. `docs/specs/unit-test-audit.md` section "Deferred findings (closeout ledger
   sync, 2026-10-19)" - MANDATORY intake read before touching lang front-ends.
   Planner intake result 2026-10-23: zero deferred items name
   `dependency_rewriter.rs` or the `consumer_rewrite_spec` seam (the lang-cpp
   items name detector/planning/shaper/backend/parser surfaces). Carry any
   new finding you discover; report, do not fix.
5. Model (structure/doc style, do NOT imitate its grammar):
   `crates/lang-scala/src/dependency_rewriter.rs:150-182` - the Scala
   implementation of the same hook; and the trait doc contract
   `crates/lang/src/rewriter.rs:77-92`.
6. Literal sources (exact bytes, read them): `crates/lang-cpp/src/dependency_rewriter.rs`
   (whole file, 244 lines); fixture consumer texts
   `tests/fixtures/basic/tests/consumer.cpp` and `tests/fixtures/basic/src/thing.cpp`
   (each holds the bare `#include "thing.h"` on line 1); the new-include order
   pinned by `tests/integration/tests/m20_apply_migrate_characterization_test.rs:72-82`
   (`expected_thing_plan_new_includes`).
7. Your two rider tests in full: `tests/integration/tests/m20_apply_migrate_characterization_test.rs`
   (268 lines; you amend ONLY the test starting :179) and
   `tests/integration/tests/m19_optimize_characterization_test.rs` (215 lines;
   you amend ONLY the test starting :110).
8. `python tools/checkpoint_guard.py status` - FIRST HAND OF THE MILESTONE: run
   it as your bootstrap before any mutation (the architect hop had no bash;
   the expected tree at authoring was clean at `a38c7aaa` except the
   pre-existing dirty `extensions/ariadne-extension/ariadne-extension-0.1.0.vsix`).
   A dirty tree beyond that stray is a STOP-and-report.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/extraction-consumer-rewrite-cw1-journal.md (checkpoint journal; create it)
- crates/lang-cpp/src/dependency_rewriter.rs (the hook + helper + unit tests)
- tests/integration/tests/m20_apply_migrate_characterization_test.rs
  (DECLARED RIDER 1: hunks ONLY inside
  `apply_thing_plan_locks_transaction_outcome_and_disk_effects`)
- tests/integration/tests/m19_optimize_characterization_test.rs
  (DECLARED RIDER 2: hunks ONLY inside
  `optimize_writes_all_extracted_headers_and_leaves_consumer_sources_unchanged`,
  plus that one test's name)

You MUST NOT touch: `crates/cli/**`, `crates/transform/**` (the D6 ledger row's
named `IncludeRewriter` seam is REJECTED - shape section 0 adjudicates; the
ledger row stands as boundary of record, this handoff is the named seam inside
it, not a deviation), `crates/validate/**`, `crates/lang/**` (trait-default
pins are frozen locks), `crates/lang-scala/**` (its hook is already correct),
`tests/fixtures/**` (byte-frozen), `tools/**`, every other test in m19/m20,
any other pre-existing test. No `git stash`, no branch change.
Cargo.lock: no new dependencies (std::path only), so it must not appear in
your diff at all; if cargo rewrites it, revert.

## What to build (shape C1/C2/C3)

Inside `impl DependencyRewriter for CppDependencyRewriter` in
`crates/lang-cpp/src/dependency_rewriter.rs` add:

1. `fn consumer_rewrite_spec(&self, old_includes: &[std::path::PathBuf],
   new_includes: &[std::path::PathBuf], _moved_module_ids: &[String]) ->
   Option<ariadne_lang::RewriteSpec>` - ALWAYS `Some` (C++ always has directive
   grammar): maps every path through the bare-file-name rule
   `dependency_spelling`; `moved_module_ids` is unused (C++ directive space
   names files, not modules). Doc comment must state: the SAME bare-file-name
   rule `consumer_discovery_targets` probes with (trait default
   `crates/lang/src/rewriter.rs:66-75`), so discovery's predicate and the
   rewrite's matcher agree by construction.
2. Free fn `fn dependency_spelling(path: &std::path::Path) -> String` -
   `path.file_name().map(|name| name.to_string_lossy().into_owned())`
   else the display string (honest fallback for a path without a file name,
   mirroring the discovery default's refusal to invent a probe; same ASCII
   channel as today's fallback - X7).
3. The two characterization riders (next section) - the amendments ARE part of
   this hand's Red, not a follow-up.

No other function changes. `rewrite`, `find_dependency_targets`, `render_diff`
and `emit_file_preamble` stay byte-identical (X3/X8; m18 parity locks them).

## Frozen-lock amendment clause (kickoff section 6 - binding)

You are the ONLY hand that may ever amend m19/m20, only these two tests, only
under this clause, and only after capturing the Red (below). Proof obligations
the reviewer enforces:
- Every other assertion in both files byte-identical: consumer-set counts
  (`report.affected_consumer_files == 2` etc.), the m20 manifest/outcome/tier/
  generated-list/migrate assertions, the m19 report assertions (:144-145) and
  every other test - none of it moves a byte.
- m10/m14/m17/m18/m23 green UNEDITED (gate 6; the m18 parity diff text keeps
  rendering `- #include "include/thing.h"` - it is path-space DISPLAY, the fix
  lives below the payload layer, X3).
- The amended assertions must first FAIL on the pre-hook tree for the
  documented reason (captured Red, below).
- The new truth is independently witnessed, never only by your amendments:
  the C1 unit tests in this hand (esp. unit test 4) plus, landing next, CW-2's
  end-to-end equality.
- R8: if you discover any OTHER pre-existing test needing an edit, STOP and
  report; do not widen the amendment table.

## RGB protocol (single Red capture covering all three files)

1. Red: author ALL of - the four C1 unit tests, the m20 amendment, the m19
   amendment - before the hook exists. One Red capture, two scoped invocations
   (gates 2-3). Expected failures and WHY:
   - the 4 new unit tests fail because the trait default
     `consumer_rewrite_spec` returns `None` and every test expects `Some`
     (panic shows `left: None` / the expect message
     "C++ always has directive grammar");
   - amended m20 `apply_thing_plan_locks_transaction_outcome_and_disk_effects`
     fails its include-flip equality: the pre-hook apply is vacuous, so
     `tests/consumer.cpp` still reads `#include "thing.h"` (your panic message
     names the milestone key);
   - amended m19 `..._rewrites_consumer_includes` fails the same way via
     `EndToEndOptimizer::optimize_repository`.
   Capture the failing-test list verbatim in the journal. NEVER weaken an
   assertion to pass.
2. Green: implement the hook + helper; re-run gates 4-7 until green. NEVER
   weaken an assertion to pass.
3. Refactor: dedupe the expected-block constant between the two riders into
   each file locally (no cross-file extraction - they are separate test
   binaries); honest names; re-run.

## Required tests (exact)

In `crates/lang-cpp/src/dependency_rewriter.rs` `mod tests` (4 new tests;
house naming, no banned names):

1. `consumer_rewrite_spec_maps_bare_name_paths_identity`: paths
   `["old.h"] -> ["new.h"]`, moved `[]` map to `Some(RewriteSpec { old_deps:
   ["old.h"], new_deps: ["new.h"] })`. Journal note: this is exactly why the
   frozen byte-parity test `crates/validate/tests/transaction_apply_test.rs:48`
   (bare `old.h` payload) survives UNEDITED - the hook spec equals the path
   fallback there.
2. `consumer_rewrite_spec_maps_paths_to_bare_file_names`:
   `["include/thing.h"] -> old_deps ["thing.h"]`;
   `["generated/include/Thing_extracted.h", "include/ThingConfig_extracted.h"]
   -> new_deps ["Thing_extracted.h", "ThingConfig_extracted.h"]` (order
   preserved, no sorting).
3. `consumer_rewrite_spec_fallback_keeps_display_string_without_file_name`:
   `std::path::Path::new(".")` keeps its display string in the spec
   (`old_deps [ "." ]`), never a fabricated probe.
4. `discovery_hit_iff_rewrite_changes_text_over_the_spelling_classes`
   (X1(a) fidelity pair - the pair that kills the D6 class forever):
   spec built via the hook from old `["include/thing.h"]`, non-empty new deps;
   discovery target is `consumer_discovery_targets(Path::new("include/thing.h"), &[])`
   (= `"thing.h"`). For EACH spelling class assert
   `(find_dependency_targets(text, "thing.h") non-empty) ==
   (rewrite(text, &spec) != text)`:
   bare quoted (`#include "thing.h"`), angled (`#include <thing.h>`), CRLF
   (`#include "thing.h"\r\n...`), comment-embedded (`// #include "thing.h"` -
   pin BOTH sides explicitly: discovery hits AND the text changes; the
   substring symmetry is the invariant, R3), unrelated path spelling
   (`#include "sub/thing.h"` - neither hit nor change), no-match
   (`int main() {}\n` - byte-identical).

RIDER 1 - m20 `apply_thing_plan_locks_transaction_outcome_and_disk_effects`
(:179) ONLY. Replace the two identity assertions and the
"latent vacuous-rewrite bug, characterization only" message at :222-230 with
the fixed truth (message must name `extraction-consumer-rewrite`, house
provenance idiom):
- expected block literal (the four bare extracted includes, newline-joined, in
  the SAME BTreeSet order `expected_thing_plan_new_includes` pins -
  `ThingConfig_extracted.h`, `Thing_Thing_extracted.h`, `Thing_extracted.h`,
  `Thing_get_id_extracted.h`);
- `assert_eq!(consumer_after, consumer_before.replace("#include \"thing.h\"",
  EXPECTED_BLOCK), ...)` and the same for `src/thing.cpp` - one clause pins
  old-gone, four-in-place, order, and all-other-bytes-unchanged, and is
  byte-safe under either checkout EOL (do NOT hardcode full file bytes);
- explicit gone assert: `!consumer_after.contains("#include \"thing.h\"")`;
- X5 copy-only assert: `thing_impl_after.contains("Thing::Thing(ThingConfig config)")`
  (the flip is an include rewrite, not a move; declarations survive).
The manifest/outcome/tier/generated-list assertions (:192-220) and every other
test stay byte-identical.

RIDER 2 - m19
`optimize_writes_all_extracted_headers_and_leaves_consumer_sources_unchanged`
(:110) ONLY. Rename it to
`optimize_writes_all_extracted_headers_and_rewrites_consumer_includes`
(declared amendment; the name states the bug today). Test count and suite count
unchanged so census-by-name reconciliation stays mechanical. Replace the two
identity assertions + "latent vacuous-rewrite bug" message at :135-143 with
the same flip pair as RIDER 1 (the fixture preconditions at :113-119 stay).
`:144-145` (`report.committed_transformations == 1`,
`report.affected_consumer_files == 2`) and every other test stay
byte-identical (X2). The optimize path's payload quirks (its `generated/`-
prefixed `new_includes` spelling) are unchanged: the file-name rule maps both
spellings to the same bare identifiers, which is why one hook fixes all verbs
(R5 - intended symmetry).

## Hygiene (review-enforced)
Functions < 100 lines, files < 1000 lines (dependency_rewriter.rs 244 ->
~300-330; m19 215 -> ~230; m20 268 -> ~290 - all fine), no banned names, no
single-letter locals, intention-revealing identifiers
(`dependency_spelling`, `expected_block`, the renamed m19 test).
`python tools/hygiene_check.py` exit 0.

## Frozen locks (commands that prove them)
- m10/m14/m17/m18/m23 UNEDITED: gate 6 + reviewer per-commit diff (empty for
  those files).
- Scala hook + suites + m17 cone guard UNEDITED: gate 7 (`-p
  ariadne-lang-scala`) + `crates/lang-scala/**` in no diff (X4/R6).
- `crates/lang` trait-default pins: the crate is frozen (touch-set); its pins
  run inside the gate 8 census capture - journal the capture lines for
  `pin_neutral_consumer_rewrite_spec_default_is_none` staying green (that pin
  locks the DEFAULT, which you do not change).
- `crates/validate` byte-parity `transaction_apply_test`: inside gate 8 census;
  the file must appear in no diff.
- goldens/fixtures byte-locked; `UPDATE_GOLDENS=1` forbidden;
  `tools/**` operator-law; `tests/golden/**` untouched.
- Preview/dry-run bytes (X3): `old_includes` path payloads stay
  `vec![target_file.clone()]` (`crates/cli/src/commands/transform.rs:340`
  frozen) - you change no payload.

## Gates (exact commands, timeout 600000 ms each; scoped only; every raw cargo
run is a reviewer blocker - distilling entry points only)
1. `python tools/testrun.py --distill -p ariadne-lang-cpp` - clean baseline
   BEFORE any edit; journal the canonical line verbatim.
2. `python tools/testrun.py --distill -p ariadne-lang-cpp` - Red: exit 1,
   exactly the 4 named unit-test failures.
3. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m20_apply_migrate_characterization_test --test
   m19_optimize_characterization_test` - Red: exit 1, exactly the 2 amended
   tests failing (each for the documented reason; journal the two panic lines).
4. `python tools/testrun.py --distill -p ariadne-lang-cpp` - Green: exit 0,
   all 4 new tests plus the pre-existing ones.
5. same command as gate 3 - Green: exit 0.
6. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m10_validation_test --test m14_end_to_end_test --test
   m18_cpp_plugin_parity_test --test m17_plugin_seam_test --test
   m23_symbol_isolation_test` - exit 0, UNEDITED (X2/X3; m18 diff-text keeps
   the `- #include "include/thing.h"` display).
7. `python tools/testrun.py --distill -p ariadne-lang-scala` - exit 0 UNEDITED
   (X4/R6).
8. `python tools/testrun.py --distill` - full workspace census; reconcile from
   your journal start baseline and attribute every delta (expect +4 passed,
   suites and ignored held; anchor of record
   `1830 passed / 0 failed / 13 ignored / 119 suites @ 765df7b`; the
   `--baseline` FILE form stays superseded by this `--distill` substitute).
   Journal the capture lines proving `transaction_apply_test` and the
   `crates/lang` pins green.
9. `python tools/distill.py -- python tools/bench_compare.py` - fresh recording
   (timing gate runnable since `f20ad35`); machine-class WARNs recorded, NOT
   attributed (PR-2 persists); `crates/cli/tests` appears in no diff (standing
   producer proof - cite the empty `git diff --numstat` in the journal).
10. `python tools/hygiene_check.py` - exit 0.
11. Commit cycle (template standing row, itemized per the 2026-10-06 M28 law):
    `python tools/encoding_guard.py --staged` (or the `--all` superset if the
    staging seam refuses, journal the form used) then
    `python tools/commit_scoped.py --scope
    crates/lang-cpp/src/dependency_rewriter.rs --scope
    tests/integration/tests/m20_apply_migrate_characterization_test.rs --scope
    tests/integration/tests/m19_optimize_characterization_test.rs --scope
    tasks/extraction-consumer-rewrite-cw1-journal.md -m "fix(lang-cpp):
    implement consumer_rewrite_spec with the bare-file-name rule; amend
    m19/m20 characterizations (extraction-consumer-rewrite CW-1)"`.
    Never `git add .`. One landing commit; the journal rides inside it.

Gate-investment note (declared up front, per the 2026-10-06 M28 finding): the
honest routed count is 10 scoped invocations plus the commit cycle, inside
`<= ~10 + commit-cycle trio`; itemize per-gate counts in the journal and
declare any further overshoot as a dated D-row - silence is the deviation.

## Discipline clauses
- UTF-8: never Set-Content/Out-File/`>`/`Get-Content | -replace`; Write/Edit
  tools only; authored content pure ASCII (AGENTS.md File-Edit Discipline).
  Your amendments DELETE two pre-existing em-dash bytes inside the replaced
  messages and add only ASCII around them - the encoding guard must stay
  silent.
- Touch only the touch-set; out-of-scope breakage: report, do not fix.
- Amendment policy (kickoff section 6) and the R8 STOP-and-report clause above
  are the highest-priority contract after this file.
- After EACH Green gate append a 1-3 line checkpoint to the journal (module +
  commit/rev, any deviation, any lock finding).
- Context exhaustion looming: STOP, return a disk-state inventory; a dead
  session is never resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec
  tasks/extraction-consumer-rewrite-cw1-handoff.md --scope crates/lang-cpp
  --scope tests/integration/tests --cmd ...`.
- Never pin warning counts; never state a count without the command that
  produces it (M23.5 F14 standing rules).

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence: the 4 unit-test names +
   the 2 integration failures with verbatim panic lines (first assert only).
3. Green evidence: distill digest headers + canonical summary lines for gates
   4-8 + capture path. 4. Any deviation + reason. 5. Out-of-scope observations
   (report-only). Never paste raw tool output; cite command + canonical line +
   exit code.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill. Additional MUST checks for this hand: per-commit diff against the start
rev shows hunks ONLY in the touch-set; inside the two rider files, hunks sit
ONLY inside the two named tests (RIDER 2 additionally carries the declared
test-name change - a hunk on the `fn` line of that test is sanctioned, any
other is a rejection); X8's closed amendment list is the reject basis
(kickoff section 6, R1/R8).

Sizing (planner, itemized from the mandated bodies - M23.5 rule 4): hook +
helper ~30 lines; 4 unit tests ~95; m20 hunk ~+35/-9; m19 hunk ~+35/-9;
journal ~15 - total ~210-320 produced lines vs the ~1200 bound; 10 scoped
gate invocations + commit cycle vs the ~10 bound; one crate dir
(+ the two DECLARED rider files under `tests/integration/tests/` - the A-3
rider precedent; the split rationale for keeping riders with the hook: the
amendment hunks ARE the statement of the hook's truth - any seam separating
them forces a half-built tree between hands: hook-without-amendments leaves
m19/m20 red, amendments-without-hook cannot go green).
