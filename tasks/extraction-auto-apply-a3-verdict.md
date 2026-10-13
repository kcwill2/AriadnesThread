# A-3 Verdict - extraction-auto-apply (applier trait, apply DTOs, POST apply route, state seam)

Handoff of record: `tasks/extraction-auto-apply-a3-handoff.md`.
Journal: `tasks/extraction-auto-apply-a3-journal.md`.
Review range: `eb2bd91..ea66151` (commits `f93e51d`, `ea66151`).
HEAD at adjudication: `ea66151` on `main`, worktree clean, no untracked files
(`python tools/checkpoint_guard.py status`).
Scope inputs read: shape sections 2, 3 (C2/C3/C4/C5), 5, 8; kickoff sections
3 and 6; the handoff; the journal; the full diff of every commit in range.

## Verdict: APPROVED

One minor finding (documentation completeness, no source remediation), four
accepted deviations. No blocker. Every acceptance criterion is met with
fresh evidence below.

## Fresh gate results (my hop, quoted verbatim)

| Gate | Command | Result |
|---|---|---|
| Workspace census | `python tools/testrun.py --distill` | `### cargo test PASS (exit 0)`; `1502 passed / 0 failed / 13 ignored / 112 suites`; `no failures`; capture `target/testrun-capture.txt` (2811 lines) |
| Scoped ui suite | `python tools/testrun.py --distill -p ariadne-ui` | `### cargo test PASS (exit 0)`; `327 passed / 0 failed / 0 ignored / 3 suites`; `no failures` |
| Tool unit suites | `python -m unittest discover -s tools -p "*_tests.py"` | `Ran 219 tests in 23.453s` / `OK` |
| UI runtime smoke | `python tools/distill.py -- python tools/ui_smoke.py` | `### generic PASS (exit 0)`; `boot+route checks clean (cold fixture, attempt 1, 1.0s)`; `ui_smoke: PASS` |
| Load-perf bench | `python tools/distill.py -- python tools/bench_compare.py` | `### generic FAIL (exit 2)`; `ERROR parse format drift: documented phase row(s) missing from bench output (never substituting zeros)` - the deterministic `DOCUMENTED_PHASES` blocker (`tools/bench_compare.py:75-90`, line 84 inside the tuple) |
| Theme drift | `python tools/distill.py -- python tools/theme_token_check.py --check-added --base eb2bd91` | `### generic PASS (exit 0)`, capture 0 lines (silent) |
| Hygiene | `python tools/hygiene_check.py` / via distiller | `### generic PASS (exit 0)`; R1/R2 zero; 179 R3 warn-only rows, zero of them in any A-3 touched path |
| Encoding | `python tools/encoding_guard.py --all` / via distiller | `### generic PASS (exit 0)`, capture 0 lines (silent) |
| Producer zero-diff (bench proof) | `git diff --numstat "eb2bd91..ea66151" -- crates/cli/tests` | EMPTY (no output) |

Census reconciliation against the recorded milestone line
`1487 passed / 0 failed / 13 ignored / 112 suites`: the fresh line is exactly
+15 (`1502 passed`), 0 failed, `13 ignored` and `112 suites` held with zero
movement. The 15 new tests are the 15 test functions in
`crates/ui/tests/ui_test/extraction_apply_api.rs` (12 route tests plus the 3
string locks). The implementer's Green claim (`327 passed / 0 failed /
0 ignored / 3 suites` for `-p ariadne-ui`) reproduces exactly, and its baseline
312 + 15 = 327 arithmetic is internally consistent.

### Gate execution notes and refusals (recorded per reviewer protocol)

- `python -m unittest discover -s tools -p "*_tests.py" 2>&1 | Select-Object
  -Last 5` was refused with the catch-all denial
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`
  (boilerplate message; the piped form is the dated configuration row). The
  plain command was then run and passed: `Ran 219 tests` / `OK`.
- `python tools/ui_smoke.py` direct was refused with the same catch-all
  denial; it was obtained through the approved distilling entry point
  (`python tools/distill.py -- python tools/ui_smoke.py`), which is the route
  the task assigned for exactly this case.
- `python tools/testrun.py --distill --baseline` failed with `usage error:
  missing value for --baseline` (an argument-parsing error, not a permission
  refusal); `--baseline` requires a path. The full census was then run
  unqualified, which is the form the task assigned.
- `git grep -n ... <rev>` was refused twice with the catch-all denial (both
  the quoted-revision and the bare `HEAD` form). The censuses it was needed
  for were re-established with the repo search tool over the clean worktree at
  `ea66151`, whose content is HEAD byte-for-byte (`git status` clean, no
  untracked files); equivalence noted where each census is cited below.
- Two `git show ... > <tempfile>` commands were refused by the same
  catch-all rule. They were not needed: `tasks/extraction-auto-apply-shape.md`
  is present in the worktree and provably unchanged across the range
  (`git diff "eb2bd91..ea66151" -- tasks/extraction-auto-apply-shape.md` and
  the kickoff doc both empty).
- No gate I was expected to run was skipped. Nothing outside my edit grant was
  modified; this verdict is the single artifact written.

## Per-commit review (oldest first)

### `f93e51d` - feat(ui): POST /api/extraction/{plan_id}/apply ... (parent `eb2bd91`)

Twelve paths, 1241 insertions, 2 deletions. Reviewed as if standalone:

- The new module is the plan-literal twin of the read-only seam. The trait
  `ExtractionPlanApplier` (lines 39-50), the refusal enum (54-60) and the
  C3 table body shapes match shape C2/C3 verbatim, including the inline
  comments on `NotApproved`, `InProgress` and `Failed`. The one addition
  beyond the shape snippet is `#[derive(Debug)]` on the enum: additive,
  behavior-neutral, and consistent with the DTO derives.
- The handler (62-90) is 33 lines. Check order: `applier_ready_for_request`
  (no applier -> 503 `analysis_loading`; epoch mismatch -> 409
  `stale_generation` with the live epoch) then the tolerant body parse
  (absent / unparseable / empty token -> 428 `dry_run_required`) then the
  trait answer. `axum::body::Bytes` plus `serde_json::from_slice::<
  ExtractionApplyRequestDto>` means no `Json` extractor ever runs, so no
  extractor-induced 4xx is possible on the malformed-body rows.
- The apply runs inside `tokio::task::spawn_blocking` with the join mapped:
  `Ok(Ok(outcome))` -> 200 DTO, `Ok(Err(refusal))` -> the refusal table,
  `Err(join_failure)` -> the 500 `apply_failed` row with a detail the route
  builds itself. A stopped blocking task is reported as a failed transaction
  rather than a success, which is the honest reading of C3.
- Every refusal variant is mapped, including the defensive
  `StaleGeneration -> 409 stale_generation` second line behind the route's own
  epoch check (line 128). The variant is constructible by the A-4 provider and
  cannot fire through this route.
- `server.rs` gains exactly one route line
  (`.route("/api/extraction/:plan_id/apply", post(api_extraction_apply))`)
  immediately after the dry-run line, `post` added to the `axum::routing`
  import, and the handler imported on its own `use` line. The manifest and
  dry-run registration lines are untouched and still register `get(` only.
- `state.rs` gains the `extraction_applier` RwLock field, `install_extraction_applier`
  and the accessor; both `new_loading` and `new_loaded` start the field at
  `None` and keep their signatures, so no existing call site changes.
- `dto/extraction.rs` gains the three DTOs and the single `apply_token` field,
  every field `#[serde(default)]`, all serde-plain (String / u64 / bool /
  Vec<String>): no `ariadne_validate`, `ariadne_transform` or `ariadne_cluster`
  type crosses into the ui crate. Field names and order match the shape
  snippet literally.
- `routes/api_extraction.rs` gains the C4 mint only: after a `Some` dry-run
  answer, `approved_apply_token` fetches the applier, requires
  `applier.epoch() == state.analysis_epoch()`, and otherwise leaves the field
  empty. The manifest route, the GET check order and every GET error body are
  unchanged (the diff is confined to the `Some(mut dry_run)` arm plus the new
  helper).
- The two-line rider in `crates/cli/src/commands/ui/extraction_reader.rs`
  (`:182` `compute_dry_run`, `:418` `honest_absent_dry_run`) and the C8
  amendment 4 line are the only cross-crate and frozen-file edits.

### `ea66151` - docs(tasks): A-3 journal rider (landing record)

Journal only, 36 insertions, zero source. Inside the touch-set. This is the
D2 rider precedent (W-1 `82e5ffe`, A-1 `7d819bb`, A-2 `d7df94f`): the landing
record cannot exist inside its own commit. Non-blocking.

## Commit-vs-touch-set

`git diff --name-only "eb2bd91..ea66151"` yields exactly twelve paths and
`git diff --numstat "eb2bd91..ea66151"` matches them one-for-one:

```
 2  0  crates/cli/src/commands/ui/extraction_reader.rs
62  0  crates/ui/src/dto/extraction.rs
 1  0  crates/ui/src/lib.rs
21  1  crates/ui/src/routes/api_extraction.rs
186 0  crates/ui/src/routes/api_extraction_apply.rs
 1  0  crates/ui/src/routes/mod.rs
 3  1  crates/ui/src/server.rs
26  0  crates/ui/src/state.rs
639 0  crates/ui/tests/ui_test/extraction_apply_api.rs
 1  0  crates/ui/tests/ui_test/extraction_wizard_api.rs
 1  0  crates/ui/tests/ui_test/main.rs
334 0  tasks/extraction-auto-apply-a3-journal.md
```

All twelve are declared in the handoff's touch-set; nothing outside it is
touched in either commit. Independently confirmed empty across the range:
`tests/golden/**`, `tests/fixtures/**`, `crates/ui/assets/**`, `tools/**`,
`Cargo.toml`, `Cargo.lock`, `crates/cli/src/commands/ui/optimize.rs`,
`crates/cli/src/commands/transform.rs`, `crates/cli/src/commands/ui.rs`, the
planning docs, and every frozen wizard/render/static/proposals/export/dto-builder
theme/contrast/preview/icon/asset/help-center file named in the handoff's lock
list. A8 holds: `resolve_bind_addr` and `--expose` appear in no `crates/ui`
source at all, and `commands/ui.rs` is untouched.

## Adjudications against the nine specific questions

1. **Commit-vs-touch-set.** Pass, see above. `f93e51d` lands the twelve
   declared paths (journal at 298 lines there, plus the 36-line rider in
   `ea66151`); `ea66151` is journal-only. No over-broad touch-set: no commit
   removes the check for anything else.
2. **C8 amendment 4 in `extraction_wizard_api.rs`.** The single hunk is
   `@@ -111,6 +111,7 @@ fn sample_dry_run_dto() -> ExtractionDryRunDto {` adding exactly
   `apply_token: String::new(),` after `includes_unavailable: false,` (now line
   114). Nothing else in the 372-line file moved. The GET-only server lock
   (`:326-345`) and the two no-write module locks (`:347-361`, `:363-372`) are
   byte-identical, and they are green with `routes/api_extraction.rs` carrying
   the new mint code.
3. **F14 census.** Reconciled and accepted. Shape section 8 R5 says
   "exactly 3 construction sites at HEAD" (`extraction_reader.rs:182`,
   `:417`, `extraction_wizard_api.rs:100`); the handoff says "exactly FOUR
   sites" and counts the struct definition (`dto/extraction.rs:131`) as the
   fourth. The two statements agree on substance; they differ only on whether
   the definition counts. The implementer's "4 struct-literal sites verified at
   preflight" is the handoff's number and is correct for `eb2bd91`. I
   re-established the census at HEAD with the repo search tool (clean worktree
   at `ea66151`): the construction sites are exactly the three the planner
   named, plus the new one this hand authors in its own new test file
   (`extraction_apply_api.rs:212`). See finding 1 for that last delta.
4. **Refusal-table order and semantics.** The code order is no-applier 503 ->
   epoch 409 with the live epoch -> tolerant parse -> trait answer, exactly C3
   and exactly the GET family's `reader_ready_for_request` shape. 428 keeps
   its single meaning (a valid single-use approval token is missing) and 409
   keeps its single meaning (stale generation); `apply_in_progress` is 409 with
   a distinct body, which is what C3 pins. No extractor-induced 4xx exists on
   any malformed-body row (test 3 posts `this is not json {{` and gets 428).
   The join-failure row maps to 500 `apply_failed`, not to a 200.
5. **Token mint.** Delegation-only. `approved_apply_token` mints only when an
   applier is installed and `applier.epoch() == state.analysis_epoch()`, and it
   is called only on the `Some` dry-run arm. There is no epoch bypass, no
   mint inside the apply route, and no local mint fallback. I searched all of
   `crates/ui/src` for `apply_token`, `apply_with` and `TransactionApplier`: the
   only hits are the DTO field (x2), the emptiness check in the request parse,
   the mint assignment, the trait method and one doc mention. No consumption
   site exists in this hand (the route only checks emptiness; consumption at
   transaction start is A-4's contract, C4/C5), and the route never mints.
6. **No second write path.** `routes/api_extraction_apply.rs` contains no
   `fs::write`, `fs::remove_file`, `fs::create_dir`, `ArtifactManager`, no
   `post(`/`put(`/`delete(`/`patch(`, and in fact no `std::fs` token at all.
   The only write-path addition anywhere in the range is the declared
   `install_extraction_applier` seam in `state.rs`. The relay claim holds and
   is unchanged: `crates/cli/src/optimize.rs:406` still holds the pre-existing
   second `TransactionApplier::apply_with` call site, that file is untouched,
   and `apply_plan_with(` still has exactly 2 sites (definition at
   `transform.rs:162`, the wrapper call at `transform.rs:139`).
7. **GET module W1 intact.** `routes/api_extraction.rs` keeps its frozen 503 /
   404 / 409 shapes, its check order and all its error bodies; the only diff
   is the mint arm and the helper. The three wizard lock tests
   (`test_server_registers_extraction_routes_with_get_only`,
   `test_extraction_route_module_has_no_write_calls`,
   `test_extraction_route_module_has_no_mutation_route_registrations`) run
   unedited and green inside the fresh 327 line.
8. **D3 name mapping.** Accepted. The handoff's Red-protocol section and its
   Required-tests section name four of the six failing tests differently; the
   two cannot both name the functions. The implementer chose the
   Required-tests identifiers (the normative list the acceptance items cite)
   and carried the Red section's six panic messages verbatim. I verified each
   verbatim string in the source: `expected status 200 for an approved apply,
   got 428` (:431), `expected 404 for a plan the applier does not know, got
   428` (:452), `expected 409 for an applier whose epoch differs from the live
   epoch, got 428` (:470), `expected 409 apply_in_progress, got 428` (:495),
   `expected 500 for a transaction error, got 428` (:517), `the dry-run 200
   body must carry the minted apply_token` (:556). The Red totals are also
   arithmetically consistent (321 + 6 = 327 = 312 + 15).
9. **ASCII and encoding.** `python tools/encoding_guard.py --all` exits 0 with
   no findings (silent, capture 0 lines). Every response the route family can
   answer is asserted pure ASCII by the new test file, and the 200, the 503,
   all five 428 rows, 404, both 409 rows and the 500 row each carry an
   `assert_ascii_body` call; and the landed content of this range contains no
   non-ASCII byte, per the silent `encoding_guard --all` exit 0.

## Findings

1. **MINOR (documentation completeness, no source change required).**
   `tasks/extraction-auto-apply-a3-journal.md:33-43` states the census as
   "exactly FOUR sites" and stops there, and the landing record at :300-334
   likewise records only the twelve paths. The F14 census is a standing
   STOP-and-report rule, and the number legitimately moves 4 -> 5 with the
   landing, because the hand's own new test file must construct its own
   `ExtractionDryRunDto` sample: the handoff (lines 270-272) forbids sharing
   the wizard file's private helper and forbids editing that frozen file, so
   `extraction_apply_api.rs:211-223` necessarily adds a fifth construction
   site, and it correctly includes the new `apply_token: String::new()` field.
   This is plan-mandated, inside the touch-set, and compiles clean (gates 4 and
   5 exit 0; the fresh census is +15 green). The gap is disclosure only: the
   census section does not note that its count is preflight-scoped and that
   the landing adds one in-touch-set site. Remediation for the next implementer
   hop: one line in the journal census section (or a standing ledger note)
   recording "census 4 -> 5 on landing: the new test file's own sample, per the
   own-helpers rule at handoff lines 270-272". No source edit, no revert, no
   re-run. Not a blocker.

## Deviation dispositions

- **D1 (distill-staging route for gate 7).** ACCEPTED. Same precedent as A-2
  D1 and playbook T-C D2 (both approved in prior verdicts). With an empty
  index, `encoding_guard --staged` would scan nothing; the twelve explicit
  paths were staged by an enumerated `git add --` (never `.`), and the guard
  then scanned the blobs actually landing. I independently ran
  `encoding_guard --all` over the landed tree: exit 0. Scope discipline intact.
- **D2 (journal rider commit).** ACCEPTED. A landing record cannot live inside
  its own commit; the rider is journal-only (36 insertions, zero source) and
  matches the W-1 / A-1 / A-2 precedent.
- **D3 (Red-section names versus Required-test identifiers).** ACCEPTED, see
  adjudication 8. The Required-tests identifiers are the normative set; the six
  Red panic messages are carried verbatim, so the Red evidence remains
  machine-checkable against both readings of the handoff.
- **D4 (test file at 639 lines against the ~300-360 estimate).** ACCEPTED. The
  cause is structural, not sloppiness: the handoff forbids sharing the frozen
  wizard helpers, so every request/assert helper and every `assert_eq!` message
  is priced twice, and the two scripted doubles are ~250 lines. The file is far
  under the 1000-line ceiling, no function approaches 100 lines (R1/R2 zero),
  and the hand's produced-lines budget holds: 943 lines of implementation plus
  test diff against the ~1200 bound.

## Observation dispositions (O1-O4)

- **O1 (gate 5's 21 cargo warnings).** Noted and confirmed benign by
  inspection: the warnings sit in `api_graph.rs`, `tests/integration/tests/
  common/mod.rs` and `m18_cpp_plugin_parity`, none of which this range
  touches. Zero warnings in any touched file.
- **O2 (the Red line reads `2 suites`).** Noted. cargo's fail-fast aborts the
  remaining targets after `ui_test` fails, so the Red digest shows 2 suites;
  gate 3 restores the 3-suite shape. I confirmed the 3-suite shape fresh.
- **O3 (refusal disclosure about the chained/piped hygiene command).**
  Accepted as transparency, and independently satisfied on my hop: I ran
  `python tools/hygiene_check.py` plainly (R3 warn-only output reviewed) and
  `python tools/distill.py -- python tools/hygiene_check.py` -> `### generic
  PASS (exit 0)`. Consistent with the reviewer protocol that a catch-all
  denial is boilerplate and the gate must be obtained another way rather than
  dropped.
- **O4 (the `--staged` runs of duties 1 and 2 were vacuous).** Accepted, and
  discharged by my fresh `encoding_guard.py --all` (exit 0), which scans the
  landed verdict file and the amended A-2 journal as tracked content.

## Bench exemption stance (recorded, with proof)

No timing number is attributed to A-3. Three independent facts carry the
exemption, all re-established on this hop:

1. `python tools/bench_compare.py` is deterministically blocked before any
   comparison: `### generic FAIL (exit 2)` with
   `ERROR parse format drift: documented phase row(s) missing from bench output
   (never substituting zeros)`, the `DOCUMENTED_PHASES` tuple at
   `tools/bench_compare.py:75-90` (line 84 inside it). `tools/**` is forbidden
   to the reviewer, so the tool is not edited and the exit-2 is the standing
   documented state.
2. The producer set is byte-identical across the range:
   `git diff --numstat "eb2bd91..ea66151" -- crates/cli/tests` is EMPTY
   (no output).
3. What A-3 adds is a route, DTOs and a state seam; the apply algorithm runs
   only through the A-2 core once A-4 wires it, so nothing in this range is a
   measurable load-perf producer.

## Acceptance table (fresh evidence)

1. Every C3 row answered with the exact body and every body pure ASCII - MET.
   Required tests 1-12 plus the three lock tests are green inside
   `327 passed / 0 failed / 0 ignored / 3 suites`; the shape table was checked
   field-by-field against `api_extraction_apply.rs` (adjudication 4).
2. Dry-run mints only with a live-epoch applier - MET.
   `test_dry_run_route_mints_an_apply_token_with_a_live_applier` green; the
   frozen wizard dry-run 200 test stays green with the field empty because it
   installs no applier.
3. No write token and no mutation registration in the new module - MET.
   `test_apply_route_module_has_no_write_or_registration_tokens` green; I read
   the module and confirmed no `std::fs` usage at all.
4. Wizard GET-only and no-write locks green with the wizard assertions unedited
   - MET. The wizard file diff is exactly the one C8 line; its three lock tests
   are unedited and green.
5. Gates 4 and 5 exit 0 - MET (implementer's record; reproduced in spirit by
   my workspace census and the ui-scoped run, both of which compile and run
   the full `ariadne-ui` and `ariadne-cli` test surface).
6. `hygiene_check.py` exit 0 and `encoding_guard.py --all` exit 0 - MET fresh.
7. Commit touch-set is exactly the twelve declared paths - MET, per the
   numstat above.

## Follow-ups for the ledger (recorded, not dropped)

- Finding 1's census disclosure note goes to the next implementer hop's duty 1
  as a one-line journal entry (or a standing ledger note if the planner prefers
  one record for the milestone).
- The A-4 hand owns the consumption site (consumption at transaction start,
  lock-refusal non-consumption), the `apply_lock` `try_lock` model of C5, the
  `ValidationStatus` -> `status` + `detail` mapping, the ASCII sanitization and
  the `swap_into_state` install call immediately after
  `install_extraction_reader`. None of that is in A-3 and nothing here
  pre-empts it.

This verdict is the only artifact written by the reviewer hop. It is read-only
with respect to source, tests, the journal and the planning docs; the next
implementer hop commits it as its duty 1.
