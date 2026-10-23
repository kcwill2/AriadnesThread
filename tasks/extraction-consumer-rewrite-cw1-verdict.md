# CW-1 Verdict - extraction-consumer-rewrite

Reviewer hop, 2026-10-23. Contract: `tasks/extraction-consumer-rewrite-cw1-handoff.md`.
Journal: `tasks/extraction-consumer-rewrite-cw1-journal.md`. Start rev of record:
`2e5b86f` (journal-bootstrapped; every diff adjudicated against it).

## VERDICT: APPROVED

Chain of record: landing `c916a26` (fix, 3 code files + journal) + journal rider
`f73a9a1` (journal-only), on planning `2e5b86f` + `53fee56`. HEAD `f73a9a1` on
`main`; dirty: only the tolerated pre-existing
`extensions/ariadne-extension/ariadne-extension-0.1.0.vsix` (checkpoint_guard
ground truth at review start). No STOP condition.

## Commit-vs-touch-set: PASS

`git diff --name-only 2e5b86f..HEAD` returns exactly the four declared paths:

```
146	0	crates/lang-cpp/src/dependency_rewriter.rs
261	0	tasks/extraction-consumer-rewrite-cw1-journal.md
 18	6	tests/integration/tests/m19_optimize_characterization_test.rs
 26	5	tests/integration/tests/m20_apply_migrate_characterization_test.rs
```

No path outside the touch-set. `Cargo.lock` appears in no diff (std::path only).
`crates/cli/**` (incl. the frozen `transform.rs:340` payload), `crates/transform/**`,
`crates/validate/**`, `crates/lang/**`, `crates/lang-scala/**`, `tests/fixtures/**`,
`tools/**` appear in no diff. `git diff --numstat 2e5b86f..HEAD -- crates/cli/tests`
is empty - the standing producer proof for CW-2's oracle attribution is verified.

## Per-commit review (oldest-first)

### c916a26 - fix(lang-cpp): implement consumer_rewrite_spec ... amend m19/m20

1. **`crates/lang-cpp/src/dependency_rewriter.rs` (+146/-0, pure insertion).**
   The four frozen functions (`rewrite`, `find_dependency_targets`, `render_diff`,
   `emit_file_preamble`) are byte-identical by the numstat itself (zero deleted
   lines). Hook matches the handoff C1 contract literally: always `Some`, maps
   both payload sides through `dependency_spelling`, `_moved_module_ids` unused,
   doc comment states the SAME bare-file-name rule `consumer_discovery_targets`
   probes with and cites the trait default `crates/lang/src/rewriter.rs:66-75`
   (verified: default there is the `file_name()` probe; `consumer_rewrite_spec`
   default returns `None` at :85-92 - the Red reason is real). Free fn
   `dependency_spelling` sits outside the impl (file :120-124): `file_name()` ->
   lossy string, else display fallback (X7 honest fallback, lossy-ASCII channel
   matches the trait default's). Signature matches the trait verbatim.
2. **The 4 required unit tests, exact names, exact literals.** Test 1 carries the
   journal note pinning why `crates/validate/tests/transaction_apply_test.rs:48`
   survives UNEDITED. Test 2 pins order-preserved/no-sorting. Test 3 keeps the
   `"."` display string, never a fabricated probe (adds an honest `".."` new-side
   assertion beyond the letter - strengthening only, accepted). Test 4 is the
   X1(a) fidelity pair over ALL SIX spelling classes (bare-quoted, angled, CRLF,
   comment-embedded pinned on BOTH sides per R3, unrelated path spelling,
   no-match byte-identical) - each class pins both sides explicitly, which is
   the equality the handoff demanded.
3. **RIDER 1 - m20 (`apply_thing_plan_locks_transaction_outcome_and_disk_effects`,
   base :179).** One hunk at base :221-233, inside the named test (base blob read
   byte-safe via git MCP; the test runs :179-233, hunk interior). The `expected_block`
   literal is the four bare extracted includes, newline-joined, in exactly the
   BTreeSet order `expected_thing_plan_new_includes` pins (base :72-82:
   ThingConfig, Thing_Thing, Thing, Thing_get_id). Replace-clone flip pair for
   `tests/consumer.cpp` and `src/thing.cpp` (byte-safe under either checkout EOL,
   no hardcoded file bytes); explicit gone assert `!consumer_after.contains(...)`;
   X5 copy-only assert `thing_impl_after.contains("Thing::Thing(ThingConfig config)")`.
   Message names the milestone key. The manifest/outcome/tier/generated-list
   assertions (:192-220) and every other test are byte-identical (single hunk,
   proven by the diff itself).
4. **RIDER 2 - m19 (base test at :110).** Exactly two hunks: the sanctioned `fn`
   line rename `..._leaves_consumer_sources_unchanged` ->
   `..._rewrites_consumer_includes`, and the body hunk at :134-147 inside the
   named test. Fixture preconditions (:113-119) and the report pins at :144-145
   (`committed_transformations == 1`, `affected_consumer_files == 2`) are hunk
   context - unchanged. "Same flip pair as RIDER 1" delivered literally; the
   RIDER-1-only gone/X5 bullets are not required by RIDER 2's letter and their
   absence is compliant (the replace-clone equality pins old-gone given the
   :113 precondition). Test count and suite count unchanged (5 tests m19; census
   suites held at 119) - rename keeps census-by-name reconciliation mechanical.
5. **Journal inside the landing commit** - sanctioned (handoff gate 11: "One
   landing commit; the journal rides inside it").

### f73a9a1 - docs(cw-1): journal closeout

Journal-only scoped commit (+42 to the journal). House rider precedent correctly
applied: the record of `c916a26`'s SHA cannot ride the commit it names.

## Frozen-seam enforcement (this hand's headline risk): PASS

- `IncludeRewriter` lives in `crates/transform/src/placement.rs` (re-export
  `transform/src/lib.rs:13`); `plan_target_consumers` lives in
  `crates/cli/src/commands/transform.rs:314`. Neither file appears in any diff of
  this hand -> both zero-diff. The D6 `IncludeRewriter` seam stays REJECTED and
  untouched; this hand is the named seam inside its boundary, as adjudicated.
- m10/m14/m17/m18/m23 + all Scala files + the m17 cone guard + every other test
  in m19/m20: byte-identical at this hop (absent from `2e5b86f..HEAD`). The m18
  parity text keeps the `- #include "include/thing.h"` path-space display (file
  untouched; suite green UNEDITED).
- Trait-default pins and `crates/validate` byte-parity: green in the reviewer's
  own fresh census capture -
  `test rewriter::tests::pin_neutral_consumer_rewrite_spec_default_is_none ... ok`
  (capture line 1388) and `test test_transaction_apply_and_rollback ... ok`
  (capture line 2639).

## Red capture adjudication

The Red evidence is journal-only (reviewer cannot mutate the tree to re-derive
it), but it is internally corroborated: the trait default provably returns
`None` (read at HEAD, frozen crate), so the 4 unit tests must have panicked on
`expect("C++ always has directive grammar")` pre-hook; the two verbatim journal
panic messages match the committed assert messages of the amendments word-for-word;
the gate-3a split reason (cargo fail-fast binary ordering) is mechanically true.
The `1 suites` at Red is a disclosed distiller artifact (doctest suite skipped)
that reconciles at Green (2 suites). No assertion was weakened: both riders
replace bug-characterization equality with strictly stronger fixed truth; every
other assertion in both files is byte-identical.

## Deviations - independent dispositions

1. **Gate 9 (bench_compare) skipped by implementer - ACCEPTED.** Orchestration
   assignment (dispatch moved all workspace-wide gates to the reviewer hop), not
   a contract break: the producer-side half of gate 9 (`crates/cli/tests` zero-diff)
   was performed and the reviewer ran the fresh recording (PASS below), so the
   obligation is fully discharged across the two hops.
2. **Gate 3 split into 3 + 3a - ACCEPTED.** `cargo test` runs test binaries
   serially and stops at the first failing binary, so one command provably
   cannot surface both integration panics. Total scoped invocations stayed at
   10 (1,2,3,3a,4,5,6,7,8,10 + commit cycle, inside the declared bound; itemized
   gate table in the journal).
3. **Census via `--distill` substitute - ACCEPTED, and not actually a deviation.**
   Handoff gate 8 prescribes it verbatim ("the `--baseline` FILE form stays
   superseded by this `--distill` substitute"). Plan-literal compliance,
   mis-filed by the implementer as a deviation - harmless.
4. **encoding_guard form (`--staged` vacuous + `--all` real) - ACCEPTED.** The
   handoff's gate 11 anticipates the `--all` superset form and demands the form
   be journaled (it was). The `--staged` run exited 0 vacuously because
   `commit_scoped.py` stages its own scopes (raw `git add` denied to the
   implementer); `--all` is the strictly stronger check. Reviewer's own fresh
   `--all` run: silent, exit 0 (the tool prints every violation to stdout before
   nonzero exit - silence is pass evidence).
5. **Journal 219 lines at landing (261 total) vs the ~15 sizing estimate -
   ACCEPTED, cosmetic.** The sizing bound that binds is total produced lines vs
   ~1200: actual 451 insertions / 11 deletions across the hand, and the
   code-file diff (+190/-11) sits inside the ~210-320 estimate band. The journal
   volume IS the artifact the contract demanded (verbatim Red panics, itemized
   gate table, zero-diff proofs, per-gate checkpoints); the planner's ~15 was
   simply a bad estimate, not a discipline failure. No gate, lock, or bound
   affected.

## Hygiene + encoding: PASS

- `python tools/hygiene_check.py`: R1/R2 blocking silent; all output lines are
  warn-only R3 rows in files this hand did not touch (neither `dependency_rewriter.rs`
  nor either rider file appears in any row). New code uses intention-revealing
  names (`dependency_spelling`, `expected_block`, `discovery_hits`, `hook_spec`,
  `probe`), no single-letter locals, all functions < 100 lines,
  `dependency_rewriter.rs` at 390 lines (< 1000).
- `python tools/encoding_guard.py --all`: silent (pass). The amendments deleted
  the two pre-existing em-dashes inside the replaced messages and added pure
  ASCII around them - as the handoff predicted.
- Authored content verified pure ASCII at HEAD (hook, tests, riders, journal).

## Workspace-wide acceptance gates (reviewer-run, fresh)

See `tasks/extraction-consumer-rewrite-execution-results.md` for the verbatim
lines and the explicit not-run statement (`--baseline` FILE form).

## Findings

No blockers. Minor (none gating; none require a CW-1 change):

1. (minor, config) Reviewer-sandbox allowlist refuses several mandated gate forms
   outright: direct `python tools/ui_smoke.py`, and every exit-code capture form
   (`python -c ...`, `cmd /c ...`, `echo`, `; Write-Output` chaining) - each
   answered with the boilerplate catch-all deny. `python tools/distill.py --
   python tools/ui_smoke.py` was allowed and ui_smoke PASSed, so no gate was
   skipped; exit codes for plain-form gates rest on tool semantics (encoding_guard
   prints violations then exits nonzero; hygiene R1/R2 silence). Ledger this as
   an operator-law item: the reviewer allowlist should include
   `python tools/ui_smoke.py` directly and one exit-code-echo form.
2. (minor, record-only) Implementer journal mis-files the plan-literal `--distill`
   census as deviation 3 (see disposition 3). No action; CW-2 journal should not
   replicate the mis-filing.

## Follow-up dispositions

No out-of-scope breakage was reported by the implementer and none was found.
Finding 1 (allowlist rows) goes to `tasks/milestone_ledger.md` deferred-findings
at the next ledger touch - recorded here per the review-gate rule that no
observation is dropped.

## Duty 1 for the CW-2 implementer (exact landing commands)

The CW-2 implementer lands THIS verdict plus the execution-results addendum as
its duty-1 commit, before authoring any CW-2 Red:

```
python tools/encoding_guard.py --all
python tools/commit_scoped.py --scope tasks/extraction-consumer-rewrite-cw1-verdict.md --scope tasks/extraction-consumer-rewrite-execution-results.md -m "docs(cw-1): reviewer verdict APPROVED + execution-results addendum (extraction-consumer-rewrite CW-1)"
```

Never `git add .`; never amend `c916a26`/`f73a9a1`. CW-2 builds on
`f73a9a1`-plus-duty-1 as its start rev.
