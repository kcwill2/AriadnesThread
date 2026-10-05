# V1 Journal - M26 boot-progress-visibility (implementer log)

Handoff: `tasks/boot-progress-visibility-v1-handoff.md`.
Start rev (HEAD before first commit): `7aeee44ab1a3006e9e752fde4a4ed7aa5f05d19d`.
All `--base` / diff reconciliation args use this rev.

## Bootstrap findings (targeted reads)

- `TerminalStepBars::with_sink` already exists in the FROZEN `progress.rs`
  (`crates/cli/src/commands/progress.rs:168`): the `*_with_sink` builders
  delegate without touching the frozen renderer. No `progress.rs` diff needed.
- Frozen-test import surface verified against
  `progress_contract_tests.rs:1-8` and `boot_progress_tests.rs:1-8`:
  `stage_report`, `ProgressReport`, `ProgressFn`, `stage_progress_callback`,
  `read_state` stay in `ui.rs`; the percent/phase/span names move to
  `stage_spans.rs` and are re-exported into `ui.rs`'s namespace so the
  frozen `use super::{...}` lines resolve unchanged.
- `pub mod ui` (commands/mod.rs:10); `stage_spans` is a private child module,
  so moved items get `pub` on the move (module-boundary visibility, values
  byte-identical). Child test modules see parent-private `use` bindings,
  which is what the frozen imports rely on today.
- `UiState::new_loading(PathBuf)` and `ServerProgress` fields confirmed in
  `crates/ui/src/server.rs:54`, `crates/ui/src/dto.rs:27` (read-only).

## Deviations register

- (open) `ui.rs` landing size vs the "~700" target - to be measured after
  the bridge lands; hard gate is "smaller than 839".

## Checkpoints

(append 1-3 lines per per-module Green gate below)
