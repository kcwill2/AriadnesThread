# rust-language R-5 - Verdict

Handoff: `tasks/rust-language-r5-handoff.md` (syn analyzer, fingerprint
`"rust-1"`, `semantic_index` flip, INV-7 span truth).
Chain from start rev `c54035d`: duty 1 `80a2e28` + landing `673b22b`
(implementer) + orchestrator rider `8622b2c` (flip-lock amendment).
Reviewed fresh at HEAD `8622b2c`, tree clean (2026-10-26, reviewer
`checkpoint_guard.py status`: HEAD 8622b2c, dirty none, untracked none).

Verdict: **APPROVED** - zero blockers, findings below are minor/recorded.

## Fresh verification (reviewer's own runs, canonical lines verbatim)

| Gate | Result |
|---|---|
| `-p ariadne-lang-rust` | `43 passed / 0 failed / 0 ignored / 2 suites` exit 0 |
| `--test rust_language_semantic_index_test` | `11 passed / 0 failed / 0 ignored / 1 suites` exit 0 |
| `--test rust_language_discovery_test` (post-rider) | `7 passed / 0 failed / 0 ignored / 1 suites` exit 0 |
| `--test m21_scala_semantic_index_test` UNEDITED | `5 passed / 0 failed / 0 ignored / 1 suites` exit 0 |
| Full census `python tools/testrun.py --distill` (bare fail-fast form) | `1916 passed / 0 failed / 13 ignored / 123 suites` exit 0 (capture 3310 lines) |
| Hygiene | `### generic PASS (exit 0)` (capture `target/hygiene-r5-reviewer.txt`, 173 lines; all rows warn-only R3, none names a file of this hand) |
| Encoding | `python tools/encoding_guard.py --all` -> `### generic PASS (exit 0)` (capture 0 lines) |
| Tool suites (CI form) | `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 296 tests in 32.080s` / `OK` |
| Bench (kickoff sec. 4 fully-run) | `python tools/distill.py -- python tools/bench_compare.py` -> `### generic PASS (exit 0)`; `bench compare: 3 run(s), no regressions past 1.25x baseline`; zero WARN lines in capture (PR-2 stance moot) |

Census attribution, done independently against the duty-1 ledger anchor
`1886 passed / 0 failed / 13 ignored / 122 suites @ a9b6864`: +19 lang-rust
lib net (`43 = 24 - 1 stub-pin + 4 analyzer + 1 plugin + 8 parser_items +
7 parser_uses`, all pins counted by name at source), +11 new-suite tests
(11 `#[test]` in `rust_language_semantic_index_test.rs`), +0 rider (restores
the one flip-touched discovery lock to passed). 1886 + 19 + 11 = 1916,
suites +1, ignored flat 13. The implementer's `1915/1/13/123` reconciles to
`1916/0/13/123` by name: their single failure was the stale
`capabilities_lock_the_registration_flip_literal`, fixed by rider `8622b2c`.

## Commit-vs-touch-set

`git diff --name-only c54035d..HEAD` = 10 paths. Nine inside the declared
touch-set (analyzer.rs, lib.rs, parser_items.rs, parser_uses.rs, plugin.rs,
new suite, journal, r4-verdict + ledger for duty 1). The tenth -
`tests/integration/tests/rust_language_discovery_test.rs` - is the rider
commit's file and the ONLY `tests/**` hunk outside fixtures; adjudicated as
rider-of-record below (the implementer correctly never touched it).

Per-commit, oldest first:
1. `80a2e28` (duty 1): `tasks/rust-language-r4-verdict.md` +237 and the
   1-line ledger flip carrying BOTH named clauses verbatim (R-4 LANDED
   clause + `fixture rust-basic APPROVED 2026-10-26 at ec552bd via R-4
   verdict - L3 approval path discharged`). Established duty-1 shape (R-2/R-3/
   R-4 pattern); memory mirror `ledger.milestone.rust-language` verified
   present with the same clauses.
2. `673b22b` (landing): 7 files, +1395/-40, exactly the touch-set six code/
   suite paths + journal inside the same commit (gate 8 "Journal inside").
   Fingerprint `"rust-1"` lands in the FIRST commit shipping an analyzer
   (handoff extra-MUST) - satisfied, both in this commit.
3. `8622b2c` (rider): one file, 5+/3- - see rider disposition.

## Caps + fingerprint literal audit

- Implementation caps struct `plugin.rs:147-157`: `discovery: true,
  semantic_index: true`, six capability bits false, `requires_toolchain:
  false` - byte-exact against the handoff cap.
- THREE independent byte-literals exist and agree: in-src
  `capabilities_match_the_semantic_index_flip_literal` (plugin.rs tests,
  renamed from the R-4 pin, full-struct `assert_eq!`), integration-side
  `capabilities_lock_the_semantic_index_flip_literal` (new suite :165-187,
  full-struct via the registry seam), plus the amended rider lock in
  `rust_language_discovery_test.rs`. All three match the implementation
  literally.
- Fingerprint `"rust-1"` pinned byte-exactly TWICE: in-src
  `analysis_fingerprint_is_rust_1` (`plugin.rs`) and integration-side
  `registered_plugin_fingerprint_is_rust_1` (suite). `CURRENT_ANALYSIS_
  VERSION` untouched (read below).

## syn mechanism + span truth (audit focus)

- `analyzer.rs:51` `syn::parse_file`; item walk matches on `syn::Item`
  variants; spans exclusively via `syn::spanned::Spanned` +
  `proc_macro2::Span::start().line/.column + 1` (`parser_items.rs:138-156`).
  Grep for hand-rolled scanning (`.lines()`, `char_indices`, `find(`,
  `split(`, `matches(`) over `parser_*.rs`: zero hits - no faked spans.
- INV-7 span pins re-read by the reviewer against the frozen fixture bytes,
  ALL EXACT: `lib.rs` `crate::greeting (6,5)`, `crate::models (7,5)` (the
  handoff's named `mod models;` oracle), `crate::fixture_marker (9,8)`;
  `greeting.rs (4,8)`; `models/mod.rs crate::user (4,9)`, `models_marker
  (6,8)`; `models/user.rs` `User (4,12)` (attribute-immune: derive sits on
  line 3), `default_user (8,8)`; `macro_specimen.rs` `written_code_only
  (20,8)`.
- Macro-fabrication audit: every pinned symbol has written source; the
  specimen's `mod generated_by_macro`/`invisible_without_expansion` exist
  only inside the `macro_rules!` body and are pinned absent both by
  full-surface equality and an explicit anti-fabrication assert (suite
  :326-332); in-src pin `macro_items_record_nothing_and_absorb_no_names`
  likewise.
- Pre-split holds: analyzer.rs 143, parser_items.rs 338, parser_uses.rs 219
  (all well under ceilings); suite 425; functions < 100 (hygiene R1/R2 zero).
- `use` facts are raw path + span only (references channel), Incomplete
  stance mirroring Scala; renames record the left side; globs carry `*` at
  the star span - nothing resolved, nothing invented.

## Frozen locks (reviewer-run proofs)

`git diff c54035d..HEAD -- Cargo.toml Cargo.lock crates/index crates/repo
crates/cli crates/pipeline crates/lang crates/lang-cpp crates/lang-scala
tests/fixtures tests/golden tools` -> EMPTY. Fixture `rust-basic` read-only
this hop (zero new bytes; suite copies it into tempdirs - m19/m21 isolation
pattern). m00/m18-m23/ar27 witnessed green UNEDITED inside the fresh census.
L12 encoding exit 0.

D1 read-check: `crates/index/src/cache.rs:30` reads
`CURRENT_ANALYSIS_VERSION = "2.2.0"` - the handoff's "frozen 2.1.0" quote is
STALE (raised by the registered M28 T4 DP-2 exception documented in the
cache.rs doc block itself; m21's UNEDITED suite pins `"2.2.0"`). Zero bytes
touched - confirmed by the empty lock diff above.

## Deviation dispositions

- **D1** (2.2.0 vs handoff's 2.1.0): **ACCEPTED - plan erratum.** Binding
  rule (zero bytes under `crates/index`, epoch never the vehicle) honored;
  the handoff's quoted value was stale at dispatch. Planner note: cite the
  read-value pattern, not a literal, in future freeze clauses.
- **D2** (suite-red run beyond the seven named gates): **ACCEPTED.** The
  handoff's own Red protocol ("Journal the failing list") requires the suite
  red run; journal-first declared; gate 4 is the same invocation green-side.
  8 scoped invocations total, no budget abuse elsewhere.
- **D3** (stale external flip-lock): **ACCEPTED - report path correct,
  resolved by rider.** The file sits outside the touch-set; the implementer
  posted the board ASK, landed in-scope work with evidence, and per "report
  don't fix" never touched it. See rider disposition.
- **D4** (stale `"v1"` seam assertion retired with the override):
  **ACCEPTED - retirement-of-superseded, not weakening.** The old assertion
  was false by construction once the override landed; the fingerprint is now
  POSITIVELY pinned byte-exactly twice (in-src + integration-side), strictly
  stronger than the retired default-seam leg; sibling seam assertions
  (`aggregation_filter`/`target_path_for` `None`) retained untouched.
- **D5** (source-only qualified ids; cross-file module truth deferred):
  **ACCEPTED - design judgment, verified.** The `SemanticAnalyzer::analyze`
  seam genuinely receives only hashed `FileId` + context name (analyzer
  signature + the journal's `worker.rs:95`/`tu_store.rs:208` cites); a file-
  position prefix would be fabricated input - the D-2 class the handoff
  forbids. The handoff item 3 itself defers path resolution to the R-6
  resolver. PARKED as relay (below): R-6 resolver owns cross-file module-tree
  truth via `rel_path`; the `crate::greeting` mod-record/fn-record id-sharing
  class rides the existing `ReducedSemanticModel::reduce` fold and lands for
  adjudication at R-9's symbol-isolation suite.

## Rider disposition (`8622b2c`, rider-of-record)

**ACCEPTED.** Orchestrator commit, one file, 5+/3-: exactly the one literal
`semantic_index: false -> true` inside the R-4-era integration-side lock in
`rust_language_discovery_test.rs`, plus its doc-comment and assert-message
(amendment-style evolution, never weakening - the full-struct byte-exact
`assert_eq!` discipline is intact and now locks the post-flip truth).
Precedent verified on disk: `m19_scala_discovery_test.rs:236` `EVOLVED at
M23 T7 (D-M23-6 flip)` - same shape, name kept, comment records the flip.
Fresh proof: discovery suite `7 passed / 0 failed / 0 ignored / 1 suites`.
The implementer's board-ASKed alternative (leave it red) is census-
incompatible (gate 6 demands failed 0); the touch-set law made the rider the
ONLY lawful path; the implementer's journal had already named this exact
one-liner as the R-6/orchestrator option.

## Transient-noted (Windows fixture-copy race)

The implementer's first bare census died early on
`commands::ui::extraction_apply::tests::test_a_valid_token_applies_the_plan_
end_to_end` ("copy fixture: Os { code: 2, NotFound }"). My fresh bare census
(123 suites, fail-fast, single run) passed it green - NOT reproduced.
Disposition: TRANSIENT-NOTED. Pre-existing cli-side Windows race class
(copying the untouched `basic` fixture), outside this hand's bytes; recorded
here for the closeout's pattern census (observer pattern-count law), not a
finding on R-5.

## Findings

1. (minor, plan-level, no code change) Handoff `CURRENT_ANALYSIS_VERSION`
   freeze clause quotes the stale `2.1.0`; live value `2.2.0` since M28 T4
   DP-2. Planner to use the read-value pattern in R-6..R-10 handoffs.
2. (minor, record) Ledger row 62 continues per-hop duty-1 growth (the
   established duty-1 shape; pre-existing over the 2000-char read-
   truncation convention before this append). Closeout consolidation is the
   natural cure; hygiene/encoding gates exit 0 unaffected. Noted, not
   blocking.
3. (record) Journal Red gate 2 shows `24 passed / 2 failed / 0 ignored /
   1 suites` - the suite-count drop vs the `2 suites` baseline is the
   fail-fast seam (doc-tests never ran after the lib failure); named
   failures and both `left:` values match the handoff's Red prediction
   exactly. Historical Red state (unreproducible post-landing, R-1
   criterion-11 precedent); the Green state I re-ran carries the truth.

## Not run / refused (explicit statement)

- `python tools/testrun.py --baseline` (bare): refused - `usage error:
  missing value for --baseline`. The F9-errata FILE form, superseded
  track-wide (no baseline file exists for this track); the governing fresh
  gate is the `--distill` census above, per the handoff's gate 6.
- `python tools/ui_smoke.py`: not required by this hand (no UI/runtime-
  surface bytes); NOT run.
- `python tools/golden_guard.py --base <rev>`: tool not landed; substituted
  by the handoff's prose byte-check (lock zero-diff + name-only proofs).
- Catch-all-deny refusals (verbatim `{"permission":"bash","pattern":"*",
  "action":"deny","source":"global"}`): chained `git show --stat a; echo ...`
  (substituted by three individual `git show --stat` runs), `python -c`
  probes, `findstr`, `Get-Content .Count` line counts, and one chained
  exit-code probe (all substituted by the Read tool's line totals and the
  distiller's exit-0 lines). No expected gate was skipped.

## Relay line for R-6 duty 1

`R-5 LANDED 673b22b (+rider 8622b2c) APPROVED 2026-10-26, census anchor
1916 passed / 0 failed / 13 ignored / 123 suites @ 8622b2c, R-6 dispatched
(resolver hop).`

R-6 relay notes (from D5 + the macro/`use` stance): the resolver owns
cross-file module-tree truth (`resolve_edges` receives `rel_path` - the one
seam that does) and consumes `parser_uses`' raw facts verbatim (relative,
`self::`, `::`-absolute, glob `*` at star span, rename left-side); the
`crate::greeting` mod/fn id-sharing class is R-9's named adjudication;
`tasks/rust-language-r5-verdict.md` + this file's R-5 addendum land via
R-6's duty 1 (precedent chain), execution-results append-only as always.
