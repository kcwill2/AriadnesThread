# T5 profile-lever Handoff - M27 tu-index-hang-throughput: release launch route + release overflow-checks

Hop id: `t5-profile-lever`. Micro-handoff: one config stanza + doc lines.
Ceilings: 0 crates of src, 5 touched files + journal, 8 gate invocations max,
well under all sizing-law bounds. USER-RATIFIED decision (2026-10-05): land
the release launch route AND evaluate `overflow-checks = true` for the
release profile. This hop discharges the dated OPEN obligation row
(`tasks/milestone_ledger.md` deferred-findings row: "M27 profile-lever
landing (OPEN obligation, NOT closed by recording)").

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/milestone_ledger.md` - M27 row (line 33) + the profile-lever dated
   row in Deferred findings (the obligation being discharged).
3. `tasks/tu-index-hang-throughput-t1-measurement.md` - sec. (a) table
   (:22-33, the T1a medians you compare against), sec. (c) (:81-105, the
   launch-profile finding that names the doc sites), sec. (d-final)
   (:143-173, the ratified ACCEPT; note :157-158).
4. `tasks/tu-index-hang-throughput-t1a-verdict.md` - :117-121 (launch-profile
   finding verification), :190-193 (route-to-named-landing disposition),
   :130-135 (the EXACT release-bench invocation form - copy it verbatim).
5. `tasks/tu-index-hang-throughput-t1a-handoff.md` gate 1 (canonical bench
   invocation, reuse verbatim).

## Objective and invariants

Objective: (A) the normal human launch route points at the RELEASE build at
every doc site the T1a finding named; (B) `[profile.release]
overflow-checks = true` ships in the workspace root, keeping underflow-class
bugs LOUD in release (precedents: M25 `parser.rs` brace-depth, M27
`tokenizer.rs` char/byte - both silent-wrapped in release); (C) the cost of
(B) is measured once, honestly, against the T1a medians, and reported.

Invariants (must hold at landing):
- I1 debug behavior unchanged: `overflow-checks` is already ON in debug;
  debug suites are bit-identical before/after by construction (the flag
  names ONLY `[profile.release]`).
- I2 `tools/ui_smoke.py` NOT touched - it stays debug (overflow-checks
  already on there; PR-2/bench binding untouched).
- I3 no new tests, no test edits; census line MUST be UNCHANGED after this
  hop: `1235 passed / 0 failed / 6 ignored / 88 suites`.
- I4 T1a audit artifacts append-only: sec. (a)-(d-final) + Deviations of the
  measurement doc never edited; only a NEW dated section is appended.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/tu-index-hang-throughput-t5-profile-lever-journal.md` (checkpoint
  journal; always in-set; create alongside)
- `Cargo.toml` (workspace root; add the `[profile.release]` stanza - it does
  not exist today, verified 2026-10-05 at HEAD: file ends at
  `[workspace.dependencies]` rayon, :42)
- `AGENTS.md` (Build section, :74-78: one dated bullet pointing the normal
  human launch at the release build)
- `EXTENSIONS.md` (:20-23 and :111-113 blocks: release variant line each)
- `extensions/ariadne-extension/README.md` ("Start the Analysis Server"
  block, :79-82: release variant line; anchor away from the emoji heading
  lines - do not re-write them)
- `tasks/tu-index-hang-throughput-t1-measurement.md` (APPEND-ONLY: one new
  dated section "(e) Post-flag release cost (T5, 2026-10-05)" with the
  single-run table + per-stage % cost vs sec. (a) medians; NEVER edit
  anything above the appended section)

You MUST NOT touch: `tools/**` (incl. `ui_smoke.py`, `bench_compare.py`,
`bench_baseline.json`), `crates/**` (all src/tests/benches), `tests/**`,
`Cargo.lock` (a profile stanza cannot change it - zero-diff is a lock),
`tasks/milestone_ledger.md` (planner-owned; the executor's memory save is
the orchestrator's relay, not this hop), `tasks/tu-index-hang-throughput-t1a-*`.
No `git stash`, no branch change.

## Frozen locks

Everything else byte-identical: `tests/golden/**`, `tests/fixtures/**`
(incl. `tests/fixtures/m27-nonascii-underflow/`),
`crates/cli/src/commands/progress.rs`, `progress_contract_tests.rs`,
`crates/pipeline/tests/boot_stages_bench.rs` + `common/mod.rs` (the T1a
bench sources), `tools/bench_baseline.json`. Proof:
`git diff --name-only <start-rev>..HEAD` must equal the touch-set exactly
(reviewer-enforced commit-vs-touch-set). Note: `overflow-checks` under
`[profile.release]` affects ONLY the release profile - goldens/fixtures
suites (all debug) cannot move; state that one-liner in the journal.

## What to build (exact edits)

1. `Cargo.toml` - append at end of file (exact lines, pure ASCII):

   ```
   [profile.release]
   # AriadnesThread: keep integer underflow LOUD in release. Two incidents
   # (M25 parser.rs brace-depth, M27 tokenizer char/byte columns) both
   # silent-wrapped in release; debug already checks (M27 T0 proof).
   # Cost measured at T5: see tasks/tu-index-hang-throughput-t1-measurement.md sec. (e).
   overflow-checks = true
   ```

2. `AGENTS.md` Build section - add one bullet after :77 (anchor
   `- Use \`cargo build --release\` for release builds`):

   `- Normal human launch uses the RELEASE build (\`cargo build --release\`
   first, or add \`--release\` to any documented \`cargo run\` line): debug
   indexing measured ~20x slower than release at TU scale (M27 T1a
   launch-profile finding 2026-10-05; E1's ~3.1 s/TU was the debug profile)`

3. `EXTENSIONS.md` - inside each of the two code blocks (:22 and :113, both
   currently `cargo run --bin cli --features server`), add above the
   existing line:
   `cargo build --release --features server`
   and change the run line to `cargo run --release --bin cli --features server`
   (minimal surface: same command plus `--release`; keep one debug-form line
   visible below the block as the "quick debug" alternative with a dated
   note). Do not invent new flag semantics.
4. `extensions/ariadne-extension/README.md` :81 (`cargo run --bin cli --
   features server`) - same treatment: `--release` variant, debug form kept
   as labeled alternative.
5. Measurement doc - append section (e): header, method one-liner (single
   release run of the T1a bench under the new flag, same invocation as T1a
   gate 1), the 8-label table (`label | post-flag ms | T1a median | delta
   %`), and the STOP bar outcome line.

## RGB protocol (config-item carve-out)

Config/doc hop - no compile-failing Red test exists and adding a behavior
lock is UNNECESSARY (debug already checks; I1). The machine-checkable
substitute, in order:
1. RED-equal: before any edit, journal-record that `overflow-checks` is
   ABSENT from `Cargo.toml` (Read/grep-tool proof) and that the launch docs
   are debug-de-facto (cite measurement sec. (c)) - the incoherent state
   this hop exists to close.
2. GREEN: make the edits in "What to build"; run gates 1-5; never weaken an
   assertion (none are touched).
3. REFACTOR/doc pass: re-read added lines for honesty (no claim exceeds the
   evidence: "~20x slower" is E3/T1a-grounded - 151.4 ms/TU release vs E1
   ~3.1 s/TU debug), hygiene exit 0, journal.

## Measurement obligation (binds before commit)

After the flag lands, ONE release run of the T1a bench, invocation copied
verbatim from `tasks/tu-index-hang-throughput-t1a-handoff.md` gate 1 (the
form the reviewer re-ran at t1a-verdict :130-131; allow a full release
rebuild - first run recompiles every dep under the new profile; timeout
600000 ms, distill route). Emit the 8 release labels + the tu-index warm
row into measurement sec. (e) as the single-run table above.

STOP-AND-REPORT bar (concrete): any stage whose post-flag single value
exceeds its T1a median by MORE THAN +10 percent -> STOP, journal + report,
do NOT commit; you may spend ONE confirming re-run on the tripped stage
counts; if the re-run lands inside the T1a recorded run spread for that
stage (e.g. W=1 spread 56563-68495 ms), that is NOT a trip - record both
numbers and proceed. Within-bar results: report the % table; the REVIEWER
adjudicates final acceptance of the flag (they own the accept call; a
uniform +2..+8% cost is acceptable on the loud-panics rationale, a
stage-level cliff is not).

## Gates (exact commands, timeout 600000 ms each; all via distilling entry
points; MUST lines below are quoted from GREEN-tree runs - row-65 lesson,
never from fail-fast truncations)

1. `python tools/testrun.py --distill -p ariadne-pipeline` -> MUST print
   `25 passed / 0 failed / 4 ignored / 6 suites` (t1a-verdict :222, green).
2. `python tools/testrun.py --distill -p ariadne-cli` -> MUST print
   `110 passed / 0 failed / 1 ignored / 5 suites` (M26 V3 verdict, green).
3. Release bench run per the Measurement obligation above -> exit 0, eight
   release labels present in the capture; transcription promise honored
   (capture file is overwritten by later distill calls - transcribe sec.
   (e) while fresh).
4. `python tools/testrun.py --distill -- --workspace --no-fail-fast` ->
   MUST print exactly `1235 passed / 0 failed / 6 ignored / 88 suites`
   (unchanged census is I3).
5. `python tools/hygiene_check.py` -> exit 0.
6. `python tools/encoding_guard.py --staged` -> exit 0 before committing.
7. Commit (direct route - sanctioned precedent T0b `4ef8412` / T2 D1; the
   distill-wrapped form is allowlist-refused for commit_scoped):
   `python tools/commit_scoped.py --scope Cargo.toml --scope AGENTS.md --scope EXTENSIONS.md --scope extensions/ariadne-extension/README.md --scope tasks/tu-index-hang-throughput-t1-measurement.md --scope tasks/tu-index-hang-throughput-t5-profile-lever-journal.md -m "feat(release): overflow-checks=true for release profile + release launch route (M27 T5 profile lever; cost table in measurement sec. (e))"`
   Total invocations: 7 (+1 confirming re-run only if the bar trips) <= 8.

## Hygiene / ceilings at risk

Files < 1000 / functions < 100: none at risk (docs + 6-line config stanza).
The real ceiling here is AUDIT integrity: the measurement doc's existing
sections are the frozen T1a gate artifact - append-only discipline (I4) is
the hygiene target; a single edited line above the append is a blocker, not
a fix. Added doc lines: pure ASCII, short-line rule (< 2000 chars), no
emoji, dated citations for every performance claim.

## Discipline clauses

- UTF-8: Write/Edit tools only, never Set-Content/Out-File/pipes; pure
  ASCII content; the extension README already contains emoji - anchor edits
  away from those lines and do not rewrite them.
- Touch only the touch-set; out-of-scope breakage (e.g. any doc command that
  seems semantically wrong, such as the `--bin cli` name vs the binaries
  declared in `crates/cli/Cargo.toml`) -> journal + report-only, do NOT fix.
- No bare high-volume commands; everything through distilling entry points
  (docs/specs/tool-output-distillation.md). `bench_compare` NOT RUN -
  PR-2-bound to T1b, exemption stands (this hop's bench obligation is the
  ignored cargo test form, not bench_compare).
- Checkpoint journal after each Green step (1-3 lines). Context exhaustion:
  STOP, return disk-state inventory; successor uses
  `python tools/checkpoint_guard.py resume --spec
  tasks/tu-index-hang-throughput-t5-profile-lever-handoff.md ...`.

## Acceptance (machine-checkable verdict lines)

- Gates 1/2/4 print their MUST lines verbatim, exit 0.
- Gate 3 exit 0 with 8 release labels; sec. (e) table complete, every delta
  within the +10% bar (or a documented STOP-and-report exchange).
- `git show --stat <landing>` = exactly the six touch-set paths.
- Census UNCHANGED: `1235 passed / 0 failed / 6 ignored / 88 suites`.
- Reviewer gate: adjudicate via `review-gate` skill (per-commit oldest-first
  `git show`, commit-vs-touch-set vs this file, locks byte-identical, and
  the reviewer OWNS the flag-acceptance call on the % table). On APPROVED:
  relay to planner to discharge the ledger row 67 obligation and record the
  sec. (e) outcome; the ledger anchor stays `1235/0/6/88`.

## Sequencing (why this rides BEFORE PR-2 + T1b, not after)

`overflow-checks = true` changes release codegen. T1b lands the
`bench_baseline.json` boot-stages rows via operator `--update-baseline` on
VERIFIED-IDLE (PR-2). Baseline and gate runs MUST share one profile regime:
T5 -> PR-2 baseline refresh -> T1b, in that order, or the fresh baseline
inherits a step-change and re-opens the row-61 flapping-gate saga. This hop
therefore CLOSES as the M27 pre-T1b rider; it does not wait for T1b.

## Report back (final message)

1. Commit hash + `git show --stat`. 2. Red-equal proof line (flag-absent
  pre-state) + doc-before state. 3. Gate 1-6 canonical lines + exit codes +
  capture path for gate 3. 4. The sec. (e) delta table (labels/medians/delta
  %) + bar outcome. 5. Deviations (declare any) + report-only observations
  (e.g. `--bin cli` name drift). Never paste raw tool output.
