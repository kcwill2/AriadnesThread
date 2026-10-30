# rust-language R-6 - Verdict

Verdict: APPROVED (zero blockers).

Milestone `rust-language`, hand 6 of 10, serial. Adjudicated fresh 2026-10-26
at HEAD `470ae22` (worktree clean: `checkpoint_guard.py status` shows dirty
none, untracked none). Chain from start rev `f18f57f`:

- duty 1 `bd44e41` - R-5 verdict (+219) + ONE-line ledger flip carrying the
  R-5 relay clause verbatim; memory mirror `ledger.milestone.rust-language`
  verified present and holding the R-5 dispatch state.
- landing `baf24f8` - the four contract paths (resolver, plugin caps flip +
  pin rename, new suite, journal), +1293/-34.
- orchestrator rider `470ae22` - three-witness amendment per the
  implementer's board ASK option (a), m23 precedent shape.

All verification below is my own fresh run; nothing relayed was trusted.

## Fresh verification (reviewer's own runs, canonical lines verbatim)

- `python tools/testrun.py --distill -p ariadne-lang-rust` ->
  `56 passed / 0 failed / 0 ignored / 2 suites` (exit 0).
- `python tools/testrun.py --distill -p ariadne-integration-tests --test
  rust_language_dependency_graph_test` ->
  `9 passed / 0 failed / 0 ignored / 1 suites` (exit 0).
- `python tools/testrun.py --distill -p ariadne-integration-tests --test
  m20_scala_dependency_graph_test` ->
  `5 passed / 0 failed / 0 ignored / 1 suites` (exit 0) - UNEDITED (m20 file
  in no hunk of `git diff --name-only f18f57f..470ae22`).
- Rider-touched suites, fresh: semantic_index ->
  `11 passed / 0 failed / 0 ignored / 1 suites`; discovery ->
  `7 passed / 0 failed / 0 ignored / 1 suites`.
- m18.7 SCC-sharing lock (`crates/cluster/tests/planner_injection.rs::
  cycle_policy_variants_share_the_scc_coproposal_path_for_now`) fresh:
  `6 passed / 0 failed / 0 ignored / 1 suites`; `crates/cluster` zero-diff.
- FULL CENSUS (bare fail-fast form, now completing post-rider):
  `python tools/testrun.py --distill` ->
  **`1938 passed / 0 failed / 13 ignored / 124 suites`** (exit 0, capture
  3356 lines).
- Hygiene: `python tools/distill.py -- python tools/hygiene_check.py` ->
  `### generic PASS (exit 0)`; every row warn-only R3, none naming
  resolver.rs, plugin.rs, the new suite, or the journal.
- Encoding: `python tools/distill.py -- python tools/encoding_guard.py
  --all` -> `### generic PASS (exit 0)` (capture 0 lines).
- Tool suites (CI form): `python -m unittest discover -s tools -p
  "*_tests.py"` -> `Ran 296 tests in 29.323s` / `OK`.
- Bench (kickoff fully-run at this src-touching hop):
  `python tools/distill.py --timeout 3000 -- python tools/bench_compare.py`
  -> `### generic PASS (exit 0)`; `bench compare: 3 run(s), no regressions
  past 1.25x baseline`; zero WARN lines, so the PR-2 record-not-attribute
  stance was moot this run. No baseline/corpus change (D-3 held).

## Census attribution (reconciled by name)

Duty-1 anchor `1916 passed / 0 failed / 13 ignored / 123 suites @ 8622b2c`.

- +13 lang-rust net (43 -> 56): -1 stub-contract pin
  `resolver::tests::resolve_edges_returns_no_edges` (superseded by dispatch,
  replaced not weakened), +14 named resolver tests (counted by reading the
  test module).
- +9 new suite `rust_language_dependency_graph_test` (+1 suite).
- Pre-rider, the three superseded witnesses sat failed (journal gate-6
  `1935 passed / 3 failed / 13 ignored / 124 suites`); the rider amended all
  three, turning them green without changing the test count:
  1935 passed + 3 turned = 1938 passed / 0 failed.
- Ignored flat 13. Cross-check from the R-4 anchor: 1886 @ a9b6864 + 30
  (R-5) + 13 + 9 = 1938. Exact, both routes.

## Commit-vs-touch-set

`git diff --name-only f18f57f..470ae22` = 8 paths. Per commit, oldest first:

- `bd44e41`: `tasks/rust-language-r5-verdict.md` + `tasks/milestone_ledger.md`
  - duty 1 only, exactly the named pair.
- `baf24f8`: `crates/lang-rust/src/resolver.rs`,
  `crates/lang-rust/src/plugin.rs`, the new suite, the journal - all four
  inside the hard touch-set. plugin.rs is the caps literal flip + pin rename
  + accompanying doc lines ONLY.
- `470ae22`: the two rider files (`rust_language_discovery_test.rs`,
  `rust_language_semantic_index_test.rs`) - outside the implementer's
  touch-set but inside the orchestrator's sanctioned amendment band:
  two one-literal flip-lock evolutions + the sentinel-host relocation the
  board ASK proposed. Disposition below.

## Frozen locks

`git diff --name-only f18f57f..470ae22` over `tests/fixtures`, `tests/golden`,
`Cargo.toml`, `Cargo.lock`, `crates/repo`, `crates/pipeline`, `crates/cli`,
`crates/lang-cpp`, `crates/lang-scala`, `crates/index`, `crates/graph`,
`crates/report`, `crates/validate`, `crates/artifact`, `crates/cluster`,
`crates/transform`, `crates/scheduler`, `crates/ui`, `crates/worker-core`,
`tools` -> EMPTY. The warm-reindex sentinel is a runtime envelope mutation
inside `tempfile::TempDir` fixture copies only - fixture bytes untouched
(verified at the suite source: both suites write only under the temp root).
`crates/worker-core` zero-diff proves the merged-closure behavior needed
ZERO core edits. `crates/pipeline/src/lib.rs` in no hunk - the
`cycle_policy_by_file` lookup is consumed, not edited.

## Caps byte-check - exactly three true bits, FIVE agreeing literals

All five literals read byte-exactly and agree: `discovery: true`,
`dependency_graph: true`, `semantic_index: true`, all five other bits false,
`requires_toolchain: false`:

1. plugin.rs impl literal (flipped in `baf24f8`);
2. in-src pin `capabilities_match_the_dependency_graph_flip_literal`
   (renamed byte-exact per the handoff);
3. new integration lock `capabilities_lock_the_dependency_graph_flip_literal`
   (suite :125-147, registry seam);
4. rider-amended `capabilities_lock_the_registration_flip_literal`
   (discovery suite);
5. rider-amended `capabilities_lock_the_semantic_index_flip_literal`
   (semantic suite).

All five green in my own runs.

## Resolver audit (the named risk of this hand)

- syn per shape decision 2: `resolve_edges` parses with `syn::parse_file`
  and reuses the R-5 `parser_uses` facts verbatim; no hand-rolled source
  scanning (the only string `split("::")` segments an already-written raw
  target, not source text).
- Scala IV-5 precedent verified 1-for-1 against `crates/lang-scala/src/
  resolver.rs`: marker-first-segment External without lookup; matched
  record under a marker External; zero catalog matches External (out-of-repo
  resolves to NOTHING - never Unresolved, which would run the builder's
  on-disk fallback); >1 matches sorted Wildcard; fully-boundary match set
  External. `directive_edge_kind() -> Some(IncludeKind::Import)` mirrors
  IV-1/D10.
- R-5 D5 relay honored: the resolver is the one seam that receives
  `rel_path`, and it owns cross-file truth through it (mod candidates and
  self/super anchors derive from `source_rel_path`); the
  `crate::greeting` id-sharing class correctly stays parked for R-9.
- Invented-edge audit: every asserted edge traces to written source
  (`mod greeting;` :6, `mod models;` :7, `pub mod user;` :4 in the frozen
  fixture; suite-authored temp files for the use/cycle witnesses). External
  stance pinned by tests at both seams. Whole-scope globs honestly External
  (no catalog directory-listing query; no filesystem probe).
- Red prediction honesty: the start-rev stub returns an empty edge vec and
  inherits the trait's `resolve_target` default (`crates/lang/src/
  resolver.rs:152-159` returns `Unresolved`), so the journal's Red list -
  4x `left: []`, 2x `left: Unresolved`, 1 empty-cycle-group, 1 vacuous
  pass - is structurally true, not narrated. The vacuous witness was
  noted, not hidden (R-5 precedent).

## Merged-closure / warm-witness ruling (rider's sentinel relocation)

Ruled CORRECT-BEHAVIOR ACCOMMODATION, not test weakening. Root cause
confirmed: `compute_tu_closure_plugin` (`crates/worker-core/src/tu_store.rs`,
file zero-diff in this chain) walks resolver edges and merges declared
children into parent envelopes, so with the real resolver a symbol like
`crate::User` legitimately lives in three envelopes; a planted id in a
reachable file therefore ADDS a unique symbol (cold 8 vs warm 9). The
behavior is graph truth - exactly what the dependency_graph capability is
for - and the old single-envelope count assumption was authored under the
degenerate stub. The relocated witness stays strong: `macro_specimen.rs` is
declared nowhere in the fixture (grep over all `mod`/`use` directives -
nothing references it), so it is never merged; the sentinel id exists only
inside the stored envelope JSON, so a re-analysis would lose it - the
cache-serve proof stands; the `surface_written` precondition still fails on
an empty surface; the equality witness is now robust for the right reason.

## Deviation dispositions D1-D6

- D1 (external stale flip-locks broken by the mandated flip - report, do
  not fix, board ASK): **ACCEPTED.** Correct discipline; the m23-precedent
  rider resolved both; I re-gated both files personally.
- D2 (gate budget +1 for the Red suite run): **ACCEPTED.** Protocol-implied,
  same class as R-5 D2 (accepted there).
- D3 (census via no-fail-fast sweep + `--replay` while D1 locks sat red):
  **ACCEPTED.** Pre-declared, documented seam; post-rider the bare
  fail-fast census completes exit 0 - my run proves it.
- D4 (shape sec. 8 "five docs riders land at R-6" vs the handoff touch-set
  omitting `docs/adding-a-language.md`): **ACCEPTED - touch-set law
  governs.** Recorded disposition (ASK 2): the shape line is stale planning;
  the five dated guide riders are CONFIRMED at R-10, whose planner hand
  already carries them ("5 dated guide riders"). No action for R-7..R-9.
- D5 (new-suite caps pin authored green-side with the flip commit):
  **ACCEPTED.** A byte-pin of a not-yet-existing literal is meaningless
  red-in-isolation; the flip's red evidence was the renamed in-src pin plus
  the suite's edge-vector reds. R-5 D3 precedent.
- D6 (`git stash push -- crates/lang-rust/src/plugin.rs` + `pop` during
  gate-6 triage - the handoff bars `git stash` outright): **ACCEPTED as a
  MINOR / RECORD-class process breach, not a blocker.** Verified: no stash
  remains (`git rev-parse --verify refs/stash` -> `fatal: Needed a single
  revision` - no `refs/stash` exists at all); the landing hunk for plugin.rs
  is bounded to the caps/pin/doc lines (28 lines, no EOL churn); both suites
  re-ran green on the restored bytes in my own runs. The bar exists to
  prevent losing work across the touch-set boundary - here the stash was a
  single in-touch-set file, popped immediately, self-reported, zero residue.
  The byte-safe probe alternative was available first, so this was avoidable;
  a recurrence (any stash, any file) is a blocker at the next gate.

## Rider disposition (`470ae22`)

**ACCEPTED.** Orchestrator commit, two files, +21/-13, bounded to exactly
the three witnesses the board ASK named: two flip-lock evolutions (one
literal each, names kept, comments record the flip - the m23/R-5 rider
shape, full-struct byte-exact discipline intact) and the sentinel-host
relocation with the merged-closure truth comment. No assertion removed, no
count literal touched, no test deleted. All three witnesses green in my own
fresh runs (11/0/0/1 and 7/0/0/1; and the census's exit 0 covers the warm
witness).

## Findings (numbered; all minor/record - no blockers)

1. MINOR/RECORD (process): the D6 stash breach above - ruled acceptable this
   hop, recurrence = blocker.
2. MINOR/RECORD (stale docs, deferred): `crates/lang-rust/src/plugin.rs`
   lib-header prose still calls the resolver a never-dispatched stub;
   outside this hand's plugin grant ("caps literal + pin rename ONLY").
   Already journal-declared as R-10 rider material (same treatment R-5's
   stale headers got). Goes to the ledger deferral chain at closeout.
3. NOTE (pre-existing tooling, out of scope): `python tools/testrun.py`
   raw (non-distill) echo path dies on this console with
   `UnicodeEncodeError: 'charmap' codec can't encode characters` at
   `testrun.py:262` (`_echo`, cp1252 stdout) - confirmed while probing the
   `--baseline` FILE form. Pre-existing (tools/** zero-diff in this chain;
   already noted in the R-4 execution-results addendum). Not attributable
   to this hand; carried for closeout.
4. NOTE (planning hygiene): shape sec. 8's "five docs riders land at R-6"
   line is stale - riders confirmed at R-10 (see D4 disposition). The
   planner should retire or date-annotate the sec. 8 line when R-10 lands.

## Report-evidence check (distillation contract)

Journal is digest-shaped throughout: distilling entry points only
(`testrun.py --distill`, `distill.py -- ...`), canonical lines + exit codes +
capture paths, zero raw dumps; the D6 self-report and the probe evidence are
disclosed with restore proofs. PASS.

## Relay line for R-7 duty 1

`R-6 LANDED baf24f8 (+rider 470ae22) APPROVED 2026-10-26, census anchor
1938 passed / 0 failed / 13 ignored / 124 suites @ 470ae22, R-7 dispatched
(load_metric hop).`

R-7 relay notes: the three-witness discipline is now five agreeing caps
literals - any future caps flip amends all five in its rider band; the
merged-closure truth (tu_store walks resolver edges) is now load-bearing
for every envelope-count witness downstream (R-8/R-9 symbol-isolation work
must expect parent envelopes carrying merged children); the
`crate::greeting` mod-record/fn-record id-sharing class stays parked for
R-9's named adjudication (R-5 D5); the plugin.rs stale-header docs rider is
R-10 material; the five dated guide riders are R-10's (shape sec. 8 line
is stale - this verdict's finding 4).

## Not run / refused (explicit statement)

- `python tools/testrun.py --baseline` (bare): refused with verbatim
  `usage error: missing value for --baseline` - the F9-errata FILE form;
  no baseline file exists for this track (my probe of the FILE form also hit
  the pre-existing cp1252 `_echo` crash - finding 3). The governing fresh
  census is the bare `--distill` run above, which matches the anchor exactly.
- `python tools/ui_smoke.py`: not required by this hand (no UI/runtime-
  surface bytes; `crates/ui` zero-diff); NOT run.
- `python tools/golden_guard.py --base <rev>`: tool not landed; substituted
  by the handoff's prose byte-check (the zero-diff name-only proof above).
- Red runs at the stub: structurally unreproducible for a reviewer at
  landing HEAD; verified honest instead at source - the start-rev stub's
  empty vec and the trait's `Unresolved` default make every journalled `left:`
  value provably true.
- Catch-all-deny refusals (verbatim
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`):
  one chained `git show --stat x3 + git stash list` (substituted by three
  separate `git show --stat` runs + `git rev-parse --verify refs/stash`),
  one `git grep` (grep tool), one `Set-Content` + chained run (Write tool),
  one scratchpad Write outside the edit grant (unnecessary - dropped), one
  `> $null` chained exit-code probe (substituted by `distill.py`
  `PASS (exit 0)` lines). No expected gate was skipped.
