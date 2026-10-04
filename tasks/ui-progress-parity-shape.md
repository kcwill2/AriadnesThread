# Shape - UI/CLI Boot Progress Parity

Companion to `tasks/ui-progress-parity-kickoff.md`. This is the shape, not a
task list; the planner sizes handoffs from the split seams at the end.

## Plumbing reality (how progress reaches the browser today)

`Pipeline::with_progress` (crates/pipeline) fires `ProgressEvent` ->
`crates/cli/src/commands/ui.rs::stage_progress_callback` maps each event to a
`ProgressReport { stage, percent(global 0-100), phase: &'static str,
message }` -> `run_pipeline_stages` adds per-stage start/end reports ->
`make_boot_progress(state)` writes the report's *global* view into
`UiState.progress` (`ServerProgress`) and forwards the same report to
`TerminalStepBars`, which computes `step_percent(stage, percent)` and prints
`[k/5] Label [################____]  80%  message` (20-cell bar,
`STEP_BAR_WIDTH = 20`, `format_step_bar` in `progress.rs`).

The browser gets `ServerProgress { state, phase, percent(global), message }`
via `/api/status`, polls at 150 ms, and draws one cumulative-ish CSS bar. The
per-step view exists ONLY inside the terminal renderer's closure. **The fix
is to widen the snapshot that `make_boot_progress` already writes, so the
browser receives the same per-step frames the terminal commits.** No new
event, no new thread, no new endpoint, no polling change.

## Invariants (each independently checkable)

I1 **Per-step scale.** For any boot report stream, every snapshot row's
`percent == step_percent(row.stage, report.percent)` (clamped 0-100), and a
row reaches 100 only on its step's own terminal frame. Locked by: new
`boot_progress` unit tests replaying the stage-1..4 start/end frames and the
exact stage-5 calibrated sequence already pinned in
`crates/cli/src/commands/ui/progress_contract_tests.rs::
test_stage_five_milestone_sequence_is_monotonic_and_contained` (80,81,81x5,
82,85,85,85,86,99,100 -> row percents 0,5,5x5,10,25,25,25,30,95,100).

I2 **Terminal-parity equivalence (the oracle).** For every replayed report
stream, feeding it to BOTH (a) `TerminalStepBars::with_sink(MemorySink)` and
(b) `BootStepSnapshot`, then: each terminal line committed by the renderer's
newline logic parses to `(stage, label, step_percent, message)` equal to the
snapshot's `done` row at that moment; the final open terminal line equals the
`active` row. Frozen against the CURRENT `format_step_bar`/`StepBarRenderer`
before any consumer ships. Lives in `crates/cli` (sees both sides). Parsing
is field-position extraction from `format_step_bar`, not substring luck.

I3 **Row monotonicity.** Across a run, each row's `percent` is
non-decreasing and its status never leaves `done`. Frames for a stage older
than the active stage are ignored (defensive; global percent and stage are
monotone in `build_ui_payload`). Locked by replay property tests in the same
module as I1.

I4 **Readiness contract intact.** `progress.state` still only ever
`"loading" | "ready" | "error"`; on `ready`, `swap_into_state` writes the
same globals as today; `ui_smoke` polls readiness unchanged. Locked by:
existing `server_api.rs` asserts, unchanged, plus `python tools/ui_smoke.py`.

I5 **Additive serde.** A `ServerProgress` JSON object without `steps` /
`total_stages` deserializes to `steps == []`, `total_stages == 0`, and old
field values round-trip unchanged. Locked by: new serde test in
`crates/ui/tests/ui_test/`.

I6 **Error freeze.** The `run_ui` error arm keeps writing only
`state`/`message`; `steps`/`total_stages` are byte-equal to the last loading
snapshot, no row gains `done`. Locked by: `server_api.rs` test that installs
a loading snapshot, flips to error, compares rows.

I7 **Bar arithmetic parity.** Filled cells rendered for percent `p` equal
`Math.floor(20 * p / 100)`, matching `format_step_bar`'s
`STEP_BAR_WIDTH * percent / 100` integer arithmetic; total cells always 20.
Locked by: `static_html.rs` source assertion that the renderer computes the
bar from that formula with the shared width, plus the existing
`test_step_bar_fill_tracks_step_percent` pinning the CLI side. **No
headless-browser test locks the rendered DOM; stated plainly - residual risk
accepted (see Risks R2).**

I8 **Theme safety.** The milestone's added lines introduce zero hex
literals; every progress style uses existing `var()` tokens
(`--cyan`/`--cyan-glow`/`--text-muted`/`--bg-card`/`--border` family, present
in all three themes via D7 token-name reuse). Verdict/heat tokens untouched.
Gate: `python tools/theme_token_check.py --check-added --base <rev>` clean.

I9 **Pipeline untouched.** `crates/pipeline/src/lib.rs` byte-identical; the
exhaustive `stage_progress_callback` match stays the single mapping point.
Gate: reviewer commit-vs-touch-set (review-gate skill).

## Contracts (exact)

### Rust DTO - crates/ui/src/dto.rs (additive)

```rust
/// Lifecycle of one boot step row, serialized lowercase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgressStepStatus { Pending, Active, Done }

/// One `[k/N]` boot step as the terminal would show it: its own 0-100 bar
/// plus the in-flight detail. `label` carries the phase text with the
/// `[k/N] ` prefix stripped (the browser composes the label from
/// `stage`/`total_stages`). `message` is empty while `pending`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgressStepView {
    pub stage: usize,
    pub label: String,
    pub percent: usize,
    pub message: String,
    pub status: ProgressStepStatus,
}

pub struct ServerProgress {
    pub state: String,   // unchanged contract: loading | ready | error
    pub phase: String,   // unchanged: current report's phase string
    pub percent: usize,  // unchanged: GLOBAL pipeline percent
    pub message: String, // unchanged
    /// Per-step rows mirroring the terminal's committed + live lines.
    /// Empty only pre-seed (standalone exports, legacy fixtures).
    #[serde(default)]
    pub steps: Vec<ProgressStepView>,
    #[serde(default)]
    pub total_stages: usize,
}
```

JSON example on `/api/status` mid-boot (shape locked by a `server_api.rs`
fixture assertion; field values illustrative):

```json
{"progress": {
  "state": "loading",
  "phase": "[2/5] Indexing translation units...",
  "percent": 26, "message": "6/24 TUs",
  "total_stages": 5,
  "steps": [
    {"stage":1,"label":"Loading artifacts...","percent":100,"message":"Include graph ready","status":"done"},
    {"stage":2,"label":"Indexing translation units...","percent":30,"message":"6/24 TUs","status":"active"},
    {"stage":3,"label":"Reducing semantic model...","percent":0,"message":"","status":"pending"},
    {"stage":4,"label":"Building call graph...","percent":0,"message":"","status":"pending"},
    {"stage":5,"label":"Analyzing collateral...","percent":0,"message":"","status":"pending"}
  ]}}
```

### Snapshot producer - crates/cli/src/commands/ui/boot_progress.rs (new)

```rust
pub struct BootStepSnapshot { /* total_stages, one slot per stage */ }

impl BootStepSnapshot {
    /// All rows `pending`, labels seeded from the phase constants
    /// (`strip_stage_prefix(ARTIFACTS_PHASE .. COLLATERAL_PHASE)`).
    pub fn seeded(total_stages: usize, labels: &[&'static str]) -> Self;
    /// Apply one report: rows < stage become Done (percent 100, keeping the
    /// message of that stage's last frame); row == stage becomes Active with
    /// percent `step_percent(stage, report.percent)` and the report message;
    /// stage == total at step-percent 100 is Done (the finalize rule copied
    /// from `StepBarRenderer::frame`). Frames older than the active stage
    /// are ignored. Pure: returns the row list for the snapshot write.
    pub fn apply(&mut self, report: &ProgressReport) -> Vec<ProgressStepView>;
}
```

Wiring in `crates/cli/src/commands/ui.rs` (touched, not restructured):

- `run_ui`: after `UiState::new_loading`, write the seeded snapshot
  (`steps`, `total_stages = STAGES_PER_RUN`) into `state.progress` once.
- `make_boot_progress`: own a `Mutex<BootStepSnapshot>` beside the bar
  closure; per report, write globals exactly as today PLUS the row list,
  under the same single write-lock pass.
- Error arm and `swap_into_state`: unchanged writes (they assign only
  globals); rows freeze by construction.
- Watcher refresh path: untouched (quiet plain-line progress fn, no rows).

### Frontend - crates/ui/assets (contracts)

- `main.js` keeps `showProgress` / `updateProgress` / `hideProgress` names
  (static_html asserts + standalone callers rely on them) and adds
  `renderProgressSnapshot(progress)`:
  - `progress.steps` empty/missing -> legacy single-bar path (the current
    `loadingProgress` width behavior) - the honest fallback.
  - else render one `.step-row` per step: `[stage/total_stages]` chip,
    label, 20-cell bar (`filled = Math.floor(20 * percent / 100)`, fill
    glyph + light-glyph remainder, monospace), right-aligned `pp%`,
    message. Status classes `step-done` / `step-active` / `step-pending`
    (pending: muted, no detail text).
- `index.html` overlay: keep `loadingOverlay`, `loadingProgress`,
  `loadingText`, `loadingDetail` (legacy fallback UI, hidden when rows
  render) and add a `loadingSteps` container.
- `api.js`: the `state === 'loading'` poll branch calls
  `renderProgressSnapshot(status.progress)` (replacing the
  `updateProgress(percent, phase + ': ' + message)` call); the standalone
  export branch builds the same-shaped local snapshot with 3 synthetic steps
  and routes it through the same renderer (no second renderer).
- `styles.css`: `.step-row`, `.step-index`, `.step-label`, `.step-bar`,
  `.step-percent`, `.step-detail` + status modifiers - `var()` tokens only.

## Module map

| File | Change |
|---|---|
| `crates/ui/src/dto.rs` | + `ProgressStepStatus`, `ProgressStepView`; extend `ServerProgress` |
| `crates/ui/src/server.rs` | touch both `ServerProgress` literals (new_loading, new_with_data); `/api/status` otherwise unchanged |
| `crates/cli/src/commands/ui/boot_progress.rs` | NEW snapshot + unit/parity tests |
| `crates/cli/src/commands/ui.rs` | wire seed + snapshot write in `make_boot_progress`; declare module |
| `crates/ui/assets/index.html` | overlay markup (+ `loadingSteps`) |
| `crates/ui/assets/main.js` | `renderProgressSnapshot` + bar cell math |
| `crates/ui/assets/api.js` | poll branch + standalone branch feed the snapshot |
| `crates/ui/assets/styles.css` | token-only step-row styles |
| `crates/ui/tests/ui_test/static_html.rs` | update overlay test, add formula/width + token-only assertions |
| `crates/ui/tests/ui_test/server_api.rs` | steps mid-boot, serde back-compat, error freeze |
| MUST NOT touch | `crates/pipeline/**`, `crates/cli/src/commands/progress.rs`, `themes.css`, `theme.js`, `tests/fixtures/**`, `tests/golden/**`, `tools/theme_token_baseline.json`, verdict/heat CSS tokens, `/api/*` routes other than the `ServerProgress` shape carried by `/api/status` |

## Risks and mitigations

- **R1 DOM-less parity gap.** Nothing renders the browser in CI, so I7 is
  formula-source-locked, not pixel-locked. Mitigation: one formula, one
  width constant, static_html assertion; the Rust-side I2 oracle covers the
  data half. Residual risk accepted; revisit if a headless harness lands.
- **R2 Snapshot vs terminal drift after future edits.** Mitigation: I2
  equivalence test fails loudly if `format_step_bar` or `StepBarRenderer`
  finalize semantics change under the snapshot's assumptions.
- **R3 `static_html.rs` breakage.** The overlay test asserts ids and
  `status.progress.state === 'loading'`-style source needles; the api.js
  needle changes. Mitigation: update assertions in the same handoff as the
  assets edit (planner: keep test + asset in one scope).
- **R4 Struct-literal breakage.** `ServerProgress` is built by name in 2
  places; new fields are compile errors, not silent defaults. Compiler-
  caught; listed so the planner sizes `crates/ui` backend work honestly.
- **R5 Steampunk legibility.** Pending/done rows on the dark-brass theme are
  only guarded for hex-drift, not taste. Mitigation: token-only styling
  (same tokens the toolbar uses); flag for a manual three-theme eyeball
  during review.
- **R6 Poll/write race.** One RwLock, one writer per report, reader clones -
  unchanged pattern, poison-tolerant `write_state`. No new lock held across
  heavy work (snapshot `apply` is pure in-memory before the write).
- **R7 Empty-repo edge.** `tu_index_percent(0,0)` puts step 2 straight at
  100 (existing honest contract, pinned in
  `test_tu_index_events_stay_in_stage_two_span`); the browser shows step 2
  done immediately, same as the terminal. No new behavior, but call it out
  in the walkthrough.

## Hygiene notes (ceilings this change grows)

- `crates/cli/src/commands/ui.rs` is at 754 lines; wiring adds ~40-60.
  Extraction NOW planned: step-snapshot logic lives in the new
  `ui/boot_progress.rs` (mirrors the existing `ui/node_maps.rs` pattern),
  keeping `ui.rs` growth to wiring only.
- `crates/ui/src/dto.rs` is at 852 lines; the DTO addition is ~45-60 ->
  still under 1000, but it is the file's last comfortable DTO block; note it
  for the next ui-crate milestone.
- `styles.css` is 1422 lines and invisible to the R2 gate (known ledger
  deferred finding); this adds ~40 token-only lines - do NOT also grow
  `index.html` inline styles beyond the container div.
- `main.js` 513 -> ~560; `index.html` 562 -> ~575. All functions here stay
  far under 100 lines; no line approaches 2000 chars (step-row text is
  composed, not parked on one source line).
- Authored content pure ASCII (`░`/`█` glyphs live in the CLI's existing
  string literals; if the browser needs them, they already ship in the
  served JSON-free JS - keep them there, none in Rust).

## Split seams (for the planner; not handoffs)

1. `crates/ui` backend: DTO + constructors + server_api/dto tests. Small.
2. `crates/ui` frontend: assets + styles + static_html updates (needs the
   DTO shape to exist for test fixtures; seams 1+2 may merge - still 1 crate,
   well under the sizing law either way).
3. `crates/cli`: `boot_progress.rs` + wiring + I1/I2/I3 tests, including the
   frozen terminal-parity oracle (module dir `crates/cli/src/commands`,
   <=1 crate). Depends on seam 1's DTO only (it names `ProgressStepView`).

Suggested order 1 -> 3 -> 2 (browser lands last so no intermediate state
ships a half-renderer), but the legacy-fallback path makes 1+3 safe alone.

## Decision points surfaced to the user

- DP-1: pending rows shown dim from boot vs strict CLI parity (rows appear
  only when their step starts). Contract carries both (`status: pending`);
  browser policy. Recommended: dim pending, overlay has the room and `[k/N]`
  already names the total.
- DP-2: bar style - literal 20-cell block-glyph bar (byte-identical CLI
  look, themeable via token text colors) vs smooth CSS width fill.
  Recommended: cell bar, it is the most literal reading of "same bars".
- DP-3: done rows stay accent-colored (CLI-monochrome parity); no semantic
  green checkmarks, keeping verdict colors locked to verdicts.
