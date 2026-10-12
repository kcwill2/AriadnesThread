# AR-25 Execution Results (reviewer acceptance pass)

Workspace state at adjudication: `checkpoint_guard status` -> HEAD `b7136db`,
branch `shining-enthusiasm`, tracked tree clean (untracked `tasks/ar-*` docs
only). The parallel AR-21 lane had already landed (`3e874ab`, `9fa9a1c`) at
review time; per the dispatch constraint this pass ran SCOPED gates only -
no workspace-wide census. Every line below is reviewer own-run stdout
(capture files treated as clobber-prone per the standing race ruling).

## Reviewer-dispatched scoped gates (own stdout, canonical lines verbatim)

| # | Command | Result line (verbatim) | Expected | Status |
|---|---|---|---|---|
| 1 | `python tools/testrun.py -p ariadne-cli` (bare form first crashed `UnicodeEncodeError` at `tools/testrun.py:262`; re-run `$env:PYTHONIOENCODING="utf-8"; ...`) | `118 passed / 0 failed / 8 ignored / 7 suites` | 118/0/8/7 (= 114 base + 4 new ar25 locks; +1 suite) | MATCH - all four `ar25_*` tests green in-run |
| 2 | `python tools/testrun.py -p ariadne-report` | `47 passed / 0 failed / 0 ignored / 4 suites` | 47/0/0/4 | MATCH - byte-identical to AR-24-era census (report untouched) |
| 3 | `python tools/testrun.py -p ariadne-integration-tests --test m24_language_identity_test` | `8 passed / 0 failed / 0 ignored / 1 suites` | 8/0/0/1 | MATCH (pinned AR-24 census; the one-token hunk green) |
| 4 | `... --test m00_golden_baseline_test --test m00_facts_path_parity_test` | `4 passed / 0 failed / 0 ignored / 2 suites` | 4/0/0/2 (2/0/0/1 each) | MATCH, both files file-ZERO-diff |
| 5 | `... --test m18_cpp_plugin_parity_test` | `23 passed / 0 failed / 0 ignored / 1 suites` | 23/0/0/1 | MATCH, file-ZERO-diff (error-text authority green) |
| 6 | `python tools/distill.py -- cargo check --workspace --all-targets` | `ok (exit 0, 31 warnings emitted) -> capture: target/distill-capture.txt` | exit 0 | STRICT COMPILE-GREEN. Warning census: 31 here vs journal's 21 = cold lang-cpp lib-test cache adds the 10 AR-21 `IncludeResolver` deprecation warnings; capture grep: ZERO warnings name `registry.rs` / `analyze.rs` / `ar25*` / `m24*` |
| 7 | `python tools/hygiene_check.py` | zero R1/R2 rows; every printed row `R3 (warn)`; none in AR-25-touched files | zero blocking | PASS - the NEW zero-blocking regime (2-server.rs rows superseded by AR-13's split) confirmed current |
| 8 | `python tools/encoding_guard.py --all` | (no output - silent/clean) | silent | PASS |
| 9 | grep lock: grep tool over `crates/**` + all `*.rs` workspace-wide for `map_err(anyhow::Error::msg)` / `Error::msg(` | 4 hits: test witness line 149 + two doc-comment mentions in `ar25_seam_error_chain_test.rs` + one comment mention in `analyze.rs:39` - ZERO in `crates/*/src` production code | zero production occurrences | PASS |
| 10 | `git diff --stat d4c608c..HEAD -- crates/report/src/manager.rs tests/.../m00_golden_baseline_test.rs tests/.../m00_facts_path_parity_test.rs tests/.../m18_cpp_plugin_parity_test.rs crates/lang-api` | (empty) | all frozen | PASS - StaleArtifact fold = leave with zero-diff; AR-24 D1 consumed, not moved/re-created |

## Commit-vs-touch-set (per-commit, oldest-first)

| Commit | Paths | Inside touch-set? |
|---|---|---|
| `2aeae5e` | `tasks/ar-25-handoff-journal.md` | EXACT |
| `96739ad` | `crates/cli/src/registry.rs`, `crates/cli/tests/ar25_seam_error_chain_test.rs`, journal | EXACT (lock test = "cli/report lock tests"); production `analyze.rs` untouched at RED - correct red shape |
| `2f3b5e7` | `crates/cli/src/commands/analyze.rs`, `crates/cli/src/registry.rs`, `crates/cli/tests/ar25_seam_error_chain_test.rs`, `tests/integration/tests/m24_language_identity_test.rs` | 3/4 EXACT; m24 = declared deviation D1 (verdict §4, ACCEPTED) |
| `b7136db` | journal | EXACT |

Range-wide foreign paths (`crates/lang-cpp/**`, `tests/integration/Cargo.toml`,
`tasks/ar-21-handoff-journal.md`) belong to AR-21 commits `3e874ab`/`9fa9a1c`,
absent from every AR-25 commit's diffstat; disclosed in the landing record.

## Acceptance criteria (contract) vs what actually happened

1. **Both re-wrap sites dropped; typed errors flow as typed anyhow sources**
   - MET. `analyze.rs:39` now plain `?`; `incremental_backend` errors carry
     `IncrementalDispatchError` (typed kind); the RED->GREEN downcast locks
     passed fresh (gate 1).
2. **New lock downcasts on a forced error where the re-wrap made it
   impossible (RED before, GREEN after)** - MET. Red run documented
   (`114 passed / 3 failed / 0 ignored / 4 suites`, the 3 predicted downcast
   lines with byte locks passing FIRST); GREEN verified this pass; non-vacuity
   re-derived by mutation reasoning (verdict §4 D2) - the lock also catches a
   reintroduced call-site re-wrap over the typed `anyhow::Result`.
3. **CLI-visible error STRINGS byte-identical; m00/m18 locks UNEDITED** -
   MET. All three renderings re-derived byte-exact from `d4c608c` blobs
   (continuation whitespace and `{languages:?}` first-seen order included);
   m00/m18 file-ZERO-diff and green; m18 parity is the external proof.
4. **StaleArtifact fold onto shared pattern where trivial** - DISCHARGED via
   the contract's own conditional: fold is non-trivial AND vacuous (already
   the shared pattern; zero bespoke cli handling); `report/manager.rs`
   file-ZERO-diff; reason journalled. Verdict §5.
5. **STRICT COMPILE-GREEN; pure ASCII; commit_scoped + journal** - MET
   (gate 6; `encoding_guard --all` silent gate 8; per-commit diffstats carry
   only in-scope paths).
6. **No new `.map_err(anyhow::Error::msg)` on seam paths (grep lock)** - MET
   (gate 9; the last production occurrence is gone workspace-wide).
7. **Re-anchor of stale contract cites** - MET: re-wrap at `analyze.rs:39`
   (contract's :49 stale), `incremental_backend`'s `Result<_, String>` at
   `registry.rs` ~:105, both verified against base blobs.
8. **AR-24 D1 transport lock CONSUMED, not moved/re-created** - MET:
   `crates/lang-api` file-ZERO-diff; consumption leg =
   `Error::new(PluginError)` -> downcast `Some` + Display verbatim in the new
   suite, green.

## D1-D4 + StaleArtifact verdicts (detail in verdict file §4/§5)

- D1 (m24 `.to_string()` token): ACCEPTED - compile-forced, wording locks
  byte-untouched, green at pinned census; AR-24 §3 D2 precedent class.
- D2 (anyhow mechanism correction): ACCEPTED, independently CONFIRMED from
  `anyhow-1.0.104/src/error.rs:77-82,167-188` (`construct_from_adhoc` +
  `object_downcast::<M>`: msg preserves payload type identity). The contract's
  assumed mechanism was wrong; the lock's teeth verified by mutation reasoning
  (RED mechanically String-payload-forced; reintroduced re-wrap ->
  `M = anyhow::Error` payload -> seam-enum downcast fails -> locks red).
- D3 (`parse_language_arg` stays `String`): ACCEPTED - clap `value_parser`
  contract (`cli/src/lib.rs:148`, `:181`), not an anyhow seam; not mandated.
- D4 (F4 double-`default_registry()` skipped): ACCEPTED - the contract text
  never mandates it; the bank row is "informational ... candidate"; row stays
  OPEN for closeout (verdict §4).
- StaleArtifact conditional: EXISTS in contract (`ar-25-handoff.md:12-13`);
  LEAVE disposition verified correct; `manager.rs` ZERO-diff (verdict §5).

## Not run / refused (explicit statement)

1. **Workspace-wide gates NOT RUN per dispatch constraint** (parallel lane;
   scoped-gates-only): bare `python tools/testrun.py` census, `unittest
   discover -s tools`, `ui_smoke.py`, `bench_compare.py`. Rationale:
   ui_smoke/bench_compare are not required by the AR-25 contract (zero
   UI/hot-path surface); census is the orchestrator's per Planning Law with
   the projected anchor `1393 + cli +4 + AR-21 delta` from `1393 passed / 0
   failed / 13 ignored / 102 suites` @AR-24.
2. `testrun.py --baseline`: FILE-mode with no baseline file in the repo -
   standing errata governs (bank `ar-closeout-deferred-rows.md:84-86`);
   bare-form exits 2. Not applicable to this scoped pass.
3. `python -c`, `git grep`, chained `echo`: refused by this session's global
   deny rule (verbatim refusal:
   `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`).
   Every intended check completed via the grep/glob tools, `git show` blobs,
   and the sanctioned `tools/*.py` entry points. No gate was skipped or
   silently substituted.
4. Re-running the RED commit's test suite: reviewer is read-only and cannot
   check out intermediate commits to rerun; RED adjudicated from commit
   structure (production re-wrap intact at `96739ad`, harness mirrors it,
   byte-lock-first assertion ordering) plus this pass's GREEN run.
5. Mutations (read-only constraint): non-vacuity adjudicated by mutation
   REASONING over the pinned anyhow source, not a mutated tree (verdict §4 D2).

## Refused-gate log (verbatim)

- `git show --stat ...; echo ---; git show ...` (chained echo): denied -
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
  Split into four separate `git show` calls, all succeeded.
- `git show "d4c608c:crates/cli/src/registry.rs" | Select-Object -Skip 90 -First 80`:
  denied (same boilerplate). Replaced by full `git show` blob read per the
  never-pipe-git-show rule.
- Bare `python tools/testrun.py -p ariadne-cli`: not a denial - the banked
  cp1252 tooling crash (`UnicodeEncodeError`, `tools/testrun.py:262`);
  re-ran with `PYTHONIOENCODING=utf-8` and the canonical line was captured.
