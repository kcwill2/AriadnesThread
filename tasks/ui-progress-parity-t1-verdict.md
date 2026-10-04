# Verdict - ui-progress-parity T1 (ServerProgress additive step-rows DTO)

Reviewer adjudication via `review-gate`, 2026-10-04. Range under review:
`e276910..000238b` (commits `5eff824` code+tests+ledger, `000238b` journal
evidence follow-up), reviewed oldest-first, each as if alone.

## VERDICT: APPROVED

## Fresh verification (reviewer's own runs)

- `python tools/checkpoint_guard.py status` - HEAD `000238b`, branch `main`,
  worktree clean except untracked planning docs (handoff/shape/kickoff/T2/T3,
  plans; not part of the commit range).
- `python tools/testrun.py --distill -p ariadne-ui` - exit 0, canonical line
  verbatim: `184 passed / 0 failed / 0 ignored / 3 suites` (capture
  `target/testrun-capture.txt`, 203 lines). Matches the implementer's claim
  exactly.
- `python tools/hygiene_check.py` - R3 warnings only (warn-only per AGENTS
  rules); every warning sits on a line predating this commit (new code spans
  dto.rs ~1-40 and server_api.rs 767-943; zero warnings in those ranges).
  Successful invocation with no error = exit 0.
- `python tools/encoding_guard.py --all` - clean, exit 0.

## Commit-vs-touch-set

`git diff --name-only e276910..HEAD`:

- `crates/ui/src/dto.rs` - in set.
- `crates/ui/src/server.rs` - in set; both hunks are the two named struct
  literals only (server.rs:60 `new_loading`, :84 `new_loaded`), 2 added lines
  each; nothing else in the file.
- `crates/ui/tests/ui_test/server_api.rs` - in set; additions-only except one
  deletion: the 1-line `use ariadne_ui::{...}` widened to a 3-line import. No
  existing assertion weakened or deleted;
  `test_status_route_locks_ready_response_shape` untouched.
- `tasks/ui-progress-parity-t1-handoff-journal.md` - in set.
- `tasks/milestone_ledger.md` - OUTSIDE the declared touch set; adjudicated
  as finding 1 / deviation D1 below.
- `Cargo.lock` - untouched (as expected; no new deps).

## Frozen locks (all proven empty vs base e276910)

`git diff e276910..HEAD` empty for: `crates/pipeline` (I9),
`crates/cli/src/commands/progress.rs`, `crates/cli` (entire crate),
`crates/ui/assets`, `themes.css`, `theme.js`, `tests/fixtures`,
`tests/golden`, `tools/theme_token_baseline.json`,
`crates/ui/tests/ui_test/static_html.rs`, `crates/ui/tests/ui_test/common.rs`.

Independent literal-site check: grep `ServerProgress {` across `crates/`
returns exactly server.rs:60, server.rs:84, plus the struct definition at
dto.rs:27. The implementer's claim is confirmed, not trusted.

## Plan-literal checks

- DTO block vs shape lines 89-119: enum derives, `rename_all = "lowercase"`,
  struct derives, field names/types/order (`steps` then `total_stages` after
  `message`), `#[serde(default)]` on both new fields, no
  `skip_serializing_if`, doc comments verbatim for the three new items -
  all match.
- Four legacy fields unchanged in name/type/meaning (I4 intact).
- Both constructors carry `steps: Vec::new(), total_stages: 0` (handoff
  lines 59-62).
- Mid-boot fixture test locks the shape's JSON example (shape 125-137)
  field-for-field: phase `[2/5] Indexing translation units...`, percent 26,
  message `6/24 TUs`, `total_stages` 5, five rows with stripped-prefix
  labels, percents 100/30/0/0/0, lowercase wire strings done/active/
  pending x3.
- I5 test deserializes the exact old four-field JSON and round-trips all four
  legacy values, asserting `steps == []`, `total_stages == 0`.
- I6 test mirrors the real error arm at `crates/cli/src/commands/ui.rs:720-722`
  (assigns ONLY `state`/`message` under one write guard); asserts `steps`
  Vec-equality to the installed rows and no row gaining `Done` (plus
  `total_stages == 5`). The Vec-equality assertion strictly dominates the
  Done-count check.
- No retained dual representation: the four globals keep their exact meaning;
  the new fields are the only additions.
- `dto.rs` is 880 lines (was 852 + 28), still < 1000; functions well under
  100 lines.

## Report-evidence check (distillation contract)

Journal is digest-shaped: canonical summary lines + capture path + exit-code
semantics, zero raw output; gate commands all use distilling entry points
(`testrun.py --distill`, `commit_scoped.py`, `hygiene_check.py`,
`encoding_guard.py --staged`). Compliant.

## Findings

1. (minor, ACCEPTED deviation) `tasks/milestone_ledger.md` was co-committed
   inside `5eff824` - outside the handoff's hard touch-set. Disposition:
   accepted on merits: the delta is exactly one PLANNED row describing this
   milestone itself (artifacts, handoff list, locks, DP resolutions, the
   `new_loaded` name note); the deviation was pre-recorded in the journal
   before any code was touched; ledger-code co-landing is repo convention
   (AGENTS.md "the ledger is the source of truth"). Process gap belongs to
   the handoff document, not the implementer: future handoffs on this
   milestone (T2/T3) should list `tasks/milestone_ledger.md` in the touch-set
   if co-landing remains the plan. No rework required.
2. (minor, disposition) Name deviation `new_with_data` -> `new_loaded`:
   ACCEPTED. Pre-recorded in the handoff itself (lines 16-18) with an
   explicit no-rename instruction; code renames nothing; journal re-records
   it. Correct behavior.
3. (disposition) Row-level tests (I5, mid-boot fixture, I6) reached Green via
   a compile-error Red, not assertion-Red: ACCEPTED - this is the handoff's
   own prescribed sequence (handoff lines 82-85), and the journal records it
   honestly as not assertion-Red.
4. (informational) Red run reported `2 suites` vs green `3 suites`; the
   reviewer's fresh run confirms `3 suites` with successful parse and exit 0
   both ways. No action; noted for the T3 hop to confirm the workspace-wide
   suite census.

## Gate execution notes

- Two initial combined invocations (`cmd; echo $LASTEXITCODE` chains and one
  `python -c` probe) were refused with the boilerplate catch-all rule
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
  Each gate was then re-run individually and succeeded, so no gate was
  skipped on refusal evidence alone.
- Workspace-wide milestone gates (`testrun --baseline`, full `unittest
  discover`, `ui_smoke`, `bench_compare`) were NOT run here, by explicit
  review instruction: they belong to the final T3 reviewer hop.

## Follow-up dispositions

- Finding 1's touch-set amendment for T2/T3 handoffs: orchestrator-side,
  record at next handoff authoring; not ledger deferred-finding material.
- Suite-count census (finding 4): verify during T3's workspace gate run.
