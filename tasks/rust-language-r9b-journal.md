# rust-language R-9b journal - enrollment hop (planning hooks, probe validator, F2 freeze, all-true flip, symbol-isolation suite)

Successor hand (the first R-9b attempt died at zero work - no commits, no dirty
files; this hand started clean at HEAD `6b9dae0`).
Governing contract: `tasks/rust-language-r9b-handoff.md` under
`tasks/rust-language-r9-amendment-1.md` (the amendment is law where they
conflict). Context chain: `tasks/rust-language-r9-journal.md` (sec. 2/3
findings, F6 note), `tasks/rust-language-r9a-journal.md` (mechanism hop),
`tasks/rust-language-r9a-verdict.md` (F1 FINAL WORD byte-pairs consumed
verbatim), `tasks/rust-language-r8-verdict.md` (29 witnesses).

## Section 1 - duty 1 (ledger relay + verdict landing)

- Gate 1 (baseline, at start rev `6b9dae0`): `python tools/testrun.py --distill
  -p ariadne-lang-rust` -> `94 passed / 0 failed / 0 ignored / 2 suites`
  verbatim = the R-9a anchor (`94 passed / 0 failed / 0 ignored / 2 suites` per
  the r9a verdict, census anchor `1981 passed / 0 failed / 13 ignored /
  125 suites @ fcb078c`). Capture: `target/testrun-capture.txt`.
- Landed `tasks/rust-language-r9a-verdict.md` as authored (was untracked).
- Ledger `tasks/milestone_ledger.md` rust-language live row: appended the
  handoff's clause verbatim after `... R-9a dispatched.` inside the row,
  before the state-cell terminator:
  `R-9a LANDED fcb078c APPROVED 2026-10-26, census anchor 1981 passed / 0
  failed / 13 ignored / 125 suites @ fcb078c, R-9b dispatched (hooks +
  validator + all-true flip).`
  Count-of-1 verified against the landed commit blob.
- Clause-vs-draft difference (journal duty-1 requirement): the verdict's relay
  section carries the longer draft phrasing
  `... R-9b dispatched (hooks + validator + F2 freeze + all-true flip + suite
  hop; four caps locks ride the architect rider).` - the handoff commands the
  shorter clause above as "the one of record per the orchestrator dispatch
  2026-10-26"; the shorter clause is what landed. The longer draft stays
  untouched in the verdict file as authored.
- Deleted untracked `tasks/rust-language-r9-handoff-resume.md` (verdict
  disposition; never staged, outside every commit scope).
- Not re-landed: duty 1 `c52817c`, amendment `f2387f1`, mechanism `fcb078c`.

## Section 2 - F6 consumption mapping (the tier-3 Skipped route)

F6 (r9 journal sec. 2): `crates/validate` maps only `ToolchainStatus::Missing`
to tier-3 `Skipped`, and cargo/rustc ARE on PATH here, so a wired apply reports
tier-3 `Pass`. The handoff (item 5) names the Skipped route as
`RustToolchainValidator::with_runner(scripted all-None)` consumed by
`TierValidator::validate_affected_tus`. That consumption requires the suite to
name the `ariadne-lang-rust` type, and the suite crate
(`tests/integration/tests/...`) has no dependency on that crate; the only
resolution is a `tests/integration/Cargo.toml` dev-dependency row, which the
handoff's hard touch-set forbids (`Cargo.toml`/`Cargo.lock` edits: "no new
deps"; dispatch law: "no Cargo.toml/Cargo.lock edits"). A landing commit that
names an undeclared crate does not compile - a manifest edit inside this hand
is therefore impossible-by-its-own-rules, and the literal with_runner
consumption is impossible-by-the-same-rules from outside.

Disposition (board ASK posted 2026-10-26; report-don't-fix law, no manifest
hunk): the Skipped chain is pinned as two honestly-witnessed links instead of
one composed call -

1. `validator.rs` in-src (probe test set): an all-None probe walk answers
   `Missing` (probe test `probe_reports_missing_when_no_probe_is_usable`), and
   the wired registry validator answers `Available` with a real cargo banner on
   THIS machine (suite probe) - both ends of the Rust probe contract pinned at
   the plugin crate.
2. Suite tier-3 test: the `Missing -> Skipped` mapping is CONSUMED from
   `crates/validate::TierValidator::validate_affected_tus` with a
   suite-local validator carrying exactly the all-None probe outcome
   (`Missing`), never re-implemented. The composition of THAT validator type
   with the real Rust probe class is the one byte the manifest freeze withholds;
   if the orchestrator grants a manifest rider, the suite swaps the stand-in
   for `RustToolchainValidator::with_runner(scripted all-None)` byte-for-byte.

Apply-side posture (per the amendment's own words, "a wired apply would report
`Pass`"): the apply stage asserts `Committed` with tiers 1-2 `Pass` and tier 3
`Pass` (never asserted `Skipped` against the real environment, INV-10 honored -
the apply flow's only Rust-toolchain spawn is the `--version` probe walk inside
`toolchain_status`, which `validate_affected_tus` performs; it never reaches
`validate_compile`).

## Section 3 - red protocol note (validator probe tests)

The handoff's gate-2 red list includes "the validator probe tests fail on the
stub". The new probe tests call `with_runner` / `TOOLCHAIN_PROBE_ORDER` - API
the stub does not have - so they cannot COMPILE against the stub, and a compile
error is not Red under the handoff's own TDD law ("a compile error ... is not
Red"). Their named semantic red is therefore not executable pre-mechanism; the
superseded stub pins they retire (amendment sec. 9) are green-until-replaced
characterizations of the STUB era (unconditional `Missing`) that CANNOT survive
the flip (a real probe answers `Available` here). The three other named red
categories ARE executed true-Red (renamed parity pin, hook-default seam legs,
constraints Module/two-decl pins - verbatim `left:` lines in section 4). The
probe tests land in the Green step together with the production API, exactly as
the R-9a hop did for its hook defaults (r9a verdict D4 precedent).

## Section 4 - gate 2 (Red, executed)

`python tools/testrun.py --distill -p ariadne-lang-rust` (test-code-only edits:
renamed caps pin, evolved seam legs, two constraints positive pins - zero
production bytes touched). Canonical line:
`92 passed / 4 failed / 0 ignored / 1 suites`
(the `1 suites` shape is the preexisting distiller observation from the r9a
verdict D5: a failing lib target skips the doc suite; baseline and green
report `2 suites`). Exactly the four named failures; verbatim `left:` lines
from `target/testrun-capture.txt`:

1. `plugin::tests::capabilities_match_the_parity_flip_literal` -
   `left: LanguageCapabilities { discovery: true, dependency_graph: true, semantic_index: true, load_metric: true, planning: false, extraction: false, rewriting: false, validation: false, requires_toolchain: false }`
   (the predicted `planning: false` vs `right: true` - the flipped fields are
   exactly planning/extraction/rewriting/validation).
2. `plugin::tests::plugin_default_seams_stay_at_trait_defaults` -
   `left: None` (hook-default `None` vs the Some-dispatch pins - the
   handoff's own named hook-default red).
3. `constraints::tests::module_kind_records_are_planning_frozen` - panicked at
   the frozen-contains assert (stub `frozen_symbols` answers empty).
4. `constraints::tests::two_declaration_fold_records_are_planning_frozen` -
   panicked at the >=2-declaration frozen assert.

Harness note (not a red-expectation change): two assert MESSAGES contained
literal `{` braces (`mod x { .. }`, `{symbol}.rs`) which Rust format-strings
hijack; escaped to `{{ }}` (renders identically) before the named reds ran.
The first run therefore showed a compile failure (`parse format drift`), fixed
test-side only, then the line above.

## Section 5 - R-9b1 successor bootstrap + adopt-and-verify (amendment-2 sec. 2)

SUCCESSOR HAND (R9B-DEADSESSION-2 was this journal's authoring hand; it died
after section 4. This hand is R-9b1 per amendment-2, bootstrapped from disk at
HEAD `cef3b0a` with the inventoried dirty state, never resumed, no stash -
`refs/stash` re-proved absent by the reviewer).
Adopt-and-verify three legs, amendment-2 sec. 2:
- Leg 1 (RELAY, not re-observed by the author of these bytes): sections 1-4
  above - the dead hand's durable record, quoted as relay.
- Leg 2 (FRESH, this hand's own machine truth): gate 1 re-ran
  `python tools/testrun.py --distill -p ariadne-lang-rust` against the DIRTY
  tree and RE-OBSERVED the journalled red exactly:
  `92 passed / 4 failed / 0 ignored / 1 suites` with exactly the four named
  failures and matching `left:` bytes - caps pin `left:` all-false-on-the-four
  struct, seam pin `left: None`, both constraints pins panicking at their
  frozen-contains asserts (constraints.rs:81 / :111). Capture:
  `target/testrun-capture.txt`. NO disk-vs-journal contradiction.
- Leg 3 (falsifiability, source-read): the renamed caps pin asserts an
  INDEPENDENT all-true full-struct literal the four-bits-false production
  struct cannot satisfy; the seam pins assert Some-dispatch bytes the trait
  defaults (None) cannot give; the F2 pins assert frozen-contains answers the
  empty-HashSet stub cannot give. Probe tests owe NO red (sec. 3 stands):
  they landed WITH the API in the Green step (D4 precedent), every spawn
  `--version`-shaped (INV-10b), the two stub pins retired with named positive
  replacement (never weakening).

## Section 6 - Green landing + gate attribution (gates 2-6)

Production landed: NEW `planning.rs` (aggregation_filter + target_path_for +
shared-grammar witnesses; `sanitize_module_stem` single source of truth),
`validator.rs` stub -> probe-only (probe order ["cargo","rustc"],
CommandRunner/CommandOutcome/ProcessCommandRunner/with_runner pub surface 1:1
with the Scala file for R-9b2's external construction; rustc-specific
options-before-sources compile with the single-input honesty doc; offline Err
names the D-M23-12 class deferral), `plugin.rs` production (caps FINAL all-true
literal - exactly four bits gained, `load_metric` byte-untouched,
`requires_toolchain: false`; hook-pair overrides delegating to planning,
registry-dispatched, zero CLI edit), `constraints.rs` production freeze +
honest reword of the stale R-3 header/IV3 prose, `lib.rs` one `mod planning;`,
shape-table :290 IV3 row dated edit (amendment-2 sec. 1 erratum).
- Gate 2/3 (green + refactor re-run): `102 passed / 0 failed / 0 ignored /
  2 suites` = 94 + M, M=8 BY NAME: probe set net +3 (five named probe tests
  `probe_order_is_cargo_then_rustc`,
  `probe_reports_missing_when_no_probe_is_usable`,
  `every_probe_spawn_is_a_probe_order_entry_with_version_flag`,
  `validate_compile_offline_errors_naming_the_deferral_spawning_nothing_beyond_the_probe_walk`,
  `probe_reports_available_with_version_banner_of_first_usable_probe` minus the
  two retired stub pins), +2 adopted constraints pins, +3 planning witnesses
  (`aggregation_filter_detects_out_of_line_mod_declarations`,
  `target_path_lands_beside_the_aggregation_file`,
  `target_stem_grammar_is_the_shared_module_stem`). R-8's 29 witnesses + R-9a's
  11 pins green UNEDITED inside the attribution.
- Gate 4: `7 passed / 0 failed / 0 ignored / 1 suites` (m23 UNEDITED).
- Gate 5 DEVIATION: the gate line names `-p ariadne-integration-tests --test
  planner_injection`; that target exists ONLY in `ariadne-cluster` (cargo
  help proved it). Ran the named lock in its real package, UNEDITED green:
  `6 passed / 0 failed / 0 ignored / 1 suites` (frozen_symbols_are_skipped_
  from_proposals + m18.7 SCC lock - the freeze stays neutrally locked).
- Gate 6 RED (see section 7): cli `173 passed / 1 failed / 0 ignored / 1
  suites` - attribution in section 7, zero cli bytes touched.

## Section 7 - gate 7 census: SIX failures, not the designed four (board ASK posted)

`python tools/testrun.py --distill -- --no-fail-fast` (the tool's `--`
passthrough; testrun has no native no-fail-fast flag - form deviation, same
no-fail-fast semantics). OBSERVED line, verbatim:
`1983 passed / 6 failed / 13 ignored / 125 suites`
Arithmetic: anchor `1981 / 0 / 13 / 125 @ fcb078c` + 8 in-src = 1989 total;
1983 + 6 = 1989 exact; suites 125 flat; ignored 13 flat; zero collateral.
Designed four (AMENDMENT-1 sec. 4 table, each `left:` shows the all-true
flipped struct - the rider band turns these):
1. `capabilities_lock_the_registration_flip_literal` (discovery_test:302)
2. `capabilities_lock_the_semantic_index_flip_literal` (semantic_index_test)
3. `capabilities_lock_the_dependency_graph_flip_literal` (dependency_graph_test:130)
4. `capabilities_lock_the_load_metric_flip_literal` (load_analysis_test:217)
TWO EXTRA flip-superseded witnesses the rider band undercounts (report-don't-
fix; board ASK posted 2026-10-26; options (a) rider expansion / (b) re-home):
5. `rs_claimed_case_exact_and_plan_backend_resolves` (discovery_test:281,
   INSIDE the rider-band file, asserts "Rust keeps extraction false at this
   hop (INV-5)" - an R-7-era statement superseded by the mandated flip).
6. `pin_failed_transaction_burns_the_token_and_reports_failed_text`
   (crates/cli/src/commands/ui/extraction_apply/tests.rs:854, ZERO-HUNK zone):
   "a .rs target must fail the core routing, got Ok(committed=true)" - its
   doomed plan retargets to `solver.rs`, "a .rs target no plugin CLAIMS";
   core routing keys on exactly the extraction bit this hop truthily flips
   (precedent: the R-1 rider moved the registry.rs unclaimed-path witnesses
   from .rs to .f90 for the same reason). Report-only: worth one orchestrator
   glance that the committed=true through-engine path is honest frozen
   semantics (zero engine bytes touched by this hand).
Gate 6 (cli UNEDITED) fails for failure 6; the landing proceeds per the R-7
precedent (in-src caps literal INV-5-byte-exact; the census carries the
frozen-zone attribution; the rider/re-home decision is the orchestrator's).

## Section 8 - F6 stand-in supersession (dated append; sec. 2 stands byte-unchanged)

Dated 2026-10-26: the user's manifest grant (amendment-2 sec. 0 decision 2,
R-9b2's granted dev-dep row) supersedes this journal's sec. 2 two-link F6
stand-in DISPOSITION. This hand landed the consumed surface in full:
`RustToolchainValidator::with_runner` + `CommandRunner`/`CommandOutcome`/
`ProcessCommandRunner` pub, mirroring the Scala file 1-for-1; R-9b2 builds
the REAL composition (its suite swaps the stand-in "byte-for-byte", the
journal's own words) under the granted row. Section 2 itself remains the
as-executed history of the pre-grant hand.

## Section 9 - deviations, disk state, next steps

Deviations (all with reason): (1) gate 5 package correction (sec. 6);
(2) census no-fail-fast form via `--` passthrough (sec. 7); (3) six-red
designed-window amendment (sec. 7, ASK pending); (4) offline validator error
KIND is `ToolchainUnavailable` (Scala 1:1 mirror; the stub-era
`ValidationNotWired` kind retired WITH the superseded stub pin - amendment-1
sec. 9 authorizes the positive replacement; the deferral-message truth
survives in `validate_compile_offline_errors_naming_the_deferral_...`);
(5) `checkpoint_guard.py resume` (sec. 7 line) hung with no output for
600 s and was abandoned; bootstrap proceeded from the amendment + disk
inventory directly; (6) journal append ~110 lines vs the ~30-45 estimate
(honest evidence demanded; R-7 precedent on estimate deviations).
F5 exact-named test NOT authored here - reserved, lands in R-9b2's suite
(amendment-2 sec. 3 ruling). Next: orchestrator rider call (four + the two
extras), reviewer fresh bare census (expect >= 1981+8 / 0 failed / 13
ignored / 125 @ landing+rider), then R-9b2 (suite + manifest dev-dep).

