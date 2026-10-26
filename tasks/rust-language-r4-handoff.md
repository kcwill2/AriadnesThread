# R-4 Handoff - rust-language: registration, the .rs flip, amendments A1-A4+A7, fixture rust-basic, discovery suite

Milestone: `rust-language` (UNREGISTERED post-M28 track, NO M-number). Hand 4
of 10, serial. THE registration hop: one line in the composition root, the CLI
closed set + error text, the four cli/m19 enumeration amendments that the
registration flip makes red-or-false, the `rust-basic` fixture (approval path)
and the first milestone suite. Start rev: your dispatch head (R-3's landing +
verdict via duty 1).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/rust-language-shape.md` sec. 2 INV-1/INV-2/INV-4/INV-5/INV-6/INV-9,
   sec. 3.2 (the registration line, verbatim), sec. 3.4 (capability schedule:
   this hop flips `discovery` ONLY), sec. 3.5 (detector contract - already
   shipped at R-3), sec. 3.6 (closed set + error-text literal), sec. 4
   (fixture properties), sec. 8 (the R-10 riders - not yours). Kickoff A1-A4,
   A7, L3/L5.
3. Contract files, current state: `crates/pipeline/src/registry.rs` (39; the
   register block :23-27), `crates/cli/src/registry.rs` (761; error text
   :48-51, `closed_set_language_by_serde_name` :59-70, A1 test :302-318, A2
   tests :367-372 and :383-402, A3 test :406-439 (witness :411/:416/:419), A4
   test :736-760 (loop list :743)), `tests/integration/tests/m19_scala_discovery_test.rs`
   (:213-218, the vec at :217), `crates/lang-rust/src/plugin.rs` (caps literal).
4. Model suites to mirror: `tests/integration/tests/m19_scala_discovery_test.rs`
   (fixture -> `tempfile::TempDir` -> CLI handlers -> model + manifest asserts)
   and `tests/integration/tests/m24_language_identity_test.rs` (identity byte
   claims - READ-ONLY, cite never edit; copy its mechanism shape for the
   positional-escape evidence).
5. Your duty-1 targets: `tasks/rust-language-r3-verdict.md` + ledger.

## Duty 1 (before any R-4 mutation)
Land R-3's verdict + ledger flip (shape of R-2's duty-1 clause; ledger
clause: `R-3 LANDED <SHA> APPROVED <date>, census anchor <line> @ <SHA>,
R-4 dispatched (registration hop).`).

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/rust-language-r4-journal.md (create)
- tasks/rust-language-r3-verdict.md + tasks/milestone_ledger.md (duty 1 only)
- crates/pipeline/src/registry.rs (the registration line + doc sentence)
- crates/cli/src/registry.rs (closed set, error text, amendments A1-A4 -
  nothing else in the file)
- crates/lang-rust/src/plugin.rs (the caps literal flip + pin-test rename +
  one detect-language pin test; the modeling literals stay byte-identical)
- tests/fixtures/rust-basic/** (NEW fixture - approval path, see clause below)
- tests/integration/tests/rust_language_discovery_test.rs (NEW suite)
- tests/integration/tests/m19_scala_discovery_test.rs (A7: the ONE hunk at
  :215-218 - the vec literal; the rest of the file byte-identical)
- tests/integration/Cargo.toml - CONDITIONAL: only if the discovery suite
  genuinely needs a direct `ariadne-lang-rust` import; the preferred form
  (m19 precedent) goes through `ariadne_cli::registry::default_registry()` /
  `ariadne_pipeline` and then this file MUST NOT appear in any hunk.

You MUST NOT touch: any other `crates/cli/**` file (the three
`CyclePolicy::Reject` planner literals at `commands/plan.rs:203`,
`optimize.rs:305`, `commands/ui.rs:419` DO NOT change - shape sec. 3.6, the
hooks are already registry-dispatched), `crates/repo/**`, `crates/lang*/**`
beyond the two named files, `m24_language_identity_test.rs`, `m17_*`, `m18_*`,
`m20-m23*`, `tools/**`, existing fixtures, goldens. No `git stash`/branch change.

## Fixture-approval clause (kickoff L3; M27-T0b precedent)
`tests/fixtures/rust-basic/` lands through the documented frozen-set approval
path: your reviewer's APPROVED verdict of THIS landing commit is the approval;
R-5's duty 1 must carry the dated ledger note naming the fixture landing SHA
("fixture rust-basic APPROVED <date> at <SHA> via R-4 verdict - L3 approval
path discharged"). No existing fixture gains a byte (`basic`/`expensive`:
zero new files - reviewer path-diff proof).

## What to build

1. Registration (shape sec. 3.2 verbatim, appended LAST - order is the
   contract): in `build_default_registry`, after the Scala line:
   `.register(Box::new(ariadne_lang_rust::RustPlugin::new()))`
   plus a doc-comment sentence: C++ first stays first; Rust appends last
   (INV-1/INV-2). Update the fn's leading doc "C++ first, then Scala" to name
   the append.
2. CLI closed set (shape sec. 3.6): `closed_set_language_by_serde_name` gains
   the arm comparing `name == SourceLanguage::RUST` returning
   `SourceLanguage::Rust` (pattern of the CPP/SCALA/OTHER arms); error text
   becomes EXACTLY: `unknown language '{raw}'; expected cpp, scala, rust, or
   other (serde names: Cpp, Scala, Rust, Other)`.
3. Amendments (CW-1 amendment discipline - each is a justified edit of a
   pin, riding in THIS commit because THIS is the hop that would otherwise
   turn it red):
   - A1 (:302-318): test name `default_registry_enums_cpp_then_scala` STAYS
     (a name is not a contract); the vec literal becomes
     `vec![LanguageId::Cpp, LanguageId::Scala, LanguageId::Rust]`; keep the
     ORDER message's C++-precedence claim true; add
     `assert!(registry.plugin_for_id(LanguageId::Rust).is_some());` and the
     detection row `assert_eq!(registry.detect_language(Path::new("src/a.rs")),
     Some(LanguageId::Rust));`.
   - A2: `parse_language_arg_rejects_unknown_values` (:367-372): remove
     `"rust"` from the rejected list (list becomes `["", "py", "Cpp20"]`);
     `pin_parse_language_arg_exact_accepted_set_not_case_insensitive`
     (:383-402): add accepted rows `assert_eq!(parse_language_arg("rust").unwrap(),
     SourceLanguage::Rust);` and `assert_eq!(parse_language_arg("Rust").unwrap(),
     SourceLanguage::Rust);` plus rejection row
     `assert!(parse_language_arg("RUST").is_err(), "RUST keeps its case and
     matches no serde name - same class as CPP/SCALA");` (the exact-set
     discipline, not case-insensitivity).
   - A3 (:406-439): the witness path moves from `src/unclaimed.rs` to
     `src/solver.f90` (m24's mock vocabulary - still unclaimed): update the
     `plan_backend(...)` path arg (:411), the `.expect(...)` message (:416)
     to "no registered detector claims `.f90`", and the assert text (:419) to
     contain `"no language plugin claims the plan target src/solver.f90"`.
   - A4 (:736-760): the loop list at :743 becomes
     `["src/solver.f90"]` (drop the `"src/unclaimed.rs"` entry - the f90
     witness alone answers "no plugin claims").
   - A7 (m19 :215-218): `vec![LanguageId::Cpp, LanguageId::Scala,
     LanguageId::Rust]`. One hunk only.
4. Capability flip (INV-5): in `crates/lang-rust/src/plugin.rs`, the caps
   literal gains EXACTLY one bit true: `discovery: true` (detector + shaper
   + fixture evidence exist NOW); everything else stays false,
   `requires_toolchain: false`; rename the pin test to
   `capabilities_match_the_registration_flip_literal` (full-struct
   `assert_eq!` against an independent literal - the D-M23-6 style Scala
   proved catches slips). Add an in-src pin:
   `RustPlugin` answers `detect_language` for `.rs` (post-registration the
   plugin is registry-dispatchable; the detector behavior itself is already
   pinned from R-3).
5. Fixture `tests/fixtures/rust-basic/` (shape sec. 4 properties; ALL bytes
   pure ASCII, tiny and deterministic): root `Cargo.toml` (package manifest -
   the shaper's build marker at the fixture root); `src/lib.rs` declaring
   `mod models;` and `mod greeting;` (the two module forms:
   `src/models/mod.rs` + `src/models/user.rs` sibling style AND
   `src/greeting.rs`); `src/macro_specimen.rs` with one small `macro_rules!`
   whose generated `mod` stays unexpanded (the honest D-2 caveat specimen);
   `tests/cli_smoke.rs` (the test-side file the shaper must EXCLUDE);
   `vendor/legacy/legacy.rs` (the external/frozen-side directory - `target/`
   cannot be committed, so the external side rides `vendor/` per shape sec. 4).
   No build artifacts; nothing that requires a toolchain.
6. New suite `tests/integration/tests/rust_language_discovery_test.rs`
   (milestone-named - the mNN trap: this is NOT `m29*`-anything; guide Step 5
   rule, the name carries the track key `rust_language_`), m19-idiom, asserting:
   - INV-6 unit truth: every `src/**.rs` yields exactly one unit of context
     per nearest `Cargo.toml` ancestor; the `tests/`-carrying file is excluded;
     `Cargo.toml` itself never becomes a unit; `vendor/` files are not units
     of context (external marker); no C++/Scala-flavored context appears
     (D8-lock idiom from m19).
   - INV-9 `.rs` claimed: `ariadne_cli::registry::parse_language_arg("rust")`
     is `Ok`; case-folded `"Rust"` Ok; `"RUST"` Err; `detect_language` on a
     fixture path answers `Some(LanguageId::Rust)`; and the A3/A4 discipline
     witnesses live: `plan_backend` on `src/unclaimed.rs` now RESOLVES (do
     not pin this as rejection anymore).
   - INV-4 identity bytes: mirror m24's claims 1+2 for the REAL plugin id -
     (1) serde_json round-trips `"Rust"` as the bare name; (2) the positional
     (bincode) form takes the tag-3 Open escape (tag 3 + u64 LE length + the
     four UTF-8 bytes) and the stored trio keeps indices 0/1/2 (copy m24's
     mechanism; cite its claims, edit nothing).
   - INV-1 enumeration from the integration side: `default_registry()
     .languages() == [Cpp, Scala, Rust]`.
   - INV-2 first-wins witness (cheap, same suite): a `.cpp` and a `.scala`
     fixture-name probe still resolve to Cpp/Scala (the shipped detectors
     answer before Rust's claim - first registration order held).

## RGB protocol (true Red this hop)
1. Red, in this order: (a) author the fixture + discovery suite; run gate 2 -
   expected failures include `detect_language(...) == Some(LanguageId::Rust)`
   failing `left: None, right: Some("Rust")` (plugin unregistered yet) and the
   languages-vec assert failing `left: [Cpp, Scala]`. (b) Apply amendments
   A1/A2/A7 test edits and re-run: `default_registry_enums_cpp_then_scala`
   fails `left: [Cpp, Scala], right: [Cpp, Scala, Rust]`;
   `pin_parse_language_arg_exact_accepted_set_not_case_insensitive` fails on
   the new `rust` accepted row (`Err("unknown language 'rust'; ...")`).
   (c) A3/A4 are pre-flip amendments (CW-1): edited to the `.f90` witnesses
   they compile and pass IMMEDIATELY (green-on-landing by construction) -
   their falsification is inverted: if the registration line landed WITHOUT
   them, the `src/unclaimed.rs` witness would start resolving and both tests
   would go red; state that reasoning verbatim in the journal. All red edits
   (a-c) and the flip (below) land in ONE commit; the journal captures the
   red runs before the green.
2. Green: items 1-6; re-run gates 3-5 until green. NEVER weaken an assertion.
3. Refactor: docs plain, honest names; re-run.

## Hygiene (review-enforced)
cli/registry.rs 761 -> ~790 (margin ~210, shape sec. 7); pipeline registry
~42; suite ~300 (< 1000); functions < 100; pure ASCII incl. fixture bytes
(the macro specimen stays ASCII). `hygiene_check.py` exit 0.

## Frozen locks (commands that prove them)
- L5: `git diff <start>..HEAD -- tests/integration/tests/m19_scala_discovery_test.rs`
  = exactly the :215-218 vec hunk; m20/m21/m22/m23 paths absent.
- L6: m18 absent from all diffs (byte-identical); m17 absent (R-3 rode A8).
- L2/L3: `git diff --name-only <start>..HEAD -- tests/golden` EMPTY; fixture
  path-scoped diff shows ONLY `tests/fixtures/rust-basic/` additions.
- L7/L8/L9: lang-cpp/lang-scala/tools absent from any diff.
- L4: m00 green UNEDITED (gate 6 census + zero path hunk).
- L1: identity.rs (repo) absent from this hand's diffs.
- L12: `python tools/encoding_guard.py --all` exit 0.

## Gates (exact commands, timeout 600000 ms each; distilling entry points only)
1. `python tools/testrun.py --distill -p ariadne-cli` - journal the clean
   baseline BEFORE test edits.
2. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   rust_language_discovery_test` - Red: the named failures above.
3. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   rust_language_discovery_test` - Green: exit 0.
4. `python tools/testrun.py --distill -p ariadne-cli` - Green with A1-A4;
   passed = baseline + 0 or +1 (the new rust pins ride amended tests); the
   amended tests' names journal-logged.
5. `python tools/testrun.py --distill -p ariadne-pipeline` - green UNEDITED
   except the register line (suite-level proof via gate 6 census).
6. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m19_scala_discovery_test` - A7 green; rest UNEDITED.
7. INV-9 grep witness: `grep` via the Read/Grep tooling or
   `python tools/distill.py -- findstr /s /m "unclaimed.rs" crates\*` route -
   the string `unclaimed.rs` must occur NOWHERE under `crates/` (shape INV-9;
   grep-count zero is the oracle - journal the command and the zero count).
8. `python tools/testrun.py --distill` - full census: failed 0, ignored 13,
   suites +1 (discovery file); passed delta = suite N (+ amended tests are
   not new counts) attributed by name.
9. `python tools/hygiene_check.py` - exit 0.
10. Commit cycle: `python tools/encoding_guard.py --staged` (or `--all`) then
   `python tools/commit_scoped.py --scope crates/pipeline/src/registry.rs
   --scope crates/cli/src/registry.rs --scope crates/lang-rust/src/plugin.rs
   --scope tests/fixtures/rust-basic --scope
   tests/integration/tests/rust_language_discovery_test.rs --scope
   tests/integration/tests/m19_scala_discovery_test.rs --scope
   tasks/rust-language-r4-journal.md -m "feat(pipeline): register RustPlugin,
   flip the .rs enumeration amendments, add fixture rust-basic and the
   discovery suite (rust-language R-4)"`. Journal inside. If (and only if)
   the conditional Cargo.toml import was used, add that scope.

Declared gate budget: 10 scoped invocations + commit cycle - at the ceiling,
declared; journal per-gate counts.

## Bench stance (reviewer's acceptance pass, not yours)
`python tools/distill.py -- python tools/bench_compare.py` - RUN fresh (this
hand touches `crates/pipeline/src` AND `crates/cli/src`; kickoff sec. 4
fully-run stance). The `.rs` flip must not move the baseline: bench corpora
are fixture paths and `tools/**` is zero-diff (RISK-SELFSCAN; D-3 - no
own-tree corpus). WARNs recorded, not attributed (PR-2).

## Discipline clauses
- UTF-8 rule set (AGENTS.md); Write/Edit only; pure ASCII.
- Touch-set hard; report, don't fix.
- Journal checkpoint after each Green gate.
- STOP on context exhaustion; successor runs `checkpoint_guard.py resume
  --spec tasks/rust-language-r4-handoff.md --scope crates/cli --scope
  crates/pipeline --scope tests --cmd ...`.
- Landing SHA relay: report + reflog; R-5's duty 1 cites it AND carries the
  fixture-approval dated note (clause above).
- F14 rules.

## Report back (final message)
1. Duty-1 SHA + landing SHA + `git show --stat`. 2. Red evidence (gate 2/4
canonical lines + the named failing assertions). 3. Green evidence (gates
3-8 canonical lines + capture path + the grep-zero witness line). 4.
Deviations. 5. Out-of-scope observations. No raw output.

## Reviewer acceptance pass
- bench_compare (above); per-commit `git show` oldest-first vs touch-set;
  m19 diff = one hunk; the registration line appended LAST; error-text
  literal byte-checked against this file; A2's rejected-list no longer
  contains "rust" and `RUST` still rejected; fixture path-diff proof (only
  rust-basic; basic/expensive untouched); caps literal = discovery-only;
  census delta attribution; INV-9 zero-count witness re-run.

Reviewer-gate reference: `review-gate` skill. Extra MUSTs: the three
`CyclePolicy::Reject` literals untouched; m24 suite untouched; if
`tests/integration/Cargo.toml` appears without a demonstrated import need,
that is a blocker.

Sizing (planner, itemized): fixture ~110, suite ~300, amendments ~55/-12,
closed set + error text ~12, registration ~6, caps flip + pins ~20, journal
~15 - total ~560-750 produced lines; 10 gates + commit trio (at ceiling,
declared); crate-span note: pipeline+cli+lang-rust/plugin.rs+tests are ONE
vertical registration seam (shape sec. 5's own R-1 split kept the amendments
with the registration - the registration cannot ship its own red), declared
against the 1-crate law exactly as SM-4's hard-chain precedent declares
multi-file verticals: each touched file's delta is minimal and lock-named.
No further split exists: any cut separates the flip from its amendments and
lands a red hop.
