# AR-01 Handoff - Path-traversal containment + export script escape (S1 CRIT, S3 HIGH, S4-partial MED)

Wave 0 / Security Hotfix. Single handoff.

## Objective

1. **Path traversal containment (S1 CRITICAL)**: Canonicalize `repo_root.join(...)` inputs
   in `api_source` (`/api/source`) and assert `starts_with(canonical repo_root)`. Reject
   absolute `file=` paths and `local_prefix=` escapes (or sandbox prefix into repo).
2. **Export JSON escape (S3/S4)**: In `export.rs`, post-serialize `<` -> `\u003c`, U+2028/2029
   in the embedded JSON. Add an attribute-safe escape helper used by shared renderers.

## Impact

- Closes the only confirmed whole-disk read primitive in the product.
- Removes script tag break-out from shareable export artifact.
- Legitimate same-repo file viewing is unchanged (byte-identical responses for contained paths).

## Touch-set (hard)

- `crates/ui/src/server.rs` — `api_source` function (lines 288-315)
- `crates/ui/src/export.rs` — `export_standalone_html` function (lines 40-55)
- `crates/ui/assets/file-view.js` — use the helper (if applicable, minor)
- `crates/ui/assets/hotspots.js` — use the helper (if applicable, minor)
- `crates/ui/tests/ui_test/server_api.rs` — new security lock tests
- `tasks/ar-01-handoff-journal.md` — checkpoint journal

## Constraints

- TDD: red-first. Add failing test for path traversal attempt before implementing fix.
- All file edits pure ASCII only (repo UTF-8 hazard rule).
- Use Read/Edit/Write tools, never PowerShell file writes.
- Commit via `python tools/commit_scoped.py --scope <glob> -m "<msg>"`.
- Gates via distilling entry points only (`testrun.py --distill`, `distill.py --`).

## Gate commands

1. `python tools/testrun.py --distill -- -p ariadne-ui` (scoped ui tests)
2. `python tools/ui_smoke.py` (route + no-store assertion)
3. `python tools/encoding_guard.py --staged`
4. `python tools/hygiene_check.py`

## Invariants / locks

- Byte-identical responses for contained (legitimate) paths.
- Export payload JSON must escape `<`, U+2028, U+2029 to prevent script injection.
- `starts_with(canonical repo_root)` assertion must reject any path that escapes repo.

## Reviewer-gate reference

`review-gate` skill: commit-vs-touch-set EXACT enforcement, encoding + hygiene + locks.
