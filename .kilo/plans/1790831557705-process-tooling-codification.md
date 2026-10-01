# Process Tooling Codification — Spec & Plan

Status: READY TO EXECUTE · Prepared 2026-10-01, self-audit dispositions applied same day (see final section) · Base: `main` @ `1aa03b3`

Goal: codify the process rituals that sessions keep re-authoring as prose into deterministic scripts (`tools/`) and project-local skills (`.kilo/skills/`), so implementer/reviewer/orchestrator handoffs shrink to *referencing* tools instead of restating protocols.

## Evidence (why each item exists)

| # | Repeated behavior / incident | Codification |
|---|---|---|
| E1 | M18.11 implementer died mid-task; compaction "empty summary" unrecoverable; successor had to reverse-engineer intent from disk | T1 |
| E2 | PowerShell `Set-Content`/`>` mojibake (`Â§`, `â†’`) in tracked files recurred ≥3 sessions; `git show` pipeline decoding faked "missing" lines; unquoted `^` nearly compared wrong blob | T2 |
| E3 | UI runtime smoke (boot `run_ui` on ephemeral port, poll `/api/status`, assert routes + `Cache-Control: no-store`) hand-written per handoff; Pass 1.6 re-invented a helper | T3 |
| E4 | Canonical test reporting (`679 passed / 0 failed / 3 ignores`) re-parsed from `cargo test` output in every session/tool-runner call | T4 |
| E5 | Pass 3 cross-crate pollution incident; handoffs always repeat "touch only your scope, never `git add .`, retry index.lock ~10s" | T5 |
| E6 | Reviewer gate prompts re-authored each time (fresh re-measure, per-commit `git show`, independent deviation adjudication, capability-literal check) | T8 |
| E7 | M19 T-A/T-B handoff docs proved the right shape; now re-typed per milestone | T7 |
| E8 | Kickoff docs carry milestone state + deferred findings ("guard gaps → M23", D9) only in prose + agent memory; memory index already truncates | T9 |
| E9 | Plans get an "execution results" addendum after approval (`ef6b7a0`, `76707df`) — always as manual prose | T9 |
| E10 | Artifact landing is hit-or-miss: the M19 branch DID commit its walkthrough deck (`32ecf33`), but the UI-theming deck + entire theming implementation are absent from every branch (see E16) | T12 |
| E11 | Clippy 125-warning `-D warnings` flip promised only in a ci.yml comment; naming rules (`tmp`/`res`/`data`) mandated but unenforced | T10 |
| E12 | A0–A6 load-perf story (69 % stage-5 reduction) lives as plan prose + one bench; no regression tripwire | T11 |
| E13 | 104 hex literals across `crates/ui/assets` (styles.css 61, main.js 26, index.html 10, preview.js 5, graph.js 2); token sweep + steampunk semantic locks keep recurring as plans | T13 |
| E14 | Golden flow (`UPDATE_GOLDENS=1`, byte-locked `tests/golden/**`, manifest hash resync `0xddc3…`, "no new files inside basic/expensive") restated per milestone handoff | T6 |
| E15 | `.kilo/run-script.ps1` is the untouched Agent Manager template | T14 |
| E16 | **Verified-in-worktree work never reaches git, and memory then records it as decided/shipped**: the three-theme plan (`.kilo/plans/1790656207440-ui-theming-token-sweep-light-theme.md`) is fully specced (D1–D8), memory carries "steampunk semantic locks" decisions, yet `themes.css`/`theme.js`/selector exist in ZERO commits on any branch — implementation was lost the same way as the E10 walkthrough. One worktree (`troubled-concrete`) sits at detached HEAD with its branch deleted. The whole plan already specifies its own enforcement tests (T8 parity + zero-hex) — so the THEMING fix is to execute that plan, and the PROCESS fix is landing-verification (T1 census, T12 closeout) | T1, T12 |

## Decisions

- **D1 — Language:** guard tools are Python 3 stdlib-only, mirroring `tools/hygiene_check.py` (already CI-wired, 41-test pattern). No new dependencies, no PowerShell logic (E2 is precisely why).
- **D2 — Skills placement:** new skills are project-local under `.kilo/skills/<name>/SKILL.md` — verified loadable (Kilo discovers `{skill,skills}/<name>/SKILL.md` inside project config dirs); global `.agents/skills/` stays for the generic idd-workflow. Note: `tasks/` does not exist on `main` (M19 handoffs live on the milestone branch) — T9 creates it.
- **D3 — Testing:** every `tools/*.py` ships with `tools/<name>_tests.py` unittest suite (same convention as `hygiene_check_tests.py`); CI gains ONE consolidated step `python3 -m unittest discover -s tools -p "*_tests.py"` (replacing/absorbing the existing hygiene-tests step). Tools must be ASCII-only themselves (E2 discipline); test fixtures that need mojibake bytes construct them via byte escapes (`b"\xc3\x82\xc2\xa7"`), never literals — otherwise the guard flags its own tests (F2 below).
- **D4 — Graduation:** warn-only checks (T10, T11, T13) flip to blocking ONLY via a dated row in `tasks/milestone_ledger.md` (T9) — "flip X at/before milestone M" — owned by whoever runs the next kickoff. No automatic flips. Guards T2/T6 block immediately; T5 is advisory by nature (see F4).
- **D5 — Branch:** all tooling is milestone-independent → implement on `main`. Exception: T6 `golden_guard.py` verifies `tests/golden/**` which exists only on `multi-language-support`; T6 is specced here but EXECUTE AFTER that branch merges (or land it there).
- **D6 — Naming:** commands are `python tools/<name>.py <subcommand>`; exit 0 = clean, 1 = violation/fail, 2 = internal error (matches hygiene_check).

## Phase 1 — Incident-class guards (highest measured cost)

### T1 · `tools/checkpoint_guard.py` + skill `resume-checkpoint`
Deterministic recovery from a dead implementer session (E1).
- `python tools/checkpoint_guard.py status`: prints canonical repo state — HEAD, branch, dirty files with per-crate `diff --numstat`, untracked files, last 5 commits, **plus a worktree census** (per `.kilo/worktrees/*` + `git worktree list`: branch-or-detached, ahead/behind vs `main`, dirty-file count) so orphaned verified work (E16 — `troubled-concrete` at detached HEAD with a deleted branch is the live example) is visible at session start, not discovered weeks later. (Also serves the orchestrator "verify ground truth before delegating" preamble.)
- `python tools/checkpoint_guard.py resume --spec tasks/<handoff>.md [--scope <glob>]… [--cmd <line>]…`: extracts the touch-set + scoped commands from the handoff (M19 shape: touch-set bullet + fenced/bulleted `cargo test` command lines), classifies each dirty path in-scope/out-of-scope, runs the scoped test commands (600 s budget each; report-only on failure, never "fixes"), emits `tasks/<handoff>-resume.md` skeleton: disk-state inventory, green/red verdict per command, in-scope vs stray edits, remaining handoff steps. Ordering caveat: T1 lands before T7 defines the canonical handoff format, so `--scope`/`--cmd` explicit args are the primary interface and markdown parsing is best-effort — on any parse ambiguity the tool prints what it parsed and asks the operator to pass explicit args (E1 lesson: never reverse-engineer intent, report state).
- Rule baked in: sessions whose compaction summary was empty are NEVER resumed — the tool's output is the fresh-session bootstrap.
- Skill `resume-checkpoint`: wraps the invocation + the "start fresh, never resume, run parity/guard suite FIRST before continuing mutations" clauses (learned in M18.11 recovery).
- Red test: temp git repo with scripted dirty state → assert classification + resume skeleton content.

### T2 · `tools/encoding_guard.py`
Kills mojibake and revision-resolution silent errors (E2).
- `python tools/encoding_guard.py [--staged|--all]`: scans tracked (or staged) blobs (via `git cat-file` plumbing, raw bytes) for double-encoded-UTF-8 mojibake signature sequences (`Â`, `â€`, `Ã`, `ï¿½` — constructed internally from byte sequences, not literals) and for non-UTF-8-decodable bytes. Allow-list: `.kilo/plans/**` and `plans/**` (docs may cite signatures — this very plan contains them) plus any byte-exact fixture paths that legitimately carry non-UTF-8 bytes. Exit 1 on hit. Wire into CI next to hygiene gate.
- Baseline verified clean 2026-10-01: a repo-wide scan found zero mojibake outside this plan file → blocking-immediately is safe with no debt paydown.
- Companion (no code): AGENTS.md clause block added in the same commit — quote git revisions (`git show "rev^:path"`), never pipe `git show`/`Set-Content` through PS file writes, use Read/Edit tools or byte-level writes.
- Red test: temp repo with a file containing `b"\xc3\x82\xc2\xa7"` → exit 1; clean repo → exit 0.
- Known limit: this is a DETECTOR, not prevention — it catches corruption at commit/CI time; the AGENTS.md clause is the prevention.

## Phase 2 — Ritual scripts (stop re-authoring prompts)

### T3 · `tools/ui_smoke.py`
The mandated runtime smoke as one command (E3): builds `target/debug/tool.exe` (or `--no-build`), copies the fixture to a temp dir and (default `--cold`) strips `artifacts/`/`.ariadne` from the copy — `tests/fixtures/basic` SHIPS prebuilt `artifacts/`, so an unstripped copy silently tests the cache-hit path and hides cold regressions (also matches the known `run_ui` live-progress unobservability fragility note); `--warm` opts into cache-hit coverage. Spawns `tool ui <tempcopy> --no-open --no-watch --port <p>` — `--no-open` verified to exist (crates/cli/src/lib.rs:238), so NO product change is needed — probes a free port with a Python bind test and passes it explicitly (the CLI takes a fixed `--port`, there is no ephemeral-port handshake; TOCTOU race handled by 3× port-retry), polls `/api/status` until `progress.state==ready` (bounded `--timeout`), asserts the route table (`/`, `/styles.css`, `/app/*.js`) with content types and `Cache-Control: no-store`, kills the server, exit code = verdict. CI-nightly eligible.

### T4 · `tools/testrun.py`
Canonical test reporting (E4): `python tools/testrun.py [scoped cargo test args]` → runs `cargo test` (default `--workspace`) with internal `--timeout` (default 600 s) plus `--retry-on-timeout` (one cached re-run). Parse spec pinned: sum `test result: ok. N passed; M failed; K ignored` lines across all test binaries + doc-test groups; suite count = number of `Running`/`Doc-tests` section lines. Prints the canonical one-line summary `N passed / M failed / K ignored / S suites`. If the output does not match the pinned shape (cargo version drift), exit 2 with "parse format drift" — NEVER print a silently wrong number. Optional `--baseline <file>` compares counts; intentional deltas (new `#[ignore]`) are accepted by updating the baseline in the same commit so the diff is reviewable. Tool-runner agents quote the line verbatim in reports.

### T5 · `tools/commit_scoped.py`
Mechanizes the git discipline clause (E5): `python tools/commit_scoped.py --scope <glob>… -m <msg>` — resolves scopes to explicit pathspecs (aborting if a scope resolves to nothing or matches out-of-allowed roots), commits via `git add <paths>` + `git commit --only -- <paths>` so the commit contains ONLY the scope's working-tree content even if a sibling concurrently staged foreign files (the shared-index race otherwise unfixable); rejects a `.`/repo-root scope; retries on index.lock up to 3× with 10 s backoff; prints `git show --numstat` for the handoff log. Honest limits: this is ADVISORY — plain `git commit` still bypasses it; the enforcement net is the review gate (T8 checks commits against the declared touch-set) and CI. Structural fix recorded in the resume protocol: run sibling implementers in separate Agent Manager worktrees, not one shared worktree; `commit_scoped` is the fallback for shared runs. Red tests in a temp repo with two concurrent dirty scopes.

### T6 · `tools/golden_guard.py`  (execute after `multi-language-support` merges — D5)
Byte-lock enforcement (E14): verifies `tests/golden/**`, `tests/fixtures/**`, and M18 vendored captures are byte-identical vs a given base revision (`--base`), reports which byte-lock manifests changed with new hashes, and fails if new files appeared inside `basic`/`expensive` fixture dirs. `--allow <path>` for sanctioned regenerations. Referenced by every milestone handoff's frozen-locks section instead of prose.

## Phase 3 — Skills (authored prose → templates)

### T7 · Skill `handoff-spec` (`.kilo/skills/handoff-spec/`)
Template + checklist reproducing the M19 T-A/T-B shape (E7): Objective/invariants, hard scope boundary + touch-set, frozen-lock list (invokes T6 command line), acceptance as machine-checkable commands (scoped-only until final gate), Red-capture requirements ("never weaken an assertion"), discipline clauses (T5 command, T4 summary line, UTF-8 clause, 600 s timeouts), reviewer gate reference to T8. Orchestrator fills the template; nothing load-bearing is left to memory.

### T8 · Skill `review-gate`
The reviewer prompt structure that worked (E6) as a template: fresh full verification first (`tools/testrun.py`), per-commit `git show` in oldest-first order, **commit-vs-touch-set check** (`git diff --name-only <base>..HEAD` must be inside the handoff's declared touch-set — this is what makes the advisory T5 actually enforced), independent adjudication of implementer-declared deviations ("do not trust either side"), plan-table literal checks (parameterized per milestone, e.g. capability flags), hygiene via `tools/hygiene_check.py` + T2 + (T6 when applicable — until T6 lands, the frozen-locks check stays prose), verdict format APPROVED/REJECTED with numbered findings + follow-up dispositions.

### T9 · `tasks/milestone_ledger.md` + skill `milestone-lifecycle`
Tracked ledger table: milestone → status (HEAD commit), frozen locks, deferred findings with target milestone, byte-locked sets, plus the graduation backlog rows (D4 flips, clippy `-D warnings`). **Authority rule:** the ledger is the source of truth for milestone state; kilo-memory entries MIRROR it and are updated in the same commit — on conflict, trust the ledger and fix the memory. Skill covers kickoff (generate kickoff doc from plan section + ledger, à la M19 kickoff) AND closeout: append execution-results section to the timestamped plan file, update ledger row, **verify `walkthroughs/<feature>-walkthrough.html` is committed on the branch** (the E10 loss prevention — enforcement lives HERE, not in the global `walkthrough-guide` skill, which lives in `~/.agents/skills/` and is out of this repo's scope; a global-skill amendment is a separate, user-level action), commit plan+ledger+walkthrough+code together per `repo.tracks_plan_files` (E9). Replaces the "milestone state lives in agent memory" fragility (E8).

## Phase 4 — Gate extensions & hygiene-of-process

### T10 · `hygiene_check.py` R3 naming rule (warn-only until ledger flip, D4)
Scan IDENTIFIERS only (reuse the existing Rust/Python scanners; strings, comments and raw text are out of scope) against a small provably-banned set: `tmp`, `res`, `foo`, `bar`, `test1`, and single-letter locals outside closures. **`data` deliberately NOT banned (F5)** — it is a legitimate identifier at high frequency (serde fields, `LoadMetric` payloads, API surfaces); a rule with permanent allow-lists is noise, and noise kills gates. `res`/`tmp` get a warn allow-list mechanism per file+line so existing code can't block adoption. Keep R1/R2 behavior byte-identical; add unit tests per existing convention. Closes most of the "meaningful names" mandate (E11). Same commit: create the `tasks/milestone_ledger.md` backlog rows for the R3 flip decision and the clippy `-D warnings` flip (currently only a ci.yml comment).

### T11 · `tools/bench_compare.py` (LOCAL tool, warn-only, D4)
Verified: `crates/cli/tests/payload_tail_bench.rs` is a hand-timed bench printing a `phase | ms` table via stdout (`--ignored --nocapture`) — parse pinned to its documented row labels, MISSING ROWS = loud exit 2, never silent zeros. Runs the bench 3× and compares MEDIANS against `tools/bench_baseline.json` keyed by platform/CPU marker (GitHub-hosted-runner timings are incomparable to this Windows box — baselines are per-machine-class). Threshold: 25 % median regression on any stage → warn (exit 1 in local mode). Invocation is the M19-3 local gate, not CI blocking; CI adoption only if a dedicated self-hosted runner class ever exists (ledger row records that dependency).

### T12 · Skill `walkthrough-commit` clause + fix
The `walkthrough-guide` output must be written into `walkthroughs/` and committed in the same commit as its finalization (E10/E16: theming deck AND the entire theming implementation were verified in a worktree and lost; M19's deck `32ecf33` shows the pattern works when followed). Encode as: skill instructs Write-to-repo-path + `tools/commit_scoped.py --scope walkthroughs/...`; plus TWO `milestone-lifecycle` closeout checklist items: "walkthrough present on branch?" and **"worktree census clean: every participating worktree merged or abandoned-with-ledger-row"** (E16's lost-implementation class — census output from T1 `status` is the input).

### T13 · `tools/theme_token_check.py` (drift-guard only — Phase B deferred)
Verified state: `crates/ui/assets/styles.css` has exactly ONE `:root` block (dark only), no `[data-theme]`, no `themes.css`/`theme.js` — the three-theme system is specced (`.kilo/plans/1790656207440-…-light-theme.md`, D1–D8) but NOT implemented anywhere in git (E16). Therefore the ORIGINAL T13 spec (assert semantic-role tokens exist) CANNOT pass today — those tokens don't exist. Re-scoped:
- **Ships now:** `--check-added --base <rev>` — flag NEW hardcoded hex literals in `crates/ui/assets/**` in a diff, with a baseline snapshot file of every current location (generated at first run, committed). Stops drift without depending on the theming milestone. Note `styles.css` is R2-exempt (`.css` not in `R2_SCANNED_SUFFIXES`, hygiene_check.py:19) so file-size growth there is a separate, unguarded vector — the baseline also freezes the current hex COUNT in styles.css so new hardcoded colors can't hide in an unscanned file.
- **Deferred to the theming execution (product milestone, OUT OF SCOPE here):** token-parity + semantic-role assertions. The theming plan already specifies them as Rust tests (its T8 parity + zero-hex) — do NOT build a competing Python mechanism; when the theming plan lands, its suite owns Phase B and this tool stays the drift-guard. Ledger row added (T9): "execute theming plan 1790656207440 — implementation lost per E16, needs redo."
- Red test: temp base rev with clean assets; child rev adds `#ff0000` line to main.js → exit 1.

### T14 · Fill `.kilo/run-script.ps1`
Agent Manager run script: `cargo build --bin tool` then `target\debug\tool.exe ui tests\fixtures\basic` so worktree reviews serve the UI automatically (E15). Two lines; also documents the asset-rebuild rule (assets are embedded into the exe).

## Execution order & dependency graph

Phase 1 first (T1, T2 independent, parallel-safe). Phase 2: T3–T5 independent; T6 waits on milestone merge. Phase 3: T7/T8 depend only on tool names existing (reference T4–T6 command lines); T9 ledger file precedes T10's backlog row. Phase 4 fully independent. Each T = one Red/Green commit pair + its `_tests.py`; commit style per repo (`feat(tools): …`, `test(tools): …`).

## Validation (whole plan)

1. `python3 tools/hygiene_check.py` green including new files; consolidated CI step `python3 -m unittest discover -s tools -p "*_tests.py"` green.
2. Warn-only tools exit 0 on the current tree; `encoding_guard --all` exits 0 on `main` (baseline verified clean 2026-10-01) AND must NOT flag this plan file — the `.kilo/plans/**` allow-list is exercised by a test using the plan's own signatures.
3. Drill: simulate a dead session (dirty synthetic edits) → `checkpoint_guard resume` produces a correct inventory; `checkpoint_guard status` census correctly reports `troubled-concrete` as detached-HEAD; `encoding_guard --staged` rejects a planted byte-escape mojibake file; `ui_smoke.py` passes on a temp copy of `tests/fixtures/basic` (default `--cold`), and fails loudly when the fixture copy still carries `artifacts/` while `--cold` was requested.
4. Skill check: `.kilo/skills/` entries discoverable in a fresh session; `handoff-spec` template reproduces the M19 T-A handoff section list.
5. AGENTS.md updated with the revision-quoting + UTF-8 clause block (T2 companion) so all future agents inherit it, and pointers to the new tools from the Testing/Build sections.

## Review dispositions (2026-10-01 audit of this plan)

F1 **`:0`-port claim false** — CLI takes fixed `--port`, no ephemeral handshake; T3 rewritten (bind-probe + retry). F2 **Guard would flag its own spec/tests** — this plan and test sources contain signature literals; byte-escape construction + plan allow-list added. F3 **`--no-open` already exists** (crates/cli/src/lib.rs:238) — T3 needs no product change; claim verified. F4 **Shared-index race** — T5 switched to `git commit --only` + honestly labeled advisory, enforced by T8's touch-set check; per-worktree isolation recorded as structural fix. F5 **`data` un-bannable** — too-common identifier; R3 scope cut to `tmp/res` + single-letter rule. F6 **Bench baselines not cross-platform** — T11 made local-gate, platform-keyed, median-of-3; verified bench is a nocapture table (`payload_tail_bench.rs`). F7 **T13 asserted non-existent tokens** — three-theme system was never landed (E16); T13 cut to drift-guard, Phase B deferred to the theming plan's own Rust tests. F8 **T1 parsed a format that doesn't exist yet on `main`** — explicit `--scope/--cmd` args made primary. F9 **Warm-cache smoke blind spot** — `tests/fixtures/basic` ships `artifacts/` (verified); T3 gets `--cold` default. F10 **Graduation had no owner** — flips now ledger rows (D4). F11 **CI step proliferation** — D3 consolidated to one discover step. F12 **Global-skill assumption** — enforcement moved into T9/T12 closeouts inside this repo.

## Residual risks (accepted, mitigated)

- **E16 already happened once** — the theming implementation is unrecoverable from git (worktree gone); only the plan survives. Ledger row mandates redo; census + closeout reduce recurrence, cannot guarantee it.
- Cargo-output and bench-table parsers break on toolchain/format drift — both exit 2 loudly, never guess.
- `resume --spec` markdown parsing stays best-effort forever by design; malformed handoffs degrade to state-reporting.
- Warn-only checks decay if nobody owns the flip — mitigated only by dated ledger rows; audit them at each kickoff.
- T6 remains blocked on `multi-language-support` merging; until then frozen-locks enforcement is prose in handoffs (T8 carries that caveat).
- Skills under `.kilo/skills/` require a session restart to enumerate; plan commits must not depend on same-session skill invocation.
- `.css` files are invisible to the R2 size gate (hygiene_check.py:19) — recorded; expanding `R2_SCANNED_SUFFIXES` is a separate decision with a live 1422-line `styles.css` failing day one.

## Execution results (2026-10-01)

Status: EXECUTED on `main`, T1-T5 + T7-T14 landed; T6 deferred (D5, `tests/golden/**` exists only on `multi-language-support`; ledger row records the merge trigger). 13 Red/Green-style commit pairs + 3 docs/chore commits.

| Task | Commit (feat) | Verification |
|---|---|---|
| T2 encoding_guard | `1e50fb8` | 11 tests; `--all` exit 0 on `main` incl. this plan file via allow-list (validation #2 met); `--staged` scope proven |
| T1 checkpoint_guard + `resume-checkpoint` skill | `a5893ba` | 13 tests; real-repo census flags `troubled-concrete` DETACHED + surfaces the extension-worktree orphan; resume classification + skeleton proven in temp repos |
| T5 commit_scoped | `fa387cb` | 10 tests incl. foreign-staged immunity and index.lock retry |
| T7-T9, T12 skills + `tasks/milestone_ledger.md` | `4e1d851` | 5 project-local skills under `.kilo/skills/`; ledger carries D4 graduation rows incl. theming-redo mandate |
| T4 testrun | `9cad1c9` | 20 tests; live smoke `-p ariadne-lang` -> `7 passed / 0 failed / 1 ignored / 2 suites` |
| T10 hygiene R3 | `9b40f89` | R1/R2 behavior unchanged (all 48 pre-existing hygiene tests green); 6 new R3 tests; 150-location legacy allow-list committed, repo exit 0 with baseline |
| T13 theme_token_check | `75833c5` | 10 tests; baseline: 132 literals / 8 asset files frozen; `--check-added --base HEAD` exit 0 |
| T3 ui_smoke | `28cc57b` | 23 offline tests + LIVE cold AND warm smoke PASS on this box; cold guard proven (strips copy only) |
| T14 run-script | `2ce8f08` | 2-line Agent Manager run script + asset-embed note |
| T11 bench_compare | `592de60` | 21 tests; baseline generated from 3 live bench runs (`Windows-AMD64-AMD64_Family_26...` class); compare run exit 0 |

Whole-plan gates at execution: `python -m unittest discover -s tools -p "*_tests.py"` = 153 OK (3x consecutive clean + 9x by T3 agent; one transient 9-fail run during concurrent process teardown never reproduced); `hygiene_check.py` exit 0; `encoding_guard.py --all` exit 0; CI consolidated per D3.

Deviations from plan text (all recorded in ledger where actionable):
1. **T2**: added vendored/binary skip policy (`node_modules`/`target`/NUL-byte) — `extensions/.../node_modules` is tracked and carries legitimate non-UTF-8 + cp1252 artifacts; without this the exit-0-on-main validation is unsatisfiable. Also found: `test_output.txt`/`test_results.txt`/`parse.py` are UTF-16 PowerShell debris (invisible to the guard via NUL) -> ledger deferred-findings row, delete at next hygiene pass.
2. **T3**: assets DO send `Cache-Control: no-store` (server.rs:168) -> asserted, not warned; poll fast-fails on dead child; `--target-dir` flag added.
3. **T10**: two pre-existing hygiene-test fixtures renamed (`f`/`x` -> `tally`/`total`) — single-letter fixture names legitimately triggered R3; R1/R2 assertions untouched.
4. **T11**: phase labels pinned from LIVE bench rows (doc-comment table is stale: `hand-out`/`facts+closures`); debug profile per approved command; 3 phases at 0.0 baseline can fire +inf% WARN — ledger row.
5. **T13**: baseline regex (3/4/6/8-hex) counts 132 vs E13's 104 (E13 tallied 6-hex in 5 files only).
6. Test-count baseline drifted during execution (plan assumed 41 hygiene tests at authoring; actual pre-existing was 48+).
