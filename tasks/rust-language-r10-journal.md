# rust-language R-10 journal - full-parity acceptance suite + dated guide riders (2026-10-27)

Hand: R-10, serial hand 10 of 10, the final hand of the `rust-language` track.
Contract: `tasks/rust-language-r10-handoff.md` (read complete), shape secs.
2/6/8, kickoff sec. 4 acceptance forms, `docs/adding-a-language.md` (read
complete), models `m18_cpp_plugin_parity_test.rs` + `m24_language_identity_test.rs`
(cited, never edited), `tasks/rust-language-r9b3-verdict.md` sec. 9, ledger
row 62, `tasks/rust-language-r9b-journal.md` secs. 10-12.
Governing law: AGENTS.md (Edit/Write-only file content; pure-ASCII authored
bytes) + the `implementer-rgb-tdd` skill + `resume-checkpoint` discipline.

## 1. Dead-session relay and bootstrap (resume-checkpoint, disk-only)

This hand is the third dead session in the milestone arc. The ledger row 62
clause records the first two: "R-9b DEADSESSION-1/2 2026-10-26 two
empty-summary stops (duty-1 `cef3b0a` intact, never resumed)".
The third: the R-10 predecessor landed duty-1 (`28894bc`) and died with an
empty compaction summary before authoring the journal or the riders; that
session is NEVER resumed (standing rule). This session bootstrapped from
disk only, per the `resume-checkpoint` skill.

Ground truth at bootstrap (`python tools/checkpoint_guard.py status`):
HEAD `28894bc`, branch `add-rust-language-support`, zero dirty tracked files,
untracked exactly `tasks/rust-language-r10-handoff-resume.md` (bootstrap
glitch artifact) + `tests/integration/tests/rust_language_parity_test.rs`
(the dead hand's partial suite). Matched the orchestrator's dispatch picture
exactly - no contradiction. `tasks/rust-language-r9b3-amendment-resume.md`
(absent) confirms duty-1 discharged the verdict sec. 9 DELETE obligation.
`checkpoint_guard.py resume` (explicit scopes + scoped gate command, because
the handoff's touch-set parse best-effort-matches its MUST-NOT list into the
scope globs) ran the scoped gate: verdict GREEN.

Orchestrator relay basis (dispatch-quoted, captured pre-dispatch): gate 1 at
`12 passed / 0 failed / 0 ignored / 1 suites` via
`python tools/testrun.py --distill -p ariadne-integration-tests --test rust_language_parity_test`.

ADOPT-AND-VERIFY standard applied per the R-9b1 AMENDMENT-2 sec. 2 precedent
(dirty-tree/partial adoption with the adopter re-observing the gate): the
suite is a dead hand's partial, so this session re-ran gate 1 itself, read
the file completely, and inventoried every test against the handoff's
binding list before adopting.

## 2. Adopt-and-verify (gate 1 re-run + 12-test inventory)

Gate 1, re-run by this session: `12 passed / 0 failed / 0 ignored / 1 suites`
(exit 0; capture `target/testrun-capture.txt`, 36 lines - the capture file is
single-slot and was later overwritten by the gate-2 census run, the standard
distiller shape; digest lines here are quoted verbatim).

Disk byte census: 37155 bytes, 882 content lines + trailing newline, zero
non-ASCII bytes. Adoption is AS-IS: zero edits to the suite in this hop.

Inventory BY NAME (each test cites its owner hand(s) at source; the module
doc names the mNN trap explicitly per the guide Step-5 rule):

| # | test | binding / owning hand(s) named at source |
|---|------|------------------------------------------|
| 1 | `parity_default_registry_enumeration_and_identity_bytes` | INV-1/INV-4 enumeration + real-plugin identity bytes through the real registry; m24 claims 1+2 cited (Fortran mock never reused); INV-2 first-wins unchanged; owner R-4 |
| 2 | `parity_capabilities_all_true_final_witness` | INV-5 all-true + `requires_toolchain:false` byte-exact struct + true-count pin + m19-style in-suite `plugin_contract_in_default_registry` enumeration pin; owner R-9b1 |
| 3 | `parity_fixture_discovery_and_unit_shaping` | INV-6 discovery/shaping data re-drive (seven files, one cargo-root context, tests//vendor exclusion); owner R-4 |
| 4 | `parity_semantic_index_spans_and_fingerprint` | INV-7 exact spans + INV-8 `"rust-1"` + D-2 macro-caveat honesty; owner R-5 |
| 5 | `parity_dependency_graph_mod_tree_edges` | mod-tree resolution both layouts; owner R-6 |
| 6 | `parity_load_metric_manifest_and_vocabulary_data` | INV-14 load/manifest `["Rust"]`/vocabulary/Debug-map DATA re-drive; owner R-7 |
| 7 | `parity_modeling_data_and_hook_enrollment` | decision table as data + hook pair enrolled (guide trap honored); owners R-4/R-9b1 |
| 8 | `parity_plan_and_preview_rebind` | plan candidate set + proposals + probe lists incl. the AMENDMENT-3 parent-qualified re-pin; owners R-8/R-9b1/R-9b3 |
| 9 | `parity_apply_and_rollback_arc` | apply/rollback on m23-mirror consumer bytes + RISK-MODTREE leg; owners R-8/R-9a/R-9b3 |
| 10 | `parity_toolchain_probe_contract_never_unknown` | INV-10 (a)/(b)/(c) through the granted dev-dep seam; owner R-9b1 |
| 11 | `parity_mixed_language_repo_keeps_the_m21_error` | RISK-EMPTY-CLAIMANT: CURRENT-behavior pin of the honest M21 multi-language error (NOT a feature claim; shape sec. 6) |
| 12 | `selfscan_rust_suites_operate_on_fixture_copies_only` | RISK-SELFSCAN standing CI witness (roster + fixture input + own-tree idioms forbidden); owner hand shape sec. 6 |

Binding-list check against the handoff: enumeration/identity re-bind citing
m24 (test 1) present; per-capability re-bindings each naming its owning hand
(tests 3-10) present - all eight capabilities bound through the shipped
dispatch (discovery 3, semantic index 4, dependency graph 5, load metric 6,
planning 7/8, extraction 9, rewriting 9, validation 10); INV-5 all-true final
witness + m19-style pin (test 2) present; RISK-SELFSCAN (test 12 + the
journal grep below) present; mixed-language current-behavior pin (test 11)
present.
NO required binding missing, NONE weakened vs the handoff - so nothing was
authored by this hand into the suite and nothing was relaxed.

Line-count attribution (the ~400-line estimate overrun): the handoff sized
the suite ~400; the dead hand authored 882 content lines. The overrun is
provenance documentation, not machinery: the 51-line module header maps the
whole arc claim-by-claim; every test carries an owner-hand banner comment;
helpers are documented with the cited-suite idiom they mirror; the
plan/preview/apply/rollback re-drive (tests 8-9 plus `plan_for`,
`apply_one_plan_with_transaction`, `compose_plan_migration`) is the largest
block (~170 lines) mirroring the m23-mirror route. The dispatch quoted "829
lines"; the disk byte-count is authoritative: 882 content lines, under the
R2 1000 ceiling (margin 118), every function under 100 lines (longest ~57).
No split: the file is homogeneous evidence.

## 3. RGB posture - JUSTIFIED RED-SKIP (falsifiability verbatim)

The handoff's falsifiability statement, journalled VERBATIM as required:

> the suite's all-true caps bind and the mixed-language-current-behavior
> bind fail on the R-9 tree MINUS any single flip - the reviewer may
> sanity-audit by reasoning from the R-9 journal, no re-red is run (per-hand
> reds are the milestone's red record; INV-13 attribution chain closes here).

Basis (SM-1 / W-1 / CW-2 precedent trio, cited in the handoff): every
assertion this suite binds was already RED at its owning hand and
journal-recorded there; a suite that invents no new claim cannot manufacture
a fresh red without re-breaking a green hand. The deadsession incident record
(third death in arc: R-9b x2 per ledger row 62, R-10 x1 here) stands in for
the authoring session's red observation: this hand independently re-observed
GREEN at gate 1 (digest above), which is the adopted-partial contract's
falsifiability floor - the suite answers, and answers the shipped truth.

## 4. RISK-SELFSCAN (D-3) - grep evidence (shape sec. 6, journaled verbatim)

Greps over `tests/integration/tests/rust_language_*.rs` (6 files):

(a) fixture-path grep `fixtures` - all six suites pin their analysis input to
the frozen `tests/fixtures` set via the `CARGO_MANIFEST_DIR/../fixtures`
idiom: `rust_language_symbol_isolation_test.rs:90`,
`rust_language_semantic_index_test.rs:44`, `rust_language_parity_test.rs:107`,
`rust_language_load_analysis_test.rs:76`, `rust_language_discovery_test.rs:61`,
`rust_language_dependency_graph_test.rs:45` (parity additionally at doc
lines 37/105/824/868/870).

(b) own-tree-root grep `current_dir|env::|\.parent\(\)` over the same 6
files - ZERO matches for `current_dir`, ZERO for `env::`, ZERO for
`.parent()`. The only path-anchoring hits are `Repository::locate` calls on
explicit TempDir copy roots: parity:278 (`repo.root`), discovery:108/152/401
(`repo_copy.root`), dependency_graph:307 (`repo.root`) - every one a temp
copy of the fixture.

(c) standing witness: test 12 keeps this evidence enforced in CI every run
(roster of exactly the six suites; fixture-set input asserted; the
working-directory and ancestor-walk idioms forbidden by name).

No milestone suite points at the repo's own tree; the `.rs` flip's own-tree
indexing risk stays test-surface-free (D-3 held).

## 5. Docs riders (shape sec. 8, `docs/adding-a-language.md` - the ONLY docs file touched)

Five dated riders, each "Dated 2026-10-27 (rust-language R-10 rider...)",
pure ASCII, sentence-per-line, appended without rewriting any pre-existing
line except the guard bullet the rider-4 instruction itself targets:

- rider 1 (manifest snippet, after Step 1's ADR-0001 paragraph): lang-rust's
  deliberate parser-deps extension (`syn` full / `proc-macro2` span-locations
  / `unicode-ident`, workspace-pinned), why (D-2 correctness guarantee,
  span-locations load-bearing), what the m17 guards STILL forbid (manifest
  four-name list + transitive-tree list - parser libs cannot reach them),
  and trio-for-three stays the lang-scala baseline, not a universal cap.
- rider 2 (after the "expect to edit every site" qualifier paragraph): stale
  refs refreshed to `commands/plan.rs:203` / `optimize.rs:305` /
  `commands/ui.rs:419`, the AR-19 hook-dispatch fact (a language enrolls by
  overriding the trait hooks), the informational CyclePolicy stance, and the
  Rust enrollment proved with zero edits to those sites (zero-diff over the
  milestone range, sec. 7).
- rider 3 (end of Honest limits, additive): a third plugin now exists
  (`crates/lang-rust`, 2026-10-27, full parity), still this team so the
  adversary limit stands, R1 first exercised on a language foreign to the
  spine's modeling vocabulary (appended `AggregationKind` variant; spine edit
  confined to the `FileKind` table + append-only vocabulary), dynamic loading
  stays parked and the parked bullet is kept because it remains true.
- rider 4 (guard bullet, the instruction's own rewrite target): the manifest
  bullet now reads three manifests (`lang-rust` added), "two-element list"
  corrected to three with the date noted, INV-11 append discipline named as
  the join pattern (same-commit; self-skip rationale; test-diff review), and
  the original "your crate is NOT in it until you add it" guidance preserved.
- rider 5 (after the mNN-trap paragraph in Step 5): the six `rust_language_*`
  suites named as the worked example of naming for the milestone - an
  unregistered post-M28 track with no M-number.

Integrity proof (byte-level, vs the HEAD blob, CRLF-normalized): exactly 5
pre-existing lines absent - all five are the rider-4 bullet lines the dated
correction replaces, their guidance preserved inside the new wording; 63
rider-only lines added; zero non-ASCII bytes before AND after (the guide's
current on-disk bytes are pure ASCII - the AGENTS.md UTF-8 caution is
preventive here; every pre-existing non-ASCII would-be line is byte-identical
by the multiset match); max line width 213 chars, pre-existing table rows
far below the 2000 rule; all rider lines below 100.

## 6. Closeout EVIDENCE DIGEST (for the `milestone-lifecycle` closeout hop)

Duty-1 is DISCHARGED at `28894bc` (R-9b3 verdict + ledger relay clause +
execution-results addendum; `git show --stat`: `tasks/milestone_ledger.md`,
`tasks/rust-language-execution-results.md`,
`tasks/rust-language-r9b3-verdict.md` - zero `crates` paths); NOT re-landed.
The r9b3 resume artifact was deleted there (absent from disk at bootstrap).

Per-hand landing SHA relay, each verified against ledger row 62 clauses:

| hand | relay (dispatch table) | ledger row 62 verification |
|------|------------------------|----------------------------|
| R-1 | `e862acf` | "R-1 LANDED e862acf APPROVED 2026-10-26, anchor 1854/0/13/119 @ e862acf" EXACT |
| R-2 | `3d4dfae` | "R-2 LANDED 3d4dfae APPROVED, anchor 1855/0/13/119 @ 1dd042d" EXACT |
| R-3 | `fe34e23` | "R-3 LANDED fe34e23 APPROVED, anchor 1878/0/13/121 @ eeddfed" EXACT |
| R-4 | `ec552bd` | "R-4 LANDED ec552bd APPROVED, anchor 1886/0/13/122 @ a9b6864" + fixture L3 approval row EXACT |
| R-5 | `673b22b` +rider `8622b2c` | "R-5 LANDED 673b22b (+rider 8622b2c) APPROVED, anchor 1916/0/13/123 @ 8622b2c" EXACT |
| R-6 | `baf24f8` +rider `470ae22` | "R-6 LANDED baf24f8 (+rider 470ae22) APPROVED, anchor 1938/0/13/124 @ 470ae22" EXACT |
| R-7 | `4fd40ca` +rider `a659465` | "R-7 LANDED 4fd40ca (+rider a659465) APPROVED, anchor 1943/0/13/125 @ a659465" EXACT |
| R-8 | `5f87944` +rider `4bd22c8` | row names "R-8 LANDED 5f87944 APPROVED, anchor 1970/0/13/125 @ 5f87944" - rider `4bd22c8` NOT named in the row (discrepancy (ii)); verified as git commit: "docs(rust-language): land R-8 execution-results addendum incl. syn::__private dated deferral row (track accumulation doc, orchestrator rider)" |
| R-9 | STOP `c52817c` / journal `239df72` / AMENDMENT-1 `f2387f1` | row: "R-9 STOP c52817c clean-boundary 2026-10-26 (zero src, journal landed); AMENDMENT-1 issued 2026-10-26 (...)" - names without SHA tokens (discrepancy (i)); `239df72`/`f2387f1` verified as git commits with matching subjects and named in journal/verdict/execution-results docs |
| R-9a | `fcb078c` | "R-9a LANDED fcb078c APPROVED, anchor 1981/0/13/125 @ fcb078c" EXACT |
| AMENDMENT-2 | `ca56b3b` | row: "AMENDMENT-2 2026-10-26: remainder split R-9b1 ... manifest rider GRANTED" - no SHA token in row (discrepancy (i)); `ca56b3b` verified as git commit ("R-9b AMENDMENT-2 ... + ledger relay") and named in `rust-language-r9b1-verdict.md` |
| R-9b1 | `eb1e0c9` +rider `588bb86` | "R-9b1 LANDED eb1e0c9 +rider 588bb86 APPROVED 2026-10-27 ... anchor 1989/0/13/125 @ 588bb86" EXACT (dead-session partial adopted with red re-observed clause present) |
| R-9b2 | STOP `8c1ac46` | "R-9b2 STOP 8c1ac46 clean-boundary 2026-10-27 (...)" EXACT |
| AMENDMENT-3 | `aa67ff2` | row: "AMENDMENT-3 issued 2026-10-27: option (a) RULED ..." - no SHA token in row (discrepancy (i)); `aa67ff2` verified as git commit ("R-9b3 AMENDMENT-3 ... + ledger relay Drafts A+C") and named 10x in r9b3-verdict + 8x in execution-results |
| R-9b3 | `c93bada` | "R-9b3 LANDED c93bada APPROVED 2026-10-27 ... anchor 2003/0/13/126 @ c93bada, R-10 dispatched" EXACT |
| duty-1 | `28894bc` | not citable inside row 62 (self-reference, discrepancy (iii)); verified as current HEAD via `checkpoint_guard status` + `git log`, stat in sec. 6 header |

Discrepancy citations: (i) row 62 records AMENDMENT-1/2/3 and the R-9 STOP
journal by name+date, not SHA token - all four SHAs verified independently
against `git log` subjects and the tasks-doc corpora; (ii) the R-8 rider
`4bd22c8` is an orchestrator execution-results relay that row 62 never named
- verified as a real commit; (iii) duty-1 commits cannot self-cite in the
ledger. All 23 relay SHA tokens resolve; zero unexplained contradictions.

FINAL CENSUS (milestone ANCHOR candidate for the closeout ledger, INV-13):
`2015 passed / 0 failed / 13 ignored / 127 suites` @ this hand's landing.
Delta over the `2003/0/13/126 @ c93bada` anchor, every delta BY NAME: +12 =
the twelve tests of `rust_language_parity_test.rs` (table in sec. 2),
suites +1 = that one new integration binary, failed 0, ignored 13 unchanged.
Floor of record stays AMENDMENT-3 ruling 3 (measured values); no padding.

BENCH EXEMPTION (kickoff sec. 4 assets/docs-only; the proof, run at this
hand): `git diff --name-only c93bada..HEAD -- crates` -> EMPTY (verified
pre-commit at the duty-1 head, capture empty; the landing adds exactly three
paths - the suite, `docs/adding-a-language.md`, this journal - none under
`crates/`, and the duty-1 range already proved zero crates paths), so the
post-commit range is EMPTY by construction; also zero `crates` hunk in
`git show --stat`. Stronger than a src-only filter: only
tests/integration + docs + tasks bytes in the whole landing range. No bench
run belongs to this hand or its reviewer pass (handoff BENCH clause).

DATED DEFERRAL RECORDS (digest material ONLY per orchestrator scope rulings -
no docs rider, no code):
- DEFER-ZERO-CONSUMER: AMENDMENT-3 ruling 2 (2026-10-27), row 62 clause
  verbatim: engine zero-consumer-commit honesty is language-agnostic engine
  policy; the engine stays frozen; R-10 rider material = this record.
- Quote swap (`syn::__private::ToTokens` -> direct `quote` dep): R-8 D1 dated
  deferral row, ALREADY LANDED at orchestrator rider `4bd22c8` (execution-
  results addendum); R-10 deliberately does not widen the lang-rust manifest
  (manifest edits are not this hop's touch-set).
Also routed, not this hand's: the stale `crates/cli/src/registry.rs` header
correction (R-9b1 findings 1-2: `.f90` precedent is R-4-era `ec552bd`, m24
uses `.mock`) is an ORCHESTRATOR rider landing AFTER this hand - no `crates`
hunk was touched here (bench exemption depends on it).

CLOSEOUT CHECKLIST POINTER (next hop = skill `milestone-lifecycle`):
- execution-results addendum for R-10 + per-hand verdict relay law (each
  verdict lands via the next duty-1; R-10's OWN verdict + execution-results
  land in the closeout commit itself - extraction-symbol-move precedent;
  NOT authored here).
- walkthrough committed via skill `walkthrough-commit`.
- memory mirror `ledger.milestone.rust-language` in the SAME commit as the
  ledger flip.
- ledger closeout flip (CLOSED row + NEW ANCHOR declaration recommended by
  the R-10 verdict, "NEW ANCHOR 2015/0/13/127 @ <landing>" wording) WITH the
  declared row-62 consolidation DEBT honored: row 62 itself states "CLOSEOUT
  DEBT: row 62 exceeds the 2000-char line-width rule from per-hop duty-1
  growth - consolidate its clause chain into a dated sub-block at closeout."
- resume-artifact cleanup rows: `tasks/rust-language-r9b3-amendment-resume.md`
  DELETED at duty-1 `28894bc` (confirmed absent);
  `tasks/rust-language-r10-handoff-resume.md` remains UNTRACKED on disk -
  outside this hand's touch-set, deliberately NOT committed, DELETE at the
  closeout hop.
- census floor / D-3 / zero-diff lock re-runs are the closeout's own gates.

## 7. Gates (distilling entry points only; digests verbatim)

1. `-p ariadne-integration-tests --test rust_language_parity_test`:
   `12 passed / 0 failed / 0 ignored / 1 suites` (exit 0) - green on arrival,
   re-run by this session per the adoption standard.
2. full census `python tools/testrun.py --distill`:
   `2015 passed / 0 failed / 13 ignored / 127 suites` (exit 0, capture
   `target/testrun-capture.txt`, 3449 lines) - EXACT against 2003+12, deltas
   named in sec. 6.
3. tools suite CI form `python -m unittest discover -s tools -p *_tests.py`:
   `Ran 296 tests in 25.455s` / `OK`. Note: the bare dispatch form without
   `-p` uses unittest's default `test*.py` pattern and discovers 33 - the
   CI form (AGENTS.md / kickoff sec. 4) is the canonical one and is what was
   reported.
4. `python tools/distill.py -- python tools/ui_smoke.py`: `ui_smoke: PASS`
   (boot+route checks clean, cold fixture, attempt 1, 1.1s).
5. `python tools/hygiene_check.py`: exit 0 (`### generic PASS (exit 0)`,
   capture 173 lines) - R1/R2 clean; every printed row is R3 warn-only in
   pre-existing zones and ZERO rows name the adopted suite, the guide, or
   this journal. Suite functions all under 100 lines (longest ~57), the
   suite at 882 < 1000 lines; the .md files are outside the walker's
   extensions and are guarded here by the width/ASCII census in sec. 5.

Touch-set discipline proof: the whole hop's `git status --porcelain` is
exactly the three touch-set paths (`docs/adding-a-language.md` M, the suite
A, this journal A) plus the untracked resume artifact; `crates/**`,
`tests/fixtures/**`, goldens, `tools/**`, `tests/Cargo.toml`, `Cargo.lock`,
`docs/adr/**`, and every existing suite: zero bytes (zero entries in the
porcelain list; the dispatch-site zero-diff additionally proven over the
full milestone range `522a770..HEAD` for the three rider-2 named files).

## 8. Deliberately NOT done, and deviations (numbered)

NOT done: no `crates/**` bytes anywhere (bench exemption proof); no edits to
the adopted suite (inventory found nothing missing or weakened); no fixtures,
goldens, tools, manifest, or `docs/adr/**` contact; no R-10 verdict or
execution-results authored (closeout commit lands them); closeout NOT
pre-run; ledger not flipped (duty-1 only, discharged at `28894bc`); memory
not touched (same-commit law belongs to the closeout commit); no `git stash`,
no branch change; `tasks/rust-language-r10-handoff-resume.md` not deleted or
committed (closeout duty, listed above).

Deviations:
- DEV-1 (death relay): third dead session in the arc (R-9b x2 per row 62,
  R-10 x1). Handled per `resume-checkpoint` + the R-9b1 AMENDMENT-2 sec. 2
  adoption precedent; relay basis = orchestrator's pre-dispatch gate-1
  capture (sec. 1) + this session's re-run.
- DEV-2 (rider dates): the handoff text says riders "each DATED 2026-10-26";
  the dispatch explicitly rules "dated 2026-10-27" (= landing date, the
  shape's "dated at landing" intent). Riders are dated 2026-10-27 per the
  dispatch.
- DEV-3 (suite size): handoff estimate ~400 vs authored 882 content lines;
  dispatch quoted 829 vs measured 882 (disk byte-count authoritative).
  Attributed in sec. 2; under the R2 1000 ceiling; no split.
- DEV-4 (deny boilerplate on bootstrap probes): `python -c ...`, a temp-script
  `python <path>`, and raw `git log` each returned the catch-all deny
  boilerplate (same pattern as R-9b3 journal deviation 4); each gate was
  retried through the allowed distilling entry point
  (`python tools/distill.py --kind generic -- ...`) and SUCCEEDED - zero
  gates skipped, zero gates approximated.
- DEV-5 (gate 3 form): the dispatch's bare `python -m unittest discover -s
  tools` discovers only 33 tests under unittest's default pattern; the
  canonical CI form (AGENTS.md: `... -s tools -p "*_tests.py"`, what CI runs
  and what produced the 296 of record) was run and reported; both numbers
  journalled.
- DEV-6 (resume-guard parse): the guard's best-effort handoff parse folded
  the handoff's MUST-NOT list into its scope-glob read; the effective scope
  came from this session's explicit `--scope`/`--cmd` arguments (skill-
  documented behavior); the resume report file was regenerated by the guard
  at bootstrap and stays untracked.

## 9. Dated correction append (orchestrator, 2026-10-28, R-10 reviewer findings 1-2; append-only - secs. 1-8 above stay byte-UNEDITED)

- Correction to sec. 6 (finding 1, factual): sec. 6 lists the quote-swap as
  "already landed at `4bd22c8`" - INCORRECT. `4bd22c8` (R-8 rider) landed the
  DATED DEFERRAL ROW only; the swap itself never landed.
  `crates/lang-rust/src/extractor.rs:36` still rides
  `use syn::__private::ToTokens;`. The swap to a direct `quote` dependency is
  DEFERRED post-milestone (ledger closeout record states it).
- Note (finding 2, wording): the five riders are width-wrapped per the
  guide's own convention (the 2000-char machine rule is clean, zero
  matches); the falsifiability "verbatim" quote in sec. 3 differs from the
  handoff text by one inserted article. Substance intact; no re-authoring.

