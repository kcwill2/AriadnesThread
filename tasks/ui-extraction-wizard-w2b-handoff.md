# W-2b Handoff - ui-extraction-wizard: read-only reader trait, wizard DTOs, two GET routes

Depends on W-1 landed (the wizard DTOs extend `dto/extraction.rs`; W-1 is the
hard predecessor because `dto.rs` sits at exactly 1000 lines at HEAD
`97dd836`).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/ui-extraction-wizard-shape.md` sections 2 (invariants W1/W6/W9), 3
   (C1 flow table, C2 trait + DTOs + state seam, C3 error shapes), 4 (scope
   boundary), 9 (names); `tasks/ui-extraction-wizard-kickoff.md` sections 3-5.
3. `crates/ui/src/routes/api_graph.rs` (route-handler idiom; the 503 loading
   convention is locked at `crates/ui/tests/ui_test/graph_api.rs:269-289`).
4. `crates/ui/src/state.rs` (install_analysis at :152, analysis_epoch at :84
   and :177, new_loading :94, new_loaded :115) and `crates/ui/src/server.rs`
   (route table :48-76, every route registered with `get(...)` only).

## Objective and preserved invariants

Land the wizard backend contract in `crates/ui`: the plain-data
`ExtractionManifestReader` trait, the additive wizard DTOs, one new
`extraction_reader` field on `UiState` with install/accessor, the two GET
routes, and the uniform error contract (503 loading / 409 stale generation /
404 unknown plan). No mutation surface: this milestone's invariant W1 says
every new route is a GET and the new route module contains no filesystem
write.

Invariants this hand must preserve:
- W1: both routes registered with `get(...)` only; `routes/api_extraction.rs`
  contains no `fs::write`, `fs::remove_file`, `fs::create_dir` and no
  `ArtifactManager` call (string/import locks in the new test file).
- W6: a reader whose `epoch()` differs from the live `analysis_epoch` is
  refused with 409 and the documented body; the wizard resets on that
  response (the reset itself is W-3b's render concern).
- W9: the boot payload does not grow - this hand adds no payload field, no
  builder change, no cli change.
- `UiState::new_loading` / `new_loaded` signatures UNCHANGED (shape C2), so no
  existing call site changes.
- Error bodies are ASCII and exact: `{"error":"analysis_loading"}`,
  `{"error":"stale_generation","epoch":<live>}`,
  `{"error":"unknown_plan","plan_id":"<id>"}` (shape C3). Check order in each
  handler: no reader installed -> 503; reader epoch != live -> 409; None ->
  404; Some -> 200.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- tasks/ui-extraction-wizard-w2b-journal.md (checkpoint journal)
- crates/ui/src/dto/extraction.rs (wizard DTOs + the three caps)
- crates/ui/src/state.rs (the reader field, `install_extraction_reader`, the
  accessor)
- crates/ui/src/routes/api_extraction.rs (NEW: trait + both handlers)
- crates/ui/src/routes/mod.rs (one `pub(crate) mod api_extraction;` line)
- crates/ui/src/server.rs (two route registrations + the use line)
- crates/ui/src/lib.rs (module visibility only: one `pub use` so the cli
  provider of W-2a can implement the trait)
- crates/ui/tests/ui_test/extraction_wizard_api.rs (NEW: route contract locks)
- crates/ui/tests/ui_test/main.rs (one mod line: insert
  `mod extraction_wizard_api;` immediately after `mod extraction_contrast;`
  and before `mod file_view;`; mod order is behavior-neutral, the site is
  fixed for review-diff stability)

You MUST NOT touch: `crates/cli/**`, assets, the card test files
(`extraction_card_story.rs`, `extraction_card_playbook.rs` and the other 16
card-family locks), `themes.css`, `tools/**`.

## What to build

Trait (shape C2 verbatim; doc comment carries the no-write rule):

```rust
/// Read-only extraction computation the wizard routes need. The host (cli)
/// implements it against the isolation engine and the include rewriter; no
/// implementation may write to disk (W1, W2).
pub trait ExtractionManifestReader: Send + Sync + 'static {
    fn epoch(&self) -> u64;
    fn manifest(&self, plan_id: &str) -> Option<ExtractionManifestDto>;
    fn dry_run(&self, plan_id: &str) -> Option<ExtractionDryRunDto>;
}
```

DTOS, additive, in `dto/extraction.rs`, every field `#[serde(default)]`, all
serde-plain types (shape C2 struct list verbatim: `ExtractionUnitManifestDto`,
`ExtractionManifestDto`, `ExtractionDryRunDto`, `ExtractionRewriteDto` with
the exact field names and comments). Caps shared by provider and renderer:

```rust
pub const EXTRACTION_MANIFEST_MAX_FILES: usize = 64;
pub const EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE: usize = 200;
pub const EXTRACTION_DRY_RUN_MAX_REWRITES: usize = 200;
```

State seam (shape C2): one field
`extraction_reader: RwLock<Option<Arc<dyn ExtractionManifestReader>>>` with
`install_extraction_reader` and an accessor; `new_loading` and `new_loaded`
initialize it to `None` and keep their signatures. Routes (registered in
`server.rs`): `GET /api/extraction/{plan_id}/manifest` and
`GET /api/extraction/{plan_id}/dry-run`; the `{plan_id}` path parameter is
read as a plain string (no parsing into repo paths).

## RGB protocol (per module, in this file order)

Red: author `crates/ui/tests/ui_test/extraction_wizard_api.rs` first, and land
the compile-complete skeleton of the contract (trait, DTOs, state field, both
routes, handlers whose ONLY implemented branch is the no-reader 503). The
reader-consulting branches (200/404/409) do not exist yet, so the route tests
fail semantically. Expected failures (verbatim shape): the manifest and
dry-run tests that install a reader report `expected status 200` /
`expected 404` / `expected 409` while every response is
`503 {"error":"analysis_loading"}`; the no-write string locks pass already.
A stale-reader test injects a reader with epoch 0 after one
`install_analysis` (live epoch 1) and asserts 409 with `"epoch":1` in the
body.
Green: wire both handlers to the reader accessor and the live epoch in the
C3 check order; re-run until exit 0.
Refactor: handler shared helper for the error bodies if duplication appears;
keep every function under 100 lines.

## Required tests (in extraction_wizard_api.rs; model: graph_api.rs route tests)

1. Manifest: no reader installed -> 503 `analysis_loading`.
2. Manifest: reader for the live epoch, known plan -> 200 with the DTO JSON.
3. Manifest: same reader, unknown plan id -> 404 `unknown_plan` with the id.
4. Manifest: reader with epoch != live -> 409 `stale_generation` with the
   live epoch; body ASCII.
5. Dry-run: the same four shapes (503/200/404/409).
6. Lock: `server.rs` registers both paths with `get(` only.
7. Lock: `routes/api_extraction.rs` source contains no `fs::write`,
   `fs::remove_file`, `fs::create_dir`, `ArtifactManager` (W1).
8. Lock: the module contains no `post(`, `put(`, `delete(`, `patch(` route
   registration (W1).

## Hygiene (review-enforced)

Functions < 100 lines, files < 1000 lines, no banned identifier names, no
single-letter locals, intention-revealing names (shape section 9 name list is
binding for the new items). `python tools/hygiene_check.py` exit 0; note
`state.rs` and `server.rs` are already-sized files - keep both under their
current line counts plus the small deltas.

## Frozen locks

- `tools/theme_token_baseline.json` diff-empty; zero added hex (no asset is
  touched, so the check is trivially silent - still run it).
- The 18 card-family locks stay byte-identical (W7): this hand's diff must not
  touch either card test file.
- The ui suite minus the two new modules runs green UNEDITED.
- `crates/cli/**` diff-empty except by W-2a's later hand.

## Gates (exact commands, timeout 600000 ms each; scoped only)

1. `python tools/testrun.py --distill -p ariadne-ui` (baseline canonical line
   recorded in the journal)
2. `python tools/testrun.py --distill -p ariadne-ui` (Red: exit 1; journal
   records the named failing tests verbatim)
3. `python tools/testrun.py --distill -p ariadne-ui` (Green: exit 0;
   canonical line = baseline + the new tests)
4. `python tools/distill.py -- cargo check -p ariadne-cli --all-targets`
   (exit 0: F14 standing rule 2, the live reverse dependency; proves the
   state.rs seam did not move any cli call site)
5. `python tools/hygiene_check.py` (exit 0)
6. `python tools/encoding_guard.py --staged` (exit 0)
7. `python tools/commit_scoped.py --scope crates/ui/src/dto/extraction.rs
   --scope crates/ui/src/state.rs --scope crates/ui/src/routes/api_extraction.rs
   --scope crates/ui/src/routes/mod.rs --scope crates/ui/src/server.rs
   --scope crates/ui/src/lib.rs --scope
   crates/ui/tests/ui_test/extraction_wizard_api.rs --scope
   crates/ui/tests/ui_test/main.rs --scope
   tasks/ui-extraction-wizard-w2b-journal.md -m "feat(ui): extraction
   manifest/dry-run routes behind a read-only reader trait (503/404/409)"`
   (never `git add .`)
8. `python tools/encoding_guard.py --all` (post-commit, exit 0)

Gate count: 8 (commit-cycle trio is gates 6-8, itemized per the M28 T0b
finding-1 journal law).

## Discipline clauses

- UTF-8: Write/Edit tools only; pure-ASCII authored content; no em dashes;
  use `&rarr;` / `&#8592;` entities if a glyph is ever wanted (none should be
  in Rust source).
- Touch only the touch-set; out-of-scope breakage: report, do not fix.
- Bare high-volume commands are a reviewer blocker: distilling entry points
  only. This hand is crates/ui src + tests; no ui_smoke/bench duty (the
  reviewer hop owns the runtime gate).
- Checkpoint the journal after each Green gate.
- Context exhaustion: STOP with a disk-state inventory; successor resumes via
  `python tools/checkpoint_guard.py resume --spec
  tasks/ui-extraction-wizard-w2b-handoff.md --scope ... --cmd ...`.

## Report back (final message)

1. Commit hash + `git show --stat`. 2. Red evidence (named failing tests,
  verbatim panic lines). 3. Green evidence (digest header + canonical line +
  capture path). 4. Deviations + reasons. 5. Out-of-scope observations.
No raw tool output in the report.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces the touch-set above exactly.
