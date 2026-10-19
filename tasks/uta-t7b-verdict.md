# Verdict - UTA-T7b (H7b): cli dispatch/progress/query/registry behavior pins

**APPROVED** — commits `83a64c2`, `0d07532`, `47c893e`, `8563a5e`, `3184122`, `2a8e9dc`
on base `ff47047` (after T7a). Reviewer session fresh-verified everything measurable.

Canonical fresh gate line (reviewer's own run, `python tools/testrun.py --distill -- -p ariadne-cli`):

```
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 254 lines]
177 passed / 0 failed / 8 ignored / 8 suites
```

Matches the expected 177/0/8/8 exactly. Arithmetic independent check: T7a baseline 147
+ exactly 30 new pins (5 dispatch route/bail + 1 collateral + 3 progress + 1 panic +
2 plan + 2 registry + 5 query + 3 includes + 2 index + 4 ui row-9 + 1 failed-transaction)
= 177; ignores (pre-existing benches) and suite count unchanged.

## 1. Ground truth + per-commit review (oldest-first)

- `checkpoint_guard.py status`: HEAD `2a8e9dc`, branch main, dirty tree = 9 untracked
  reviewer artifacts only (verdicts/results from H1-H7a + H6 resume doc). Worktree
  census carries only the pre-existing pool/scratch worktrees.
- `git log --oneline ff47047..HEAD` = the six declared commits, each reviewed via
  `git show` file-by-file:
  - `83a64c2` tier-1 pins + route seams: reviewed in full (see 3/5/6). Pure additive
    pins in progress.rs/registry.rs/boot_panic_tests.rs/ui tests.rs; lib.rs gains the
    three route enums and its inline `mod tests` becomes `mod cli_command_tests;`.
  - `0d07532` row 4: pure addition (129 lines, 0 deletions) inside existing
    `plan.rs mod tests`; both pins genuine equality asserts (see 6).
  - `47c893e` rows 7/8/12: render/resolve/selection seams reviewed line-by-line
    against the old bodies — byte-faithful (see 6); pin modules pure additions.
  - `8563a5e` row 9: the adjudicated seam relocation (see 4); plus one-line journaled
    stale-`mut` cleanup in cli_command_tests.rs.
  - `3184122` rows 10/11: extraction_apply test-module move + 2 pins (see 3);
    extraction_reader.rs pin pure addition.
  - `2a8e9dc` closeout: query.rs loop-binding renames only (`r`→`artifact`,
    `u`→`user_file`, three loops, zero logic lines) + journal close.

## 2. Commit-vs-touch-set

`git diff --name-only ff47047..HEAD` -> 17 paths: 16 under `crates/cli/src/**` +
`tasks/uta-t7b-journal.md`. All inside the declared touch-set. **The MUST NOT check:
`optimize.rs`, `transform.rs`, `measure.rs`, `discover.rs` appear NOWHERE in the
range** — neither src logic nor test modules. No Cargo.toml/lock, no other crate,
no `crates/cli/tests/**` edits. PASS.

## 3. THE BIG MOVES — byte-level verbatim verification (reviewer scratch scripts, run via distill)

**(a) lib.rs 834 -> 550 + new `cli_command_tests.rs` (603 lines).** Extracted the
inline `mod tests` block from `ff47047:crates/cli/src/lib.rs` programmatically and
difflib-compared it (dedented) against `HEAD:.../cli_command_tests.rs`: exactly
**5 non-equal hunks** —
1. new `//!` module header (5 lines);
2. `test_handle_measure` body: row-13 strengthening (real field asserts:
   `tu_measurements.is_empty()`, `affected_tus_count == Some(0)`,
   `collateral_load == None`, `repo_root.ends_with("repo")`);
3. `test_handle_analyze` body: row-13 strengthening — the tautology
   `total_tus > 0 || total_tus == 0` replaced by `total_tus == 2` +
   `reused + reanalyzed == total` + `tu_analyses.len() == total_tus`;
4. one-line stale `mut` removal on `root_command` (journaled, pre-existing warning);
5. appended row-6 route pins (helpers + 5 pin fns).
Every other moved test body is line-identical. **No silent assertion edits in
transport.** fn-name superset: lost = NONE; added = the 5 row-6 pins + 3 helpers.

**(b) extraction_apply.rs 999 -> 295 + new `ui/extraction_apply/tests.rs` (780 lines).**
Raw-vs-raw difflib of the inline block @ `8563a5e` vs `tests.rs` @ HEAD: exactly
**4 non-equal hunks** — new module header; `state` -> `_state` (pre-existing
unused-warning, journaled, assertion untouched); the two `include_str!` paths each
gaining exactly one `../` (correct relative re-derivation: `../../transform.rs` and
`../extraction_apply.rs` resolve to the same two files as before) with an explanatory
comment; the appended row-11 pin `pin_failed_transaction_burns_the_token_and_reports_failed_text`.
The moved body retained its original 4-space indentation verbatim (valid Rust).
Production region byte-identity: lines 1-294 of the file are identical between
pre-move and HEAD (`mod tests {` -> `mod tests;` is the only non-moved change), and
the pre-first-`#[cfg(test)]` region is identical back to `ff47047`. Lost names: NONE.

**(c) Item 13 authorization check:** the two row-13 tests are the ONLY test bodies
whose asserted content changed; both strengthenings are real behavior asserts (verified
above), satisfying the handoff's "strengthen with real asserts" branch. The
`mut`/`_state`/`include_str!` edits are transport/mechanical, journaled, and touch no
assertion — adjudicated on merits: accepted, not content edits.

## 4. Row 9 — ratchet-forced seam relocation ui.rs -> ui/boot_progress.rs

- The frozen ratchet test `test_ui_rs_is_under_the_extraction_ceiling` is
  **byte-identical base vs HEAD** (reviewer extracted the fn block from both blobs);
  it asserts `include_str!("../ui.rs").lines().count() <= 880` and passes inside the
  green 177 run. ui.rs at HEAD = **842 lines** (877 -> 842), i.e. the seam could not
  have lived in ui.rs without violating a frozen lock the handoff forbids touching.
- Behavior of the moved seam: `drive_stage_frames` in boot_progress.rs is the old
  `run_pipeline_stages` emission body — nine `stage_report` calls with identical
  stage/percent/phase/message arguments and identical interleaving of the five stage
  calls, `?`-short-circuit preserved per call, A4 comment verbatim;
  `run_pipeline_stages` is now a thin adapter mapping StageCall to the same five
  pipeline calls in the same order. The expose split kept the warning bytes byte-exact
  and the single emitter in ui.rs (S6 grep literals stay in ui.rs production).
  `boot_progress.rs` is the sanctioned boot-family target (precedent W-2a F2 +
  stage_spans). Journal reported the relocation BEFORE commit, per the frozen-locks
  clause. **D4 ACCEPTED.**
- Row-9 test refactor satisfies the directed "REAL function replayed": the hand-written
  frame copies are deleted; `seam_stage_frames()` drives the real `drive_stage_frames`
  once and both `stage_one_to_four_frames`/`stage_five_calibrated_frames` derive from
  it (the 13-globals pinned list itself unchanged); two new pins assert the nine-frame
  shape, five-call order, model passthrough, and per-stage short-circuit semantics.

## 5. Pin honesty (independent re-derivation)

- **D1 `pin_progress_overshoot_unclamped`: ACCEPTED.** The handoff row 2 itself reads
  "(if unclamped today, pin exactly + report)". The green byte-pin proves unclamped
  (150%, 30 filled cells for 15/10, and the 20-cell full bar at 0/0). Pinning current
  behavior with an honest name IS the contract-correct response to unadjudicated
  behavior; naming it `..._clamped` would have been the lie.
- **D2 registry case matrix: ACCEPTED.** `pin_parse_language_arg_exact_accepted_set_not_case_insensitive`
  names CPP and sCaLa outcomes explicitly (both `is_err()`), pins the exact accepted
  set (lowercase + exact PascalCase) — exactly what row 5 demanded. registry.rs diff is
  **0 removed/changed lines** (pure addition): the legacy misnamed
  `..._case_insensitively` test is untouched and its assertions are true; leaving it is
  contract-acceptable (row 5 governs the new pin's name, not the legacy one), routed to
  a closeout rename candidate (finding 3). plan_backend honest-error pair pinned with
  target+stage naming.
- **D3 `pin_subrepo_positional_precedence_silent`: ACCEPTED.** Name shipped verbatim as
  mandated. Reviewer verified the shape asymmetry against the seam code: analyze
  compares `repo_path != load_repo`; includes/plan resolve via wildcard arms that drop
  the positional un-compared; all three lanes silently favor the sub-repo; the route
  pins and the verbatim-bail pin (3 frozen strings, nothing executed) cover all four
  arms per dispatcher. Asymmetry pinned as-is + REPORTED, not "fixed" — per ALL-PINS.
- Row 12 selection pins pass (0,0) and (15,10) through unnormalized — matches the old
  if-let's pass-through exactly; `print_progress(` count in index.rs == 1 (lock holds).

## 6. Seam fidelity (rows 7/8/12) + plan-literal checks

query.rs: not-found text, section scaffolding, indentation (4-space report vs 2-space
list), blank separators, unmapped-FileId Display fallback, and lookup ORDER (file_map
built only after a hit — no new `ArtifactManager::init` side effect on a miss) are all
line-for-line preserved; deps-by-name/dangling-by-raw-id unchanged. includes.rs:
lookup -> fallback-normalize -> registration-check -> verbatim bail, emission order
untouched; pins prove the backslash spelling hashes as the normalized form AND provably
not as raw, plus a constructed crossed fixture pinning the fallback-wins arm. index.rs:
the callback stays the sole `print_progress` site with the verbatim message.
plan.rs row-4: empty-edges == colocated_groups byte-equality pinned NON-vacuously
(non-empty expected group asserted), per-language filtering pinned with a mixed
scala/cpp fixture incl. cross-language edge alone colocating nothing.

## 7. Frozen grep-locks

No grep-lock test was edited in the range (the only edited source-text-assert test is
the moved applier-delegation lock — its assertions are byte-identical, only the two
include_str! paths fixed; the needle stays constructed via `format!`, so the lock text
survived the move). At HEAD: `apply_plan_with(` count == **2 in transform.rs** (lock
value) and 1 in extraction_apply.rs (production site); `print_progress(` count == 1 in
index.rs; the S6 warning/wildcard lock, first-frame lock, payload `[k/5]` locks, and
boot-family lock all still read ui.rs/ui production sections that retain their locked
text and pass inside the green 177. **PASS.**

## 8. Hygiene / encoding / report evidence

- `python tools/hygiene_check.py` -> direct run lists R3 warn-only lines; via distill:
  `### generic PASS (exit 0)`. R1/R2: none. Provenance check of every R3 warn inside
  touched files (boot_progress.rs `bar`/`p`, boot_progress_tests.rs `p`, ui.rs `p` x2):
  each warn line's content exists verbatim in the `ff47047` blob -> all pre-existing;
  zero R3 hits on lines added by this hand (the journal's claim, verified).
- `python tools/encoding_guard.py --all` -> `### generic PASS (exit 0)`, no output.
  Additionally: zero non-ASCII bytes in any added line across the whole range
  (byte-level diff scan).
- Tools CI suite: `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 232 tests in 20.085s` / `OK`.
- Journal is digest-shaped: per-row canonical lines + red/green counts, zero raw
  dumps; gates via distilling entry points. Capture paths absent from the journal
  record (finding 2) — closed by this review's capture headers.

## 9. Findings (numbered)

1. MINOR — `tasks/uta-t7b-journal.md:18`: planned-seam note says extraction_apply.rs
   is "(998 after the move)"; the move lands it at **295** lines (the row-11
   checkpoint states 999 -> 295 correctly). Stale planning note; journal self-
   inconsistency, no action required beyond closeout accuracy.
2. MINOR — journal gate record omits capture paths (handoff "Report back" asks for
   per-row distill header + capture path; canonical lines only). Same class as H6
   finding 1; closed for this hand by the reviewer's fresh capture headers.
3. MINOR — the legacy `parse_language_arg_accepts_serde_names_case_insensitively`
   NAME still overclaims (left untouched per the honest-matrix deviation; assertions
   true). Route the rename to closeout deferred-findings; not a blocker for this hand.
4. OBSERVATION (no action) — `drive_stage_frames` carries
   `unreachable!("Reduced stage must return the model")`: an internal contract only a
   misbehaving test advance could trip; the production adapter cannot. Noted for the
   record.
5. OBSERVATION (no action, positive) — the index seam replaced an if-let fall-through
   with an exhaustive match: today identical behavior, but future ProgressEvent variants
   now force a deliberate decision at the seam instead of silently skipping the bar.

## 10. Deviation dispositions

D1 overshoot-unclamped pin name — **ACCEPTED**. D2 case-strict matrix, legacy test
untouched — **ACCEPTED**. D3 subrepo precedence asymmetry pinned as-is + reported —
**ACCEPTED**. D4 row-9 seam lives in ui/boot_progress.rs under the frozen ratchet —
**ACCEPTED**. D5 lib.rs test module extracted to cli_command_tests.rs (834 -> 550,
ceiling-forced) — **ACCEPTED** (verbatim verified). D6 extraction_apply inline tests
moved to tests.rs (ceiling-forced; indent-preserving raw move) — **ACCEPTED**
(verbatim verified; production byte-identical). D7 row-13 strengthened with real
asserts instead of a journal excuse — **ACCEPTED**. D8 hygiene renames inside touched
bodies (`r`/`w`/`u` -> descriptive) — **ACCEPTED** (pure renames).

## 11. Not run / refusals

Not run (per-hand precedent; owed at UTA closeout): `testrun.py --baseline`,
`ui_smoke.py`, `bench_compare.py`; `encoding_guard --staged` vacuous (clean tree,
superseded by `--all`); `golden_guard.py` not applicable (prose lock clause, enforced
via the byte-identity audits above).

Refusals during this review — boilerplate
`{"permission":"bash","pattern":"*","action":"deny","source":"global"}` hit five
shapes: `unittest discover ... 2>&1 | Select-Object -Last 3` (reran plain, OK),
`git show ... | Select-String` (replaced by byte-level blob script per AGENTS.md),
`git blame` chained call, `Get-Content | Measure-Object` (used the Read tool), and
direct `python tasks/<script>.py` + `python -c` (all script probes reran via
`python tools/distill.py -- python tasks/...`, exit codes in the digest). Every
expected gate obtained; no gate skipped.

## 12. Reviewer artifacts

`tasks/uta-t7b-verdict.md` (this file) + the H7b/ALL-HANDS addendum in
`tasks/uta-execution-results.md` LEFT UNTRACKED (reviewer git-write denied;
T1-T7a precedent). Scratch probes `tasks/t7b-review-verify.py`,
`tasks/t7b-verify-probe2.py`, `tasks/t7b-verify-report.txt` left for closeout
cleanup.
