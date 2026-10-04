# Kickoff - UI/CLI Boot Progress Parity

Milestone key: `ui-progress-parity`. The ledger row (number and dates) is
assigned by the `milestone-lifecycle` kickoff; the ledger tracks no M21+ rows
and this artifact does not guess a number. Artifacts: this kickoff,
`tasks/ui-progress-parity-shape.md` (the shape the planner sizes from).

## Intent

**Behavior change.** The UI boot overlay must present the same progress the
CLI terminal step-bars present: one bar per pipeline step, each filling 0->100
*within its own step*, each showing the CLI's details: `[k/N]` step index,
step label, per-step percent, and the in-flight message (e.g. `12/24 TUs`,
`21 symbols analyzed`, `Payload assembled`).

**Blast radius.** `crates/ui` (progress DTO + its two constructors + frontend
assets + ui tests), `crates/cli/src/commands` (boot progress producer +
parity tests). Nothing else.

**Why.** The UI today shows one thin bar driven by the *global* pipeline
percent plus a `phase: message` text line, so a boot that spends 70% of its
time in stage 5 looks like a crawl to 100% and hides which step stalls. The
CLI already solved this with per-step bars (A4 calibration). Same boot, two
surfaces, two stories is confusing. Parity means: the browser can draw the
same committed step rows the terminal commits.

## Scope

In:

- `crates/ui/src/dto.rs`: extend `ServerProgress` additively with per-step
  rows (`steps`, `total_stages`); add `ProgressStepView` + status enum.
- `crates/ui/src/server.rs`: update the two `ServerProgress` struct literals
  (`UiState::new_loading`, `UiState::new_with_data`) for the new fields.
  `/api/status` itself changes nothing else (it nests `ServerProgress` by
  value; serde carries the new fields).
- `crates/cli/src/commands/` : a boot step-snapshot producer next to
  `make_boot_progress` (new `ui/boot_progress.rs`), fed by the same
  `ProgressReport` stream that drives `TerminalStepBars`, plus the
  terminal-parity equivalence tests. `run_ui` seeds the snapshot before boot.
- `crates/ui/assets/` : overlay step-row renderer (`index.html` markup,
  `main.js` renderer, `api.js` feed, incl. the standalone-export synthetic
  3-step feed routed through the same renderer), token-only styling in
  `styles.css`.
- Tests: update `crates/ui/tests/ui_test/static_html.rs` + `server_api.rs`
  progress assertions; add the CLI-side parity test and DTO serde tests.

Out (with reasons):

- Background-refresh progress plumbing. The CLI deliberately prints ONE PLAIN
  LINE (no bars) during watcher refresh because stdout is shared; the UI's
  `refreshChip` is that surface's equivalent and the overlay is gone by then.
  Parity at refresh already holds. Reopening the overlay mid-session is a UX
  change nobody asked for.
- `crates/pipeline` (`ProgressEvent`, `LoadMilestone`, notify sites). The
  event set is sufficient; the mapping stays owned by `crates/cli`
  (`stage_progress_callback`). Touching it would widen the gate surface for
  zero parity gain.
- `themes.css` / `theme.js`. The `--cyan` accent token family exists in all
  three themes (D7: steampunk reuses the token *name* as its brass accent),
  so new styling needs no token changes and zero new hex.
- The CLI terminal renderer itself (`crates/cli/src/commands/progress.rs`).
  It is the model being imitated; it does not change.
- `tests/golden/**` (does not exist on `main`), `tests/fixtures/**`,
  `tools/theme_token_baseline.json`, every route other than `/api/status`.

## Invariants

Numbered, contracts and named tests in `tasks/ui-progress-parity-shape.md`.

1. **Per-step scale.** A row's percent is the step's own 0-100
   (`step_percent(stage, global)`); it reaches 100 only at that step's own
   terminal milestone. Never a cumulative percentage. (Locks the established
   semantics; no relitigation.)
2. **Terminal-parity equivalence.** Replaying any report stream into
   `TerminalStepBars` and into the boot snapshot yields matching values:
   each committed terminal line's `(stage, label, step-percent, message)`
   equals the corresponding done row; the live terminal line equals the
   active row.
3. **Row monotonicity.** Row percent never decreases; status never returns
   from `done`.
4. **Readiness contract intact.** `progress.state` values/transition
   (`loading` -> `ready` | `error`) and the legacy `percent`/`phase`/`message`
   globals mirror `ProgressReport` exactly as today.
5. **Additive serde.** `ServerProgress` JSON without the new fields
   deserializes (defaults empty/0).
6. **Error freeze.** The error path freezes rows as-is; it never marks a row
   done.
7. **Bar arithmetic parity.** Browser filled-cell count ==
   `floor(20 * percent / 100)`, the CLI `STEP_BAR_WIDTH` arithmetic.
8. **Theme safety.** Zero new hex literals; rows styled by existing `var()`
   tokens only; verdict/heat semantic colors untouched.
9. **Pipeline untouched.** `crates/pipeline` byte-identical.

## Frozen locks inherited from the ledger

| Lock | Proof command |
|---|---|
| `tests/fixtures/**` frozen (all branches) | `git diff <base> -- tests/fixtures` empty |
| `tools/theme_token_baseline.json` hex snapshot; no new literals justified outside themes.css | `python tools/theme_token_check.py --check-added --base <rev>` |
| `/api/status` readiness path `progress.state == "ready"` is the product contract smoke relies on | `python tools/ui_smoke.py` |
| `tests/golden/**` byte-locked, but NOT on `main` | nothing to resync; `git ls-tree <base> tests/golden` empty on main |
| UTF-8 / PowerShell edit discipline (no `Set-Content`, pure-ASCII authored content) | `python tools/encoding_guard.py --all` |
| Code hygiene R1/R2 blocking | `python tools/hygiene_check.py` |
| Planning Law (handoff <=1 crate/module dir, <=~10 gates, <=~1200 diff lines) | planner sizing; `review-gate` commit-vs-touch-set |

## Acceptance gate for the milestone

All of:

- G1 `python tools/testrun.py --distill -p ariadne-cli -p ariadne-ui` green,
  including the new boot-snapshot parity test, updated `static_html.rs`
  overlay test, extended `server_api.rs` status test, and the DTO
  back-compat serde test.
- G2 The frozen terminal-parity equivalence oracle (shape invariant 2)
  passes against the UNCHANGED terminal renderer.
- G3 `python tools/ui_smoke.py` green (readiness + `no-store` route table
  unchanged).
- G4 `python tools/theme_token_check.py --check-added --base <rev>` reports
  zero added hex literals in the milestone's touch set.
- G5 `python tools/hygiene_check.py` reports no new R1/R2 blockers;
  `python tools/encoding_guard.py --all` clean.
- G6 `python -m unittest discover -s tools` green (no tool regressions).
- G7 Milestone close: workspace-wide `python tools/testrun.py --baseline`
  in a fresh session / the orchestrator's bounded run, per the sizing law.
