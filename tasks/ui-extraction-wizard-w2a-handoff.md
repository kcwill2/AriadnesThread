# W-2a Handoff - ui-extraction-wizard: read-only engine-backed provider (cli)

Depends on W-2b landed (this provider implements `ExtractionManifestReader`
and returns the wizard DTOs W-2b added).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/ui-extraction-wizard-shape.md` sections 1 (capability rows 4, 7, 8),
   2 (W2/W3/W4), 3 (C2 provider contract, C4 dry-run semantics), 8 (R3 cache),
   9; `tasks/ui-extraction-wizard-kickoff.md` sections 3-5.
3. `crates/ui/src/routes/api_extraction.rs` (the trait; verify its exact shape
   before implementing - W-2b is the authority).
4. `crates/cli/src/commands/ui.rs`: `swap_into_state` at :537 (the
   `install_analysis` call at :539), the watcher swap at :605, the boot call
   at :723, `UiState::new_loading` at :795; the boot plan computation the
   shape cites at :383-436 (read it; plans are computed then dropped today).
5. Model files for the engine surface (READ ONLY, never modified):
   `crates/transform/src/engine/mod.rs:95-128` (`required_deps`),
   `crates/transform/src/placement.rs:12-71` (`ConsumerRewrite` /
   `MigrationPreview`), `crates/cli/src/commands/transform.rs:81-127`
   (`build_migration_preview`) and :216-235 (`plan_rewrite_targets` passes
   empty deps on the preview path - the gap this provider closes).
6. `crates/cli/src/commands/ui.rs` `#[cfg(test)]` modules (`builder_tests`,
   `target_files_tests`): the fixture-bootstrap idiom to mirror when
   constructing the provider in tests.

## Objective and preserved invariants

Implement the provider that answers the wizard's two routes read-only, and
wire it into the boot and watcher swap so every generation installs a reader
for its own epoch. This is the gap closure for per-file required includes:
today the includes exist only inside the isolation engine at extract time and
the CLI preview path deliberately passes empty deps.

Invariants this hand must preserve:
- W2 read-only: running a manifest or a dry-run over a fixture leaves the
  fixture byte-identical (a directory-hash before/after is a required test).
  No write call exists in the provider.
- W3 engine-sourced includes: every `required_includes` entry comes from
  `GeneratedUnit.required_deps`; when the engine cannot resolve deps the list
  is empty and `includes_unavailable` is true. Never fabricate an include.
- W4 copy-vs-move honesty: on a C++ fixture the dry-run reports
  `move_removals == 0` and `copy_only == true` (the C++ backend does not
  strip moved declarations today).
- C4 dry-run parity: the provider reproduces `build_migration_preview`
  exactly - the same `plan_rewrite_targets`, the same consumer discovery, the
  same `IncludeRewriter::build_preview` - and a test compares the route-level
  DTO against the CLI preview on a fixture.
- W9 boot payload untouched: a test snapshots the serialized
  `extraction_proposals` payload bytes for the standard fixture and proves
  them byte-identical to the starting-tree capture (green byte-oracle
  precedent: captured from live code at Red time, before any mutation).
- R3 cache: the dry-run is cached per (plan id, epoch) behind a mutex; a test
  calls it twice and proves the consumer discovery ran once.
- The reader is installed inside the same `swap_into_state` sequence as
  `install_analysis` (after it), for boot and the watcher swap alike, so a
  reader can never outlive its generation.
- `m00_facts_path_parity` stays green: the planner is untouched; this hand
  adds no planning run and changes no plan output.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- tasks/ui-extraction-wizard-w2a-journal.md (checkpoint journal)
- crates/cli/src/commands/ui/extraction_reader.rs (NEW: the provider; its
  `#[cfg(test)]` tests may live here)
- crates/cli/src/commands/ui.rs (the `mod` line, the install call inside
  `swap_into_state`, and bounded test additions only)

You MUST NOT touch: `crates/ui/**` (W-2b already shipped the trait; this hand
consumes it), `crates/transform/**`, `crates/validate/**`,
`crates/cluster/**`, `crates/lang*/**`, `crates/artifact/**`, `tools/**`,
`tests/**` (fixtures and goldens frozen). No `git stash`, no branch change.

## What to build

`crates/cli/src/commands/ui/extraction_reader.rs` implements
`ExtractionManifestReader` over the analysis generation:

- `epoch()` returns the generation epoch the reader was built for.
- `manifest(plan_id)` resolves the plan by id within that generation and
  builds `ExtractionManifestDto`: per destination file the engine-sourced
  symbol names (symbol_id order) plus `symbol_count` (uncapped), the engine's
  `required_deps` rendered repo-relative (lossy-rendered) as
  `required_includes`, `includes_unavailable` true iff the engine could not
  resolve deps; caps enforced here
  (`EXTRACTION_MANIFEST_MAX_FILES` files, `EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE`
  symbols per file) with `files_truncated` honest; `cycle_error` carries the
  engine unit-graph cycle when one exists.
- `dry_run(plan_id)` builds `ExtractionDryRunDto` through the same code path
  `tool apply` uses (`build_migration_preview`): `new_files` are the paths the
  apply would write, `rewrites` map `ConsumerRewrite` fields
  (`consumer_file`, `old_includes`, `new_includes`,
  `side_effect_preservations`) capped at
  `EXTRACTION_DRY_RUN_MAX_REWRITES` with `rewrites_truncated`, plus
  `move_removals` / `copy_only` and `includes_unavailable`.
- Caching: `dry_run` memoizes per (plan id, epoch) behind a mutex (R3: the
  consumer discovery is the expensive half).
- `ui.rs` change is bounded: the `mod` declaration, the reader construction +
  `install_extraction_reader` call in `swap_into_state` right after
  `install_analysis` (target: about 30 lines of call sites; shape section 9),
  and the install's test wiring.

## RGB protocol (per module, in this file order)

Red: author the provider tests first, with the module compiling under an
honest-absent implementation: `manifest()` returns `Some(dto)` with empty
`files` and `includes_unavailable: true`, `dry_run()` returns `Some(dto)` with
empty `rewrites` and `new_files` (a real, wrong implementation - never
`todo!()`/`unimplemented!()`; the honest-absent fallback is the documented W3
degraded mode). The tests fail on the first real assertion. Expected failures
(shape): the fixture manifest test reports the missing per-file entries (for
example `assertion failed: files.len() == <n>` or an empty
`required_includes` where the fixture's inter-unit dependency is known), and
the dry-run parity test reports the empty rewrite list against the CLI
preview's known rewrites.
Green: wire the engine and the rewriter; re-run until exit 0.
Refactor: extract the shared DTO-mapping helpers; every function under 100
lines; `extraction_reader.rs` under 1000 lines.

## Required tests (in extraction_reader.rs's test module)

1. Fixture manifest: known plan -> per-file symbol names (symbol_id order),
   `symbol_count` uncapped, engine-sourced `required_includes` non-empty for
   the fixture file with a known inter-unit dependency (W3).
2. Fixture dry-run: `rewrites` equal `build_migration_preview`'s rewrites for
   the same plan (old/new includes and side-effect preservations field by
   field), and the new-file paths match the apply's write set (C4).
3. Cache: two `dry_run` calls -> the consumer discovery ran once (R3).
4. Read-only proof: a directory hash of the fixture tree before and after
   manifest + dry-run calls is identical (W2).
5. Copy-vs-move: on a C++ fixture, `move_removals == 0` and
  `copy_only == true` (W4).
6. Boot payload byte lock: the serialized `extraction_proposals` bytes for
   the standard fixture equal the capture taken at Red time from the starting
   tree (W9); the capture is verbatim from live code, never hand-written.
7. Epoch: the reader reports the generation epoch it was built for, so a
   stale reader is refused by the route layer (W6 surface, tested at W-2b).

## Hygiene (review-enforced)

Functions < 100 lines, files < 1000 lines, no banned identifier names, no
single-letter locals, intention-revealing names (shape section 9's provider
names are binding where they apply). `crates/cli/src/commands/ui.rs` is 885
lines at HEAD: the fee for this hand is call sites only (about 30 lines), so
it must land under 1000 - if the honest wiring needs more, STOP and report
rather than growing `ui.rs`.
`python tools/hygiene_check.py` exit 0.

## Frozen locks

- `crates/ui/**` diff-empty (W-2b shipped the contract; this hand compiles
  against it, never edits it).
- `tests/fixtures/**` and `tests/golden/**` byte-frozen; the W2 test reads
  fixtures and writes nothing. `UPDATE_GOLDENS=1` is FORBIDDEN.
- `m00_facts_path_parity` green UNEDITED.
- `tools/theme_token_baseline.json` diff-empty (no asset touched).

## Gates (exact commands, timeout 600000 ms each; scoped only)

1. `python tools/testrun.py --distill -p ariadne-cli` (baseline canonical
   line recorded in the journal)
2. `python tools/testrun.py --distill -p ariadne-cli` (Red: exit 1; named
   failing tests verbatim)
3. `python tools/testrun.py --distill -p ariadne-cli` (Green: exit 0;
   canonical line = baseline + the new tests)
4. `python tools/testrun.py --distill -p ariadne-ui` (the consumer crate
   still green after the install call sites)
5. `python tools/distill.py -- cargo check -p ariadne-cli --all-targets`
   (exit 0: bench and test targets compile against the changed ui.rs)
6. `python tools/hygiene_check.py` (exit 0)
7. `python tools/encoding_guard.py --staged` (exit 0)
8. `python tools/commit_scoped.py --scope
   crates/cli/src/commands/ui/extraction_reader.rs --scope
   crates/cli/src/commands/ui.rs --scope
   tasks/ui-extraction-wizard-w2a-journal.md -m "feat(cli): read-only
   extraction manifest/dry-run provider over the isolation engine"` (never
   `git add .`)
9. `python tools/encoding_guard.py --all` (post-commit, exit 0)

Gate count: 9 (commit-cycle trio is gates 7-9, itemized per the M28 T0b
finding-1 journal law). This hand touches `crates/*/src`, so the assets-only
bench exemption does NOT apply: bench duty is the reviewer/operator hop per
the kickoff lock table (the PR-2 stale-baseline row governs).

## Discipline clauses

- UTF-8: Write/Edit tools only; pure-ASCII authored content; no em dashes.
- Touch only the touch-set; out-of-scope breakage: report, do not fix. The
  engine and rewriter are READ-ONLY models: if the honest implementation
  needs a change inside `crates/transform/**`, STOP and report - that is a
  contract discovery, not a license.
- Bare high-volume commands are a reviewer blocker: distilling entry points
  only.
- Checkpoint the journal after each Green gate.
- Context exhaustion: STOP with a disk-state inventory; successor resumes via
  `python tools/checkpoint_guard.py resume --spec
  tasks/ui-extraction-wizard-w2a-handoff.md --scope ... --cmd ...`.

## Report back (final message)

1. Commit hash + `git show --stat`. 2. Red evidence (named failing tests,
  verbatim panic lines). 3. Green evidence (digest header + canonical line +
  capture path). 4. Deviations + reasons. 5. Out-of-scope observations
  (including any engine-surface friction, report-only). No raw tool output.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces the touch-set above exactly.
