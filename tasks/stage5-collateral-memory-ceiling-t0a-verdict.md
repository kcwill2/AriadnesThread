# Verdict - M28 `stage5-collateral-memory-ceiling` T0a (t0a-rss-bench) - ROUND 2

Reviewer: review-gate skill, fresh verification, second adjudication after
rework. Reviewed at HEAD `69885ce` (main, ahead=0, behind=0; 22 foreign dirty
paths, none inside the touch-set). Rework commits (oldest-first): `407fe66`
(bench: decomposable ratio), `eec5646` (append-only errata + journal REWORK),
`69885ce` (journal hash record). History: round 1 at `8d34a3c` was REJECTED
(blocker: release growth-ratio row unauditable and irreconcilable with the
bench's own peak rows; minors 2-5). This file supersedes it; the round-1
findings' dispositions are below, and the round-1 record is mirrored in
`tasks/stage5-collateral-memory-ceiling-execution-results.md`.

## VERDICT: APPROVED

The round-1 blocker is closed on the merits: the ratio row is decomposable,
self-auditing, and the corrected central reading (4.0) is independently
supported. A fresh reviewer invocation reproduced a pressure-window anomaly —
this time on the FULL operand — which the new format exposed in-invocation
exactly as designed, and which refutes one overclaim in the errata's
mechanism text. That text needs an erratum before the DP-4 evidence review
consumes it (finding 1, review-gate condition); it does not reopen the hop,
because the corrected ratio VALUE, the row grammar, and every gate line hold.

## Round-1 finding dispositions

- Finding 1 (BLOCKER, decomposability): CLOSED. `emit_and_parse_peak_row`
  echoes each child's peak row verbatim and asserts the row contract (one
  pipe, verbatim label, finite positive); ratio label byte-identical; 26-label
  set unchanged; `run_isolated_runner_child` and the children byte-identical,
  so operand measurement is unchanged by construction. Reviewer fresh release
  `--exact` run at HEAD: operands and ratio printed and the quotient
  self-audited (7720.5/2260.2 = 3.416 -> `3.4`). Implementer's five corrected
  decomposed readings (release 2257.5/9033.8/4.0; 2256.7/9042.0/4.0;
  2255.9/9040.2/4.0; 2259.3/9029.5/4.0; debug control 2258.6/9048.7/4.0)
  are arithmetically consistent (all quotients 4.00 within 0.15%).
  Mechanism adjudication: the ENVIRONMENT cause (working-set high-water
  timing, not bench computation) is ACCEPTED — independently confirmed by the
  reviewer's fresh anomalous run on different code state and different side;
  the CONFINEMENT claim is REFUTED — see finding 1.
- Finding 2 (filename): CLOSED as declared. D6 in journal; canonical-path
  header added (pure insertion); planner has ALREADY corrected the T0b
  supplier pointer (`t0b-handoff.md:16-19`, planner-corrected 2026-10-06,
  pointer errata recorded there). No rename — correct (T0b handoff was
  planner-owned; implementer must not touch it).
- Finding 3 (label count): CLOSED. "26 table-row labels" in both docs;
  parameterized-scale parenthetical under the ratio label; multiset note in
  journal label section (see follow-up 3).
- Finding 4 (exemplar/4096): CLOSED with residual nit -> finding 2. Errata
  E2 pair-set arithmetic verified exactly: 9047.2-4951.1=4096.1;
  9035.7-4939.7=4096.0; 9034.4-4938.4=4096.0; 9033.7-4937.6=4096.1.
  Exemplar now in-band; "1/2/4" numbering fixed. The ~4096.0 +/- 0.1 MiB
  save-span transient law STANDS.
- Finding 5 (capture citations): CLOSED. Every rework digest cites its
  capture path; read-immediately-after-run discipline stated and evidenced.
- Scratch experiment (`silent_peak_mib`): proper disposition — never
  committed/staged, grep zero matches in `crates/` at review; committed code
  carries no scratch residue.

## New findings (round 2)

1. [MAJOR-to-evidence-review / non-blocking-for-hop] Errata mechanism text
   is partially falsified by a fresh reviewer witness. At HEAD `69885ce`,
   release `--exact` ratio invocation (exit 0, capture
   `target/distill-capture.txt`): operands `2260.2` / `7720.5`, ratio `3.4`.
   Half in band; FULL operand −14% out of band. E1's claim "Full-scale
   operands never diverged (~9030-9048 across every reading...)" is
   empirically dead; exposure to ambient working-set trimming is NOT confined
   to the short N=640 boot. Required before/at the DP-4 evidence review
   (append-only, same discipline as E1-E4): (a) erratum the mechanism text
   with this witness; (b) extend the journal's proposed >8% operand-divergence
   rejection to BOTH operands at both scales (my run is rejected by that rule
   on the full operand alone — −14% vs the ~9035 per-slot median); (c)
   outlier-frequency witness now ~1-in-7 release ratio invocations this
   session -> the median-of->=5 protocol (journal) is the minimum, and the
   PROPOSED 1.15x tolerance/median protocol for the PEAK bars must be
   ratified as mandatory (peak rows share the susceptibility — trimming shaves
   the high-water, so a trimmed sample biases LOW and a single max/low sample
   is not gate-grade). Nothing here changes the corrected ratio value (4.0,
   median of six clean decomposed readings incl. round-1 peaks 9034.8/2258.7
   = 4.000 and today's debug control 4.0) or the PROPOSED bars (a
   pressure-trimmed 7720.5 still fails the 5200 bar pre-fix; the bar's
   discriminative margin is unaffected).
2. [minor] E2 table row 4 mislabeled: `9033.7 / 4937.6` is release
   invocation 3's partial-capture pair (evidence doc's own protocol section
   attributes both values to release invocation 3), labeled "earlier
   fully-read debug"; and the journal's "(5th confirmation...)" count
   overstates the four-pair table by one. Arithmetic unaffected; fix in the
   finding-1 append.
3. [follow-up -> ledger/planner] Multiset consequence ("peak labels occur
   twice per scale in suite runs; parse as multiset per label") is recorded
   in evidence E1, the journal label section, and the bench test doc-comment
   — but NOT in `tasks/...t0b-handoff.md`'s parse/gate text (grep: zero
   "multiset" matches). T0b is planner-owned and not dispatched; the planner
   must propagate the note when appending the RATIFIED lines (same edit as
   finding-2's pointer era). Recorded here as the relay, per round-1
   disposition style — never dropped.

## Deviations

D6 (filename, declared) ACCEPTED (pointer corrected by planner — closed
loop). Scratch non-commit disposition ACCEPTED. No new undeclared deviations:
rework commits touch only the bench file and the two docs; journal edit to the
ratio-label bullet is an implementation record edit (the append-only rule
binds the evidence doc, which is +110/−0 pure insertion).

## Ratio-row ratifiability statement (for the DP-4 evidence review)

The `stage5 RSS growth ratio` row is ratifiable AS A GATE ROW at value 4.0
(median of the clean decomposed readings; label unchanged; operands now
emitted make every future invocation self-auditing and any outlier
identifiable in-run). It is NOT ratifiable together with E1's robustness/
mechanism text as written — that text is falsified in its "full never
diverged" clause (reviewer witness 2260.2/7720.5/3.4 at `69885ce`) and must
carry the finding-1 erratum, and the outlier protocol must cover both
operands at both scales. All constants remain PROPOSED; zero authored
`RATIFIED:` lines (verified by grep); DP-4 holds.

## Per-commit scope verdict

- `407fe66`: `crates/cli/tests/stage5_memory_bench.rs` only (+28/−27). INSIDE
  touch-set; stayed in `crates/cli/tests`; zero src/Cargo delta.
- `eec5646`: evidence (+110/−0, append-only honored) + journal. INSIDE.
- `69885ce`: journal hash record (+3/−2). INSIDE.
- Full range re-proof `git diff --name-only 5169483..HEAD` = AGENTS.md,
  `tools/ui_smoke.py`, `tools/ui_smoke_tests.py`, t3-journal (T3's approved
  paths) + T0a's three paths: ZERO `crates/*/src`, zero Cargo, zero
  fixtures/goldens, zero `bench_baseline.json`, zero ledger — frozen locks
  byte-identical across T3+T0a+rework; `bench_compare` producer-crate
  exemption RE-PROVEN.

## Gate lines (verbatim, fresh at HEAD 69885ce)

- Scoped cli census: `### cargo test PASS (exit 0)` [capture:
  `target/testrun-capture.txt`, 158 lines] /
  `111 passed / 0 failed / 8 ignored / 6 suites` — expected line, exact.
- Bench probe: `### cargo test PASS (exit 0)` [capture: 15 lines] /
  `1 passed / 0 failed / 7 ignored / 1 suites`.
- Fresh release ratio (`--exact`, distill route): `ok (exit 0, 0 warnings
  emitted)`; capture rows:
  `stage5 peak RSS (release, N=640) | 2260.2` /
  `stage5 peak RSS (release, N=1280) | 7720.5` /
  `stage5 RSS growth ratio (release, N=640->1280) | 3.4` — decomposed,
  self-audit consistent; anomaly = finding 1.
- Workspace census: `### cargo test PASS (exit 0)` [capture: 2190 lines] /
  `1236 passed / 0 failed / 13 ignored / 89 suites` — unchanged vs round-1
  line; zero test-count delta from the rework (one test file, no tests added
  or removed — verified by the delta and the range diff).
- Tools CI-form discover: `### generic PASS (exit 0)` /
  `Ran 194 tests in 21.016s` / `OK` — unchanged.
- Hygiene: `### generic PASS (exit 0)` (capture 152 lines, all pre-existing
  R3 warns; zero rows for the bench file; R2 holds at 995 lines).
- Encoding: `### generic PASS (exit 0)`, capture 0 lines.
- `bench_compare`: NOT RUN — exemption re-proven above; no producer path
  exists to move.
- `ui_smoke`: NOT RE-RUN this round — zero UI/tools/binary paths in the
  rework range (`407fe66..69885ce` diff = one test file + two task docs);
  the round-1 same-day PASS witness plus zero-delta scope covers it.
- Zero-hex/theme guard: NOT APPLICABLE — zero UI paths in any T0a range.

## Methodology-truth statement (round 2, for the evidence review)

The oracle is now honest in both parts: the probe itself was verified live in
round 1 (raw FFI/procfs, semantic red, loud-failure on stubs — unchanged by
the rework), and the ratio row is now honest end-to-end: every invocation
prints the exact operands it divided, asserts their row contract, and
therefore cannot hide an environment-skewed sample from a reader. My own
fresh invocation proved the design works — a −14% pressure-trimmed full-scale
operand produced a visible, arithmetically true 3.4 that any median-of-N +
divergence-rejection protocol will filter — while also proving the review
must treat BOTH scales' peak rows as pressure-susceptible: ratify the
median-of->=5 + operand-divergence protocol as binding on T0b, erratum E1's
"full never diverged" clause (finding 1), fix the E2 label nit (finding 2),
and propagate the multiset note into the T0b handoff (finding 3). With those
three paper-relay items carried into the evidence review, every measured
value in the doc — peaks, checkpoints, memo attribution, the 4096.0±0.1 MiB
save-span transient, and the corrected 4.0 growth ratio — is ratifiable on the
documented protocol, and nothing in this hop has been self-ratified.
