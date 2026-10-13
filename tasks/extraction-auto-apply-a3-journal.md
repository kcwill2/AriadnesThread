# A-3 Journal - extraction-auto-apply (applier trait, apply DTOs, POST apply route, state seam)

Handoff of record: `tasks/extraction-auto-apply-a3-handoff.md`.
Shape binding inputs: `tasks/extraction-auto-apply-shape.md` sections 2
(invariants A1-A5, A9 + oracles), 3 (C2 trait/DTOs, C3 route table, C4
token semantics, C5 mutex + epoch model), 5 (module map A-3 row and the
declared rider), 8 (hygiene); kickoff sections 3 (items 1-3) and 6.
Start rev: `eb2bd91` (the post-duty-2 commit; duty 1 landed the A-2
verdict as `e2dd2c2`, duty 2 landed the A-2 journal appendix as `eb2bd91`
in this session). Branch `main`. Implementer hop 2026-10-13.

## Boot verification

- Duty 1 sequence, verbatim: `python tools/encoding_guard.py --staged`
  silent exit 0 (the index was empty at guard time - the verdict file was
  untracked; its byte-level coverage lands in this hand's gate 9 `--all`
  run, which scans it as tracked content); `python tools/commit_scoped.py
  --scope tasks/extraction-auto-apply-a2-verdict.md -m "review(extraction-auto-apply): A-2 APPROVED verdict + gate evidence"`
  landed `e2dd2c2`, numstat `131  0  tasks/extraction-auto-apply-a2-verdict.md`.
- Duty 2 (docs-only rider discharging A-2 verdict finding 1): the two
  disclosures (O2 refused combined guard+echo command; phantom mid-session
  narration transparency note) appended to the A-2 journal's Deviations
  section as a clearly-marked appendix rider, no other journal content
  touched. `python tools/encoding_guard.py --staged` silent exit 0 (same
  empty-index vacuity, same `--all` coverage discharge), then
  `commit_scoped.py --scope tasks/extraction-auto-apply-a2-journal.md`
  landed `eb2bd91`, numstat `36  0  tasks/extraction-auto-apply-a2-journal.md`.
- Post-duty-2 boot check: `python tools/distill.py -- git status
  --porcelain` -> `### generic PASS (exit 0)`, capture 0 lines - tree
  fully clean, so `crates/**` is clean and HEAD is the post-duty-2 commit.
  No foreign dirt; no STOP.

## F14 caller census at preflight (start rev `eb2bd91`)

`ExtractionDryRunDto {` struct-literal census (grep over the tree, the
handoff's mandated preflight re-run of the planner's census at `023558c`):
exactly FOUR sites - the struct definition
(`crates/ui/src/dto/extraction.rs:131`), the two cli construction sites
(`crates/cli/src/commands/ui/extraction_reader.rs:182` `compute_dry_run`,
`:417` `honest_absent_dry_run`), and the wizard test sample
(`crates/ui/tests/ui_test/extraction_wizard_api.rs:100`). No fifth site -
no STOP. The declared 2-line rider covers the two cli sites; the C8
amendment 4 line covers the test sample.

## Gate log

Canonical lines quoted verbatim; captures at
`target/testrun-capture.txt` / `target/distill-capture.txt` (path printed
by each distilling entry point).

- Gate 1 (ui baseline, `python tools/testrun.py --distill -p ariadne-ui`
  at the clean start rev): `### cargo test PASS (exit 0)`, canonical line
  `312 passed / 0 failed / 0 ignored / 3 suites`. The immovable bar for
  gate 3 (expected: +15 passed = the 15 new test functions, suites 3 held,
  zero movement elsewhere).

## Red evidence (gate 2)

RGB order per the handoff: the compile-complete skeleton landed FIRST
(trait, refusal enum, all DTOs, state field + install + accessor, POST
registration, handler whose only implemented branch is the no-applier
503, every other request answered by the `dry_run_required` placeholder),
then the test file. The prerequisites - the `main.rs` mod line, the C8
amendment 4 line and the two rider lines - are compile prerequisites, not
part of the Red. A pre-Red `python tools/distill.py -- cargo check -p
ariadne-ui --all-targets` confirmed the state compiles (first two
invocations surfaced two test-harness type errors: `Arc::clone` in
argument/binding position cannot infer the unsizing coercion to
`Arc<dyn ExtractionPlanApplier>` - fixed with the method-call form plus an
explicitly-typed binding; no source defect, harness-only), final
invocation: `ok (exit 0, 0 warnings emitted)`.

Gate 2 (`python tools/testrun.py --distill -p ariadne-ui`):
`### cargo test FAIL (exit 1)`, canonical line
`321 passed / 6 failed / 0 ignored / 2 suites`. The totals are exact:
321 + 6 = 327 = baseline 312 + the 15 new tests. The `2 suites` shape is
the digest reflecting cargo's fail-fast abort: the lib binary ran first
(0 tests), the `ui_test` binary failed, and the one remaining target did
not run (same digest phenomenon as the A-2 gate-3 record; gate 3 restores
the 3-suite shape). Exactly the six named failing tests, verbatim panic
messages (capture `target/testrun-capture.txt`, 406 lines, no other
failure anywhere):

- `test_apply_route_returns_200_with_a_valid_token ... FAILED` -
  `assertion `left == right` failed: expected status 200 for an approved
  apply, got 428`
- `test_apply_route_returns_404_for_a_plan_the_applier_does_not_know` -
  `expected 404 for a plan the applier does not know, got 428`
- `test_apply_route_returns_409_stale_generation_for_a_stale_applier` -
  `expected 409 for an applier whose epoch differs from the live epoch,
  got 428`
- `test_apply_route_returns_409_apply_in_progress_while_the_lock_is_held` -
  `expected 409 apply_in_progress, got 428`
- `test_apply_route_returns_500_apply_failed_on_a_transaction_error` -
  `expected 500 for a transaction error, got 428`
- `test_dry_run_route_mints_an_apply_token_with_a_live_applier` -
  `assertion `left == right` failed: the dry-run 200 body must carry the
  minted apply_token`

The five token-refusal tests and the 503 test (required tests 1-6) pass
at Red by construction - the placeholder IS the 428 refusal - which is
the handoff's documented order, not a weak Red. The `extraction_wizard_api`
suite is untouched (apart from its C8 sample line) and stayed green
throughout: its dry-run 200 test installs no applier, the field stays
empty, and its byte-equality assertion holds against the amended sample.

## Green evidence (gate 3)

Full handler per the C3 check order (`applier_ready_for_request`: no
applier -> 503, epoch mismatch -> 409 with the live epoch), the tolerant
body parse (`axum::body::Bytes` + `serde_json::from_slice::<
ExtractionApplyRequestDto>`; absent/unparseable/empty-token -> the 428
`dry_run_required` refusal), the `tokio::task::spawn_blocking` wrap with
the join mapped (`Ok(Ok(..))` -> 200 DTO; `Ok(Err(refusal))` -> the
refusal table; `Err(join)` -> the 500 `apply_failed` row - a stopped
blocking task never commits), and the C4 mint point
(`approved_apply_token` in `routes/api_extraction.rs`: live-epoch applier
only; without an applier or on epoch mismatch the field stays empty; the
manifest route, the GET check order and every GET error body untouched).
Every refusal variant is mapped, including the defensive
`StaleGeneration -> 409` second line behind the route's own epoch check.
Token semantics for the route: it never consumes or validates a token
itself - it mints by delegation and applies by one `applier.apply(&
plan_id, &token)` call per request (consumption at transaction start and
lock-refusal non-consumption are the A-4 provider's contract, C4/C5).

- Gate 3 (`python tools/testrun.py --distill -p ariadne-ui`):
  `### cargo test PASS (exit 0)`; canonical line
  `327 passed / 0 failed / 0 ignored / 3 suites` = baseline + exactly the
  15 new tests, suites 3 restored (the Red's 2-suite shape was cargo's
  fail-fast abort), no other movement. The frozen
  `extraction_wizard_api.rs` locks (GET-only + two no-write) and the
  amended suite ran inside this line, green, with their assertions
  unedited.
- Gate 4 (`python tools/distill.py -- cargo check -p ariadne-cli
  --all-targets`): `ok (exit 0, 0 warnings emitted)` - the declared
  2-line rider compiles the first reverse dependency (F14 rule 2).
- Gate 5 (`python tools/distill.py -- cargo check
  -p ariadne-integration-tests --all-targets`): `ok (exit 0, 21 warnings
  emitted)` - exit 0 carries it; all 21 warnings sit in files this hand
  does not touch (the pre-existing `unused variable gen` in
  `crates/ui/src/routes/api_graph.rs:719`, the dead-code helper family in
  `tests/integration/tests/common/mod.rs`, and the dated-deprecated
  `IncludeResolver` oracle uses in `m18_cpp_plugin_parity`) - the
  baseline population, zero warnings in any touched file.
- Gate 6 (`python tools/hygiene_check.py` via the distiller for the exit
  digest): `### generic PASS (exit 0)`. R1/R2 zero; every R3 row is a
  pre-existing warn-only population in files outside this hand (the new
  files contribute ZERO rows).

## Refactor (measured at HEAD-of-work)

One helper per error body (`analysis_loading_response`,
`stale_generation_response`, `unknown_plan_response`,
`dry_run_required_response`, `apply_in_progress_response`,
`apply_failed_response`, `error_response`); the handler is 33 lines; the
module is 186 lines (band ~150-200, far under the 1000 ceiling). File
sizes after the hand: `dto/extraction.rs` 166 -> 228; `state.rs`
241 -> 267; `server.rs` 84 -> 86; `lib.rs` 29 -> 30; `routes/mod.rs`
7 -> 8; `routes/api_extraction.rs` 103 -> 123 (band ~112 + the C4 mint's
doc comment); `extraction_apply_api.rs` 639 (band ~300-360 - see D4);
`main.rs` 34 -> 35; `extraction_wizard_api.rs` 371 -> 372 (C8 single
line); `extraction_reader.rs` 839 -> 841 (the 2-line rider). No file
approaches 1000; no source line near 2000 characters; all authored
content pure ASCII (verified by gate 9 `--all`).

## Lock and census checks (pre-commit)

- `git status --porcelain` (via distiller): exactly the 12 touch-set
  paths (9 modified, 2 new source/test files, 1 journal). Zero paths
  outside the touch-set.
- `git diff --stat`: `extraction_reader.rs 2 +` (the rider, no other
  change), `extraction_wizard_api.rs 1 +` (C8 amendment 4 only),
  `main.rs 1 +`, `lib.rs 1 +`, `routes/mod.rs 1 +`, `server.rs` 3+/1-,
  `state.rs +26`, `dto/extraction.rs +62`, `api_extraction.rs +22/-1`
  (the `Some(mut dry_run)` arm plus the mint helper).
- Wizard file hunk-by-hunk: the ONLY hunk adds `apply_token:
  String::new(),` inside `sample_dry_run_dto` at :114 - exactly C8
  amendment 4; the GET-only and no-write locks and all route tests are
  byte-identical (verified by the full `git diff` of the file).
- A1 single-write-path relay honored: `routes/api_extraction_apply.rs`
  performs no writes (required test 13 locks `fs::write`,
  `fs::remove_file`, `fs::create_dir`, `ArtifactManager` and the four
  mutation-verb tokens out of the module); the POST verb appears in
  `server.rs` only (required test 14: exactly one apply registration,
  carrying `post(`, with the manifest/dry-run lines verb-free); the GET
  module keeps its W1 shape across the mint (required test 15). The
  handler mints/validates by delegation only - no second write path was
  created anywhere. The A-2 relay note stands unchanged:
  `crates/cli/src/optimize.rs:406` holds the pre-existing second
  `TransactionApplier::apply_with` call site (untouched, out of scope,
  ledger property).
- `apply_plan_with(` census unchanged at 2 (definition + wrapper in
  `transform.rs`): `transform.rs` is untouched and the new ui doc
  references mention `apply_plan_with` without a call paren, so the
  census regex `apply_plan_with(` moves nowhere.
- Card-family files, assets, goldens, fixtures, tools, manifests: appear
  in no diff of this hand (touch-set check above).

## Bench stance (producer zero-diff proof, exemption with proof)

This hand touches `crates/ui/src` and (by the declared rider)
`crates/cli/src/commands/ui/extraction_reader.rs`, so the
DOCUMENTED_PHASES dated-row stance applies. `bench_compare.py` was NOT
run in this hop (the handoff assigns it to the reviewer hop; the timing
gate is deterministically blocked at `tools/bench_compare.py:84`, exit 2
before any comparison). Producer zero-diff proof over the landing range
(`eb2bd91..landing`): `git diff --numstat "eb2bd91..HEAD" --
crates/cli/tests` is recorded in the landing record (rider) as EMPTY -
the bench producer set is byte-identical across the range. What this
hand adds is a route + DTOs + state seam; the apply algorithm itself
runs only through the A-2 core (A-4 wires it), so NO timing number is
attributable to A-3 and the reviewer hop carries the exemption with this
proof.

## Deviations (numbered, with reasons; never silent)

- D1 (gate 7 staging route): identical to the A-2 D1 precedent - with an
  empty index, `encoding_guard --staged` would scan nothing and the gate
  would be vacuous. The twelve touch-set paths are staged via
  `python tools/distill.py -- git add -- <twelve explicit paths>`
  (never `.`, no glob), the guard then runs over the blobs actually
  landing, and gate 8's `commit_scoped.py` commits with its own resolved
  scopes (`--only`). Precedent: playbook T-C D2, A-2 D1 (both ACCEPTED
  in verdicts). Scope discipline unchanged.
- D2 (journal rider commit): the landing record (landing hash, post-
  commit proofs) cannot exist inside its own commit; gate 8's commit
  carries the journal as-is and a second scoped, journal-only commit
  lands the landing record. Precedents: W-1 `82e5ffe`, A-1 `7d819bb`,
  A-2 `d7df94f` (all ACCEPTED). The rider adds no source change.
- D3 (handoff name mismatch, resolved by choice): the Red-protocol
  section names four of the six expected-fail tests with identifiers
  that differ from the Required-tests section for the SAME tests
  (`test_apply_route_refuses_an_unknown_plan` vs required 8
  `test_apply_route_returns_404_for_a_plan_the_applier_does_not_know`;
  `..._refuses_a_stale_generation_applier` vs required 9
  `..._returns_409_stale_generation_for_a_stale_applier`;
  `..._refuses_while_another_apply_holds_the_lock` vs required 10
  `..._returns_409_apply_in_progress_while_the_lock_is_held`;
  `..._reports_a_failed_transaction` vs required 11
  `..._returns_500_apply_failed_on_a_transaction_error`). The two
  sections cannot both name the functions. Implemented with the
  Required-tests identifiers (that section defines the file's contents
  and the acceptance items reference tests by those numbers) with the
  Red section's six panic messages carried verbatim as the assertion
  texts; the Red evidence above maps both name sets to the same six
  tests.
- D4 (test-file size): `extraction_apply_api.rs` landed at 639 lines
  against the handoff's ~300-360 estimate. Cause: the model's explicit
  `assert_eq!`-with-message idiom priced per test, duplicated per the
  handoff's own rule that the wizard helpers may not be shared (frozen
  file), plus the two scripted doubles (~250 lines). Within the 1000-
  line ceiling and within the hand's ~1200 produced-lines budget (total
  diff ~118 source lines + 639 test lines + journal); no rule crossed.
- O1 (observation): gate 5's 21 cargo warnings are the pre-existing
  population in untouched files (see Green evidence); zero new warnings
  in touched files (gate 4's check of the ui-dependent cli crate emitted
  0 warnings).
- O2 (observation): the Red canonical line reads `2 suites` - cargo's
  fail-fast aborts remaining targets after `ui_test` fails (digest
  shape, same phenomenon as the A-2 Red record). Gate 3 restores the
  baseline 3-suite shape.
- O3 (refusal disclosure): the attempt to capture the hygiene exit code
  through a chained/piped form
  (`python tools/hygiene_check.py 2>$null | Out-Null; echo ...`) was
  refused with the catch-all denial
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`
  (boilerplate message). Per protocol the gate was obtained
  independently: the plain `python tools/hygiene_check.py` run (executed,
  output reviewed) plus `python tools/distill.py -- python tools/
  hygiene_check.py` -> `### generic PASS (exit 0)`. The refused form
  executed nothing; zero tree impact. Refusal class is the ledger's
  dated configuration row (chained/pipe forms), same class as A-2's O2.
- O4 (note, not a deviation): the duty 1 and duty 2
  `encoding_guard --staged` runs were vacuous (empty index at guard
  time, the A-2 duty-1 precedent note applies verbatim); their byte-level
  coverage is discharged by gate 9 `--all`, which scans all tracked
  content including both landed files.

## Acceptance table (as adjudicable pre-commit)

1. Every C3 row answered with the exact body, ASCII asserted (required
   tests 1-12 + per-test ASCII asserts covering the 200 and all six
   refusal shapes = required 16) - MET (gate 3 green).
2. Dry-run mints only with a live-epoch applier; presence/emptiness
   locked (required test 12 + the wizard suite's empty-field byte
   equality) - MET.
3. `routes/api_extraction_apply.rs` no write token, no mutation
   registration (required test 13) - MET.
4. Wizard GET-only/no-write locks green with `extraction_wizard_api.rs`
   and `routes/api_extraction.rs` assertions UNEDITED (required tests
   14-15 + the untouched wizard suite inside gate 3) - MET.
5. Gates 4 and 5 exit 0 - MET.
6. `hygiene_check.py` exit 0; `encoding_guard.py --all` exit 0 (gate 9,
   recorded in the landing rider) - MET/pending gate 9.
7. Commit touch-set = exactly the twelve gate-8 files (reviewer
   commit-vs-touch-set enforcement) - landing rider records `git show
   --stat`.

## Landing record (journal rider, D2)

- Landing commit: `f93e51d` (parent `eb2bd91`), message verbatim from
  gate 8. `commit_scoped.py` numstat digest: exactly the twelve touch-set
  paths - `api_extraction_apply.rs 186+`, `extraction_apply_api.rs
  639+`, `dto/extraction.rs 62+`, `state.rs 26+`, `api_extraction.rs
  21+/1-`, this journal `298+`, `server.rs 3+/1-`,
  `extraction_reader.rs 2+`, `lib.rs 1+`, `routes/mod.rs 1+`, `main.rs
  1+`, `extraction_wizard_api.rs 1+`. Nothing else.
- Gate 7 (with the D1 staging route): the twelve paths staged via
  `python tools/distill.py -- git add -- <twelve explicit paths>`
  (`### generic PASS (exit 0)`; git's informational LF/CRLF warnings on
  the working copies only - the staged blobs are what the guard
  scanned), then `python tools/encoding_guard.py --staged` silent exit 0.
- Gate 9: `python tools/encoding_guard.py --all` silent exit 0
  post-commit. This also discharges the byte-level coverage of duty 1's
  verdict file and duty 2's amended A-2 journal (O4).
- Range proof: `git diff --name-only "eb2bd91..HEAD"` = exactly the
  twelve touch-set paths. `crates/ui/assets/**`, the card-family files,
  `extraction_wizard_render.rs`, `transform.rs`, `commands/ui.rs`, every
  other `crates/cli/**` file, `tests/**`, `tools/**`, `Cargo.toml` and
  `Cargo.lock` appear in NO hunk of the landing range.
- Producer zero-diff proof (bench stance): `git diff --numstat
  "eb2bd91..HEAD" -- crates/cli/tests` returned EMPTY (capture 0 lines)
  - the bench producer set is byte-identical across the landing range.
  Combined with the DOCUMENTED_PHASES deterministic blocker at
  `tools/bench_compare.py:84` (exit 2 before any timing comparison;
  `bench_compare.py` not run in this hop), NO timing number is
  attributable to A-3; the reviewer hop carries the exemption with this
  proof.
- Post-landing tree: `git status --porcelain` shows only this journal as
  modified (the rider's own content) - no other dirt.
- Acceptance 6 completion: `encoding_guard --all` exit 0 - MET.
- Acceptance 7: the numstat digest above is the touch-set record for the
  reviewer's commit-vs-touch-set check - MET.
