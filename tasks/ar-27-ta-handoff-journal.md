# AR-27 T-a Handoff Journal - STOP-AND-REPORT (zero code landed)

Session: implementer, sole cargo-heavy lane. Dispatch HEAD `85f40d5`
(docs(tasks): AR-27 T-a/T-b contracts + SUPERSEDED banner per shape ruling).
Verdict: HALT at contract step 1 (bootstrap), BEFORE landing anything, on the
handoff's and ruling's own absolute STOP conditions. This journal is the only
file landed by this hop (journal-only landing precedent: AR-20 `d63c0d0`,
AR-26 `e946f57`, AR-27 predecessor `6f02b3c`).

## 1. Verdict

The re-shape (Option B, dual-format serde) fixes the BYTE problem the
predecessor proved, but the ruling's COPY-LOSS forecast is empirically wrong
by a factor of four and lands inside explicitly frozen territory.

Ruling section 4 pre-blesses out-of-set pure copy-semantics `.clone()`
insertions up to 6 sites total ("then STOP - re-split = C-class ruling",
section 8 risk row "Rider sprawl"). The compiler surface measured at this
dispatch HEAD (section 4-5 below) is 24 production out-of-set sites in 9
files, of which 6 sites are in files the T-a contract lists under "You MUST
NOT touch" (`crates/build/**`, `crates/artifact/**`) and 4 more are in a
byte-frozen existing test file (`crates/pipeline/tests/for_incremental_parity_test.rs`,
AR-22 suite). Two further sites sit inside gate-lock files
(`m22_scala_load_analysis_test.rs:280`, `m23_symbol_isolation_test.rs:406`)
under "LOCK AMENDMENTS: NONE - any contemplated lock edit = STOP-AND-REPORT".
The integration package cannot even reach the compiler while the cli lib is
broken, so the frozen-test surface is not yet fully enumerable at all.

No byte or serde finding triggered this STOP - the dual-format encoding
contract itself was not contradicted. The blocker is move-semantics blast
radius: `Cow<'static, str>` cannot be `Copy`, so the alias `pub type
SourceLanguage = LanguageId` removes `Copy` from every consumer that today
reads `.language` out of a borrowed record, and those consumers are not
inside the T-a touch-set.

## 2. Pre-change baselines (own stdout via the distilling entry points; capture-race rule honored - quoted from stdout, capture path is the shared file the digest prints)

- gate 1 PRE: `python tools/distill.py -- cargo check --workspace --all-targets`
  -> `ok (exit 0, 31 warnings emitted)` (capture: target/distill-capture.txt).
- `-p ariadne-repo`: `15 passed / 0 failed / 0 ignored / 2 suites` (pin match).
- `-p ariadne-lang-api`: `41 passed / 0 failed / 0 ignored / 4 suites` (pin match).
- `-p ariadne-cli`: `118 passed / 0 failed / 8 ignored / 7 suites` (pin match).
- `-p ariadne-index`: `15 passed / 0 failed / 0 ignored / 2 suites` (digest locks included).
- `-p ariadne-ui`: `272 passed / 0 failed / 0 ignored / 3 suites` (pin match).
- rider crate (ruling section 4 pre-mutation pin) `-p ariadne-pipeline`:
  `40 passed / 0 failed / 4 ignored / 9 suites`.
- rider crate `-p ariadne-worker-core`: `8 passed / 0 failed / 0 ignored / 2 suites`.
- m23 bincode serde lock `-p ariadne-lang-api --test m23_bincode_serde_locks`:
  `5 passed / 0 failed / 0 ignored / 1 suites`.
- m23 symbol isolation `--test m23_symbol_isolation_test`: `7 passed / 0 failed / 0 ignored / 1 suites`.
- m00 pair `--test m00_golden_baseline_test --test m00_facts_path_parity_test`:
  `4 passed / 0 failed / 0 ignored / 2 suites`.
- m24 + m24a + m18 `--test m24_language_identity_test --test m24a_scala_plan_path_test --test m18_cpp_plugin_parity_test`:
  `33 passed / 0 failed / 0 ignored / 3 suites` = m24 8 + m24a 2 + m18 23 exactly.
- `python tools/distill.py -- python tools/ui_smoke.py`:
  `boot+route checks clean (cold fixture, attempt 1, 0.7s)` / `ui_smoke: PASS`.
- `python tools/hygiene_check.py`: all reported rows R3 (warn); ZERO R1/R2 blocking rows.
- `python tools/encoding_guard.py --all`: silent (exit clean).
- Census anchor `1400/0/13/105` NOT re-measured: reviewer's acceptance gate per
  Planning Law + contract.
- Workspace census NOT re-run (sole-lane discipline, scoped-only gates).

## 3. Which hard clauses fire (verbatim authority quotes)

- T-a contract, Clone-rider protocol (ruling section 4, embedded VERBATIM):
  "General rule: any other out-of-set file needing a pure copy-semantics
  '.clone()' insertion is pre-blessed up to 6 sites TOTAL out-of-set; any
  out-of-set edit that is not a copy-semantics clone => STOP-AND-REPORT."
- Ruling section 8: "Rider sprawl: 6-site cap out-of-set, then STOP
  (re-split = C-class ruling)."
- T-a contract touch-set: "You MUST NOT touch: crates/build/**, ...
  crates/artifact/** ... EVERY existing file under tests/** and
  crates/*/tests/** (suites and goldens UNEDITED)".
- Dispatch + contract: "LOCK AMENDMENTS: NONE - frozen clause; any
  contemplated lock/fixture-expectation edit = STOP-AND-REPORT."

Measured reality (section 5): 24 production out-of-set sites (cap 6), 6 sites
inside MUST-NOT-touch files, >=4 sites inside existing byte-frozen test files
(+2 analyzed gate-lock sites), and an unenumerable remainder hidden behind the
broken cli lib. Every one of these clauses is individually exceeded.

## 4. Probe method (temp mutation, NEVER committed, reverted byte-clean)

Analysis alone repeats the predecessor's near-miss risk, so the copy-loss
class was measured EMPIRICALLY at the untouched types, following the
in-repo probe precedents (AR-27 predecessor temp test file deleted after
capture; AR-21 off-state absence lock never committed; AR-26a temp-dir probe;
AR-16/AR-20a mutation proofs reverted).

The enum-to-newtype move changes copy-semantics exactly once: `Copy` is lost
(`Cow<'static, str>` cannot implement `Copy`). The faithful, minimal
simulation was therefore to remove `Copy` from the `SourceLanguage` derive
list and re-derive nothing else, plus the one in-set coherence adaptation the
contract already commands for the `&`-From body (probe-world form
`Self::from(source.clone())`; the alias-world form is `source.clone()`).
`cargo check --workspace --all-targets` was then re-run per tier; each
compiler-surfaced site got a THROWAWAY clone/ref token so the next tier could
surface (exactly the mutation-proof protocol every approved handoff used),
and finally ALL probe edits were reverted.
`python tools/distill.py -- git status --porcelain -- crates` -> EMPTY after
revert (capture: target/distill-capture.txt, 0 lines); full-tree porcelain
shows only the foreign churn that existed at dispatch (`tasks/ar-closeout-deferred-rows.md`,
report.json CRLF, untracked plans/docs). Zero probe residue in history.

The probe does NOT weaken the byte analysis: serde attributes, field order,
and the enum's variants were never touched; only Copy was.

## 5. Site ledger (compiler-captured, own runs; tiers in dependency order)

Tier notes: `error[E0507]` = move out of shared/mutable reference,
`error[E0382]` = use-after-move / partial move; all fixable ONLY at or
before the use site, i.e. in the listed file.

Production, IN-SET (contract-blessed or touch-set files; 8 sites):
1. `crates/lang-api/src/identity.rs` `&`-From body - already commanded.
2. `crates/cli/src/registry.rs:29` E0382 (bail borrows after move; clone at :28).
3. `crates/cli/src/registry.rs:175` E0382 (clone at :174).
4. `crates/cli/src/registry.rs:192` E0507 (the ruling's predicted :191 clone).
5. `crates/cli/src/registry.rs:448` E0507 `*language` (in-file test mod).
6. `crates/cli/src/registry.rs:520` E0507 (in-file test mod).
7. `crates/pipeline/src/stage_index.rs:110` E0507 (named rider).
8. `crates/pipeline/src/stage_index.rs:122` E0507 (named rider; note the
   ruling cited ":103/:110" - the `match self.language_filter` at :103 with
   a discriminant-only `Some(_)` pattern does NOT move; :122 does).

Production, OUT-OF-SET (24 sites, 9 files - cap is 6 sites TOTAL):
9-10. `crates/build/src/discovery.rs:253`, `:272` - E0507. MUST-NOT-touch crate
   (ADAPT-8 frozen territory: "do not 'help' it").
11-13. `crates/artifact/src/manager/discovery_io.rs:51`, `:130`, `:131` -
   E0507. MUST-NOT-touch crate.
14-16. `crates/artifact/src/manager/m22_manifest_languages_tests.rs:39`,
   `:154`, `:169` - E0507 (`*lang` derefs in an existing src test mod).
17. `crates/pipeline/src/stage_load.rs:183` - E0507 (stage_load is NOT the
   named rider file; the AR-08 lock suite lives in its own file UNEDITED,
   but this production fn still needs a token edit).
18-22. `crates/pipeline/src/for_incremental.rs:66/67 (E0382), :85 (E0507
   FnMut closure), :120-125 (E0382 partial move), :141/:143 (E0382 moved)` -
   5 fix sites in the AR-22 constructor. This file was predicted NOWHERE:
   the journal predicted "registry.rs + possibly 1-2 worker-core", ruling
   section 4 added "one missed site" (stage_index) - neither enumerated
   `for_incremental.rs`, `lib.rs`, `stage_load.rs`, or any cli command file.
23. `crates/pipeline/src/lib.rs:681` - E0507.
24-29. `crates/cli/src/commands/analyze.rs:93 (E0382), :143, :148, :154,
   :169, :262 (E0507)` - 6 sites.
30. `crates/cli/src/commands/index.rs:77` - E0382.
31-32. `crates/cli/src/commands/ui/node_maps.rs:59` (E0507 `.entry(key)`),
   `:62` (E0507 `*language`) - 2 sites.

Existing TEST files, byte-frozen per contract ("EVERY existing file under
tests/** and crates/*/tests/**"):
33-36. `crates/pipeline/tests/for_incremental_parity_test.rs:98, :114, :153,
   :163` - 4 sites, compiler-captured (AR-22 parity suite; UNEDITED gate
   expectation `5/0/0/1` in the pipeline suite set).
37-38 (read-analysis; compiler-blocked: the integration package cannot
   compile while the cli lib errors, so these never reached rustc in the
   probe): `tests/integration/tests/m22_scala_load_analysis_test.rs:280`
   `record.language.into()` and `tests/integration/tests/m23_symbol_isolation_test.rs:406`
   `LanguageId::from(record.language)` - both read `SourceLanguage` out of
   `&FileRecord`; identical E0507 class to the empirically confirmed
   `stage_load.rs:183` twin. m23 is the pinned 7/0/0/1 gate lock.
39. UNMEASURABLE remainder: any further frozen-test or bin-target sites in
   `ariadne-integration-tests` / `ariadne-cli` bins stay hidden until the
   production riders land; the count of 38 is a LOWER bound.

What the probe confirmed INTACT (good news for any ruling):
- `crates/worker-core/src/tu_store.rs:210` truly is a NO-OP (partial move of
  an owned local) - worker-core never errored in any tier. Ruling right there.
- `crates/ui/**`, `crates/index/**`, `crates/graph/**`, `crates/cluster/**`,
  `crates/report/**`, `crates/scheduler/**`, `crates/lang/**`, `crates/lang-cpp/**`,
  `crates/lang-scala/**`, `crates/build/**` (besides the 2 sites), `tools/clang-worker`:
  ZERO copy-loss errors - every `SourceLanguage::Cpp/Scala/Other` CONST,
  struct-literal, and `assert_eq!` use compiles through the alias unchanged,
  exactly as ruling section 3 claimed (m24:199-212 literal class included).
- `crates/artifact/src/manager/metrics_io.rs:241` compiles UNEDITED via the
  kept `From<&SourceLanguage>` (it maps over `&SourceLanguage`), exactly as
  ruled. Design fact for the ruling: by-reference iteration
  (`.map(|f| &f.language)`) rides that kept impl and is clone-free wherever
  the consumer accepts `Item = &SourceLanguage` (`collect_stage_languages`
  takes `Item = impl Into<LanguageId>` and `&LanguageId: Into<LanguageId>`
  through the same kept impl) - several ledger sites could adapt WITHOUT
  clones, but every one of those adaptations is still an out-of-set FILE edit.
- serde attributes / field order / bincode framing were untouched by the
  probe, so this STOP does not endanger any byte claim of the ruling; it
  endangers only the touch-set size and the frozen-file guarantees.

## 6. Ruling claims vs measurement (errata for the successor ruling)

(i) Ruling section 4: "The journal missed ONE forced out-of-set consumer" -
measured: 24 production sites in 9 files. (ii) Section 3 "the std reflexive
blanket covers every value site (registry.rs:28, stage_index.rs:213,
m17:78/92, m19:115, m22:93/101)" - that list is right about the sites it
names (all are by-value params or consts, all fine) but never enumerated the
borrow-based reads (`analyze.rs`, `index.rs`, `node_maps.rs`, `stage_load.rs`,
`for_incremental.rs`, `pipeline/lib.rs`, `build`, `artifact`, `m22:280`,
`m23:406`). (iii) stage_index `:103` does NOT need a clone (discriminant-only
match); `:122` does - a net wash, but worth correcting so the next brief
does not bless a non-site. (iv) `crates/pipeline/src/for_incremental.rs`
(AR-22) structurally cannot avoid it: it holds `Option<SourceLanguage>` and
uses it after move in four places.

## 7. Options for the re-ruling (for the architect; implementers do not self-authorize)

A. (RECOMMENDED) Enumerated-rider amendment to ruling section 4: replace the
   6-site cap with the EXACT site list above; bless, per site, the minimal
   copy-semantics adaptation (clone token, or `&`-iteration riding the kept
   `From<&T>` impl where clone-free); explicitly relax "MUST NOT touch
   crates/build/** / crates/artifact/**" for copy-semantics-only tokens
   (ADAPT-8 is about types and bytes - a `.clone()` at `:253/:272` moves
   neither); rule the frozen-test question ONCE: "copy-semantics tokens in
   existing test files are sanctioned class-wide (assertion wording
   byte-preserved, suite counts unchanged; AR-24 D2 / AR-25 D1 accepted
   class), implementer journals the full ledger + any newly surfaced site
   stops again". Cost: +~30 single-token edits across 12 files, zero bytes,
   zero test-count drift; T-a grows toward the upper bound but stays one
   compile-green unit.
B. Re-split per ruling section 8's own prescription: new hop T-a0 = pure
   consumer adaptation (all 24 production + frozen-test sites) landed
   green-first with its own tiny locks, then T-a (move+serde) and T-b
   unchanged. Larger ceremony, but keeps every handoff under Planning Law
   and gives the reviewer a token-only commit to adjudicate (strongest
   precedent: AR-31 characterization-first).
C. Identity-representation redesign to keep `Copy` (e.g. interned/'static-only
   ids) - REJECTED here: JSON/cache.json deserialization must own
   heap-backed open ids (`serde_json::from_str("\"Fortran\"")` unit lock
   `language_id_round_trips_the_three_legacy_names_through_json` proves
   open ids from owned strings are load-bearing); a Copy representation
   kills the open-set mission. This is C-class territory.
D. Epoch bump (2.2.0) - does NOT touch this problem at all (copy-loss is a
   compile-time class, not a byte-format class; no warm-rebuild mechanism is
   involved). Not an option; listed to prevent misdiagnosis in the queue.

## 8. Contract-step disposition

- Step 1 (bootstrap + journal): DONE (this file + section 2 baselines).
- Step 2 (ORACLE COMMIT, fixtures green PRE-mutation): NOT AUTHORED. A byte
  oracle for a contract that cannot compile would be dead weight in history;
  the predecessor's byte evidence is still the durable fixtures-to-be and the
  ruling's encoding contract stands untouched. The successor lands the four
  oracle files unchanged against the (then-final) scope.
- Steps 3-7 (move, serde, riders, gates, commit): NOT STARTED. Zero code,
  zero test, zero lock lines touched; tree byte-clean at 85f40d5 (foreign
  churn untouched: `tasks/ar-closeout-deferred-rows.md` + report.json CRLF +
  untracked plans; docs lane owns `tasks/ar-27-*handoff.md`/`shape-ruling` -
  never staged from this lane).

## 9. Gate record for THIS journal-only hop

- Gates 2-7 measured as PRE-mutation baselines (section 2) and re-green is
  moot: the tree is byte-identical to dispatch HEAD.
- gate 9/10: `encoding_guard.py --all` silent before the commit; raw
  `git add` is off this lane's allowlist, so `--staged` runs after the
  scoped commit (empty-index trivial pass) - disclosed per AR-27 D2 /
  AR-20b D2 precedent, `--all` is the superset covering this commit.
- `hygiene_check.py`: zero blocking rows before commit (section 2); this
  journal is Markdown, out of the scanner's code scope.

## 10. Refused-command log (verbatim; catch-all boilerplate
`{"permission":"bash","pattern":"*","action":"deny","source":"global"}` is
reported regardless of true cause)

- `git log --oneline -6` (raw) -> refused -> substituted
  `python tools/distill.py -- git log --oneline -3` (PASS).
- `python tools/distill.py -- rg -n "^(error|warning: unused|error\[)" ...`
  -> refused (catch-all; the `|`/bracket pattern class is the known
  environment limitation from AR-26a2) -> substituted three `-e` flags in
  ONE invocation (PASS, identical semantics).
- `python tools/distill.py -- rg -n "^error\[E" -e ... "-e --> crates.cli" target/distill-capture.txt`
  second variant -> refused -> substituted Read-tool on the capture (distill
  self-clobber finding re-confirmed: the rg run overwrote the check capture;
  check was re-run and the capture read directly - AR-21 journal finding).
- `git rev-parse HEAD` / `git status --porcelain` ran ONLY via distiller per
  contract (PASS). No `python -c` was attempted (known refusal).
- Capture files quoted from own stdout with paths per the capture-race rule.

## 11. Successor bootstrap

Fresh session, after the re-ruling lands (do NOT resume this one):
`python tools/checkpoint_guard.py resume --spec tasks/ar-27-ta-handoff.md
--scope ... --cmd ...`
Read first: `tasks/ar-27-ta-handoff.md`, this journal (sections 5-7), the new
ruling/erratum, then the old predecessor journal sections 2/5 for the byte
evidence. Baselines in section 2 are current as of `85f40d5` and cheap to
re-verify. Board: HOLD posted to main with this verdict.
