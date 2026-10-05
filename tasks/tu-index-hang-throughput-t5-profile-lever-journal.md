# T5 profile-lever journal - M27 tu-index-hang-throughput: release launch route + release overflow-checks

Hop: `t5-profile-lever`. Handoff:
`tasks/tu-index-hang-throughput-t5-profile-lever-handoff.md`.
Start rev: `41f7ad4` (main; M27 T1a APPROVED ledger closeout).
Date: 2026-10-05. USER-RATIFIED: land the release launch route AND evaluate
`[profile.release] overflow-checks = true`.

## RED-equal record (pre-edit, config-item carve-out per handoff)

No compile-failing Red test exists for this config/doc hop (debug already
checks; I1). Machine-checkable substitute, recorded BEFORE any edit:

- Flag-absent proof: grep tool on `Cargo.toml` for `overflow-checks` ->
  0 matches; for `profile` -> 0 matches (no `[profile.*]` stanza at all).
  File ends at :42 (`rayon = "1"` under `[workspace.dependencies]`),
  matching the handoff's verified pre-state.
- Launch docs debug-de-facto: measurement doc sec. (c) :81-105 - AGENTS.md
  Build default is `cargo build`, `ui_smoke.py` builds+launches
  `target/debug`, `EXTENSIONS.md` x2 + extension README say
  `cargo run --bin cli ...` (no profile flag). Verified against the live
  tree at `42d4a6e` by the T1a verdict (:117-121); re-verified by Read at
  `41f7ad4` at this hop's start.
- This is the incoherent state the hop closes: users launch debug (E1's
  ~3.1 s/TU) while underflow-class bugs silent-wrap in the unreleased
  release profile (M25 brace-depth, M27 tokenizer precedents).

## Green: edits (handoff "What to build" items 1-5)

1. `Cargo.toml` - `[profile.release] overflow-checks = true` + 4-line
   rationale comment, appended at end (exact handoff lines, pure ASCII).
2. `AGENTS.md` Build section - one dated bullet after
   `- Use \`cargo build --release\` for release builds` pointing the normal
   human launch at the release build (~20x claim is E3/T1a-grounded:
   151.4 ms/TU release vs E1 ~3.1 s/TU debug).
3. `EXTENSIONS.md` block :22 - release build+run lines inside the block;
   debug form kept visible below as labeled quick-debug alternative.
4. `EXTENSIONS.md` block :113 - same treatment.
5. `extensions/ariadne-extension/README.md` :81 block - same treatment;
   debug form kept verbatim as labeled alternative; emoji headings
   untouched (edit anchored inside the bash block).
6. Measurement doc - sec. (e) appended AFTER gate 3, transcribed from the
   fresh capture (append-only; nothing above the append touched - I4).

Profile-scope one-liner (frozen-locks proof): `overflow-checks` under
`[profile.release]` affects ONLY the release profile - goldens/fixtures
suites (all debug) cannot move. `Cargo.lock` cannot change from a profile
stanza (zero-diff holds; verified absent from the landing commit).

## Report-only observations (discipline clause: journal + report, NOT fixed)

- `--bin cli` name drift: `crates/cli/Cargo.toml` declares bins `ariadne`
  and `tool` - no bin named `cli` exists. Every doc route copied here
  says `--bin cli`. Pre-existing drift, named as report-only by the
  handoff itself; commands copied verbatim, only `--release` added.
- `EXTENSIONS.md` block 1 says `cd crates/ui` before `cargo run --bin cli`
  - `crates/ui/Cargo.toml` is a pure lib (no `[[bin]]`), so the cd is
  wrong twice over. Report-only; not touched.
- Extension README :81 literal is `cargo run --bin cli -- features server`
  (stray `--` puts `features server` behind cargo's arg separator).
  Debug line kept verbatim as the labeled alternative per handoff; the
  new release lines use the EXTENSIONS.md-correct `--features` form.

## Gate log

(gates appended as run; MUST lines quoted from GREEN-tree runs)

- Gate 1 `testrun.py --distill -p ariadne-pipeline` exit 0:
  `25 passed / 0 failed / 4 ignored / 6 suites` (MUST verbatim).
- Gate 2 `testrun.py --distill -p ariadne-cli` exit 0:
  `110 passed / 0 failed / 1 ignored / 5 suites` (MUST verbatim).
- Edits 1-5 in place (Green step 2): Cargo.toml stanza :44-49; AGENTS.md
  bullet :78-81; EXTENSIONS.md blocks :20-27 and :115-126; README block
  :80-86. Refactor/doc pass: all added lines re-read; every perf claim
  dated and cited; pure ASCII.
- Gate 3 (release bench, ONE run, distill route, verbatim T1a gate-1
  invocation) exit 0; capture
  `C:\Users\Kurt\Code\AriadnesThread\target\distill-capture.txt` - all
  eight release labels present. Deltas vs sec. (a) medians:
  discover+hash 232.3 (+3.4%), build discovery 1.0 (0.0%), fingerprints
  0.7 (0.0%), include-graph cold 1240.3 (+167.1% - TRIP, T1a spread
  460.9-491.2), include-graph warm 46.1 (+4.1%), tu-index W=1 58389.0
  (-3.6%), tu-index W=N 86201.9 (+0.6%), tu-index warm 3082.6 (-3.0%).
- TRIP record (STOP-AND-REPORT bar): include-graph cold single value
  exceeds the +10% bar. NOT committing until adjudicated. One confirming
  re-run spent per handoff allowance (invocation #8). Candidate benign
  cause (hypothesis, not conclusion): the include-graph cold row is
  I/O-bound and a first-touch disk/page-cache state is the usual suspect
  for a row like this; overflow-checks slowing a 464 ms I/O-dominated
  stage by 2.7x is mechanically implausible for integer-arithmetic
  checks. Re-run decides: inside 460.9-491.2 = NOT a trip (record both
  numbers, proceed); outside = STOP-AND-REPORT, no commit.
- Confirming re-run (invocation #8, allowed form, same command) exit 0:
  include-graph cold 478.3 ms - INSIDE the T1a recorded spread
  460.9-491.2 -> NOT A TRIP per the handoff rule. Both numbers recorded
  (1240.3 first run / 478.3 re-run); proceeded. Re-run context values
  (non-gating): 217.2 / 1.0 / 0.7 / 42.0 / 61274.2 / 78005.8 / 2718.9.
- Sec. (e) appended to `tasks/tu-index-hang-throughput-t1-measurement.md`
  (APPEND-ONLY: header, method one-liner, 8-label table with deltas vs
  sec. (a) medians, STOP-bar outcome with both re-run numbers; nothing
  above the append touched - I4 honored; single-run table transcribed
  from the fresh gate-3 capture before overwrite).
- Gate 4 `testrun.py --distill -- --workspace --no-fail-fast` exit 0:
  `1235 passed / 0 failed / 6 ignored / 88 suites` (census UNCHANGED,
  I3 MET).
- Gate 5 `hygiene_check.py` exit 0: R3 warn-only output, every finding
  pre-existing or foreign (incl. `tasks/v3-review-*.py` throwaways from
  the ledger row-66 purge queue); zero R1/R2 findings; zero findings in
  any touch-set file. The first chained exit-code echo hit the global
  catch-all (row-60 class boilerplate); each gate was then attempted
  independently, hygiene re-run standalone with capture - warnings-only,
  clean exit.

## Invocation census

- Handoff-enumerated gates 1-7: 7 invocations (direct `commit_scoped.py`
  route as sanctioned; the distill-wrapped form is allowlist-refused).
- Confirming bench re-run: +1 = 8 total gate invocations (allowed only
  because the +10% bar tripped; resolved NOT A TRIP).
- Support calls outside the gate list: 1x `checkpoint_guard.py status`
  (session-start ritual per AGENTS.md process tooling) + 1x distill-
  wrapped `git add -- <six touch-set paths>` to give gate 6
  (`encoding_guard.py --staged`) a non-empty index to check - the
  handoff's 7-gate enumeration does not include staging mechanics and
  `--staged` on an empty index would be a vacuous pass. Disclosed as
  deviation D2.

## Deviations

- D1 (within-protocol, recorded): the STOP-AND-REPORT bar DID trip on
  run-1 include-graph cold (+167.1%); the handoff's own confirming
  re-run rule resolved it (478.3 ms inside the recorded spread). No
  commit happened between trip and resolution. Reviewer owns final
  flag acceptance on the % table either way.
- D2 (minor): +1 support distill call for staging (see Invocation
  census); total distill-route calls 9, gate invocations 8 (ceiling
  honored).
- D3 (minor): chained PowerShell forms (exit-code echo after hygiene)
  refused by the global catch-all - row-60/row-65 known family; every
  gate was attempted independently; no gate was skipped.
- Start rev was `41f7ad4` (M27 T1a APPROVED ledger closeout), one
  docs-only commit above the `bd4fb1a` census anchor; gate 4 confirms
  the census is unchanged there.
- Report-only doc drifts (see section above): `--bin cli` names no
  declared bin (`ariadne`/`tool` are), `cd crates/ui` before a cli run
  (ui is a pure lib), README `-- features server` stray separator.
  Nothing outside the touch-set was fixed.

## Closeout state

Landing commit: this hop's single scoped commit (six paths; hash
recorded by the reviewer's commit-vs-touch-set check). `Cargo.lock`
zero-diff (profile stanza cannot touch it; absent from the commit
numstat). `tools/**`, `crates/**`, `tests/**` byte-identical vs the
start rev (touch-set is Cargo.toml + 3 docs + 2 task docs). Debug
suites bit-identical by construction (I1: flag names ONLY
`[profile.release]`; debug already has overflow-checks on).
`tools/ui_smoke.py` untouched (I2).
