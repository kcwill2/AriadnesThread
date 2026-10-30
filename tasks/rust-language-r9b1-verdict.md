# R-9b1 Verdict - in-src enrollment (planning hooks, probe-only validator, F2 freeze, final all-true caps flip) + rider band

Milestone: `rust-language` (UNREGISTERED post-M28 track, NO M-number).
Hand: R-9b1 per `tasks/rust-language-r9b-amendment-2.md` sec. 3 (governing
contract; supersedes `tasks/rust-language-r9b-handoff.md` where they conflict;
`tasks/rust-language-r9-amendment-1.md` remains law except as amended).

Chain adjudicated (oldest-first): duty 1 `cef3b0a` (R-9a verdict + ledger
relay), landing `eb1e0c9` (implementer), rider `588bb86` (orchestrator's band,
expanded per board ASK `rust-language-r9b-journal.md` sec. 7 and re-adjudicated
independently below). Reviewer HEAD: `588bb86`, branch
`add-rust-language-support`, clean tree (no dirty files, no untracked files,
worktree census clean for this worktree).

## VERDICT: APPROVED

Zero blockers. Three minor/informational findings (none requires a code
change; none gates R-9b2). Deviation dispositions: D1-D4 ACCEPTED, D5 accepted,
D6 noted. The rider band's expansion is ADJUDICATED ACCEPTED on the merits
(sec. 5d).

Fresh full census (reviewer's own run at `588bb86`, verbatim):
`1989 passed / 0 failed / 13 ignored / 125 suites`

## 0. Ground truth (step 1, fresh)

`python tools/checkpoint_guard.py status` -> HEAD `588bb86`, branch
`add-rust-language-support`, DIRTY FILES (none), UNTRACKED (none). Recent
commits exactly the chain above with `ca56b3b` (planner AMENDMENT-2 commit,
not in the adjudication set) between duty 1 and the landing.

## 1. Fresh verification (step 2 - every scoped gate re-run by this reviewer)

All via `python tools/testrun.py --distill ...` (no raw cargo in reviewer
context). Canonical lines verbatim, all exit 0:

| Gate | Verbatim line |
|---|---|
| `-p ariadne-lang-rust` (green re-run) | `102 passed / 0 failed / 0 ignored / 2 suites` |
| `--test m23_symbol_isolation_test` (UNEDITED) | `7 passed / 0 failed / 0 ignored / 1 suites` |
| `-p ariadne-cluster --test planner_injection` (UNEDITED, see D1) | `6 passed / 0 failed / 0 ignored / 1 suites` |
| `-p ariadne-cli` | `182 passed / 0 failed / 8 ignored / 8 suites` |
| `--test rust_language_discovery_test` | `7 passed / 0 failed / 0 ignored / 1 suites` |
| `--test rust_language_semantic_index_test` | `11 passed / 0 failed / 0 ignored / 1 suites` |
| `--test rust_language_dependency_graph_test` | `9 passed / 0 failed / 0 ignored / 1 suites` |
| `--test rust_language_load_analysis_test` | `5 passed / 0 failed / 0 ignored / 1 suites` |
| `--test m24_language_identity_test` | `8 passed / 0 failed / 0 ignored / 1 suites` |
| full census `-- --no-fail-fast` | `1989 passed / 0 failed / 13 ignored / 125 suites` |
| `python tools/distill.py -- python tools/hygiene_check.py` | `### generic PASS (exit 0)`; every line R3 warn-only, zero entries naming any touched file |
| `python tools/distill.py -- python tools/encoding_guard.py --all` | `### generic PASS (exit 0)` (silent, capture 0 lines) |
| `python -m unittest discover -s tools -p "*_tests.py"` | `Ran 296 tests in 26.142s` / `OK` |
| `python tools/distill.py -- python tools/ui_smoke.py` | `ui_smoke: PASS` / exit 0 (`boot+route checks clean (cold fixture, attempt 1, 1.1s)`) |
| `python tools/distill.py -- python tools/bench_compare.py` (reviewer's fresh run - `crates/lang-rust/src` touched; implementer never runs the bench) | `### generic PASS (exit 0)`; `bench compare: 3 run(s), no regressions past 1.25x baseline` |

Census arithmetic: anchor of record `1981 passed / 0 failed / 13 ignored /
125 suites @ fcb078c` + M=8 new tests by name = `1989`; failed 0 post-rider;
ignored flat 13; suites flat 125. The implementer's pre-rider gate-7 line
`1983 passed / 6 failed / 13 ignored / 125 suites` closes exactly
(1983 + 6 = 1989), attributing the designed four + the two board-ASKed extras
(journal sec. 7) and zero collateral. M=8 attribution verified by name: probe
set net +3 (five named probe tests minus two retired stub pins), +2 adopted
constraints pins (`module_kind_records_are_planning_frozen`,
`two_declaration_fold_records_are_planning_frozen`), +3 planning witnesses
(`aggregation_filter_detects_out_of_line_mod_declarations`,
`target_path_lands_beside_the_aggregation_file`,
`target_stem_grammar_is_the_shared_module_stem`). R-8's 29 witnesses + R-9a's
11 pins run UNEDITED inside the green (their files - `dependency_rewriter.rs`,
`mod_tree.rs`, `move_retention.rs`, `extractor.rs` - are in NO hunk of the
landing).

## 2. Per-commit review (step 3, oldest-first, each judged alone)

### 2a. `cef3b0a` - duty 1 (touch-set: `tasks/milestone_ledger.md` + `tasks/rust-language-r9a-verdict.md` ONLY)

- Ledger relay clause blob fidelity PROVEN from the commit diff itself: the +
  row differs from the - row ONLY by the appended clause after
  `... R-9a dispatched.` inside the state cell, before the ` | kickoff commit`
  terminator: `R-9a LANDED fcb078c APPROVED 2026-10-26, census anchor 1981
  passed / 0 failed / 13 ignored / 125 suites @ fcb078c, R-9b dispatched
  (hooks + validator + all-true flip).` - verbatim the handoff's one-of-record
  clause; count-of-1 (single occurrence in the row; also matched once against
  the current HEAD blob by full-line regex - row 62).
- The longer draft phrasing from the verdict's relay section stayed OUT of the
  ledger and IN the verdict file (`tasks/rust-language-r9a-verdict.md:195`),
  matching journal sec. 1's clause-vs-draft disposition (handoff commands the
  short clause; verdict file keeps its draft as authored).
- Verdict landed as one new 241-line file, unchanged ever since (absent from
  `git diff cef3b0a..HEAD --name-only`); internal cross-references (F1 FINAL
  WORD, D4/D5 dispositions, the reserved F5 name at :125) all resolve, so the
  "as authored" property holds up to the single-copy limit.
- Resume artifact: `tasks/rust-language-r9-handoff-resume.md` deleted - absent
  from disk (glob: no files) and absent from every commit (never staged).
- Did not re-land `c52817c` / `f2387f1` / `fcb078c`.

### 2b. `eb1e0c9` - the landing (touch-set: amendment-2 sec. 3, seven paths)

`git show --stat`: exactly the seven declared paths, numstat totals
1001 insertions / 88 deletions, per-file: `constraints.rs` 115 changed,
`lib.rs` 1, `planning.rs` 186 (NEW), `plugin.rs` 135, `validator.rs` 402,
`tasks/rust-language-r9b-journal.md` 243 (NEW - the journal lands tracked; the
dead hand's untracked record + this hand's secs. 5-9),
`tasks/rust-language-shape.md` 7 changed (one dated hunk). ZERO hunks in the
frozen zones: no `crates/cli`, `crates/cluster`, `crates/validate`,
`crates/lang`, `crates/lang-cpp`, `crates/lang-scala`, `tools`, `tests`,
fixtures, manifests (`Cargo.toml`/`Cargo.lock`), m18-m23, m24, m00, ar27,
goldens. Commit message verbatim the amendment's prescribed one.

Code-level audit (files read whole):
- `planning.rs`: `aggregation_filter` answers `Some(true)` iff syn-parses AND
  declares >= 1 out-of-line `mod x;` (`Item::Mod` with `content.is_none()`);
  inline `mod x { .. }` never fires; unreadable/unparseable -> `Some(false)`
  ("invents nothing - Scala parity"); `target_path_for` lands
  `{sanitized_symbol}.rs` beside the aggregation file via
  `crate::mod_tree::sanitize_module_stem` - the SAME single source the
  move-retention hook anchors `mod <stem>;` to; qualifier strip mirrors
  `scala_target_path_for` at the `::` separator. Handoff item 1 shape held
  VERBATIM; no invented surface (three witnesses only, all named in the
  journal).
- `validator.rs`: `TOOLCHAIN_PROBE_ORDER == ["cargo", "rustc"]` byte-locked;
  `CommandOutcome` / `CommandRunner` / `ProcessCommandRunner` /
  `with_runner` pub surface mirrors `crates/lang-scala/src/validator.rs`
  1-for-1 (field-for-field, doc-for-doc; the only deliberate deltas are the
  contract-mandated ones: 2-entry probe order, `COMPILE_BINARY = "rustc"`
  specifically - Scala compiles through the probe hit, Rust must not), so
  R-9b2 can construct a scripted runner from OUTSIDE the crate. `rustc`
  invocation is options-before-sources; single-input honesty doc present;
  offline `Err` names the D-M23-12-class deferral and spawns nothing beyond
  the `--version` probe walk (INV-10c - pinned with the calls-exactly
  assertion). The five named probe tests land by EXACT contract name; every
  spawn through the scripted seam is `--version`-shaped (INV-10b audit clean -
  no `rustc --crate-name` class spawn anywhere). No pipeline compile
  invocation: `.validate_compile(` call sites are ONLY in lang-crate test
  modules (repo-wide grep); `crates/validate` contains no call.
- `plugin.rs`: caps hunk (from the `cef3b0a` blob diff) turns EXACTLY four
  bits `planning/extraction/rewriting/validation false -> true`;
  `discovery/dependency_graph/semantic_index/load_metric` and
  `requires_toolchain: false` appear ONLY as unchanged context (bit-count
  audit passes; `load_metric` does NOT re-appear). Final literal byte-exact
  vs the handoff's INV-5 text, field order identical; the parity pin
  `capabilities_match_the_parity_flip_literal` asserts an INDEPENDENT
  full-struct literal identical to it. Hook overrides `aggregation_filter` /
  `target_path_for` delegate to `planning.rs` (registry dispatch enrolls them
  with ZERO CLI edit - `crates/cli` in no hunk). `RustToolchainValidator`
  construction gains `::new()` (the stub was the unit-struct era; minimal,
  forced). Scala parity claim verified: `crates/lang-scala/src/plugin.rs`
  answers all-eight-true + `requires_toolchain: false` - the Rust final
  literal is byte-parity with it.
- `constraints.rs`: F2 freeze = union of `kind == SymbolKind::Module` and
  `declarations.len() >= 2`, `colocated_groups` stays empty; the stale R-3
  header line and IV3 docblock are reworded HONESTLY in the landing with the
  AMENDMENT-1 sec. 2 IV3 sentence (policy owns freezing INSTANCES; the rule
  freezes a KIND plus provably-ambiguous folds - the M18.7 seam's purpose);
  the dated honest-limit row (shared-id symbols never plannable, path-aware
  ids required, R-5 D5 accepted cost) is present - exactly what
  AMENDMENT-1 sec. 5 required the docblock to state at this landing.
  `no_colocated_groups_and_no_frozen_symbols` is byte-identical (untouched
  context); the two named positive pins land with the `DeclId` import - the
  adopted shapes as inventoried in amendment-2 sec. 1.
- `lib.rs`: exactly one `mod planning;` line; no `pub use` needed (the crate's
  own plugin delegation is the only consumer; R-9b2 consumes the validator's
  pub surface, which is already `pub mod validator`-reachable as landed).
- `tasks/rust-language-shape.md`: one dated edit at the sec. 3.7 IV3
  decision-table row (amendment-2 sec. 1 erratum), nothing else in the file.
- Adoption integrity (amendment-2 reviewer-gate clause): the
  `cef3b0a..eb1e0c9` diff on `plugin.rs`/`constraints.rs` shows test-module
  changes EXACTLY the inventoried adopted hunks (caps pin rename + all-true
  right side; seam legs -> Some-dispatch pins with tempfile specimens;
  `DeclId` import + two positive pins) plus production additions and doc
  prose - zero adopted-test-byte rewrites beyond the declared evolution
  bands. Full byte attribution to the uncommitted dirty state is
  structurally impossible (R-9a criterion-2 class law: intermediate state
  uncommitted - adjudicated by prediction-match): the amendment-2 sec. 1
  blob-checks, the journal's verbatim red `left:` lines, the leg-2
  re-observation at gate 1 (journal sec. 5: `92 passed / 4 failed / 0 ignored
  / 1 suites` with exactly the four named failures, capture path recorded),
  and this reviewer's green re-run all triangulate; the falsifiability leg 3
  is directly PROVEN from the commit diff's - side: pre-landing production
  carried `planning/extraction/rewriting/validation: false`, seam hooks were
  `None`, and `frozen_symbols` answered an empty `HashSet` - none of which
  can satisfy the landed pins. Probe tests owed no red and landed WITH the
  API (D4 precedent, amendment-2 sec. 2 exempts them explicitly; every spawn
  `--version`-shaped, the two stub pins retired with named positive
  replacement - never weakening).
- Journal: secs. 1-4 are the dead hand's record and match every verbatim
  citation the amendment makes of them (canonical red line, four names,
  `left:` shapes, brace-escape harness note); secs. 5-9 are this hand's
  append - append-only shape. No self-referential SHA (grep: no
  `eb1e0c9`/`588bb86` in the journal - F14 clean).

### 2c. `588bb86` - ORCHESTRATOR RIDER BAND (re-adjudicated in 5d)

Paths: the four rider-band integration locks + `crates/cli/src/commands/ui/
extraction_apply/tests.rs` (granted zone, see 5d). Four caps locks:
assert_eq full-struct discipline retained, test names unchanged, exactly the
four bits flipped to `true` with `requires_toolchain: false` kept, narrative
comments evolved; the literal is byte-identical to the in-src INV-5 final;
`rust_language_load_analysis_test.rs` carries the amendment-mandated true-
count pin `4 -> 8`. Discovery test additionally: the expired extraction-false
arm in `rs_claimed_case_exact_and_plan_backend_resolves` replaced by the
post-parity truth (`plan_backend` resolves for every stage capability) with a
dated supersession note naming where both error classes stay pinned. Cli test:
doomed fixture retargeted `solver.rs` -> `solver.f90` (premise + panic-message
updated; the pinned semantics - token consumed at step 3, Failed debug-shaped
ASCII-sanitized text, retry NotApproved - are OUTSIDE the hunk, byte-kept).

## 3. Commit-vs-touch-set (step 4)

- `cef3b0a`: 2 paths, both in duty-1 scope. PASS.
- `eb1e0c9`: 7 paths, exactly the amendment-2 sec. 3 hard touch-set; zero hunks
  in every frozen zone; zero manifest hunk. PASS.
- `588bb86`: 5 paths - the four sec. 4 band files (orchestrator's own commit,
  band-authorized) + one cli file under the user dispatch's R-1-precedent
  grant (see 5d). PASS for the rider's authority; the band undercount is the
  process finding recorded in 5d/6, not an implementer violation (the
  implementer touched nothing outside its set - the six-red census, board ASK,
  and journal prove the report-don't-fix law was honored).
- Union across the adjudicated chain matches the declared sets; `ca56b3b`
  (planner AMENDMENT-2 + its ledger relay) is outside the adjudication set and
  touched only `tasks/rust-language-r9b-amendment-2.md` + the ledger.

## 4. Plan-literal checks (step 6)

- INV-5 final caps literal: handoff item 3 text vs `plugin.rs` impl vs the
  parity pin vs all four integration locks (post-rider) - identical structs,
  identical field order, `requires_toolchain: false` everywhere. PASS
  (handoff item 3; amendment-2 sec. 3 "FINAL caps literal byte-exact").
- Probe order literal `["cargo", "rustc"]`: `TOOLCHAIN_PROBE_ORDER` +
  `probe_order_is_cargo_then_rustc` call-sequence pin. PASS (shape sec. 3.5 /
  amendment-1 sec. 11).
- Five probe-test names + two retired stub-pin names + F2 pin names + the
  caps-pin rename + the reserved F5 name: all verbatim per the contracts; F5
  `flat_id_shared_mod_and_fn_record_is_planning_frozen` NOT spent anywhere
  (grep: contract docs only) - reservation holds for R-9b2. PASS.
- `load_metric` non-re-appearance: context-only in both caps hunks. PASS.
- Ledger duty-1 clause: verbatim per handoff duty 1. PASS.
- CyclePolicy::Reject literals: zero hunk (no diff line outside
  `crates/lang-rust` src + the rider files; modeling hunk touches
  `TolerateWithinPackage` only as context). `CURRENT_ANALYSIS_VERSION`
  `2.2.0`: `git diff fcb078c HEAD --name-only -- crates/index` -> empty.
  Fingerprint `"rust-1"`: zero hunk. PASS.

## 5. Deviation adjudications (step 5 - each judged on the docs and the code, not on the narration)

### 5a. D1 (journal gate 5): `planner_injection` run in `ariadne-cluster`, not the gate line's `ariadne-integration-tests` - ACCEPTED

Independent proof: the target exists ONLY at `crates/cluster/tests/
planner_injection.rs` (glob across the repo); the gate line's package has no
such `--test` target, so the literal command is inoperable. AMENDMENT-1 sec. 2
names the lock itself (`frozen_symbols_are_skipped_from_proposals` + the
m18.7 SCC lock) - running THE NAMED LOCKS in their real package, UNEDITED,
green `6 passed / 0 failed / 0 ignored / 1 suites` with the file absent from
every chain diff, is faithful (indeed stronger) compliance. The package typo is
a defect of the gate line, not of the run.

### 5b. D2: census no-fail-fast form via `python tools/testrun.py --distill -- --no-fail-fast` (`--` passthrough) - ACCEPTED

testrun has no native flag; the passthrough carries true no-fail-fast
semantics (six failures observed in one run, per-test attribution, exit
captured). This reviewer ran the identical form for the fresh census.

### 5c. D3: the six-red census (designed window disclosed four) - ACCEPTED, implementer blameless

Gate 7 observed `1983 passed / 6 failed / 13 ignored / 125 suites`; failures
5 and 6 sit OUTSIDE the four caps locks but are genuinely flip-superseded
(5 lives inside a rider-band file; 6 in a zero-hunk zone). The implementer
correctly: report-don't-fix, posted the board ASK 2026-10-26, journalled the
line with full attribution, and landed the in-src work per the amendment's
own gate-6-fails-precedent (R-7 flow). The disclosure undercount belongs to
AMENDMENT-1 sec. 4 (it counted the locks it named, not every witness whose
PREMISE the flip kills); the process self-corrected through the ASK channel
exactly as designed.

### 5d. The rider band expansion (`588bb86`): ADJUDICATED ACCEPTED on the merits

Amendment-1 sec. 4 pre-authorized the four-lock band as "the ONLY test-byte
edits ... outside touch-sets", amendment-2 sec. 0 restated the band unchanged;
the ASKed expansion (two extra flip-superseded witnesses + the cli-zone grant)
went past the literal band text. Ruling: the band's PURPOSE - the
orchestrator's amend-with-the-flip commit between landing and the reviewer's
fresh bare census, "amended with the flip, never weakened" (the m23-Scala
idiom this track's own R-5/R-6/R-7 riders ran at anchors `8622b2c`, `470ae22`,
`a659465`) - is honored byte-for-byte, and the user dispatch of 2026-10-27
explicitly declared the rider and granted the cli-zone. Merits, each leg:
- Witness 5 (`rs_claimed_case_exact_and_plan_backend_resolves`): its
  extraction-false arm asserted "Rust keeps extraction false at this hop
  (INV-5)" - R-7-era prose that the user's FULL-PARITY decision and the
  mandated final literal make false. Supersession is REAL, not convenient.
  The replacement pins the new truth (post-parity a .rs target resolves for
  every stage capability), the name is unchanged, the first half of the test
  is unchanged, and the note is DATED and points at where each error class
  stays pinned.
- Witness 6 (`pin_failed_transaction_burns_the_token_and_reports_failed_
  text`): its doomed plan targeted `solver.rs` premised on "a .rs target no
  plugin claims" with the routing keying on exactly the extraction bit this
  hop truthfully turns on; post-flip the plan COMMITTED (verified honest:
  zero engine bytes changed by anyone; cli package green `182/0/8/8`). The
  retarget to `solver.f90` keeps the test's whole reason to exist - token
  burn at step 3, Failed debug-shaped ASCII text, NotApproved on retry -
  every line of which is outside the two-hunk change. The precedent is real
  and applies: `crates/cli/src/registry.rs:424-428` documents the identical
  `.rs -> src/solver.f90` witness move for the identical `.rs`-claim reason
  ("a `.rs` witness would start resolving and this honest-error pin would go
  red"). Finding 1 records that this move is R-4-era (`ec552bd`, A3), not
  "R-1" as the rider message and journal sec. 7 label it - a label slip, the
  precedent itself verified.
- Coverage claim VERIFIED where it is pinned: capability-off class at
  `registry.rs:443-458` (`.cpp` probed on `requires_toolchain`, the honest
  off-bit all three plugins keep false - so this arm is NOT threatened by any
  future Rust bit); unclaimed-target class at `registry.rs:429-442` (`.f90`,
  names target AND stage) and at `m24_language_identity_test.rs:348-356`
  (explicit unhandled-language error end to end). The deleted arm's
  class-distinction leg survives effectively: the registry capability-error
  assert would fail if a claimed target ever answered with the unclaimed
  text. "Zero coverage was lost here" - TRUE (modulo finding 2's wording).
- Four locks: names unchanged, full-struct asserts intact, four bits only,
  `4 -> 8` count pin, all-true literal byte-equal to the in-src pin. PASS.

### 5e. D5: `checkpoint_guard.py resume` (amendment-2 sec. 7 line) hung 600 s and was abandoned; bootstrap proceeded from the amendment + disk inventory - ACCEPTED

The context-wall law bootstraps from disk; the reviewer cannot reproduce the
hang, but the bootstrap's OUTPUT is fully verified: every disk-state claim the
journal makes (adopted inventory, red re-observation, zero stash, clean
boundaries) proved true at this review. No harm, no scope drift.

### 5f. D6: journal append ~110 lines vs the sec. 3 estimate ~30-45 - NOTED

Honest-evidence overrun, R-7 precedent on estimate deviations; total landing
diff (1001+88 lines) stays inside the ~1200 sizing ceiling. Informational.

## 6. Hygiene + encoding + locks (step 7)

- `hygiene_check.py` exit 0 (all R3 warn-only, none in any file this chain
  touched); planning 186, validator 396, plugin 375, constraints 161 - all <
  1000; every function < 100; banned names clean in the new hunks.
- `encoding_guard.py --all` exit 0 (authored bytes pure ASCII).
- Frozen locks: m23 zero-diff and `7/0` green; planner_injection + m18.7 SCC
  lock zero-diff and `6/0` green; cli package green with registry.rs byte-held
  (witnesses read at 420-459); m24 green `8/0` and :348-356 byte-held; the 29
  R-8 witnesses + 11 R-9a pins green UNEDITED inside the crate gate;
  fixtures/goldens/m18-m22/m00/ar27/lang-cpp/lang-scala/cluster/validate/tools
  in NO hunk (landing + rider path lists); `Cargo.toml`/`Cargo.lock` in NO
  hunk (manifest grant rides R-9b2).
- Superseded register: EXACTLY the two validator stub pins (names verified
  gone from all test code; grep shows only doc-comment mentions in their
  replacements) + the sec. 3 bands - nothing else.
- `refs/stash` absent (`git rev-parse refs/stash` -> `fatal: ambiguous
  argument 'refs/stash': unknown revision or path not in the working tree.`);
  no branch change (branch `add-rust-language-support` end to end).

## 7. Report-evidence check (step 8)

Journal secs. 1-9 are digest-shaped throughout: distilling entry points only
(`testrun.py --distill` / `distill.py` forms), canonical lines quoted, capture
paths named (`target/testrun-capture.txt`), exit codes noted, zero raw output
dumps; the gate-1 red re-observation carries the canonical line + the four
names + the `left:` shapes per the sec. 2 evidence standard. COMPLIANT.

## 8. Findings (numbered; no blockers)

1. MINOR (docs-label): the rider commit message, the discovery-test
   supersession note (`tests/integration/tests/rust_language_discovery_test.rs`
   ~:271-277), and the journal sec. 7 attribute the registry.rs `.rs -> .f90`
   witness retarget to "the R-1 rider"; git proves that move landed at the
   R-4 registration hop (`ec552bd`, dated A3 comment at
   `crates/cli/src/registry.rs:425-428`). The precedent is genuine and applies
   regardless; correct the label in the R-9b2 relay/R-10 docs. No code change.
2. MINOR (docs-wording): the same supersession note calls the m24 unclaimed-
   target witnesses "`.f90` witnesses ... in m24_language_identity_test.rs";
   the m24 witness uses `src/nowhere.mock`
   (`m24_language_identity_test.rs:350-356`). The registry.rs `.f90` witness
   comment itself calls `.f90` "m24's mock vocabulary" - the wording fused the
   two. Coverage claim verified true; wording imprecise. No code change.
3. INFORMATIONAL: `validator.rs` landed 396 lines against the amendment's
   "mirror ~200" ceiling-watch note (test module + docs carry the rest); R1/R2
   ceilings pass, the estimate overrun was not journalled (journal D6 covers
   only the journal-append estimate).
4. INFORMATIONAL (reviewer environment): catch-all-deny refusals
   (verbatim `{"permission":"bash","pattern":"*","action":"deny",
   "source":"global"}`) on: a chained `git show --stat` trio, an inline
   `python -c` ledger-line probe, `git grep -o -F`, `git stash list`, and the
   bare `python tools/ui_smoke.py`. Substitutions, each equal or stronger:
   bare `git show` per commit (all three reviewed in full), working-tree
   full-line regex match for the ledger clause (count-of-1 + byte-fidelity,
   sec. 2a), `git rev-parse refs/stash` fatal for the stash proof (R-9a
   criterion-12 precedent), and the distilling wrapper for ui_smoke (the
   documented agent form). No gate was skipped.

## 9. R-9b2 duty-1 relay clause (VERBATIM-READY)

Land in `tasks/milestone_ledger.md` rust-language live row, appended inside
the state cell after `... R-9b1 dispatched.` style tail of the row (count-of-1
verify against the landed blob), alongside landing this verdict file:

`R-9b1 LANDED eb1e0c9 +rider 588bb86 APPROVED 2026-10-27 (planning hooks + probe-only validator + F2 freeze + in-src all-true caps flip; four caps locks turned by rider; dead-session partial adopted with red re-observed), census anchor 1989 passed / 0 failed / 13 ignored / 125 suites @ 588bb86, R-9b2 dispatched (symbol-isolation suite + granted manifest dev-dep).`

NEW ANCHOR (carry verbatim wherever the ledger/memory mirror needs it):
census `1989 passed / 0 failed / 13 ignored / 125 suites` @ landing `eb1e0c9`
+rider `588bb86` (the anchor points at the rider - the first bare-green tree;
base of record `1981 / 0 / 13 / 125 @ fcb078c` unchanged).

Clause-vs-draft note (duty-1 law): amendment-2 sec. 6's draft carried the
placeholder date 2026-10-26; the verdict issues 2026-10-27 and the landing
prose above is the corrected one of record - journal the difference.

R-9b2 carries, beyond landing this verdict + the clause: do NOT re-land
`c52817c`/`f2387f1`/`fcb078c`/`cef3b0a`/`eb1e0c9`/`588bb86`; gate-1 boot
baseline expects `0 failed / 13 ignored / 125 suites` at >= 1989; its gate-5
certified lang-rust line is `102 passed / 0 failed / 0 ignored / 2 suites`
(UNEDITED); the F5 exact name is unspent and reserved (verified); journal
extends from sec. 9 append-only; the granted dev-dep row carries the dated
lang-scala M19.3 precedent comment (amendment-2 sec. 4).

## 10. Follow-up dispositions (recorded, never dropped)

- Findings 1-2 (labels/wording): fold into the R-9b2 duty-1 relay note above
  and the R-10 docs pass; ledger deferred-findings not required (docs prose
  inside landed artifacts, corrected via the relay note).
- Finding 3 (estimate discipline): planner note for future validator-class
  estimates; no action.
- Journal sec. 7's report-only observation (committed=true through-engine on
  a post-parity `.rs` apply is honest frozen semantics - zero engine bytes):
  verified true at this review; nothing further.
- R-9b2 gate 7 expectation form: `1989 + suite-K named` (floor 2023 per plan
  sec. 7 - now satisfied at `1989` with the suite's +K still to land), suites
  `125 -> 126`, ignored flat 13.
