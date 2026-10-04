# T1 Handoff - ui-progress-parity: ServerProgress additive step-rows DTO (backend only)

Milestone key `ui-progress-parity`. Execution slot 1 of 3. No predecessor.
Reviewer adjudicates via the `review-gate` skill; its commit-vs-touch-set
check enforces the list below.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/ui-progress-parity-shape.md` lines 85-138 (exact Rust DTO + the
   mid-boot JSON example the `/api/status` fixture must lock) and lines 51-64
   (invariants I5 additive serde, I6 error freeze).
3. `crates/ui/src/dto.rs` lines 1-10 (current `ServerProgress`) - mirror its
   derive/doc style.
4. `crates/ui/src/server.rs` lines 54-68 (`UiState::new_loading`, literal at
   line 60) and lines 70-95 (`UiState::new_loaded`, literal at line 82).
   Shape/kickoff call the second constructor `new_with_data`; the real name
   on this tree is `new_loaded` - the name deviation is recorded, do NOT
   rename anything.
5. `crates/ui/tests/ui_test/server_api.rs` lines 54-135 and 240-253 - the
   existing status-route assertion style to extend.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/ui-progress-parity-t1-handoff-journal.md (checkpoint journal; create
  it alongside this handoff)
- crates/ui/src/dto.rs (add the enum + struct + 2 fields on ServerProgress)
- crates/ui/src/server.rs (update ONLY the two ServerProgress struct
  literals, lines ~60 and ~82; nothing else in this file)
- crates/ui/tests/ui_test/server_api.rs (new progress tests appended; do not
  weaken or delete any existing assertion)
- Cargo.lock (only if cargo regenerates it - it should not; no new deps)

You MUST NOT touch: `crates/pipeline/**`, any file under `crates/cli/**`,
anything under `crates/ui/assets/**`, `crates/ui/tests/ui_test/static_html.rs`
(T3 owns it), `crates/ui/tests/ui_test/common.rs`, `themes.css`, `theme.js`,
`tests/fixtures/**`, `tests/golden/**`, `tools/theme_token_baseline.json`,
any route handler other than the two struct literals named above.
No `git stash`, no branch change.

## What to build

Exact contract, copied verbatim from the shape (shape lines 89-119) into
`crates/ui/src/dto.rs`:

- `pub enum ProgressStepStatus { Pending, Active, Done }` with
  `#[serde(rename_all = "lowercase")]` (wire values `"pending" | "active" |
  "done"`) and derives `Debug, Clone, Copy, PartialEq, Eq, Serialize,
  Deserialize`.
- `pub struct ProgressStepView { stage: usize, label: String, percent:
  usize, message: String, status: ProgressStepStatus }` with derives
  `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`. `label` carries the
  phase text with the `[k/N] ` prefix already stripped - this crate never
  strips; the CLI producer (T2) does.
- Extend `ServerProgress` with, in this order after `message`:
  - `#[serde(default)] pub steps: Vec<ProgressStepView>`
  - `#[serde(default)] pub total_stages: usize`
  The four existing fields keep their exact names, types, and meaning
  (`state` stays `"loading" | "ready" | "error"` - invariant I4; `percent`
  stays the GLOBAL pipeline percent).
- Both `UiState` constructors gain `steps: Vec::new(), total_stages: 0`
  (empty pre-seed is the honest value; T2's `run_ui` seeds real rows,
  the standalone/export path stays empty - that is what T3's legacy
  fallback renders).

Serialized shape of a fresh loading state MUST gain `"steps": []` and
`"total_stages": 0` keys (serde emits `#[serde(default)]` fields on
serialize; do not add `skip_serializing_if`).

## RGB protocol
1. Red: add ONLY these two tests to `server_api.rs` first; they compile
   against today's struct and must fail with the messages named below:
   - `test_status_progress_json_has_step_row_fields`: GET `/api/status` on
     `common::loading_state()`; assert `body["progress"]["steps"].is_array()`
     and `body["progress"]["total_stages"].is_number()`. Today `steps` is
     absent, so the expected failure is
     `assertion failed: body["progress"]["steps"].is_array()` (the index
     yields `Value::Null`). Capture the failing list.
   - `test_progress_serialization_gains_empty_step_fields`: serialize
     `new_loading(...)`'s progress via `serde_json::to_value`; assert the
     object contains both keys. Same expected failure (missing key /
     `Value::Null`).
2. Green: add the DTO + constructor fields above; re-run until green. NEVER
   weaken an assertion to pass. Then, in the same green phase, add the
   row-level tests (their pre-DTO state is the compile error `struct
   ServerProgress has no field named 'steps'` - record that in the journal
   honestly; it is not presented as assertion-Red):
   - I5 back-compat: `serde_json::from_str::<ServerProgress>` of an old
     four-field JSON string (no `steps`/`total_stages`) deserializes with
     `steps == []`, `total_stages == 0`, and the four legacy values
     round-trip unchanged.
   - Mid-boot fixture (locks the shape's JSON example, shape lines 125-137):
     install a loading `ServerProgress` whose `steps` hold the exact five
     example rows (stage 1 done/100/"Include graph ready", stage 2
     active/30/"6/24 TUs", stages 3-5 pending/0/empty), `total_stages: 5`;
     GET `/api/status`; assert each row's keys `stage`, `label`, `percent`,
     `message`, `status` and lowercase status strings, field-for-field.
   - I6 error freeze: install that same loading snapshot, then perform the
     error-arm write pattern exactly as `crates/cli/src/commands/ui.rs`
     line 720 does (assign ONLY `state = "error"` and `message`, via
     `write`/`write_state` on `state.progress`); re-read and assert
     `steps`/`total_stages` are `==` the installed rows (byte-equal `Vec`)
     and no row gained `Done`.
3. Refactor: doc comments on the three new DTO items (copy the shape's doc
   wording), re-run.

## Hygiene (review-enforced)
Functions < 100 lines, files < 1000 lines (`dto.rs` is 852 + ~50 -> still
under, this is its last comfortable DTO block - no gratuitous additions),
meaningful names, no banned names (tmp/res/foo/bar/test1), no single-letter
locals. `python tools/hygiene_check.py` must stay exit 0.

## Frozen locks
- `crates/pipeline/**` byte-identical (invariant I9): reviewer proves
  `git diff <milestone-base> -- crates/pipeline` empty.
- `crates/cli/src/commands/progress.rs` byte-identical: same diff proof.
- `themes.css`, `theme.js`, `crates/ui/assets/**` untouched (T3's surface;
  you add zero CSS/JS in this handoff).
- `tests/fixtures/**` frozen (all branches); `tools/theme_token_baseline.json`
  untouched (zero hex added by this handoff - trivially true, no CSS touched).
- Existing `server_api.rs` assertions unchanged except the named literal
  updates; `test_status_route_locks_ready_response_shape` must still pass
  untouched (I4 readiness contract intact).

## Gates (exact commands, timeout 600000 ms each; scoped only)
1. `python tools/testrun.py --distill -p ariadne-ui` -- Red run first: the
   two named tests fail as described; digest header + canonical summary line
   verbatim in the report; capture path cited.
2. `python tools/testrun.py --distill -p ariadne-ui` -- Green run: canonical
   line shows `0 failed`.
3. `python tools/hygiene_check.py` -- exit 0.
4. `python tools/encoding_guard.py --staged` -- exit 0 before committing.
5. Commit with
   `python tools/commit_scoped.py --scope "crates/ui/src/dto.rs" --scope "crates/ui/src/server.rs" --scope "crates/ui/tests/ui_test/server_api.rs" --scope "tasks/ui-progress-parity-t1-handoff-journal.md" -m "feat(ui): additive step-row fields on ServerProgress (ui-progress-parity T1)"`
   (never `git add .`).

Expected verdict lines: gates 1 (exit non-zero from the two Red asserts,
parse succeeded so testrun exits 0 with M failed > 0 - cite the canonical
line), 2 exit 0 `0 failed`, 3-4 exit 0, 5 commit hash + `git show --stat`
showing only touch-set files.

## Discipline clauses
- UTF-8: never Set-Content/Out-File/>/Get-Content-replace; Write/Edit tools
  only; authored content pure ASCII (see AGENTS.md File-Edit Discipline).
- Touch only the touch-set; on discovering out-of-scope breakage: report, do
  not fix.
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, full
  `unittest discover`, `ui_smoke`, `bench_compare`, `-p` runs beyond
  `ariadne-ui`) in an implementer session are a reviewer blocker; route
  through the distilling entry points (`docs/specs/tool-output-distillation.md`).
- After EACH per-module Green gate, append a 1-3 line checkpoint to
  `tasks/ui-progress-parity-t1-handoff-journal.md`: module + commit hash,
  any deviation, any lock/contract finding.
- If context exhaustion looms: STOP, return a disk-state inventory; a dead
  session is NEVER resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/ui-progress-parity-t1-handoff.md --scope crates/ui --cmd "python tools/testrun.py --distill -p ariadne-ui"`.

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence (first failing tests +
expected failure messages). 3. Green evidence (distill-mode digest header +
canonical summary line + capture path). 4. Any deviation + reason.
5. Out-of-scope observations (report-only). Never paste raw tool output;
cite command + canonical line + exit code.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill against this file; the milestone acceptance it feeds is kickoff
gates G1/G5 (milestone-level runs belong to the orchestrator, not here).
