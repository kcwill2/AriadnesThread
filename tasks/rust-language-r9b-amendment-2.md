# R-9b AMENDMENT-2 (dated 2026-10-26, planner) - split R-9b into R-9b1 -> R-9b2, harvest the dead-session partials, grant the manifest rider

Milestone: `rust-language` (unregistered post-M28 track, NO M-number).
This amendment supersedes `tasks/rust-language-r9b-handoff.md` where they
conflict; `tasks/rust-language-r9-amendment-1.md` remains governing law except
where this document (which the user authorized 2026-10-26) amends it.
HEAD of record: `cef3b0a` (R-9b duty 1 - R-9a verdict + ledger relay clause -
LANDED; do not re-land, do not re-dispatch duty 1).

## 0. Preamble - the two deaths and the split

Incident class: two R-9b implementer sessions died at the context wall with
empty summaries (R9B-DEADSESSION-1, R9B-DEADSESSION-2).
Both are clean-boundary class: no half-written file beyond the inventoried
dirty state, no stash (`refs/stash` absent), no branch change.
Per the context-wall law a dead session is NEVER resumed; each successor
bootstraps from disk via `resume-checkpoint` / `checkpoint_guard.py resume`
(sec. 7).

The journal on disk (`tasks/rust-language-r9b-journal.md`, untracked) is the
decision log of the partial-bearing hand: it records starting clean at HEAD
`6b9dae0`, landing duty 1 (`cef3b0a`), running gate 1 (baseline) and gate 2
(Red), then dying. The dispatch preamble counts both deaths as post-duty-1;
the journal says the first attempt died at zero work before duty 1. The disk
cannot adjudicate the head-count (a post-`cef3b0a` death that wrote nothing
leaves no trace); it CAN attribute every byte of partial state to the
journal-bearing hand. The harvest below is from the actual diffs, so the
head-count dispute does not affect any contract.

User decisions of record (2026-10-26, relayed in the dispatch):
1. SPLIT the R-9b remainder into two serial hands: **R-9b1** (in-src
   enrollment: planning hooks, probe-only validator, F2 freeze, the final
   all-true caps flip + its in-src pins) -> **R-9b2** (the integration
   symbol-isolation suite + the manifest consumption).
2. GRANT the manifest rider: an `ariadne-lang-rust` dev-dependency row in
   `tests/integration/Cargo.toml` (+ `Cargo.lock` regeneration), precedent:
   the lang-scala M19.3 dev-dep row (same file, same comment idiom).

Split rationale (sizing law): the pre-split R-9b was declared AT the ceiling
(~1230 lines, 9+trio gates) and two sessions died inside it. R-9b1's landing
is one crate's src dir (~700-850 lines, 8 gates + commit pair); R-9b2's is
one new suite file + two manifest lines (~650-750 lines, 8 gates + trio).
The seam is right because R-9b1 ships GREEN-ENDABLE alone: the caps flip is
byte-complete in-src, the four designed reds ride the same rider band as
before, and the suite consumes finished mechanism.
Rider serialization: R-9b1 landing -> orchestrator's four-lock rider commit
(unchanged AMENDMENT-1 sec. 4 band) -> R-9b1 reviewer's fresh bare census ->
verdict -> R-9b2 duty 1. Nothing is parallel-safe: the hands share
`plugin.rs`, the ledger, and the suite substrate.

## 1. Dirty-state harvest - R-9b1's adopt-and-verify starting inventory

Source: `git diff` of `crates/lang-rust/src/plugin.rs` (+51/-23) and
`crates/lang-rust/src/constraints.rs` (+62/-1), the untracked journal
(secs. 1-4), and the live blobs of `validator.rs`, `constraints.rs`,
`plugin.rs`, `lib.rs`. Every line below is diff-checked, not journal-taken.

ADOPTED (already on disk, ride R-9b1's landing commit; verify, never rewrite):
- duty 1: verdict + ledger relay clause + resume-artifact deletion - COMMITTED
  at `cef3b0a`. DONE, not adoptable work. (journal sec. 1)
- journal secs. 1-4 incl. the F6 consumption mapping and the executed gate-2
  Red record. Adopted as append-only history; R-9b1 extends from sec. 5.
- F2 test-side: `constraints.rs` test module +62/-1 - `DeclId` import plus
  the two named positive pins `module_kind_records_are_planning_frozen` and
  `two_declaration_fold_records_are_planning_frozen`; the existing
  `no_colocated_groups_and_no_frozen_symbols` batch legs are byte-identical
  (AMENDMENT-1 sec. 2 shape). TEST-SIDE ONLY - production `frozen_symbols`
  still answers an empty `HashSet` (blob-checked).
- caps pin rename + red literal: `plugin.rs` test module -
  `capabilities_match_the_load_metric_flip_literal` renamed to
  `capabilities_match_the_parity_flip_literal` with the all-true
  `right:`-side literal (planning/extraction/rewriting/validation true,
  `requires_toolchain: false`). TEST-SIDE ONLY - production caps literal at
  plugin.rs:151-162 still false on those four bits (blob-checked).
- seam-leg pin evolution: `plugin_default_seams_stay_at_trait_defaults` hook
  legs rewritten to Some-dispatch pins (out-of-line `mod x;` -> `Some(true)`;
  inline `mod x {{ .. }}` -> `Some(false)`; unreadable -> `Some(false)`;
  `target_path_for` -> `Some(src/fixture_marker.rs)`), tempfile-backed
  specimen dir (tempfile IS a lang-rust dev-dep - blob-checked, compiles).
  TEST-SIDE ONLY - plugin.rs carries no hook overrides (blob-checked).
- executed gate-2 Red: `92 passed / 4 failed / 0 ignored / 1 suites` with the
  four named failures and verbatim `left:` lines (journal sec. 4). Arithmetic
  consistent with the diffs: 94 baseline + 2 new constraints pins = 96 =
  92 passed + 4 failed. The `{` brace-escape harness note is corroborated by
  the diff (`{{ .. }}`, `{{symbol}}`). NO CONTRADICTION between journal and
  diffs on any item.

NOT DONE (remaining R-9b1 work - all production):
- `planning.rs` - DOES NOT EXIST (glob-checked; the dead session never
  started it). Full new file per handoff item 1.
- `validator.rs` - still the R-3 stub: unconditional `Missing`, both stub
  pins present, no `CommandRunner` seam, no `with_runner`, no
  `TOOLCHAIN_PROBE_ORDER` (blob-checked).
- `plugin.rs` production - caps literal NOT flipped; hook overrides absent.
- `constraints.rs` production - `frozen_symbols` still empty; the R-3 header
  line "This file does not grow" still stands un-reworded and the docblock
  still says `frozen_symbols` stays empty (stale prose - AMENDMENT-1 sec. 2
  commands the reword + IV3 sentence; NOT done).
- `lib.rs` - no `mod planning;` line.
- shape-table IV3 row - `tasks/rust-language-shape.md:290` still reads
  "`frozen_symbols` empty (IV3: policy owns freezing - Scala parity)" -
  stale prose AMENDMENT-1 sec. 2 commands updating in this landing; the
  original R-9b touch-set omitted the file while commanding "no stale prose
  survives". ERRATUM: the row is granted to R-9b1's touch-set (sec. 3).
- F5 named test - NOT authored anywhere. See the sec. 3 location ruling.
- integration suite - NOT started (stays in R-9b2 by design).
- gates 3-9 of the old handoff - NOT run (baseline + Red only).

## 2. Adopt-and-verify evidence standard (relay basis, per the R-7 precedent)

R-7 successor precedent (`rust-language-r7-journal.md` "Red evidence" +
`rust-language-r7-verdict.md` Adjudication 1): adopted work whose Red was
never observed by the current hand owes three legs - (1) relay leg (the
predecessor's durable record, labeled as relay), (2) fresh mechanical
corroboration (the successor's own machine truth), (3) falsifiability (source
read that the pin cannot pass the pre-change bytes). R-7 excused re-observation
ONLY because observing the red would have meant reverting the landed flip in
the tree. That excuse does NOT exist here: the adopted Red is the current
disk state, so re-observing it costs one gate invocation.

STANDARD for R-9b1 (machine-checkable):
- Leg 1 (relay): journal sec. 4 - canonical line `92 passed / 4 failed / 0
  ignored / 1 suites` + the four verbatim `left:` lines. Quote as relay, not
  as re-observed data.
- Leg 2 (REQUIRED fresh): gate 1 below re-runs the scoped suite against the
  dirty tree and MUST reproduce the journal line and the four failing names
  exactly. A mismatch is a disk-vs-journal contradiction: STOP + board ASK,
  do not proceed.
- Leg 3 (falsifiability, journal a 2-line source-read note): the renamed caps
  pin asserts an INDEPENDENT all-true full-struct literal that the current
  production struct (four bits false) cannot satisfy; the seam pins assert
  Some-dispatch bytes the trait defaults (None) cannot satisfy; the F2 pins
  assert frozen-contains answers the empty-HashSet stub cannot give.
- Probe tests owe NO red: they cannot compile against the stub and a compile
  error is not Red. Journal sec. 3's reasoning stands and lands as history;
  the five probe tests land WITH the API in the Green step (R-9a verdict D4
  precedent). The reviewer's substitute audit: every probe-test spawn is
  `--version`-shaped (INV-10b) and the two superseded stub pins retire with
  positive replacement, never weakening (AMENDMENT-1 sec. 9).

## 3. R-9b1 CONTRACT - in-src enrollment (planning hooks, probe validator, F2 freeze, all-true flip)

Duty 1: NONE (`cef3b0a` landed - do not re-land the verdict or the ledger
clause; the resume-artifact deletion is done). Mandatory instead: ADOPT the
dirty state (sec. 1) and EXTEND `tasks/rust-language-r9b-journal.md` from
sec. 5 (secs. 1-4 are the dead hand's decision log - byte-UNEDITED,
append-only per the ar-27 ta-journal precedent; supersede the sec. 2
two-link F6 stand-in with a DATED append, not an edit - the manifest grant
supersedes it).

Objective + invariants: land every in-src byte of the enrollment hop so the
crate self-certifies the final all-true capabilities with every bit behind a
shipped component (INV-5). Invariants riding untouched: caps hunk gains
EXACTLY four bits (planning/extraction/rewriting/validation);
`load_metric` must NOT re-appear (R-7 owns it); `requires_toolchain: false`
(A3/D-2); INV-10 a/b/c as in-src scripted tests; INV-8 fingerprint `"rust-1"`
and `CURRENT_ANALYSIS_VERSION` `2.2.0`: zero hunk;
`CyclePolicy::Reject` literals: zero hunk.

Touch-set (hard; any edit outside = blocker, report don't fix):
- `crates/lang-rust/src/planning.rs` (NEW: `aggregation_filter(repo_root,
  path)` + `target_path_for(path, symbol)` + sanitize helpers mirroring
  `scala_target_path_for`; handoff item 1 shape VERBATIM; `Some(true)` iff
  syn-parses AND declares >= 1 NON-INLINE `mod x;`; unreadable/unparseable ->
  `Some(false)` - invents nothing, Scala parity; same
  `sanitize_module_stem` single source of truth R-9a rides)
- `crates/lang-rust/src/validator.rs` (stub -> probe-only real:
  `TOOLCHAIN_PROBE_ORDER == ["cargo", "rustc"]`; `CommandRunner` seam +
  `CommandOutcome` + `ProcessCommandRunner` + `with_runner` mirroring
  `crates/lang-scala/src/validator.rs` 1-for-1 INCLUDING its pub surface, so
  R-9b2 can construct a scripted runner from OUTSIDE the crate;
  `toolchain_status()` -> `Available(first banner)` | `Missing` via the
  injected runner; `validate_compile` uses `rustc` specifically,
  options-before-sources, single-input honesty doc; offline error names the
  D-M23-12-class deferral, spawning nothing beyond the probe walk (INV-10c);
  the five named probe tests from handoff item 2 -
  `probe_order_is_cargo_then_rustc`, `probe_reports_missing_when_no_probe_is_usable`,
  `every_probe_spawn_is_a_probe_order_entry_with_version_flag`,
  `validate_compile_offline_errors_naming_the_deferral_spawning_nothing_beyond_the_probe_walk`,
  `probe_reports_available_with_version_banner_of_first_usable_probe` -
  supersede the two stub pins by NAME: `toolchain_status_is_missing_never_unknown`
  and `validate_compile_errors_naming_the_deferral`, per AMENDMENT-1 sec. 9
  implementer band with positive replacement, never weakening)
- `crates/lang-rust/src/plugin.rs` (ADOPT the dirty test hunk; ADD the
  production hunks: hook-method overrides delegating to planning.rs -
  registry dispatch enrolls them with ZERO CLI edit; FINAL caps literal
  byte-exact per INV-5)
- `crates/lang-rust/src/constraints.rs` (ADOPT the dirty test hunk; ADD the
  production freeze per AMENDMENT-1 sec. 2 - union of Module-kind records and
  `declarations.len() >= 2` records; `colocated_groups` stays empty; REWORD
  the stale R-3 header line and the IV3 docblock sentence honestly in the
  same commit - IV3 sentence: the policy layer owns freezing symbol
  INSTANCES; this rule freezes a symbol KIND plus provably-ambiguous folds at
  the language layer, which is what the M18.7 constraints seam exists for)
- `crates/lang-rust/src/lib.rs` (one `mod planning;` line + `pub use` if the
  planning surface needs exporting for symmetry with siblings - minimal)
- `tasks/rust-language-shape.md` (ONE dated edit at the :290 IV3
  decision-table row, per sec. 1's erratum; nothing else in the file)
- `tasks/rust-language-r9b-journal.md` (adopt + append-only extension)
You MUST NOT touch: `dependency_rewriter.rs`, `mod_tree.rs`,
`move_retention.rs`, `extractor.rs`, analyzer/resolver/parser/shaper/
boundary/detector files (R-8/R-9a-landed - CONSUME, derive asserts from their
pinned spellings); `Cargo.toml` / `Cargo.lock` anywhere (the granted dev-dep
rides R-9b2); `tests/**` INCLUDING the four integration caps locks
(`rust_language_discovery_test.rs::capabilities_lock_the_registration_flip_literal`,
`rust_language_semantic_index_test.rs::capabilities_lock_the_semantic_index_flip_literal`,
`rust_language_dependency_graph_test.rs::capabilities_lock_the_dependency_graph_flip_literal`,
`rust_language_load_analysis_test.rs::capabilities_lock_the_load_metric_flip_literal`)
- orchestrator rider band, report-don't-fix + board ASK + journal on first
red, never an edit; `crates/cli/**`, `crates/cluster/**`, `crates/validate/**`,
`crates/lang/**`, `crates/lang-cpp/**`, `crates/lang-scala/**`, `tools/**`
(all zero-hunk); fixtures (read-only), m18-m23 incl. m23, m00, ar27, goldens.
No `git stash`, no branch change.

F2/F5 location ruling (planner ruling against the dispatch brief, stated
plainly): the dispatch named F5's test under R-9b1, but AMENDMENT-1 sec. 5
locates the exact test `flat_id_shared_mod_and_fn_record_is_planning_frozen`
IN THE SUITE, and the user's own R-9b1 rule is NO INTEGRATION SUITE in this
hand. Its legs 2-3 (wired plans contain no `crate::greeting` proposal; the
suite-authored temp `twin` pair) run only against a wired plan on a temp
copy - forcing them into the crate would mean inventing a cross-crate plan
harness the shape never authorized. RULING: R-9b1 carries the F2 freeze and
its two constraints-seam witnesses (the adopted pins, which are the F5 class
neutralized at the planning seam); the exact-named F5 test with all three
legs lands in R-9b2's suite, byte-where-AMENDMENT-1-sec-5-put-it. No scope
is lost; the name is reserved - do not spend it elsewhere.

Frozen locks (review-enforced, unchanged from the R-9b handoff):
- The 29 R-8 witnesses + the 11 R-9a pins: UNEDITED; a witness failing is a
  STOP, not a test edit.
- m23 / m18-m22 + m24 / m00 / ar27 / goldens / fixtures / lang-cpp /
  lang-scala / cluster suites (incl.
  `planner_injection.rs::frozen_symbols_are_skipped_from_proposals` + the
  m18.7 SCC lock) UNEDITED with empty-diff proofs by name.
- `CURRENT_ANALYSIS_VERSION` `2.2.0`, fingerprint `"rust-1"`, the three
  `CyclePolicy::Reject` literals: zero hunk.
- Superseded register: ONLY the two validator stub pins (named above, with
  positive replacement). The four caps locks are superseded by the
  ORCHESTRATOR'S rider only. Nothing else in any frozen lock is superseded.

RGB protocol: Red is ON DISK (adopt, re-observe per sec. 2, never rewrite).
Green: the production hunks above. Refactor: honest names, doc comments,
re-run. DESIGNED-RED DISCLOSURE (AMENDMENT-1 sec. 4, unchanged): the caps
flip turns EXACTLY the four integration caps locks red; they are outside the
touch-set; the gate-7 census runs no-fail-fast with the four failures
attributed by name; board ASK + journal the line; the rider turns them AFTER
this landing and BEFORE the reviewer's fresh census. Four-red is the designed
window, not a broken tree.

Hygiene ceilings at risk: `plugin.rs` growth (hook overrides + literal -
stays < 1000), `validator.rs` (mirror ~200), planning < 400 comfortably,
functions < 100, banned names clean (no tmp/res/foo/bar/test1, no
single-letter locals), authored bytes pure ASCII, `hygiene_check.py` exit 0.

Gates (distilling entry points only, 600000 ms each; 8 + commit PAIR - no
duty-1 commit):
1. `python tools/testrun.py --distill -p ariadne-lang-rust` - ADOPT-AND-VERIFY
   Red re-observation (sec. 2 leg 2): expected `92 passed / 4 failed / 0
   ignored / 1 suites` with exactly the four journal-sec.-4 names; mismatch =
   contradiction STOP.
2. same - Green: `94 + M / 0 / 0 / 2` with M attributed BY NAME (net probe
   set +3 after the two stub retirements, +2 adopted constraints pins, +P
   planning in-src witnesses); R-8's 29 witnesses + R-9a's 11 pins green
   UNEDITED inside the attribution.
3. same - refactor re-run, same line.
4. `python tools/testrun.py --distill -p ariadne-integration-tests --test m23_symbol_isolation_test` - UNEDITED green (`7 passed / 0 failed / 0 ignored / 1 suites`).
5. `python tools/testrun.py --distill -p ariadne-integration-tests --test planner_injection` - UNEDITED green (the freeze stays neutrally locked; AMENDMENT-1 sec. 2 names this lock).
6. `python tools/testrun.py --distill -p ariadne-cli` - UNEDITED green (registry dispatch consumed, not edited; the dispatcher's empty-frozen test feeds Scala/C++ records and stays green).
7. `python tools/testrun.py --distill` - full census, no-fail-fast form:
   journal the OBSERVED line (do not pre-state a pre-rider number); the
   failure set is EXACTLY the four sec. 3 rider locks, each `left:` showing
   the flipped struct; pass count is 4 lower than the post-rider anchor by
   design - attribute the 4, journal them, post the board ASK.
8. `python tools/hygiene_check.py` - exit 0.
Commit pair: `python tools/encoding_guard.py --staged`, then
`python tools/commit_scoped.py --scope crates/lang-rust --scope tasks/rust-language-r9b-journal.md --scope tasks/rust-language-shape.md -m "feat(lang-rust): planning hooks, probe-only validator, F2 freeze, final all-true capabilities in-src (rust-language R-9b1)"`.

Produced-line estimate (landing diff rides ALL dirty bytes): planning
~260-300, validator ~190-210, plugin production ~50-60, constraints
production + prose ~30-40, lib.rs 1-2, shape row 2-3, journal append ~30-45,
plus the adopted test hunks (+113/-24) => ~700-850. Under the ~1200 ceiling
with margin; gates 8 <= 10; scope one crate src dir (+ one prose line,
declared).
Census expectation (form, not number): landing census (pre-rider) = `1981 +
M - 4 passed / 4 failed / 13 ignored / 125 suites` observed at gate 7; after
the rider the R-9b1 reviewer's fresh bare census must show `>= 1981 + M / 0
failed / 13 ignored / 125 suites @ <landing+rider>` with the anchor
`1981 / 0 / 13 / 125 @ fcb078c` as the base of record (R-9a). Suites stay
125; ignored flat 13.
Bench posture (reviewer's acceptance pass): `crates/lang-rust/src` touched ->
`python tools/distill.py -- python tools/bench_compare.py` RUN fresh; WARNs
recorded, not attributed; no baseline/corpus change (D-3). Implementer never
runs bare bench/raw cargo.
Reviewer gate: skill `review-gate`; per-commit `git show` oldest-first vs the
sec. 3 touch-set; caps hunk bit-count audit (EXACTLY four bits gained;
`load_metric` absent; all-true + `requires_toolchain: false` final); the
adopted hunks byte-identical to the disk state at dispatch (adoption
integrity - `git diff cef3b0a..<landing>` on plugin/constraints shows the
dead session's test hunks UNEDITED plus production additions only); probe
tests `--version`-shaped spawns only; stub-pin retirement with named
positive replacement; m23 zero-diff; cli zero-diff; journal secs. 1-4
byte-identical; `refs/stash` absent; designed-red chain audit (exactly four
rider files, four bits, `4 -> 8` true-count line on the load-analysis lock).

## 4. R-9b2 CONTRACT - the symbol-isolation suite + the granted manifest consumption

Duty 1 (own commit `review(rust-language): land R-9b1 verdict APPROVED +
ledger relay clause (R-9b2 duty 1)`): land `tasks/rust-language-r9b1-verdict.md`
as authored by the R-9b1 reviewer; append the sec. 6 clause to the ledger
live row (count-of-1 verified against the landed blob); do NOT re-land
`c52817c` / `f2387f1` / `fcb078c` / `cef3b0a` / R-9b1's landing / the rider.

Touch-set (hard):
- `tasks/rust-language-r9b1-verdict.md` + `tasks/milestone_ledger.md` (duty 1 only)
- `tests/integration/tests/rust_language_symbol_isolation_test.rs` (NEW suite)
- `tests/integration/Cargo.toml` (GRANTED: one dev-dep row,
  `ariadne-lang-rust = { path = "../../crates/lang-rust" }`, with the
  dated-comment precedent mirroring the lang-scala M19.3 row: `# rust-language R-9b: the symbol-isolation suite consumes RustToolchainValidator::with_runner (F6 tier-3 Skipped chain).`)
- `Cargo.lock` (machine regen of that row - declared machine bytes, per plan
  sec. 5 the reviewer rule keeps it in the touch-set)
- `tasks/rust-language-r9b-journal.md` (append-only, from where R-9b1 ended)
You MUST NOT touch: ANY `crates/**` byte including lang-rust (everything
landed - the suite CONSUMES; if a consumed behavior is wrong, STOP + report,
report-don't-fix, the fix is an amendment not a hunk); the four caps locks;
`m23_symbol_isolation_test.rs`; fixtures; tools; goldens; cluster/cli/
validate/lang suites. No `git stash`, no branch change.

What to build - the suite, stages per the R-9b handoff item 5 and
AMENDMENT-1 sec. 7, which stand VERBATIM (fixture-driven on a temp copy,
fixture read-only L3):
- plan: aggregation candidates = `src/lib.rs` + `src/models/mod.rs` (hook
  dispatch); proposals EXACTLY the two fn extractions
  (`fixture_marker -> src/fixture_marker.rs`,
  `models_marker -> src/models/models_marker.rs`); ZERO proposals naming a
  Module-kind symbol, ZERO targeting a frozen child, ZERO naming
  `crate::greeting`.
- preview: consumer discovery targets listed; the parent-MODIFIED
  grown-oracle assert NON-VACUOUS - names the parent path AND the inserted
  `mod` line, on the engine's real dry-run bytes; STOP-and-report if the
  frozen engine cannot honestly report it (engine stays frozen).
- apply - m23-mirror assertion VERBATIM (AMENDMENT-1 sec. 1c option (b) +
  the R-9a verdict "F1 FINAL WORD"): temp consumer files authored IN the temp
  copy only (R-6 precedent, added to lib.rs's copy with `mod consumer;`);
  consumer bytes contain `use crate::fixture_marker::fixture_marker;` and NOT
  the old one-line import; same pair for `models_marker`
  (`use crate::models::models_marker::models_marker;`). WEAKENING THIS
  ASSERTION (dropping the not-old leg, asserting presence only, or
  re-wording) = BLOCKER. Parents lose the fn bytes and gain their `mod` line
  exactly once; re-apply discipline = rollback-then-re-apply byte convergence
  + the R-9a-landed hook-level idempotence pins; no frozen fixture child
  modified anywhere (byte census of `src/greeting.rs` / `src/models/user.rs`).
- rollback: bytes back (`tree_digest` m23 idiom); re-plan converges to the
  pre-apply plan.
- tier-3: `Skipped` via `RustToolchainValidator::with_runner(scripted
  all-None)` consumed by `TierValidator::validate_affected_tus`
  (`crates/validate` mapping CONSUMED, never re-implemented). The manifest
  grant dissolves the journal-sec.-2 two-link stand-in: build the REAL
  composition (the journal's own words: "swaps the stand-in ...
  byte-for-byte"); a dated journal append records the supersession of the
  stand-in disposition - sec. 2 itself stays byte-UNEDITED. The suite NEVER
  asserts `Skipped` against the real environment (cargo/rustc ARE on PATH)
  and NEVER spawns beyond `--version` probes (INV-10).
- F5 (AMENDMENT-1 sec. 5, EXACT test name
  `flat_id_shared_mod_and_fn_record_is_planning_frozen`, located HERE per the
  sec. 3 ruling): (1) the reduced model answers EXACTLY ONE record under
  `crate::greeting` with `declarations.len() == 2` (`#L4` fn decl + `#L6` mod
  decl), `declaring_files ==` the sorted pair - NEVER pinning the folded
  `kind`/`defining_file`; (2) the wired plans contain NO proposal naming
  `crate::greeting` and none targeting `src/greeting.rs`; (3) the same
  >=2-declaration freeze on a suite-authored temp pair (two temp files each
  `fn twin();` -> `crate::twin` frozen; any candidate listing it proposes
  nothing). Ruling of record stands: flat-id sharing is the ACCEPTED cost of
  the flat-id analyzer (R-5 D5), neutralized at the planning seam by the F2
  freeze, not by re-qualification; the honest limit is the dated closeout
  deferral row (R-10 rider material; `constraints.rs` docblock states it -
  R-9b1 landed).
- caps: the suite's own all-true byte-lock authored GREEN-SIDE with the flip
  (R-6 D5 precedent; the red evidence is R-9b1's renamed in-src pin + seam
  legs, already landed).

RGB protocol (stated honestly for a post-mechanism suite): author the whole
suite before ANY suite-side fix. R-9a/R-9b1 landed the mechanism first, so
stage-by-stage the run may show red (genuine) or green-at-birth (mechanism
already shipped) - capture per stage; green-at-birth stages are attributed as
justified-green-at-birth characterizations in the journal per the SM-3 /
R-9a-verdict-D4 named precedent, NEVER left silent. The non-vacuity duties
are the substituted evidence: the grown-oracle names the parent path AND the
inserted line; the apply assertion byte-compares to the AMENDMENT-1 sec. 7
VERBATIM bytes; F5 legs are meaningful, not vacuously true. Any red that
indicates a MECHANISM defect is a STOP-and-report, not a hunk in `crates/**`.

Frozen locks (review-enforced): every `crates/**` byte (zero-hunk proofs by
name), m23, m18-m22 + m24, m00, ar27, goldens, fixtures, the four caps locks,
all R-8/R-9a/R-9b1 witnesses UNEDITED (a witness failing is a STOP, not a
test edit). `CURRENT_ANALYSIS_VERSION` `2.2.0` / fingerprint `"rust-1"` /
`CyclePolicy::Reject`: zero hunk (trivially - no crates diff at all).

Gates (distilling entry points only, 600000 ms each; 8 + commit trio):
1. `python tools/testrun.py --distill` - boot baseline (post-rider, pre-suite):
   journal the observed anchor line; expected `0 failed / 13 ignored / 125
   suites` and a pass count >= the R-9b1 reviewer's certified post-rider
   anchor (proves the rider landed and the tree is clean).
2. `python tools/testrun.py --distill -p ariadne-integration-tests --test rust_language_symbol_isolation_test` - authoring run: capture red-where-real vs justified-green-at-birth, stage by stage.
3. same - Green, all stages + F5 + the caps byte-lock, each attributed by name.
4. `python tools/testrun.py --distill -p ariadne-integration-tests --test m23_symbol_isolation_test` - UNEDITED green (`7 passed / 0 failed / 0 ignored / 1 suites`).
5. `python tools/testrun.py --distill -p ariadne-lang-rust` - UNEDITED green at R-9b1's certified line (the manifest/suite landing perturbed no crate byte).
6. `python tools/testrun.py --distill -p ariadne-cli` - UNEDITED green.
7. `python tools/testrun.py --distill` - full census: post-rider clean, expected `>= 2023 / 0 failed / 13 ignored / 126 suites @ <R-9b2 landing>` - THE FORM, not the number: 2023 is the track floor (`1981 + 42`, plan sec. 7 R-9 lower bound, unchanged by the split); at dispatch time replace 2023 with `R-9b1 landing anchor + suite-K named` recomputed from R-9b1's verdict, floor never below 2023. Suites `125 -> 126` (+1: exactly one new integration binary); ignored flat 13; N attributed BY NAME.
8. `python tools/hygiene_check.py` - exit 0.
Commit trio: `python tools/encoding_guard.py --staged`, then duty-1
`python tools/commit_scoped.py --scope tasks/milestone_ledger.md --scope tasks/rust-language-r9b1-verdict.md -m "review(rust-language): land R-9b1 verdict APPROVED + ledger relay clause (R-9b2 duty 1)"`, then landing
`python tools/commit_scoped.py --scope tests/integration --scope tasks/rust-language-r9b-journal.md -m "test(integration): rust symbol-isolation suite + granted lang-rust dev-dep (rust-language R-9b2)"`.

Produced-line estimate: suite ~600-700, manifest +~4 (lock regen = machine
bytes), journal append ~25-40 => ~650-750. Gates 8 <= 10; scope one test
directory + the two granted manifest lines (declared vertical).
Bench posture: R-9b2 touches NO `crates/*/src` byte -> bench EXEMPT with the
zero-crates-diff proof (R-10 pattern). Reviewer gate: skill `review-gate`;
apply-assertion VERBATIM byte-compare vs AMENDMENT-1 sec. 7 (weakening =
blocker); `Skipped` mapping consumed not re-implemented; the dev-dep row is
the ONLY manifest hunk and carries the precedent comment; grown-oracle
non-vacuity; F5 three legs; m23 zero-diff; all-crates zero-diff; `refs/stash`
absent.

## 5. Standing law - BOTH hands (no silent narrowing)

- UTF-8: Write/Edit tools only; never Set-Content/Out-File/`>`/
  Get-Content-replace; authored bytes pure ASCII.
- Touch-set hard; report don't fix; the four rider locks: board ASK + journal
  on first red, never an edit.
- NO `git stash` (recurrence blocker; the reviewer re-proves `refs/stash`
  absent).
- `CURRENT_ANALYSIS_VERSION` `2.2.0` NEVER bumps; fingerprint `"rust-1"`,
  `CyclePolicy::Reject`: zero hunk.
- DESIGNED-RED DISCLOSURE (R-9b1): exactly the four caps locks; four-red
  pre-rider is the window, not a broken tree (AMENDMENT-1 sec. 4 oracle).
- Context-wall STOP law: at exhaustion STOP at a clean boundary, journal the
  disk inventory (what compiles, what's red, what's next), return; the dead
  session is NEVER resumed - successor runs the sec. 7 resume line for the
  CURRENT hand.
- Journal a checkpoint after EACH green gate; the RISK-MODTREE STOP clause
  still governs.
- SHA relay: each final message carries duty-1 + landing SHAs and
  `git show --stat` each; the next duty 1 cites them (reflog-verifiable
  chain; phantom-narration countermeasure). F14: no self-referential SHA in
  the journal.
- Bare high-volume commands only through distilling entry points
  (`docs/specs/tool-output-distillation.md`).
- No hand is parallel-safe (sec. 0).

## 6. Ledger relay drafts (orchestrator-owned ledger; drafts only - this doc commits nothing)

Landed WITH this amendment (the split + grant record):
`R-9b split R-9b1 -> R-9b2 per AMENDMENT-2 2026-10-26 (R9B-DEADSESSION-1/2 empty-summary deaths, clean-boundary class, never resumed; dirty partial harvested as R-9b1 adopt-and-verify inventory); manifest rider GRANTED (tests/integration dev-dep ariadne-lang-rust + Cargo.lock, lang-scala M19.3 precedent row); R-9b1 dispatched (in-src enrollment, no suite).`

R-9b1 clause (lands with R-9b2 duty 1, alongside the R-9b1 verdict - the
rider commit precedes it and rides the same band as AMENDMENT-1 sec. 4):
`R-9b1 LANDED <landing-sha> +rider <rider-sha> APPROVED 2026-10-26 (planning hooks + probe-only validator + F2 freeze + in-src all-true caps flip; four caps locks turned by rider; dead-session partial adopted with red re-observed), census anchor <A> passed / 0 failed / 13 ignored / 125 suites @ <rider-sha>, R-9b2 dispatched (symbol-isolation suite + granted manifest dev-dep).`

R-9b2 clause (lands with R-10 duty 1, alongside the R-9b2 verdict):
`R-9b2 LANDED <landing-sha> APPROVED 2026-10-26 (symbol-isolation suite: m23-VERBATIM apply, F5 named test, tier-3 Skipped via with_runner consumed through TierValidator; manifest dev-dep ariadne-lang-rust per lang-scala M19.3 precedent), census anchor <C> passed / 0 failed / 13 ignored / 126 suites @ <landing-sha>, R-10 dispatched (docs hop + parity suite + dated riders).`

## 7. Bootstrap lines (fresh sessions only; dead sessions never resumed)

R-9b1 dispatch (adopts the dirty tree at HEAD `cef3b0a`; run in the worktree,
never stash-first):
```
python tools/checkpoint_guard.py resume --spec tasks/rust-language-r9b-amendment-2.md --scope crates/lang-rust --cmd "python tools/testrun.py --distill -p ariadne-lang-rust"
```
Reads first: this amendment (sec. 1-3 + 5), `tasks/rust-language-r9b-journal.md`
(secs. 1-4, the dead hand's log), `tasks/rust-language-r9-amendment-1.md`
secs. 2/9/11, `tasks/rust-language-r9b-handoff.md` items 1-4 (shapes that
stand), then the model files: `crates/lang-scala/src/planning.rs`,
`crates/lang-scala/src/validator.rs` (+ its scripted-runner tests).
Journal: EXTEND `tasks/rust-language-r9b-journal.md` from sec. 5 - never a
new journal file.

R-9b2 dispatch (after the R-9b1 verdict + rider are landed):
```
python tools/checkpoint_guard.py resume --spec tasks/rust-language-r9b-amendment-2.md --scope tests/integration --cmd "python tools/testrun.py --distill -p ariadne-integration-tests --test m23_symbol_isolation_test"
```
Reads first: this amendment (sec. 4 + 5), the R-9b1 verdict, the journal (all
sections), `tasks/rust-language-r9-amendment-1.md` secs. 5/7,
`tasks/rust-language-r9a-verdict.md` "F1 FINAL WORD",
`tests/integration/tests/m23_symbol_isolation_test.rs` (idiom model),
`tests/integration/tests/m19_*` (dev-dep consumption idiom).

## 8. What stands unchanged / supersessions (say it plainly)

STAND: AMENDMENT-1 secs. 1-11 rulings except where sec. 0/3/4 above relocate
them; the R-9b handoff's contracts for items 1-4 shapes (hooks VERBATIM,
validator contract, caps literal, F2 freeze, IV3 sentence), suite spec
sec. 7, frozen locks, the rider band (four locks, orchestrator's commit,
serialize between R-9b1's landing and the reviewer's fresh census); INV-5
monotonicity; every prior census anchor (`1981 / 0 / 13 / 125 @ fcb078c`
remains the base of record).
SUPERSEDED HERE: the R-9b handoff's duty 1 (done at `cef3b0a` - never
re-land); the handoff's single-hand gates/commit/line budget (replaced by
sec. 3 + sec. 4); the handoff's touch-set (split by secs. 3-4); the journal
sec. 2 two-link F6 stand-in (superseded by the user's manifest grant - dated
append, sec. 4); the gate-2 red line "the validator probe tests fail on the
stub" (a defect of the handoff, not of the session: not-Red per sec. 2;
replaced by the Green-step landing with API, D4 precedent);
`tasks/rust-language-r9b-handoff.md` itself is RETIRED as dispatchable -
amend this file instead; do not dispatch it.
