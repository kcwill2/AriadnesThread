# R-4 Verdict - rust-language: registration, the .rs flip, amendments A1-A4+A7, fixture rust-basic, discovery suite

Verdict: APPROVED

Milestone track `rust-language` (UNREGISTERED post-M28 track, NO M-number).
Hand 4 of 10, serial. Contract: `tasks/rust-language-r4-handoff.md`. Chain
from start rev `750ff8d`: duty 1 `cb5e0f4`, landing `ec552bd` (ONE
registration commit; 16 files, +785/-31 measured), journal tail `a9b6864`.
All gate lines below are the reviewer's own FRESH runs (2026-10-26, HEAD
`a9b6864`, tree clean per `checkpoint_guard.py status`), canonical lines
quoted verbatim. Nothing was trusted from the relay without personal
re-measurement.

## Verdict basis

- Commit-vs-touch-set: `git diff --name-only 750ff8d..HEAD` names 18 paths;
  every path is inside the handoff touch-set EXCEPT `crates/pipeline/
  Cargo.toml` and `Cargo.lock` - both declared pre-landing (journal deviation
  1), adjudicated ACCEPTED below (finding 3 disposition). No path under
  `crates/repo/`, `crates/lang-cpp/`, `crates/lang-scala/`, `tools/`,
  `tests/golden/`, or existing fixtures appears in any range diff (L1/L2/L3/
  L6/L7/L8/L9 held). `tests/integration/Cargo.toml` is absent from every
  hunk: the conditional import need never arose (no lang-rust import, no
  bincode import - the preferred m19 delegation form held).
- Red protocol held and lands in ONE commit: the range carries exactly three
  commits; `cb5e0f4` touches two `tasks/` files and nothing else, `a9b6864`
  touches only the journal tail - so EVERY red-turning edit (amendments
  A1-A4 + A7, the caps flip, the fixture, the suite) is inside `ec552bd`.
  The journal captures the (a) suite-red (`2 passed / 5 failed / 0 ignored /
  1 suites`, the named unregistered-plugin failures including the languages
  vec `left: [Cpp, Scala]`) and the (b) amendment-red
  (`172 passed / 2 failed / 0 ignored / 1 suites`: exactly
  `default_registry_enums_cpp_then_scala` and
  `pin_parse_language_arg_exact_accepted_set_not_case_insensitive`). (c) is
  the A3/A4 inverted-falsification statement, present verbatim in the journal
  per the handoff requirement.
- Plan-literal checks (byte-checked against the handoff/shape):
  registration line `.register(Box::new(ariadne_lang_rust::RustPlugin::new
  ()))` appended LAST after Scala (C++ first, Scala second - shape sec. 3.2
  verbatim); the fn doc names the append; `unit_shaper_for_repo` marker rule
  and first-wins precedence untouched (pipeline diff = doc + one line only).
  Error text EXACTLY `unknown language '{raw}'; expected cpp, scala, rust,
  or other (serde names: Cpp, Scala, Rust, Other)`. `closed_set_language_by
  _serde_name` gains the `SourceLanguage::RUST` arm in the CPP/SCALA/OTHER
  pattern and now accepts "rust"/"Rust" (pinned green). A1 name kept
  (`default_registry_enums_cpp_then_scala`), vec literal + `plugin_for_id`
  row + `src/a.rs` detection row as specified. A2 rejected list is
  `["", "py", "Cpp20"]` ("rust" gone); accepted rows rust/Rust added; `RUST`
  rejection row with the exact-set message present. A3 witness moved to
  `src/solver.f90` with the exact expect/assert texts. A4 loop list is
  `["src/solver.f90"]`. A7 is the ONE hunk at :217 (range diff = exactly the
  1-line vec literal; rest of m19 byte-identical). Caps literal: `discovery:
  true` ONLY, all seven other bits false, `requires_toolchain: false`
  (plugin.rs:147-158) - matches the handoff's flip exactly (discovery flips
  HERE; planning/fingerprint bits ride R-5/R-9 per the in-file notes). Pin
  test renamed `capabilities_match_the_registration_flip_literal`
  (independent full-struct literal); new detect pin added; modeling
  literals byte-identical (no modeling hunk in the diff).
- A3/A4 witness oracle re-run personally: grep regex `unclaimed\.rs` over
  `crates/` -> `No files found` (zero count; the implementer's own
  first-draft comment hit is fixed out of the landed bytes).
- ar27 frozen trio: `crates/repo/src/identity.rs` and
  `m24_language_identity_test.rs` are ZERO-DIFF across the range (measured).

## Fresh gate lines (reviewer's own runs, canonical, verbatim)

- cli: `182 passed / 0 failed / 8 ignored / 8 suites` (exit 0,
  `python tools/testrun.py --distill -p ariadne-cli`) - matches the
  journal's baseline: baseline + 0, the new rust pins ride the amended tests.
- lang-rust (full package): `24 passed / 0 failed / 0 ignored / 2 suites`
  (exit 0, `python tools/testrun.py --distill -p ariadne-lang-rust`); the
  relay's `7/0/0/1` is the lib-test target alone - both green, attribution
  holds (finding 4).
- pipeline: `46 passed / 0 failed / 4 ignored / 9 suites` (exit 0,
  `python tools/testrun.py --distill -p ariadne-pipeline`) - identical to the
  R-3 UNEDITED figures; the register line + manifest edge move no test.
- discovery suite: `7 passed / 0 failed / 0 ignored / 1 suites` (exit 0,
  `python tools/testrun.py --distill -p ariadne-integration-tests --test
  rust_language_discovery_test`).
- m19 (A7): `5 passed / 0 failed / 0 ignored / 1 suites` (exit 0, `python
  tools/testrun.py --distill -p ariadne-integration-tests --test
  m19_scala_discovery_test`) - rest of the file UNEDITED.
- Full census: `1886 passed / 0 failed / 13 ignored / 122 suites` (exit 0,
  `python tools/testrun.py --distill`, capture 3274 lines) vs the R-3 anchor
  `1878 passed / 0 failed / 13 ignored / 121 suites @ eeddfed`: delta +8 =
  the 7 tests of `rust_language_discovery_test.rs` + the in-src detect pin
  `rust_plugin_answers_detect_language_for_rs_paths` (caps-pin rename is
  1-for-1, net zero); suites +1 = the new discovery file; ignored flat 13;
  failed 0. The +8/+1 reconcile arithmetically and by name.
- Hygiene: exit 0 (`python tools/hygiene_check.py` -> `hygiene exit 0`);
  every line is warn-only R3 on PRE-EXISTING files; zero findings name any
  R-4-touched path. cli/registry.rs = 784 lines (predicted ~790, margin
  ~216); pipeline registry 42; suite 425; plugin.rs 297 - all within R2.
- Encoding (L12): `python tools/encoding_guard.py --all` -> `encoding exit
  0` (silent).
- Tool suites (CI form): `python -m unittest discover -s tools -p
  "*_tests.py"` -> `Ran 296 tests in 29.899s` / `OK`.
- Bench (kickoff fully-run stance - this hand touches crates/pipeline/src
  AND crates/cli/src): `python tools/distill.py -- python tools/bench_compare
  .py` -> `### generic PASS (exit 0)`; `bench compare: 3 run(s), no
  regressions past 1.25x baseline`; zero WARN lines. The `.rs` flip did not
  move the baseline (RISK-SELFSCAN stance: bench corpora are fixture paths,
  tools zero-diff).
- Duty 1 verified: `cb5e0f4` = R-3 verdict + one-line ledger flip only; the
  clause matches the handoff shape verbatim: `R-3 LANDED fe34e23 APPROVED
  2026-10-26, census anchor 1878 passed / 0 failed / 13 ignored / 121 suites
  @ eeddfed, R-4 dispatched (registration hop).`
- Report-evidence check (distillation contract): the journal is digest-
  shaped - canonical testrun lines, exit codes, capture paths
  (`target/testrun-capture.txt`), zero raw dumps; all gate commands use the
  distilling entry points; gate 7 used the Grep tool (permitted route) with
  the zero count journalled; refusals logged with substitutions.

## Fixture approval (kickoff L3 / M27-T0b precedent - FIXTURE APPROVED)

`tests/fixtures/rust-basic/` (8 files, pure ASCII, tiny, deterministic, no
build artifacts, no toolchain) satisfies all four required properties of
`docs/adding-a-language.md` Step 4, verified individually:
1. build-manifest the shaper recognizes: root `Cargo.toml` = the shaper's
   `build_markers()` marker (suite proves `["Cargo.toml"]` and marker-based
   shaper selection);
2. sources under the conventional source root: `src/` with BOTH module forms
   (`src/models/mod.rs` + sibling `src/models/user.rs`, `src/greeting.rs`)
   plus the `src/macro_specimen.rs` D-2 caveat specimen (ASCII `macro_rules!`
   whose generated `mod` stays unexpanded);
3. a test-side file the shaper must exclude: `tests/cli_smoke.rs` (recorded
   by discovery, never a unit - pinned);
4. an external/generated directory on the frozen side:
   `vendor/legacy/legacy.rs` (recorded ThirdParty/Frozen, never a unit -
   pinned; `target/` cannot be committed, vendor rides per shape sec. 4).
Path-diff proof: the fixture diff names ONLY the eight `rust-basic`
additions; `tests/fixtures/basic`/`expensive` gain zero bytes.
THIS APPROVED verdict + landing `ec552bd` discharge the L3 approval path.

## Findings

1. (minor, journal evidence drift) `tasks/rust-language-r4-journal.md`
   Landing section reports the landing as "+729/-31"; the measured landing
   diff is +785/-31 (16 files; reviewer `git show --numstat` sum). Deletions
   and file count correct; no gate integrity affected; the orchestrator relay
   already carries the correct +785/-31. No action beyond the record.
2. (minor, deferred fix hop) `crates/cli/src/registry.rs:8-9` - the file
   header doc still reads "(C++ first, then Scala; registration order is a
   contract)" - stale after the append. Touch-set-compliant to LEAVE
   (the handoff named only the pipeline fn's leading doc for update and the
   "nothing else in the file" clause forbade touching this), so it is not a
   violation - but it is now inaccurate prose. FIX HOP: the R-10 docs rider
   (or R-5 duty 1 if the orchestrator prefers an earlier carrier); record in
   the ledger deferred-findings.
3. Deviation 1 (touch-set extension `crates/pipeline/Cargo.toml` +
   `Cargo.lock`) - disposition: ACCEPTED AS SANCTIONED RIDER. The one
   registration line is uncompilable without the `ariadne-lang-rust` path
   edge; the composition root is the line's only home (the cycle rule puts
   the builder in pipeline, not cli); `crates/pipeline/Cargo.toml:29-32`
   shows the M19.3 Scala registration left the identical manifest precedent;
   shape sec. 2 lists `Cargo.lock (regenerated)` as arc-EDITED under the
   reviewer rule; the Cargo.lock hunk is purely additive (one dependency
   line in the ariadne-pipeline block, zero deletions, no version moves -
   measured). Declared journal-first before any mutation; scopes added to
   the gate-10 commit; census/pipeline/m19 suites prove nothing else moved.
4. Deviation 2 (INV-4 positional bytes by live production witness instead of
   a synthetic in-suite bincode call) - disposition: ACCEPTED ON THE MERITS.
   Verified independently: the handoff's "copy m24's mechanism" is satisfied
   for claims 1+2 verbatim (JSON bare-name round-trip + resolution through
   detection/id-lookup/CLI catalog dispatch in
   `identity_bytes_mirror_m24_claims_for_the_real_plugin_id`); m24 itself
   carries NO bincode mechanism (grep-verified - the journal's factual claim
   is correct); the escape-arm framing is byte-frozen UNEDITED by
   `crates/lang-api/tests/ar27_dual_format_lock_test.rs`
   (`discriminator_selects_the_wire_arm_by_format` locks the trio's u32 LE
   indices 0/1/2; `positional_form_names_an_open_id_through_the_escape_arm`
   locks tag 3 (u32 LE) + u64 LE length + UTF-8 - the EXACT 16-byte pattern
   the new test searches) plus `crates/repo/tests/ar27_old_bytes_lock_test.rs`
   (FileRecord language region stays the u32 index in `files.bin`), with
   `crates/repo/src/identity.rs:25-29` documenting the arm; those locks are
   green in the census and their files are zero-diff. Adding a bincode
   dev-dep would have edited `tests/integration/Cargo.toml` outside the
   touch-set for zero new byte truth (the package's own suites document the
   no-bincode placement). The production-writer witness (real
   `ArtifactManager::save_repository_artifacts` blob, searched for
   `[3,0,0,0][4,0,0,0,0,0,0,0]"Rust"`, then decoded back to
   `LanguageId::Rust`) is strictly STRONGER than a synthetic serialize: it
   proves the real plugin id rides the escape through the writer that
   produces the warm blobs.
5. Deviation 3 (stale cli header comment) - disposition: RECORDED AS MINOR,
   fix hop assigned (finding 2); leaving it was compliant, the relay's
   "forbidden" framing inverted the clause.
6. (note) `python tools/testrun.py` raw mode crashes in THIS reviewer's
   console (`UnicodeEncodeError: 'charmap' codec ... cp1252`) when echoing
   cargo text; the `--distill` entry point (the handoff's prescribed form
   anyway) was used for every scoped gate and the census - identical count
   semantics, no weaker evidence. `--baseline` bare is the F9-errata FILE
   form ("usage error: missing value for --baseline"), superseded track-wide
   as at R-1..R-3.

## Relay line for R-5 duty 1 (carry verbatim)

`R-4 LANDED ec552bd APPROVED 2026-10-26, census anchor 1886 passed / 0
failed / 13 ignored / 122 suites @ a9b6864, R-5 dispatched (analyzer hop).`

AND the fixture-approval dated ledger note (handoff fixture-approval clause
- R-5 duty 1 MUST carry it in the same ledger flip):
`fixture rust-basic APPROVED 2026-10-26 at ec552bd via R-4 verdict - L3
approval path discharged`.

## Follow-up dispositions (recorded, never dropped)

- Finding 2 (stale cli header doc) -> ledger deferred-findings or the R-10
  docs rider; the orchestrator picks the carrier.
- Finding 1 (journal numstat slip) -> record only; no follow-up work.
- R-5 duty 1 additionally carries: the R-4 verdict landing (this file), the
  ledger flip, and the fixture dated note above.
- `tasks/rust-language-execution-results.md` R-4 addendum: appended by this
  reviewer (uncommitted) - the orchestrator lands it as a rider, per the
  R-2/R-3 pattern.

## Not run / refused (explicit statement)

- `python tools/testrun.py --baseline` (bare): refused with `usage error:
  missing value for --baseline` - the F9-errata FILE form, superseded
  track-wide; the governing fresh `--distill` census (criterion line above)
  was run instead.
- `python tools/testrun.py` raw (non-distill) scoped run: crashed this
  console's cp1252 echo (`UnicodeEncodeError` at `_echo`); substituted by
  `--distill` for all gates (finding 6). Not a weaker-evidence substitution -
  distill is the handoff's prescribed form.
- `python tools/ui_smoke.py`: not required by this hand (no UI/runtime-
  surface bytes); NOT run.
- `python tools/golden_guard.py --base <rev>`: tool not landed; substituted
  by the handoff's prose byte-check (name-only + path-scoped diff proofs
  above).
- Catch-all-deny refusals (verbatim
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`):
  chained `git log; git status`, `git status --short`, `python -c` line
  counter, `Select-Object`-piped unittest (re-ran un-piped - succeeded). All
  substituted by dedicated tooling or un-piped equivalents. No expected gate
  was skipped.
