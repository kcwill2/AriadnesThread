# Audit-Remediation Closeout Bank (orchestrator-maintained)

Sink for deferred findings + recurring patterns surfaced by reviewers
mid-wave. Feed these into the ledger workstream registration and
kilo-memory mirrors at closeout. Status: provisional until closeout lands.

## Deferred ledger rows (from reviewer dispositions)

- AR-02: `--expose` still prints localhost banner/open URL (ui.rs run_ui
  tail) — reviewer F2, b49ed06 verdict.
- AR-06: pub `edges` field bypasses `seen_edges` if future code mixes
  push+add_edge — reviewer F2, 70e1699 verdict.
- AR-07: journal cited a "handoff stated fallback" that exists nowhere —
  wording corrected next touch (record only).
- AR-08: legacy shared-name first-wins behavior fix = FUTURE HANDOFF,
  not AR-08 debt (quadratic probe table in journal).
- AR-09: adjacency-cache staleness lock is add_edge-only; no runtime lock
  against direct-push warm-cache (documented boundary).
- AR-10: `(?i)` Unicode simple-fold vs to_lowercase divergence (ASCII-only
  data by constraint; disclosure is the instrument).
- AR-18: CppPlugin inherent-vs-trait planning hook coexistence — DEFERRED
  TO AR-19 (dispatched to implementer via board 2026-10-10; verify
  discharged in AR-19 verdict, else re-surface here).
- Tools-ticket candidates for closeout (M4 family): testrun.py per-suite
  authored-vs-registered test count (raw-string swallow class, AR-11:
  `r##"` opened, `"#` closed, 2/9 tests silently nested); per-session
  `--capture` defaults;
  PLUS (AR-20b F2, MATERIAL): hygiene_check.py scanner resets string state
  at EOL — multi-line string literals cause false `unclosed fn` R1 blocks
  (forced AR-20b's D1 concat! reshape); ALSO (AR-20b F3): the m17 guard
  walker skips `[dependencies.<name>]` inline-table syntax (plan-literal,
  informational — same walker code family).
- commit_scoped.py rename-blindness (AR-20a D7, MATERIAL for AR-26/AR-20b-
  class rename hands): resolve_scopes reads `git ls-files` (index) so a
  STAGED git-mv deletion is invisible; first attempt shipped tu_store
  add-only with stale copy in tree — implementer soft-reset their own
  unpushed wrong-scope commits and reran with deletion unstaged; landed
  0861db0 verified rename-complete by reviewer. Fix candidate: candidates
  should include `git ls-files -d`.
- D1-stale-pin: m16 measures `3 passed` vs pinned `2` (pre-existing at
  AR-20a dispatch HEAD, green UNEDITED) — ledger pin needs refresh at
  closeout; also re-check every pinned m-suite row against measured counts
  during closeout census.
- AR-11 APPROVED adb54ea (verdict tasks/ar-11-verdict.md): UI suite
  265/0/0/3; graph.js 961/990 budget green; traversal.js degree-bug zone
  deliberately byte-untouched (known open bug preserved — NOT fixed here).
- AR-11 reviewer F1 (carried to AR-15 + wording fix owed): doc-level
  delegated listener in hotspots.js also reproduces suppression of the
  pre-existing QuickOpen closer (hotspots.js:634) — button clicks now close
  an open palette (direction-consistent, only doc-level listener in shipped
  JS); journal "exactly" wording overstated — correct at next journal touch.
- AR-11 reviewer F3 (CLOSEOUT DUTY): theme_token_check frozen counts
  (index.html 6>4, main.js 1>0, traversal.js 1>0) are base-independent
  inherited drift from `&#8250;`/`&#128161;` entities — informational;
  REGENERATION needs a separate approval-gated orchestrator baseline commit
  at closeout (handoff gate-5 "silent" claim was unsatisfiable at launch —
  config finding).
- AR-22: ctor eagerly `create_dir_all`s artifacts/ (mirror did at persist)
  — harmless, parity-locked; carried into AR-23 contract.
- AR-22: StageFingerprints parity = shared-formula shape lock only (no fps
  values on that analyze path) — carried into AR-23 contract.

## Pre-existing / environment rows

- `crates/build` clippy `type_complexity` (pre-existing row).
- `tools/distill.py:213` cp1252 console-echo crash (recommend tools ticket;
  audit out-of-scope M4 family: distill capture clobbering — plan recommends
  per-session `--capture` defaults).
- `tests/fixtures/basic/artifacts/**` rewritten by live m-suite runs
  (LF/CRLF + artifact churn seen all wave).
- bench clone-row chronically near 1.25x (1.20-1.23x history AR-06/08/09);
  closeout needs a quiet-tree bench_compare rerun for truth rows.
- caps_env_ablation flake-class (pre-existing, introduced f312dba): mutates
  ARIADNE_LOAD_MEM_CAPS under a mutex covering only its own 3 tests; sibling
  panic observed at stage_load.rs:223 during AR-23's workspace census
  (`got "3,x"`); workspace census retry was clean. Ledger row candidate.
- Pre-existing `unused_mut` warning, `crates/cli/src/lib.rs:804` (landed
  b49ed06 = OUR AR-02 commit) — visible under --all-targets; fold into AR-28
  slice or fix at closeout; AR-23 finding 2.
- NEW ENV HAZARD (2026-10-11, two-part): (a) plain `git commit` is simply NOT
  allowlisted in this repo — refused in a FRESH implementer too; the only
  permitted commit form is `python tools/commit_scoped.py` (worked every hop
  all run). My micro-hop denial was orchestrator instruction error, owned;
  retry uses 89 EXPLICIT per-file --scope args (explicit scopes = zero AR-07
  leak risk; the leak vector is GLOB scopes only). (b) STILL REAL: the
  RESUMED planner lane refused even raw commit_scoped — resumed sessions can
  lose write forms the original had. RECOVERY LESSON: after a provider death,
  mutation work via FRESH session; resume for analysis only. The 89-file
  staged index survived death + concurrent AR-27 commit_scoped use
  (--only-path semantics immune to foreign staged content). Record
  disposition at closeout ledger registration.
- cp1252 console-echo crashes count 2: `tools/distill.py:213` AND
  `tools/testrun.py:262` bare-mode echo (reproduced independently by AR-23
  and AR-24 reviewers; workaround PYTHONIOENCODING=utf-8). One tools ticket
  covers both.
- `testrun.py --baseline` is FILE-mode with no baseline file in repo
  (documented bare form exits 2) — closeout census should first AUTHOR a
  baseline file at the final numbers, then future hands can gate on it.
- AR-23 informational F4: analyze.rs:37 builds a second `default_registry()`
  though `pipeline.registry` is public (D9-locked, no behavior impact) —
  cheap cleanup candidate for AR-25's cli visit.

## Recurring pattern classes (counts — per observer convention)

- Journal distill evidence omits per-run capture paths: AR-18 F1, AR-22 F1,
  AR-19 F3 (count 3 — same discipline row needed in implementer briefing
  text; AR-11/AR-20/AR-23 briefings now demand capture paths explicitly).
- Allowlist catch-all denies `git grep`/`echo`/`python -c`/chained forms:
  AR-01/02/03/08/18/22 reviewers all hit it (count >= 6; adjudication fell
  back to printed contract + own stdout each time — stable workaround).
- Provider-timeout session deaths: AR-16 reviewer, AR-17 implementer,
  AR-11 implementer x2 (second death = 292s no-response on the FIRST
  resumed turn; provider flakiness window ~22:42-23:19 affected AR-11 and
  near-missed AR-19 reviewer; docs-sweep planner died at the same 292s
  no-response signature 2026-10-11 during the AR-26a2/27 window) (count 5;
  AR-11 recovered via task_id resume
  WITH disk-state briefing both times — lane identity preserved; recovery
  prompts now mandate SMALL INCREMENTS/one-file-per-cycle for
  provider-stability).
- Tool-schema session aborts: AR-30 implementer aborted after 3x malformed
  `kilo_memory_save` input (count 1; recovery = resume + explicit
  "never call that tool again, orchestrator mirrors memory post-commit"
  instruction. LESSON: do not task implementers with memory-tool work at
  all — memory mirroring is an orchestrator post-commit duty).

## Closeout checklist (when queue drains)

1. Definitive workspace census `testrun.py --distill -- --workspace` vs
   anchor `1280 / 0 / 13 / 91`; provisional latest `1393 / 0 / 13 / 102`
   (@AR-24 reviewer own-stdout) — re-check every pinned m-suite row
   (m16 2->3, worker-core 1->2) while running it
   ; + `python -m unittest discover -s tools -p "*_tests.py"` (219 OK last).
2. Quiet-tree `bench_compare` truth rows (post AR-31 stage timing); Wave-2
   expected stage-5 drop documented.
3. Hygiene exit-0 confirmation (AR-13 landed) + size census re-run.
4. Ledger: register workstream as ONE tracked row (milestone-lifecycle
   assigns the M-number; do not guess); flip/append per AR-30's corrected
   format; memory mirrors SAME commit.
5. Deferred-rows section above -> ledger deferred-findings + kilo-memory
   typed mirrors.
6. `encoding_guard --all` + `--staged` clean on final tree.

## Wave-reconciliation items for closeout:
## - AR-13 reviewer census observed `79a90d3` mid-review as foreign landing
##   (+2 empty worker-core suites; zero crates/ui paths — shape matches
##   AR-20a's D3 empty-doc-test-binaries material). AR-20a's stated commits
##   are 0861db0/4dc7237 — RECONCILE what 79a90d3 is (intermediate of the
##   D7 soft-reset saga still in history, or hash misread) before closing
##   the wave table. RESOLVED @ AR-20a review: 79a90d3/d4630d4 proven NOT
##   IN RANGE (D7 soft-reset removed them; observed only transiently as
##   branch tip mid-review). Wave table closed on this point.
## - AR-13 APPROVED: NEW hygiene baseline ratified (exit 0, zero blocking,
##   179 R3 rows as of 79a90d3; pin "zero blocking", NOT an R3 count per
##   reviewer F2).

## Census corrections pinned by planner (2026-10-11): AR-20b m17 green
## landing = `8 passed / 0 failed / 0 ignored / 1 suites` (shape's `7`
## miscount: journal RED 7-total included guard+? vs 6 pre-existing tests +
## guard + planted-edge self-test; 6 baseline verified on-disk). Ledger /
## closeout expectations must read 8, never 7.
## Planner noted: ledger kickoff rows for the AR-20 workstream are ORCHESTRATOR
## duties (ledger-ownership rule) — already folded into closeout duties.

## Memory mirror owed — ADDITIONAL entry: AR-15 journal
## (`tasks/ar-15-handoff-journal.md`) carries its own "Memory mirror" text
## (per board RESULT) — harvest at closeout alongside the AR-30 trio; save
## via a session exposing kilo_memory_save.
## AR-15 also fixed a LATENT SHIPPED BUG in scope (disclosed): api.js
## lastSeenRefresh block-scoped in init() but read by top-level
## startLiveRefresh — the delta-polling branch was ReferenceError-dead
## before AR-15. Ledger note candidate: "ETag delta polling never executed
## pre-AR-15".

## Memory-mirror owed (verbatim from AR-30 @ 0741074; orchestrator session
## had no kilo_memory_save tool — closeout session MUST save these 3, then
## cross-check ledger rows are consistent with them):

1. milestone.m19_complete_2026_10_01: "M19 (Scala discovery plugin)
   complete 2026-10-01 at 32ecf33 on branch multi-language-support; MERGED
   to main via merge f9064f4 (\"Merge branch 'multi-language-support' into
   main (M24 Part A)\"); crates/lang-scala, tasks/m19-t{a,b}-handoff.md,
   walkthroughs/m19-scala-discovery-walkthrough.html all present on main;
   AR-30 ledger truth-sync 2026-10-10."
2. m20.kickoff_dispatched (status supersede): "M20 (Scala resolver /
   StubResolver replacement) LANDED on main: kickoff+handoff docs 520cdb5;
   T-A bf1e3b0 (+ blocker fix ce31b61); T-B 64b0795; T-C c56b2c4; D11
   erratum 5f761b7; T-D acceptance 75abb8e; merged via f9064f4. On main:
   resolver.rs (ScalaImportResolver) present, stubs.rs absent. AR-30
   truth-sync flipped the ledger row from STARTING 2026-10-10."
3. ui.three_theme_set (four-theme set): "The UI provides four themes:
   light, dark, dark-grey (label Dark (Grey)), dark-steampunk; the fourth
   landed 2026-10-04 at 41ec354 (journal rider 4900e04), gated APPROVED;
   original three-theme constraint (light/dark/dark-steam-punk) extended by
   the dark-grey milestone; AR-30 truth-sync 2026-10-10 mirrors the
   dark-grey ledger row flip."

## AR-30 verdict findings folded into closeout duties (verdict APPROVED
## 0741074, tasks/ar-30-verdict.md):
##  (F1/F2) ledger M19 row + archive linen-mice note say "branch head
##  32ecf33" — actual merged branch head is d39980c (f9064f4^2); literal
##  ancestry claim true; FIX BOTH PHRASAS in the closeout ledger commit.
##  (F4) memory mirror = orchestrator/closeout duty BEFORE ledger final row
##  (3 entries verbatim above). Reviewer confirms live ui.three_theme_set
##  memory still says three-theme — entry #3 supersedes it.
## AR-30 landed 0741074 — refusals noted for closeout (audit-workstream row
## still MY duty at closeout; M20 per-hop verdicts unasserted = commit-
## presence basis; troubled-concrete/jeweled-neon/load-tail rows LEFT for
## operator confirm; linen-mice archived, worktree dir cleanup = operator).

## Census deltas ledger (test-count drift for reconciliation at closeout)

- anchor (plan): `1280 / 0 / 13 / 91`
- @AR-02: `1322 / 0 / 13 / 94`
- @AR-06+07 (incl. AR-16 uncommitted WIP +4): `1326 / 0 / 13 / 94`
- @AR-13 (landed aaffde0/14e21b1/768529f): ui count UNCHANGED 265/0/0/3;
  HYGIENE BASELINE TRANSITIONED: zero blocking rows (exit 0), 178 R3
  warn-rows = AR-29 material; server.rs split into server 78/state 114/
  payloads 116/source 87/graph 878/mod 6; downstream contract hygiene
  expectations ("only 2 server.rs rows") are SUPERSEDED — new baseline is
  zero blocking rows (pending AR-13 review confirmation).
- @AR-15 APPROVED (cd875de/cefbd6f): ui `272/0/0/3`; workspace census
  `1381 passed / 0 failed / 13 ignored / 101 suites` (reviewer own-stdout,
  retry-once after a shared-target rlib race vs AR-20b reviewer); bench
  `3 run(s), no regressions past 1.25x` — clone rows 1.4/1.2 chronic band
  with graph crate byte-untouched (closeout quiet-tree rerun still owed).
- @AR-15 reviewer F1 (LEDGER ROW CANDIDATE): export-snapshot invalidation
  sound TODAY only because the sole production payload writer
  (cli `swap_into_state`) necessarily moves both epoch+refresh keys
  (2 write sites grep-proven); no lock pins that against a hypothetical
  second writer. F2: bounded torn-publish window (pre-existing torn-swap
  class, self-healing, no regression).
- @AR-23 APPROVED (reviewer's fresh workspace census, --no-fail-fast after
  1 foreign flake retry): `1372 passed / 0 failed / 13 ignored / 99 suites`;
  tools `Ran 219 tests OK`. Latest full-tree truth point pre-AR-13/AR-20b.
- @AR-20b (dcfc813): m17 suite 6->`8` (guard + planted-edge self-test;
  GREEN-once, single git log -S line); worker-core 8/scheduler 26
  conserved; +2 suite count already in the 101 from 20a.
- @AR-26b APPROVED (3064b77/8e08600/ff526b6, tasks/ar-26b-verdict.md) —
  AR-26 CHAIN CLOSED, all 3 hops. Guard byte-exact `[&str;1]=["ariadne-report"]`
  (P2 compliant — zero absence-assertions of cluster/transform/validate),
  planted-edge self-test, whole-TOML-key `==` predicate (never substring);
  RED-first coherent (panic @ :708 = committed assert; 8e08600 carries
  guard AND swap atomically, GREEN-once per AR-20b class). scheduler cone
  report-free in source/manifest/lock; CONE-CONTENT DELTA vs pre-chain HEAD
  exactly zero. CHAIN CENSUS FOR CLOSEOUT: `1400 / 0 / 13 / 105` exact.
  LEDGER WORDING ROWS: m17:10 "seven core crates" stale vs 8-element guard
  array (contract-forbade the fix — disposition at ledger registration);
  ar-26b contract's `report 7/0/0/3` + `1398/0/0/105` baseline lines stale
  (true 7/0/0/4 / 13-ignored). F4: shape-ruling was UNTRACKED -> docs-sweep
  hop dispatched (tasks/** only); closeout must re-verify zero untracked
  workflow artifacts remain.
- @AR-26a2 APPROVED (landing 0d348ee + d32c3fa/8eec456,
  tasks/ar-26a2-verdict.md): 11/11 R100 VERBATIM (hash-level blob identity vs
  4794467; only deltas = 2 pinned shims + ruled 1+1- hunk); RIDER TRIO
  EXECUTED — widening window CLOSED, `collect_stage_languages` exists only at
  def (pub(crate)) + 3 same-crate consumers repo-wide; report lib.rs = pinned
  7 lines. Conservation 40+7=47; ANCHOR HELD `1398/0/13/105` with suites
  105->105 NO drift (artifact 2 = lib+Doc; report 4 = lib0+bincode6+
  extraction1+Doc0). G7 `-e` substitution accepted-equivalent (refusal hit
  reviewer's run too, identical boilerplate). CONTRACT-TEXT ERRATA for
  closeout planner touch: (F1) ar-26a2-handoff.md:69 anchor says "0 ignored",
  true 13 (line 104 agrees); (F2) gate-3 header pins 3 suites, own
  enumeration + measurement say 4. @AR-26b DISPATCHED (terminal hop, Option A
  guard RED-first, m17 8->10).
- @AR-26a APPROVED (hop 4794467 + journal 5b7c53d/2fb9b21/075e3bf,
  tasks/ar-26a-verdict.md): artifact skeleton + manifests R099 move (the one
  E0603-forced `pub` hunk at :22, ruled transient) + report shim; report
  47/0/0/4 byte-conserved, decomposition 40+6+1+Doc-tests-0. D3 allow-pair on
  rider line NECESSITY PROVEN via --force-warn (beats any allow) — journal
  RIDER enumerates trio (line+allow+comment) for 26a2 removal, verified.
  NEW CENSUS ANCHOR (reviewer own-stdout, tree quiet): `1398 / 0 / 13 / 105`
  = 1393@AR-24 +4/+1@AR-25 +1@AR-31 +0/+2@AR-26a, fully attributed. F1 fixed
  pre-dispatch @ 69ab3f7: 26a2 contract rider trio machine-checkable (gate 7
  exact grep shape), touch-set paths corrected, 26b stale census lines also
  fixed. Rider duty now lives IN the 26a2 contract, not just the journal.
- @AR-31 APPROVED (089a9d1/1140ed5/c8ae8ea, tasks/ar-31-verdict.md):
  tokenizer byte-slice rework, characterization-first ordering VERIFIED
  (suite blob 36bf5907 identical old->new; reviewer hand-recomputed
  word_run.cpp oracle columns as CHAR — lock carries pre-refactor semantics,
  unfittable). lang-cpp 155->`156/0/0/2` (1 lock added); CENSUS PROJECTION
  now 1398/0/13/103. M27 locks zero-diff, contents genuinely byte-column;
  mutation non-vacuity at the file's only char-count subtraction
  (tokenizer.rs:92). alloc 624->458 ops/TU-set (-26.6%), every TU improved;
  probe residue none. D1 (suite in src/tokenizer/ — pub(crate) unreachable
  from tests/, 3-mod precedent) + D3 (343+/221- holistic rewrite) accepted.
  F1 LEDGER CANDIDATE: underflow_fix_tests.rs doc prescribes release-profile
  run but contract gates are debug-only (oracles close the wrap in practice).
  F2: journal D4 "counted separately" wording vs gate #6 counting the lock
  inside 156 — reconcile on next touch. F3: bench PASS rode AR-25 concurrent
  host; neutral-or-better MARGIN confirmed only by closeout's quiet-tree row.
- @AR-26 SPLIT-CHECK STOP (journal-only e946f57/a634ae5/2679ebb): manager/* =
  2879 lines/11 files + manifests 222 forced-move (all IO imports
  `crate::manifests::*`) = ~3150 projected, 6.2x threshold. Re-planned by
  planner: chain `ar-26a` (skeleton + manifests + 2-line shim, ~255 lines) ->
  `ar-26a2` (atomic manager/* verbatim move, 2879, R100) -> `ar-26b`
  (scheduler reparent + m17 guard 8->10); original contract SUPERSEDED-bannered.
  ORCHESTRATOR RULINGS: (1) 26a2 overrun ~1200 ACCEPTED — sizing axis is
  gate-count x output-volume, git-mv ~0 churn with E0116 (7 inherent impls) +
  E0583 (test-mod welding) atomicity proof; precedent AR-20a tu_store move;
  alternative was relaxing VERBATIM = wrong trade. (2) 26b cone wording
  machine-UNSATISFIABLE as originally written (metrics_io.rs:330,
  transform_io.rs:15, apply_io.rs:16 name cluster/transform/validate types in
  PRODUCTION signatures -> artifact must dep all three) — `tasks/ar-26b-shape-ruling.md` FILED: Option A BINDING — 26b locks ONLY
  `ariadne-report` absence (m17 forbidden-set + planted edge + cargo tree
  -e normal exact-token); cluster/transform/validate = EXPECTED members.
  Audit A3 adjudicated **PARTIALLY satisfied** — honest names/plugin-cone/
  byte-locks FULLY delivered; "cone loses stack" line NOT deliverable under
  VERBATIM (transform_io.rs:37 -> ExtractionResult::write_to_disk: DTO
  carries domain behavior; report/Cargo.toml:17-19 drags stack TODAY so
  Option-A cone-content delta is zero). RESIDUAL = DEFERRED LEDGER ROW here
  (AR-26c promotion charter paragraph verbatim in the ruling): domain-neutral
  store + typed facades would be needed to make the full cone claim
  machine-lockable — promote only if a future handoff needs it. Planner's
  guard wording ratified w/ 3 precision notes — APPLIED at 957c391 (26b text
  dispatch-ready; banner + 26a2 pointer cleaned of stale HELD wording). (3) Planner flag: transient
  m17 R2-walk gap for crates/artifact between 26a2 and 26b (closed by
  GUARD_CORE_CRATES addition in 26b).
- @AR-25 APPROVED (2aeae5e/96739ad/2f3b5e7/b7136db, tasks/ar-25-verdict.md):
  cli `114/3/0/4` RED (exactly 3 predicted downcast lines) -> `118/0/8/7`
  (new ar25_seam_error_chain_test.rs +4); sole `.map_err(anyhow::Error::msg)`
  erased from all crate sources (test witness is the only code occurrence).
  CENSUS: +4 tests/+1 suite over the 1393 anchor -> projected 1397/0/13/103.
  D2 INDEPENDENTLY CONFIRMED: anyhow 1.0.104 `Error::msg` PRESERVES payload
  type (`error.rs:167-188` object_downcast binding) — contract's erasure
  premise was factually WRONG (F4: erratum — check no future handoff quotes
  the wrong mechanism; AR-27's contract must be scanned for it). Lock still
  has teeth: re-wrapped call-site -> `M = anyhow::Error` payload -> enum
  downcast fails -> all 3 seam locks red. D4 skip accepted (contract never
  mandated F4) — bank F4 row stays OPEN. report/manager.rs + lang-api + m00 +
  m18 all file-ZERO-diff. F1: testrun.py:262 cp1252 crash now FOUR reviews
  deep. F2: journal warning-count claims are CACHE-scoped (21 vs 31; delta =
  AR-21's 10 deprecations) — reviewers must claim per-cache, only
  "zero warnings naming touched files" is load-bearing. F3: registry.rs test
  mod gained 3 `.to_string()` render tokens (in-file, compile-forced) —
  record only.
- @AR-21 APPROVED (7797edb/3e874ab/9fa9a1c, tasks/ar-21-verdict.md): NOT a
  census event — legacy `IncludeResolver` gated behind default-off
  `test-resolver` feature; lang-cpp 155/0/0/2 IDENTICAL both feature
  configurations; m18 oracles byte-identical AND executing (6 legacy probes
  re-proven); production compiles legacy out (pipeline check ZERO warnings,
  feature-attribution tree, source gating — triple-verified). RED form was
  build-shaped (4x E0433 off-state) — ADMITTED precedent for build-gate
  hands. D1 = one-line test-manifest dev-dep `features` wiring ACCEPTED
  (only out-of-set path). Reviewer F1 (LEDGER DEFERRED-FINDINGS ROW): stale
  oracle doc wording ("public and non-deprecated",
  m18_cpp_plugin_parity_test.rs:11, resolver_parity.rs:270) — contract-
  forced in uneditable files; disposition = the oracle-retirement handoff
  (which is also what the `#[deprecated]` dated note times out to). F2:
  closeout wording — AR-21's contract anchor 153 was stale vs 155 at
  dispatch (AR-24 +2); contracts quote HEAD-adjacent numbers, always
  re-brief at dispatch.
- @AR-24 APPROVED (8497a67 + d4c608c, tasks/ar-24-verdict.md): lang-api
  `41/0/0/4` (new error-kind suite +4); lang-cpp 153->`155`; lang-scala
  221->`227` (characterization +1 each + kind tests); only ONE trait method
  (`validate_compile`) ever returned String — A7 scope smaller than plan
  estimated. REVIEWER CENSUS (own stdout): `1393 passed / 0 failed / 13
  ignored / 102 suites` — actual delta +12 tests / +1 suite vs +~9 expected
  (m16 was mis-pinned as one file; true census traversal `2/0/0/1` +
  clang-worker `3/0/0/1`). tools `Ran 219 tests OK`. NEW ANCHOR:
  `1393 / 0 / 13 / 102`. Reviewer F1 folded into D1-stale-pin row above
  (re-confirmed); F2 re-pinned the testrun cp1252 row at
  `tools/testrun.py:262`; F4: AR-24 journal cites "riders R1-3" but only
  R1/R2 labeled — label third on next journal touch (record only).
- @AR-15 (cd875de): ui suite 265->`272/0/0/3` (+7 payload_etag locks);
  pre-existing ui baseline math: 242@AR-01 + 3@AR-02 + 11@AR-10 + 9@AR-11
  + 7@AR-15 = 272 (for ledger row). AR-15 also fixed in-scope pre-existing
  api.js ReferenceError (lastSeenRefresh block-scoped; delta branch dead
  pre-change) — disclosed; ledger note candidate "delta polling never
  actually worked before AR-15".
- @AR-19 (landed 5de3884): cli package 117 (=118-5+1+3), +1 integration
  suite m24a; lang-cpp 153 byte-equal test attrs
- @AR-23 (landed d8559ae): cli 114 (=117 − 3 helper-unit-tests deleted with
  their helper — reviewer verifying the deleted tests were helper-scoped,
  no lost behavior; AR-19's "118" chain traced: plan-baseline figure was
  never measured at HEAD, 117 was the measured truth); pipeline unchanged
  40/0/4/9; +1 integration suite (ar23 parity, 4 tests)
- @AR-20: NOT a census event — blocked, journal-only commits d63c0d0 +
  7adbeca; A4 reshaped by architect (tasks/ar-20-shape.md pending).
  AR-24's dispatch decision (parallel vs serialized vs lang queue)
  deferred until the shape lands: if C-lite (neutral worker crate), AR-24
  may run alongside; if A (lang-api seam), AR-24 must wait.
- expected additions: AR-09 +13, AR-10 +11, AR-11 +asset locks, AR-13 0,
  AR-16 +4 (landed), AR-17 0, AR-18 +mock-parity locks, AR-19 +scala suite,
  AR-22 +5, AR-30 0, rest per journals.
