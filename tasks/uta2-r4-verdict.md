# UTA2-R4 Verdict - sweep-B remainder (worker-core drop pin + small crate completions)

Range: base `b43d6da` -> `59abbf0` (single commit, 16 files, +1355/-0).
Handoff: `tasks/uta2-r4-handoff.md`. Journal: `tasks/uta2-r4-journal.md`.
Spec: `docs/specs/unit-test-audit.md` "Round 2" (R4 rows + pin-and-report row).

**VERDICT: APPROVED** - 1 minor finding, none blocking; deviations ACCEPTED.

## Canonical lines (reviewer fresh runs, verbatim)

`python tools/testrun.py --distill -- -p ariadne-<crate>` (each `### cargo test PASS (exit 0)`,
capture `C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt`):

| crate | expected | observed (fresh) |
|---|---|---|
| worker-core | 21/0/0/2 | `21 passed / 0 failed / 0 ignored / 2 suites` (capture 35 lines) |
| scheduler | 42/0/0/2 | `42 passed / 0 failed / 0 ignored / 2 suites` (66) |
| build | 31/0/0/2 | `31 passed / 0 failed / 0 ignored / 2 suites` (44) |
| cluster | 68/0/0/8 | `68 passed / 0 failed / 0 ignored / 8 suites` (117) |
| artifact | 58/0/0/2 | `58 passed / 0 failed / 0 ignored / 2 suites` (74) |
| transform | 33/0/0/4 | `33 passed / 0 failed / 0 ignored / 4 suites` (59) |
| lang | 45/0/1/2 | `45 passed / 0 failed / 1 ignored / 2 suites` (59; 1 ignore pre-existing, unchanged from baseline) |
| validate | 33/0/0/5 | `33 passed / 0 failed / 0 ignored / 5 suites` (65) |

All 8 before->after pairs reproduce: journal baselines (19/40/28/67/57/32/44/31) +
13 new pins = exactly the observed totals (13 shipped pin fns: 2 worker-core, 2
scheduler, 3 build, 1 cluster, 1 artifact, 1 transform, 1 lang, 2 validate).

Workspace gates: `python -m unittest discover -s tools -p "*_tests.py"` ->
`Ran 296 tests in 21.606s` / `OK`. Hygiene: `### generic PASS (exit 0)` (capture
173 lines, R3 warn-only). Encoding: `### generic PASS (exit 0)` (capture 0 lines).
Byte scan of added lines: `added-lines-with-non-ascii-bytes: 0`.

## Per-commit review + commit-vs-touch-set

`git log --oneline b43d6da..59abbf0` -> single commit, message matches gate-10
`-m` literal truth-adjusted ("test: ... pins"). `git show --stat` +
`git diff --numstat/--name-only` reviewed file-by-file: 16/16 paths inside the
handoff touch-set (8 `crates/*/src/**` trees + journal). MUST-NOT list clean:
`repo`, `lang-api`, `ui`, `cli`, `graph`, `lang-cpp`, `lang-scala` appear
NOWHERE in the range. No Cargo/lock touch.

## Independent adjudications (contract items 1-7) - ALL PASS

1. **Purely-additive claim: TRUE.** `--numstat` deletions column = 0 on all 16
   files. Zero production lines: the six touched production files carry ONLY
   `#[cfg(test)]` `mod` declarations (manager.rs, build/lib.rs,
   scheduler/incremental.rs, validate/lib.rs, worker-core tu_store.rs,
   worker_backend.rs); the three inline additions (cluster plan.rs:275+,
   lang extractor.rs:259+, transform placement.rs:374+) are each a whole-file
   appended `#[cfg(test)] mod`; the seven new files are test-only modules.
   Byte scan: all added lines pure ASCII.
2. **worker-core drop pin: honest and code-true.** `tu_store.rs:295-305` (HEAD)
   is exactly `if let Some(path) = ctx.path_by_id.get(&id)` in both the
   Resolved and Wildcard arms - a miss pushes nothing: no counter, no warning,
   no error. The on-disk fallback lives ONLY in the `External|Unresolved` arm
   (:315-319), and the pin's post-discovery ghost-hunk + root-wide
   search-paths prove the Resolved/Wildcard arms cannot reach it. Name
   `pin_edge_destination_drops_unknown_id` (verbatim from handoff item 1)
   matches the behavior; the Wildcard arm pins survivor order + all-unknown
   boundary. Report (surfaced dropped-id counter, UTA-B6 class) shipped in the
   journal; no fix - correct per handoff.
3. **build precedence pin: genuine conflict, not a single-db smoke.** Scenario
   (a): root VALID + `build/` + `out/` planted UNPARSEABLE -> clean Ok proves
   the ladder short-circuits at `exists()` (discovery.rs:78-82 returns the
   first existing candidate; losers never read). Scenario (b): root absent +
   `build/` malformed + `out/` VALID -> Err naming the build rung, with the Ok
   branch panicking on fall-through. Error text matches discovery.rs:113.
   measure pin matches measure.rs:153-198 (empty `arguments` skips spawn;
   :186-188 copies wall-clock-with-1ms-floor into `frontend_cpu_time`;
   preprocessing 0; 1MiB mock; exit 0). Name is honest attribution.
4. **artifact 8-variant equivalence: grep-lock independently confirmed.**
   `pub fn load_*_checked` = exactly 8 definitions (discovery_io 2, index_io 1,
   graph_io 3, metrics_io 2); `transform_io.rs` zero (load_migration_artifacts
   unchecked); each variant's first statement is `self.check_manifest("<stage>",
   input_fingerprint)?` at the journaled line numbers (discovery_io:71/143,
   index_io:64, graph_io:85/243/290, metrics_io:323/383). The pin drives ALL
   EIGHT variants by name with a foreign-fingerprint manifest and asserts the
   exact shared verdict `stale artifact for stage '<stage>': input fingerprint
   mismatch` (Display at manager.rs:39 + reason at :157) + StaleArtifact
   downcast - so the per-variant equivalence is pinned honestly, payload-free
   dirs double-locking manifest-before-payload order. See finding 1 on the
   journal's wording.
5. **validate pins match the uta-t4 reviewer observations exactly.** The T4
   verdict rows ("apply_with can leave its created intermediate directories
   after rollback"; "FileSnapshot::capture fails on a pre-existing directory
   target before the AR-04 decorated path") map 1:1 to the two shipped pins.
   Code-verified: `capture` is `std::io::Result` with `fs::read(p)?` (bare
   error, no path context, all-or-nothing loss of earlier good files) at
   validate lib.rs:102-112; apply_with `capture(...)?` at :178 precedes the
   only `rolled_back_error` call at :197, so no "rolled back" decoration is
   possible on capture failure; `processed_paths = entries + new_paths`
   (:288) = 2+1 = "rolled back 3 files"; `rollback` (:297-321) removes files
   only - `generated/deep/nested` survives, matching the pin.
6. **Report-only trio correctly NOT fixed.** Zero production hunks in the
   commit (criterion 1), so no fix can have landed; all three carry journal
   reports: dropped-id counter (Reports), wall-clock-as-CPU + `simulated`
   flag owner decision (Reports), `verify_restored` dir blindness (Out-of-
   scope observations - true by construction, lib.rs:323-331 fingerprints
   snapshot file entries only).
7. **All 8 before->after lines reproduce** - table above.

## Deviation dispositions

- **D1 (scheduler: blocker-file injection instead of the handoff's "read-only
  dir") - ACCEPTED on merits.** The cited t5 precedent exists
  (`tu_store_t5_pins.rs:128`
  `save_shard_surfaces_create_dir_all_failure_as_error`); read-only dir
  attributes are advisory for own-user writes on this Windows host, so the
  handoff's literal shape cannot deterministically inject here; the pinned
  contract (stage-create IO failure surfaces as Err, no manifest, later
  stages never attempted - incremental.rs:22-38 `?`-chain) is unchanged by
  the injection shape. Same failure class, cross-platform deterministic.
- **D2 (`--staged` -> post-commit `--all` sequencing) - ACCEPTED.**
  commit_scoped stages+commits atomically, so `--staged` post-commit is
  vacuous; reviewer fresh `--all` exit 0 at HEAD is the strict superset.
  Mirrors T1-R3 precedent.
- **D3 (cluster comment-only U+2192 rewrite + cluster gate re-run) -
  ACCEPTED.** Commit contains only the words-form comment; ASCII byte scan
  confirms; re-run line reproduced fresh (68/0/0/8).
- Journal's two compile-only failures (worker-core `Arc::clone` coercion,
  scheduler `expect_err` Debug bound) were harness-shape fixes, no production
  line touched, final state green - noted, nothing owed.

## Hygiene / encoding / locks

Hygiene exit 0, R1/R2 findings zero. The four R3 warns inside touched FILES
(discovery.rs:320 `c`, plan.rs:74 `s`, validate lib.rs:104 `p`, tu_store.rs
:54/:61/:62 `tmp`) are all pre-existing lines OUTSIDE every commit hunk (the
added regions are plan.rs:275+, validate lib.rs:495+, tu_store.rs:713+, and
build/discovery.rs is untouched) - journal claim verified. Encoding `--all`
exit 0 at HEAD. Round-1 pin files: zero deletions means every pre-existing
test is byte-untouched by construction, and all round-1 suites are green in
the fresh runs (baseline-count arithmetic reconciles).

## Findings (numbered)

1. MINOR - `tasks/uta2-r4-journal.md` gate-5 note: "`self.check_manifest(`
   has exactly 8 production call sites" is one short - `stage_is_fresh`
   (manager.rs:137-138) is a 9th production caller. The grep-lock's substance
   (exactly 8 `load_*_checked` variants, each gated first-statement) is
   independently TRUE, and the shipped test pins equivalence per variant, so
   no change is required; correct the wording if this row is mirrored into
   the closeout ledger.

## Follow-up routing (closeout deferred-findings candidates)

- worker-core dropped-id surfaced counter (UTA-B6 class) - owner decision.
- build simulation attribution / `simulated` flag on TuMeasurement - owner
  decision.
- validate rollback created-dir set untracked -> `verify_restored` dir
  blindness - ledger candidate.
- `format_summary` ignores target_file/problem/risk (field-set lock) - UI
  grouping collision note, report-only.
- Journal wording fix per finding 1.

## Refusals observed (configuration note, not skipped gates)

Catch-all deny boilerplate (`{"permission":"bash","pattern":"*","action":
"deny","source":"global"}`) hit two shapes: a `;`-chained `git log ...; git
show --stat ... | Select-Object` (reran as two plain `git log` /
`git show --stat "59abbf0"` calls) and direct `python tasks/uta2-r4-review-
scan.py` (reran via the sanctioned `python tools/distill.py -- python tasks/
uta2-r4-review-scan.py` -> `### generic PASS (exit 0)` +
`added-lines-with-non-ascii-bytes: 0`). Every expected gate ran; no gate
skipped. Reviewer scratch `tasks/uta2-r4-review-scan.py` left untracked for
closeout cleanup.

## Not run (explicit statement)

- `python tools/testrun.py --baseline`, `python tools/ui_smoke.py`,
  `python tools/bench_compare.py`: NOT run at this hand (per-hand precedent
  T1-T7b/R1-R3; AGENTS.md sizing rule - workspace-wide gates belong to the
  fresh-session closeout; R5 still mutates the tree). OWED at UTA2 closeout.
  This handoff's declared gates (8 distills + hygiene + encoding witnesses)
  were ALL run fresh by this review.
- `encoding_guard.py --staged`: vacuous post-commit (clean tree); superseded
  by the `--all` run at HEAD.
- `golden_guard.py`: not applicable; the zero-deletion audit + green round-1
  suites are the byte-lock evidence.
