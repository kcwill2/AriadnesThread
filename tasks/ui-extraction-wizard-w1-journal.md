# W-1 Journal - ui-extraction-wizard: dto.rs module split

Handoff: `tasks/ui-extraction-wizard-w1-handoff.md`. Track: `ui-extraction-wizard`.
Start rev: HEAD `97dd836` + duty-1 planning commit `5cc4ac3` (docs-only; no
`crates/**` diff between them). Tree clean under `crates/**` at boot
(`checkpoint_guard.py status`: dirty=0, untracked=0).

## RGB protocol record

- Red: DELIBERATELY SKIPPED - declared D1 below.
- Green: perform the move; scoped ui canonical line must be identical.
- Refactor: none by contract (the move itself is the refactor).

### D1 - Red-skip declaration (per M28 T0b finding-1 dated-row law)

This hand is a pure module move under a "no test change" bound (shape module
map: "no test change"). There is no failing assertion that can be authored
without editing a test file, which would violate the touch-set; a vacuous one
would be theater. The refactor oracle is the UNEDITED scoped ui suite staying
green (frozen locks: `proposals_payload.rs`, `static_html.rs`,
`export_order.rs`, `dto_builders.rs`, the card-family locks,
`extraction_contrast.rs`, theme oracles) plus the reverse-dependency compile
proofs on gates 3-4 (`ariadne-cli`, `ariadne-integration-tests`).

## Gates

- Gate 1 (baseline, BEFORE the move):
  `python tools/testrun.py --distill -p ariadne-ui`
  -> `### cargo test PASS (exit 0) [capture: target/testrun-capture.txt, 326 lines]`
  -> canonical line: `283 passed / 0 failed / 0 ignored / 3 suites` exit 0.
  Start tree green - move authorized.

## Pre-cut verification (handoff: "verify on disk before cutting")

- `crates/ui/src/dto.rs` is exactly 1000 lines at HEAD - confirmed by read-to-EOF.
- Family boundary check (grep across the repo):
  - No function/helper in `dto.rs` consumes the proposal types; the builders
    live in `crates/cli/src/commands/ui.rs` and reach them ONLY through the
    `ariadne_ui::` root (proof of the public-path invariant).
  - `GraphExportPayload.extraction_proposals` (dto.rs:314) is the payload
    root's field - stays in `dto.rs` per the handoff ("dto.rs keeps the
    payload root"); it resolves through the new `pub use extraction::*;`.
  - Shared helpers (`path_basename`, `serialize_sorted_map`, justification
    caps) stay in `dto.rs`; none of them is proposal-family-only.
  - No test file reads `dto.rs` as source text (no source-string lock can be
    broken by the move).
- Byte check on the move region (lines 234-286): the only non-ASCII bytes are
  two LEGITIMATE em dashes (U+2014, bytes E2 80 94) at lines 248 and 262 -
  single-encoded, not mojibake (encoding_guard --all passes on HEAD). The
  verbatim move carries them byte-identical; see deviation D3.
- Anchor drift: the handoff/shape cite "dto.rs:182-184" for the
  "plain display data only" rule; on disk at `97dd836` that rule lives inside
  the moved structs' own doc comments at lines 248 and 262. The module doc
  carries the rule forward from the disk text. See deviation D4.

## Checkpoints

- Gate 2 (AFTER the move): `python tools/testrun.py --distill -p ariadne-ui`
  -> `### cargo test PASS (exit 0) [capture: target/testrun-capture.txt,
  313 lines]` -> `283 passed / 0 failed / 0 ignored / 3 suites` - IDENTICAL
  to the gate-1 canonical line. Green.
- Gate 3: `python tools/distill.py -- cargo check -p ariadne-cli
  --all-targets` -> `ok (exit 0, 0 warnings emitted)`. Public-path proof:
  the live reverse dependency compiles against the unchanged `ariadne_ui::`
  surface.
- Gate 4: `python tools/distill.py -- cargo check
  -p ariadne-integration-tests --all-targets` -> `ok (exit 0, 21 warnings
  emitted)`. All 21 warnings are pre-existing in untouched files
  (integration golden-helper dead code, deprecated AR-21 `IncludeResolver`
  call sites, `unused variable: gen` at the untouched
  `crates/ui/src/routes/api_graph.rs:719`); NONE names `dto.rs` or
  `dto/extraction.rs`.
- Gate 5: `python tools/hygiene_check.py` -> exit 0; output carries ONLY
  warn-only R3 findings repo-wide; zero findings in `dto/extraction.rs`
  (68 lines); `dto.rs` (953 lines, under the 1000 ceiling) carries the three
  pre-existing R3 single-letter `s` warnings from the untouched
  justification functions (line numbers shifted by the move only); no R1/R2
  findings anywhere - zero NEW findings in the touched files.
- Pure-move byte proof (throwaway script in the temp dir, not the repo,
  run via the distiller; HEAD blob read byte-level with `git cat-file`
  inside python per the UTF-8 discipline): the 53-line moved block in
  `dto/extraction.rs` is BYTE-IDENTICAL to `HEAD:crates/ui/src/dto.rs`
  lines 234-286 (`BYTE-IDENTICAL: True`). Non-ASCII inventory unchanged:
  the two legitimate em dashes moved with the block (extraction.rs lines
  30/44); `dto.rs` keeps only its pre-existing legitimate non-ascii lines.
- Gate 6: `python tools/encoding_guard.py --staged` -> exit 0 (silent).
  Vacuous-by-construction pass: raw `git add` is allowlist-denied and
  `commit_scoped.py` owns staging (same pattern as the duty-1 planning
  commit and the branding-assets gate-4 verdict precedent); gate 8 `--all`
  is the real post-commit check.
- Line budget: ~130 produced diff lines (move + header + module doc),
  inside the ~300-450 estimate band's ceiling; gates used: 8, exactly the
  handoff's enumerated list.
- (pending) Gate 7: scoped landing commit via `commit_scoped.py`.
- (pending) Gate 8: `encoding_guard.py --all` post-commit.

## Deliberately NOT done

- No test file touched (frozen-locks invariant; zero new assertions).
- No rename, reorder, doc rewrite, or visibility widening inside the moved
  block; `JUSTIFICATION_MAX_NAMES`/`JUSTIFICATION_MAX_HEADERS` stay private
  consts in `dto.rs` (shared with the justification family) and are imported
  by the submodule rather than widened - see D2.
- No ui_smoke/bench gate applies: this hand is Rust-source-only with a zero
  assets/JS diff by construction (stated per the discipline clause).

## Deviations (numbered)

1. D1: Red skipped - declared above per the handoff's RGB section.
2. D2: the submodule header carries two mechanical `use` lines
   (`use serde::{Deserialize, Serialize};` for the verbatim derive
   attributes; `use super::{JUSTIFICATION_MAX_HEADERS,
   JUSTIFICATION_MAX_NAMES};` because the verbatim cap definitions reuse the
   parent's private justification caps). No content edit on either side; the
   parent consts keep their exact declaration text.
3. D3: the verbatim move carries two pre-existing legitimate em dashes
   (U+2014) at source lines 248/262 into `dto/extraction.rs`. The pure-move
   bound overrides the new-authoring ASCII preference for MOVED bytes; all
   NEWLY authored lines (module docs, `use`/`mod` lines, this journal) are
   pure ASCII. Byte-identity is proven by the pure-move diff and by
   `encoding_guard.py --staged` / `--all` exit 0.
4. D4: planning-anchor drift reported (not fixed): shape/kickoff cite
   `dto.rs:182-184` for the plain-display-data rule; disk placement at
   `97dd836` is inside the moved struct docs (248/262). The handoff's own
   instruction "verify on disk before cutting" governed.
