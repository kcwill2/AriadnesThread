# AR-02 Handoff - Localhost default bind + CORS same-origin (S6 HIGH)

Wave 0 / Security Hotfix. Depends: after AR-01 (same `server.rs` file).

## Objective

1. **`cli/src/commands/ui.rs`** binds `127.0.0.1` by default instead of `0.0.0.0`.
2. Add an opt-in `--expose <addr>` flag to the `Ui` CLI command (documented, warns).
3. **`ui/src/server.rs`**: Replace `CorsLayer::permissive()` with same-origin restriction.

## Impact

- Removes the unauthenticated LAN exposure that turned S1/S2/S4 from local risks into
  remote/browser-exfiltration risks.
- Default UX unchanged for normal human launch (browser on same machine).
- Anyone who LAN-accesses the UI must pass `--expose` (one-time behavior change, noted in help).

## Touch-set (hard)

- `crates/cli/src/lib.rs` — add `--expose` CLI arg to `Ui` command
- `crates/cli/src/commands/ui.rs` — use `--expose` value for bind address, default `127.0.0.1`
- `crates/ui/src/server.rs` — replace `CorsLayer::permissive()` with same-origin restriction
- `crates/ui/tests/ui_test/server_api.rs` — lock CORS header behavior
- `crates/cli/tests/**` or inline — lock `--expose` parsing (if test infra exists)
- `tasks/ar-02-handoff-journal.md` — checkpoint journal

## Constraints

- TDD: red-first. Add failing test for default bind + CORS restriction before fix.
- All file edits pure ASCII only.
- Commit via `python tools/commit_scoped.py --scope <glob> -m "<msg>"`.
- Gates via distilling entry points only.

## Gate commands

1. `python tools/testrun.py --distill -- -p ariadne-ui` (scoped ui tests)
2. `python tools/testrun.py --distill -- -p ariadne-cli` (scoped cli tests)
3. `python tools/ui_smoke.py`
4. `python tools/encoding_guard.py --staged`
5. `python tools/hygiene_check.py`

## Invariants / locks

- Default bind is `127.0.0.1`, not `0.0.0.0`.
- CORS must not be permissive (same-origin by default).
- `--expose <addr>` opts back into multi-host binding.
- `--help` documents the `--expose` flag and warns about LAN exposure.
