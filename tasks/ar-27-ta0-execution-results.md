# AR-27 T-a0 Execution Results - Reviewer Acceptance Pass

Reviewer runs, own stdout (capture-race rule). Landing `43e7de1` + journal
addendum `046e38c` @ HEAD; parent/dispatch `3c814a2`. Tree quiet apart from
foreign closeout-docs churn + two untracked `.kilo/plans/` files.

## Workspace-wide acceptance gates (reviewer's, none carried by the handoff)

| gate | result | evidence line (verbatim) |
|---|---|---|
| workspace census vs anchor `1400/0/13/105` | EXACT — zero drift, no attribution hunt needed | `1400 passed / 0 failed / 13 ignored / 105 suites` (`### cargo test PASS (exit 0) [capture: target\testrun-capture.txt, 2684 lines]`; run after `cargo clean -p` of the four token crates so their test binaries rebuilt from the landed tree) |
| tools unittest discover | PASS | `Ran 219 tests in 21.230s` / `OK` |
| `python tools/hygiene_check.py` | PASS (zero blocking) | output = ONLY `R3 (warn)` rows; zero R1/R2 rows; of the 15 touched files only `crates/build/src/discovery.rs:320` appears (pre-existing line, not a touched line). Raw exit code not echoable on this lane (chained-echo refused); adjudicated on the printed contract per AR-01/AR-03 precedent |
| `python tools/encoding_guard.py --all` | PASS | silent (zero output) |
| `cargo check --workspace --all-targets` | PASS | `ok (exit 0, 31 warnings emitted)` — first (incremental) run: `ok (exit 0, 11 warnings emitted)`; clean-rebuild rerun (after `cargo clean -p ariadne-build -p ariadne-pipeline -p ariadne-cli -p ariadne-artifact`): `ok (exit 0, 31 warnings emitted)` — 31 matches the implementer's gate-1 count; warning class is the pre-existing legacy oracle/goldens set (verdict finding 1) |
| `python tools/testrun.py --baseline` (bare) | NOT RUN — ERRATUM | `--baseline` is a FILE value-flag (tools/testrun.py docstring) and no testrun baseline file exists in the repo; documented bare form exits 2 (ar-01/ar-03/ar-05 execution-results precedent). The workspace census above, quoted verbatim vs the anchor, discharges the milestone gate |

## Scoped pinned suites (reviewer's own runs, all IDENTICAL pre/post)

| suite | canonical line (verbatim) | expectation |
|---|---|---|
| `-p ariadne-pipeline` | `40 passed / 0 failed / 4 ignored / 9 suites` | `40/0/4/9` IDENTICAL; parity in-map `5 passed; 0 failed; 0 ignored` = `5/0/0/1`, five named tests all ok |
| `-p ariadne-cli` | `118 passed / 0 failed / 8 ignored / 7 suites` | IDENTICAL (run via `--distill` — raw-mode echo crashed on the cp1252 console: `UnicodeEncodeError: 'charmap' codec can't encode characters in position 1365-1384` at testrun.py `_echo`; `--distill` preserves the canonical line + exit codes per the tool's own I1 clause) |
| `-p ariadne-build` | `23 passed / 0 failed / 0 ignored / 2 suites` | first-ever pin CONFIRMED — ADOPTED |
| `-p ariadne-artifact` | `40 passed / 0 failed / 0 ignored / 2 suites` | first-ever pin CONFIRMED — ADOPTED |
| `-p ariadne-index` | `15 passed / 0 failed / 0 ignored / 2 suites` | `15/0/0/2` UNEDITED = ADAPT-8 digest proof (plus `git diff 3c814a2 43e7de1 -- crates/index` EMPTY; `cache.rs` blob untouched) |
| integration m23+m22 | `13 passed / 0 failed / 0 ignored / 2 suites` | per-binary capture: line 53 `test result: ok. 6 passed; 0 failed; 0 ignored` (m22 — names incl. token-site test `plan_evolved_..._planned_nothing`), line 65 `test result: ok. 7 passed; 0 failed; 0 ignored` (m23 — incl. token-site test `wired_plan_proposes_scala_extraction`); split `6/0/0/1` + `7/0/0/1` IDENTICAL |
| scoped clippy (my own addition, for the R3 transient-warning clause) | `ok (exit 0, 0 warnings emitted)` digest + capture evidence | capture lines 5-21 show `clippy::clone_on_copy` live and UNSUPPRESSED at `crates/build/src/discovery.rs:253` and `:272` (`using 'clone' on type 'SourceLanguage' which implements the 'Copy' trait`); no allow-attribute exists anywhere in the landing diff; CI runs clippy without `-D warnings` (ci.yml:26-30) |

## Acceptance criteria vs what happened

| criterion (contract/R2) | result | evidence |
|---|---|---|
| Token-only purity: all pairs class `x`->`x.clone()` / `*x`->`x.clone()` / `x.into()`->`x.clone().into()` / `From(x)`->`From(x.clone())` | MET | 39/39 pairs class A (31) or B (8); unified diff read pair-by-pair on `git show 43e7de1 -- <file>` per file + `git show --word-diff=plain --unified=0` mechanical second pass — every changed span confined to the token; zero whitespace/literal/assertion/count/ordering edits |
| Frozen files byte-equal after clone-deletion/`*x`-restoration (parity x4, m22 x1, m23 x1, ar25 x1) | MET | per-file quoted outputs in verdict, R2.4 section |
| ZERO assertion/literal/count edits; suite counts identical | MET | all pinned lines above IDENTICAL; 39 deletions = 39 token minus-lines; journal 338 = journal insertions |
| ADAPT-8 comment lines context-only; `for_incremental.rs:143` stays last-use move | MET | comment block :245-252 unchanged context in both build/discovery hunks; :143 appears only as unchanged context after the :141 token line |
| Escape-hatch additions same-class + journaled (D1); if-let-scrutinee attachments (D2) compiler-forced | ACCEPTED | verdict Claim 2 dispositions; journal §3 #17/#39, §5 tiers 2/3 pre-commit |
| Probe: 4 tiers to zero; 39/15 inside ~28-45; deleted-before-commit attestation reproduced; `identity.rs:129` token never committed | MET | `git diff 3c814a2 43e7de1 -- crates/repo crates/lang-api crates/worker-core crates/index` EMPTY (blob-level, stronger than the worktree attestation); derive list `Copy` at `types.rs:141`; `identity.rs:129` = `Self::from(*source)` at HEAD |
| build/artifact FIRST-EVER pins | CONFIRMED + ADOPTED | `23/0/0/2`, `40/0/0/2` (my own runs above) |
| Touch-set = ledger 15 + journal exactly | MET | `git diff --name-only 3c814a2..HEAD` = 16 paths, verdict section; addendum `046e38c` journal-only |
| D3 byte-exact original indentation (m23) | MET | blob-level: word-diff change span at m23:406 confined to `registry.plugin_for_id(...)` token; indentation identical |
| D4 `--staged` timing disclosure | BENIGN | commit_scoped stages atomically; `--all` superset silent pre-commit (gate 8) and in my own post-landing run |
| Workspace census EXACT `1400/0/13/105` | MET | `1400 passed / 0 failed / 13 ignored / 105 suites` — no drift, no attribution hunt |
| tools discover | MET | `Ran 219 tests in 21.230s` / `OK` |
| Report shape (digest-shaped, distilling entry points) | MET | journal is digest+capture-path shaped, zero raw dumps; all gate commands via `testrun.py --distill` / `distill.py --` |

## NOT RUN / substituted (explicit statements)

- `python tools/ui_smoke.py` — OUT per dispatch (milestone smoke not required
  on this hop; zero UI/runtime surface: token-only inert diff, census exact).
- `python tools/bench_compare.py` — OUT per dispatch (same rationale).
- `python tools/testrun.py --baseline` (bare form) — ERRATUM, not runnable:
  `--baseline` requires a FILE argument and no repo baseline file exists
  (ar-01/ar-03/ar-05 precedent). Substituted: raw workspace census quoted
  verbatim against the `1400/0/13/105` anchor — EXACT.
- Raw-mode `testrun.py -p ariadne-cli` — crashed in the tool's own raw echo
  on the cp1252 console (`UnicodeEncodeError` at `_echo`); substituted with
  the documented `--distill` mode (canonical line and exit codes identical
  per the tool's I1 clause). Pipeline ran raw successfully; its canonical
  line is quoted raw.
- Hygiene/encoding raw exit codes — cannot be echoed on this lane (chained
  `echo`/`$LASTEXITCODE` forms refused); adjudicated on printed contracts and
  absence of blocking output, same as every prior verdict in this lane.
- `cargo check` clone-warning confirmation at the check level — N/A by
  construction: `cargo check` does not run clippy lints (verdict finding 1);
  replaced by the scoped `cargo clippy -p ariadne-build --all-targets` run
  which shows the warnings live and unsuppressed.

## Refused commands (verbatim)

Three refusals this review, all catch-all boilerplate regardless of true
cause: `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`

1. `git diff 43e7de1 HEAD --stat; python tools/hygiene_check.py; echo
   "hygiene exit: $LASTEXITCODE"; python tools/encoding_guard.py --all; echo
   "encoding exit: $LASTEXITCODE"` — refused (chained-echo form) -> split
   into single commands, all three then ran.
2. `(Get-Item "crates/build/src/discovery.rs").LastWriteTime = Get-Date;
   (Get-Item ...).LastWriteTime = Get-Date; git status --porcelain` — refused
   -> attempted single touch `(Get-Item "crates/build/src/discovery.rs").
   LastWriteTime = Get-Date` — ALSO refused (property-set form) -> replaced
   with `python tools/distill.py -- cargo clean -p ariadne-build
   -p ariadne-pipeline -p ariadne-cli -p ariadne-artifact` (legitimate cargo
   route to force warning re-emission).
3. `write` of a byte-level token-diff helper script into the pre-approved
   temp dir — refused by my own reviewer edit grant (`{"permission":"edit",
   "pattern":"*","action":"deny","source":"agent"}`, tasks/**+`.kilo/plans/**`
   only) -> R2.4 recipe run directly on `git show` output (unified pair read
   + `--word-diff=plain --unified=0` mechanical pass), which covers the same
   byte-equality decision without a scratch script.

All git invocations read-only (`status`, `log`, `show`, `diff`, `rev-parse`
via checkpoint_guard). One non-allowlisted-attempt note: the first
`--word-diff` invocation placed flags after the pathspecs (swallowed as
pathspecs, plain diff printed) — my invocation error, corrected, not a
refusal; second run produced the quoted word-diff.
