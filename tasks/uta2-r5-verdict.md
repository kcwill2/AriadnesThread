# UTA2-R5 verdict — APPROVED (2026-10-19)

Handoff: `tasks/uta2-r5-handoff.md`. Commit: `765df7b` (single, base `59abbf0`,
9 paths `+680/−0` — see finding 1 re the reported `+777`). Journal:
`tasks/uta2-r5-journal.md`. Spec: `docs/specs/unit-test-audit.md` Round 2, R5
bullet (:272-276). Verdict: **APPROVED** — 3 minor findings, none blocking;
both declared deviations ACCEPTED on independent merits.

## Fresh canonical lines (reviewer's own runs, verbatim)

- `python tools/testrun.py --distill -- -p ariadne-graph` ->
  `174 passed / 0 failed / 0 ignored / 12 suites` (capture 256 lines)
- `python tools/testrun.py --distill -- -p ariadne-lang-cpp` ->
  `201 passed / 0 failed / 0 ignored / 2 suites` (capture 277 lines)
- `python tools/testrun.py --distill -- -p ariadne-lang-scala` ->
  `257 passed / 0 failed / 0 ignored / 2 suites` (capture 270 lines)
- `python tools/testrun.py --distill -- -p ariadne-cli` ->
  `178 passed / 0 failed / 8 ignored / 8 suites` (capture 254 lines)
- `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 296 tests in 25.295s` / `OK`
- `python tools/distill.py -- python tools/hygiene_check.py` ->
  `### generic PASS (exit 0)` — R3 warn-only, ZERO warns inside any committed
  file; `python tools/distill.py -- python tools/encoding_guard.py --all` ->
  `### generic PASS (exit 0)` (capture 0 lines).

Baseline-count arithmetic all reconciles: graph 170(T1)+4=174; lang-cpp
193(T2)+8=201; lang-scala 254(T3)+3=257; cli 177(T7b)+1=178; suites 11+1=12
(only the graph pin is a tests-dir file; inline modules add no suite).

## Commit-vs-touch-set

`git diff --name-status 59abbf0..765df7b`: 9 paths —
`crates/graph/tests/uta2_r5_aggregation_consumers_test.rs` (A),
`crates/lang-cpp/src/tokenizer.rs` (M, +3),
`crates/lang-cpp/src/tokenizer/raw_string_multibyte_tests.rs` (A),
`crates/lang-cpp/src/plugin.rs` (M, +3),
`crates/lang-cpp/src/plugin/plugin_probe_arm_tests.rs` (A),
`crates/lang-scala/src/boundary.rs` (M, +3),
`crates/lang-scala/src/boundary/boundary_pin_tests.rs` (A),
`crates/cli/src/commands/transform.rs` (M, +73),
`tasks/uta2-r5-journal.md` (A).
All inside the touch-set (the handoff parenthetical "test-module additions +
tests dir only" is the naming authority for the tests-dir file). MUST-NOT
clean: repo/lang-api/worker-core/scheduler/build/cluster/artifact/transform/
validate/ui appear NOWHERE in the range. Commit message = handoff `-m` literal
with the sanctioned truth-adjust `+ transform arm table`.

## Directed adjudications

1. **graph `map_aggregation_consumers` pins — CORRECT.** Read
   `graph.rs:321-360`: `closures.get(&tu_id)` skip (:342), per-header
   `tu_list.sort()` (:355), and the BFS-oracle doc (`:264-268`) fixing that
   the seed is excluded from `transitive_includes` (SCC DP byte-equivalence
   locked by `closure_equivalence_tests`). Pin 1 `len()==2` (no self-entry)
   matches the seed-exclusion rule; pin 2 ascending `[5,10]` vs descending
   input locks :355; pin 3 map-equality + neither-key-nor-consumer is the
   strongest honest form of the :342 skip; pin 4 covers both empty graphs.
   Call-site citation verified: `planner.rs:165-172` None-arm.
2. **multibyte raw-string pins — HONEST.** Specimen char built via
   `String::from_utf8(vec![0xC3, 0xA9])` (raw_string_multibyte_tests.rs:9) —
   pure-ASCII authoring, no literal mojibake; `encoding_guard --all` exit 0
   covers the tracked blobs. Assertions match tokenizer.rs semantics:
   byte-only `raw_string_open` (:326-336, `quote_byte == 0` guard — the
   handoff's `raw_string_open("\"abc", 0) == false` appears verbatim :50-53),
   raw emits NO tokens (:348 — no brace leak), char-view column math
   hand-traced (trailing token col 19 char vs 20 byte; glued col 11;
   multiline line 2 col 6). The glued `éR` probe genuinely discriminates
   byte-vs-char indexing (char-indexed probe reads 0xA9 ≠ `R`).
   scan_quote's pending-run drop is documented at :207-208 — the glue pin
   locks the documented rule, quirk preserved not unified.
3. **probe deviation — ACCEPTED as an honest pin.** Independent probe
   `python tools/distill.py -- cargo test -p ariadne-lang-cpp -- --version`
   -> `### ... FAIL (exit 101)` + `error: Unrecognized option: 'version'`:
   libtest rejects `--version` on this toolchain exactly as journalled. Probe
   code (`plugin.rs:77-93`) never inspects exit status and falls back via
   `.unwrap_or(binary)` (:88), so the spawn-Ok/empty-stdout test binary
   lands `Available(current_exe)` — pinned exactly (:75-79). The banner arm
   the handoff intended IS separately pinned by the cmd.exe real-banner pin
   with the expected value recomputed from an independent spawn (no
   machine-dependent literal), and the findstr.exe pin locks the same
   fallback row deterministically. Self-flagging drift design (red-for-
   re-pin if libtest ever implements `--version`); invariant (never
   Missing/panic) holds either way. Missing-arm lock
   (`probe_of_absent_binary_is_missing_and_graceful`, plugin.rs:349) byte-
   untouched: plugin.rs diff is a 3-line trailing `#[cfg(test)] mod` append,
   0 deletions.
4. **boundary frozen-4 + 14-row table — COMPLETE.**
   `EXTERNAL_PATH_MARKERS == ["lib","target",".ivy2",".cache"]` content+order
   + len-4 (boundary.rs:13 vs doc contract :8); predicate is exact-match
   `contains` (:16-18); all 14 rows hand-verified false across every
   discrimination axis (superstring, case, truncation, dot-absence,
   whitespace, empty); handoff's named `targets`/`LIB` rows present.
5. **ROW-5 VERIFICATION — implementer's evidence independently CONFIRMED.**
   The three H7a-era tests (`cli_command_tests.rs:97/:119/:162`) are pure
   `Cli::parse_from` + field asserts — clap parse-shape only. T7b's
   route-table seam covers analyze/includes/plan dispatch only
   (lib.rs seams :349/:435/:485; pins at cli_command_tests.rs:408+) — no
   transform arm. All four m20 rows exist
   (`tests/integration/tests/m20_apply_migrate_characterization_test.rs:136/
   :150/:235/:249`) and cover the preview/apply Err arms — none extract.
   Repo-wide grep: `extract_plan_headers` appears only at its definition
   (transform.rs:31), the dispatch call site (lib.rs:521), and this commit's
   new test — ZERO prior behavioral coverage. The handoff's "if no" branch
   was correctly invoked; the appended table test drives all three arms
   through (1) unlocatable path -> frozen `Failed to locate repository path`
   per arm (:559 — same frozen text class as the T7a-era assert at :503 and
   the m20 locate row) and (2) locatable artifact-free repo -> honest Err,
   never vacuous Ok. Arm bodies verified: `extract_plan_headers` locates
   first (transform.rs:32), so scenario 1 exercises the frozen text.
6. **Pure-additive — CONFIRMED.** numstat: deletions column = 0 on ALL 9
   files. The three src files gained only `#[cfg(test)] mod <file>_tests;`
   declarations (3/0 each, trailing append, production lines byte-untouched);
   transform.rs's +73 is a single hunk appended inside the existing inline
   `mod tests` (hunk context label `mod tests`), zero production lines.

## Findings (all minor, non-blocking)

1. **Report-stat misreport (blocker? no — accuracy).** The implementer's
   RESULT report states "9 paths +777/−0"; ground truth
   `git diff --numstat 59abbf0..765df7b` sums to **+680/−0**
   (73+121+3+80+3+163+3+59+175). Overstatement +97. The journal itself
   carries no total stat. Same class as the T4/T7a numstat findings; closeout
   should use +680.
2. **Comment/code slip in pin1** (`raw_string_multibyte_tests.rs:24` vs
   `:28`): the comment cites the handoff specimen `...e-acute...R"( { } )"`
   but the code builds the glued `...e-acuteR"( { } )"` (middle `...`
   absent). Semantics stay valid (the glued form is a stronger stress of
   byte-vs-char indexing), and the handoff specimen itself IS present
   verbatim in pin2 (:59) — so the journal deviation-2 claim "fixed the
   fixture to the handoff text" holds for the suite but not for pin1's own
   comment. Comment-level correction only.
3. **Windows-only probe binaries, ungated.** `cmd.exe`/`findstr.exe` pins
   have no `#[cfg(windows)]` gate (unlike T7a's junction precedent). No CI
   config exists in-tree and all project gates run on this Windows host, so
   informational only — ledger row for the owner if a non-Windows runner is
   ever added.

## Deviation dispositions

- D1 (libtest `--version` not supported on rustc 1.99.0 -> test-binary probe
  lands `Available(current_exe)` via empty-stdout fallback, pinned exactly):
  **ACCEPTED** — independently reproduced mechanism; honest pin of real
  behavior; intended banner arm separately pinned; drift-flagged.
- D2 (specimen initially mis-authored without its `...`; fixture fixed, red
  was harness-only): **ACCEPTED** — zero semantics involved; see finding 2
  for the one residual comment that still reads the old specimen.

## Ledger routing (closeout deferred-findings)

- `probe_compiler_status` never inspects exit status (crashing binary with
  stdout reports Available) — documented contract, now made visible by the
  findstr/current_exe pins; owner note.
- `extract_plan_headers` has no replan fallback (direct
  `load_plan_artifacts()?`) unlike `load_plan_for_action` — asymmetric arm
  behavior, owner look.
- `scan_quote` glued-run drop (e-acute vanishes with the R) — pinned, not
  unified; existing documented rule.
- Finding 1 stat correction; finding 2 comment correction; finding 3
  platform-gate note.

## Refusals observed (configuration note, not skipped gates)

Catch-all deny boilerplate (`{"permission":"bash","pattern":"*","action":
"deny","source":"global"}`) hit three compound shapes: `git show ... |
Select-String`, `git ls-files | Select-String`, `Get-Item ... |
Select-Object`. Each reran as plain calls or via Read/Glob/Grep tools; the
full `git show 765df7b -- crates/cli/src/commands/transform.rs` blob review
completed unquoted-free. All expected gate evidence obtained; no gate
skipped. No reviewer scratch files created this hand.

## Not run (explicit)

`testrun --baseline`, `ui_smoke`, `bench_compare` — NOT run at this hand
(all-hand precedent; AGENTS.md sizing rule: full-workspace gates belong to
the fresh-session closeout). R5 is the final round-2 hand: the round-2
closeout now OWNS these with the new anchor, `ui_smoke`, and bench, plus
committing this hand's + R1-R4's untracked verdict/results artifacts and the
reviewer-scratch cleanup list (`tasks/r2-probe-driver.py`,
`tasks/r2-probe.py`, `tasks/uta2-r4-review-scan.py`).
