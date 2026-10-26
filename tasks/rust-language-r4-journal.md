# R-4 Journal - rust-language: registration hop

Milestone track `rust-language` (UNREGISTERED post-M28 track, NO M-number).
Hand 4 of 10, serial. THE registration hop. Contract:
`tasks/rust-language-r4-handoff.md` (read complete, first, before any edit).
Start rev: `cb5e0f4` (my duty-1 commit: `tasks/rust-language-r3-verdict.md`
landed + ledger clause `R-3 LANDED fe34e23 APPROVED 2026-10-26, census anchor
1878 passed / 0 failed / 13 ignored / 121 suites @ eeddfed, R-4 dispatched
(registration hop).` - shape of R-2's duty-1 clause, landing SHA fe34e23
taken from the R-3 verdict, census anchor @ the reviewer's fresh-run HEAD
eeddfed).

## Declared deviations (journal-first, per gate discipline)

1. TOUCH-SET EXTENSION (blocker-level, reviewer must adjudicate): the
   registration line cannot compile without a dependency edge in
   `crates/pipeline/Cargo.toml` (`ariadne-lang-rust = { path = "../lang-rust"
   }`) plus the matching purely-additive `Cargo.lock` edge. Neither file is
   in the handoff's touch-set, but the handoff's core mandate (the ONE
   registration line landing green, amendments riding in the same commit) is
   uncompilable without them; there is no alternative home. The shape's own
   module map lists `Cargo.toml` and `Cargo.lock (regenerated)` as arc-EDITED
   files, and the M19.3 Scala registration left the precedent two-line
   comment+dep in the same manifest (pipeline Cargo.toml:29-32). Both files
   ride the ONE registration commit; scopes added to gate 10 accordingly.
2. INV-4 positional-byte sub-claim, executed by citation not duplication:
   the handoff asks the discovery suite to pin the positional (bincode)
   tag-3 escape bytes "copy m24's mechanism". Verified:
   `m24_language_identity_test.rs` contains NO bincode mechanism (its claims
   1+2 are JSON round-trip + dispatch resolution); the byte-level escape
   mechanism lives in `crates/lang-api/tests/ar27_dual_format_lock_test.rs`
   (`discriminator_selects_the_wire_arm_by_format`: trio keeps u32 LE indices
   0/1/2; `positional_form_names_an_open_id_through_the_escape_arm`: tag 3 +
   u64 LE + UTF-8). `tests/integration` has no bincode dependency and its
   own suites document why (`ar27_field_probe_test.rs:18-21`,
   `ar27_dual_format_lock_test.rs:18-21`: "the integration package ... has
   no bincode dependency (its Cargo.toml is outside the ... touch-set)");
   adding one would edit `tests/integration/Cargo.toml` + `Cargo.lock`
   outside the touch-set for no new byte truth. The suite therefore pins:
   (a) m24 claim 1 verbatim for the REAL plugin id (serde_json bare-name
   round-trip + Debug/Display), (b) m24 claim 2 mechanism for the real id
   (detect_language + plugin_for_id + CLI catalog dispatch), and (c) a LIVE
   positional witness: the production writer
   `ariadne_report::ArtifactManager::save_repository_artifacts` serializes
   the fixture's Rust-labelled records into `repository/files.bin` (bincode,
   positional arm), and the suite byte-searches that blob for the exact
   escape framing `[3,0,0,0][4,0,0,0,0,0,0,0]['R','u','s','t']` = tag 3 +
   u64 LE length 4 + the four UTF-8 bytes, then decodes the blob back
   through `load_repository_artifacts` recovering `LanguageId::Rust`. The
   stored-trio 0/1/2 byte claim rides the UNEDITED ar27 locks (green in the
   gate-8 census, named here for attribution).

## Gate ledger (per-gate invocation counts; timeout 600000 ms each)

- Gate 1 (cli baseline, BEFORE any test edit): invocation 1/1 ->
  `182 passed / 0 failed / 8 ignored / 8 suites` (exit 0).
- Gate 2 (RED, discovery suite, invocation 1/1):
  `2 passed / 5 failed / 0 ignored / 1 suites` (exit 1, capture
  `target/testrun-capture.txt`). Named failures, all predicted-by-protocol
  (unregistered plugin; no marker-selectable shaper):
  `enumeration_and_first_wins_probes` (languages vec `left: [Cpp, Scala]`),
  `shaper_selection_and_unit_truth_on_the_fixture` (pre-registration the
  marker scan finds no Cargo.toml-owning plugin, the no-marker fallback
  answers the C++ shaper - `left: ["CMakeLists.txt",
  "compile_commands.json", ".vcxproj"]`),
  `rs_claimed_case_exact_and_plan_backend_resolves` (first-firing assert is
  the parse row: Err value `"unknown language 'rust'; expected cpp, scala,
  or other (serde names: Cpp, Scala, Other)"`; the handoff's
  `left: None, right: Some("Rust")` detect row lives in this same test and
  is the same unregistered-plugin cause),
  `capabilities_lock_the_registration_flip_literal` and
  `identity_bytes_mirror_m24_claims_for_the_real_plugin_id` (both panic at
  `plugin_for_id(LanguageId::Rust)` - plugin unregistered yet).
  The 2 passes are the spine-only witnesses
  (`discovery_records_every_rust_source_and_never_the_manifest`,
  `warm_files_bin_blob_carries_the_open_escape_bytes_for_rust` - both
  green pre-registration by design: FileKind-table driven).

- Gate 3 (GREEN, discovery suite, invocation 1/1):
  `7 passed / 0 failed / 0 ignored / 1 suites` (exit 0).
- Gate 4 (cli, invocation 2 = GREEN with A1-A4; invocation 1 was the
  protocol-(b) RED capture: `172 passed / 2 failed / 0 ignored / 1 suites`
  with exactly `default_registry_enums_cpp_then_scala`
  (`left: [Cpp, Scala]`) and
  `pin_parse_language_arg_exact_accepted_set_not_case_insensitive`
  (Err value `"unknown language 'rust'; ..."`)):
  `182 passed / 0 failed / 8 ignored / 8 suites` = baseline + 0 (the new
  rust pins ride the amended tests, no new test functions - matches the
  gate wording "+0 or +1"). Amended test names journal-logged:
  `default_registry_enums_cpp_then_scala` (A1),
  `parse_language_arg_rejects_unknown_values` +
  `pin_parse_language_arg_exact_accepted_set_not_case_insensitive` (A2),
  `pin_plan_backend_honest_errors_unclaimed_target_and_capability_off`
  (A3), `planning_hooks_answer_honestly_for_paths_no_plugin_claims` (A4),
  m19 `plugin_contract_in_default_registry` (A7).
- Gate 5 (pipeline, invocation 1/1):
  `46 passed / 0 failed / 4 ignored / 9 suites` - identical to the R-3
  verdict's UNEDITED pipeline figures; the register line (+declared manifest
  edge) does not move a single pipeline test.
- Gate 6 (m19, invocation 1/1): `5 passed / 0 failed / 0 ignored / 1 suites`
  - A7 green, rest of the file UNEDITED (one-hunk proof below).
- Gate 7 (INV-9 grep witness): the Grep tool over `crates/` for regex
  `unclaimed\.rs` -> first run found 1 hit: MY OWN new A3 explanatory
  comment re-naming the old witness path - fixed to describe the move
  without the literal, re-run -> `No files found` (zero count). Journaling
  the first-run hit honestly: the string survives NOWHERE under crates/.
- Gate 8 (full census, invocation 1 DRIFT exit 2: a build failure, not a
  test failure - my lang-rust detect-pin omitted the in-scope
  `ariadne_lang::SourceFileKind` import (module imports are not test-gated
  in that file); scoped gates 3-6 never compile lang-rust's own lib-test
  target, so it surfaced here only. Fixed (one use-line added). Invocation
  2 GREEN): `1886 passed / 0 failed / 13 ignored / 122 suites` (exit 0,
  capture `target/testrun-capture.txt`). Delta vs the R-3 anchor
  1878/0/13/121 @ eeddfed: passed +8 = the 8 tests of
  `rust_language_discovery_test.rs` named in the capture
  (discovery_records_every_rust_source_and_never_the_manifest,
  shaper_selection_and_unit_truth_on_the_fixture,
  enumeration_and_first_wins_probes,
  rs_claimed_case_exact_and_plan_backend_resolves,
  capabilities_lock_the_registration_flip_literal,
  identity_bytes_mirror_m24_claims_for_the_real_plugin_id,
  warm_files_bin_blob_carries_the_open_escape_bytes_for_rust = 7 ... plus
  the lang-rust in-src detect pin
  `rust_plugin_answers_detect_language_for_rs_paths` (+1, renamed caps pin
  net-zero); suites 121 -> 122 = +`rust_language_discovery_test`; ignored
  flat 13; failed 0. (The 7+1 arithmetic: the suite file itself carries 7
  tests, the plugin.rs rename replaces 1-for-1 and adds 1 pin.)
- Gate 9 (hygiene): exit 0 via the distiller digest
  (`### generic PASS (exit 0)`); all 160+ lines are R3 warn-only and
  PRE-EXISTING - zero R3 findings under any R-4 file, zero R1/R2.
- Gate 10 (commit cycle): see Landing section.

## Green/landing content (all in ONE registration commit)

- `crates/pipeline/src/registry.rs`: the shape sec. 3.2 line appended LAST
  + doc naming the append (INV-1/INV-2).
- `crates/pipeline/Cargo.toml`: `ariadne-lang-rust` edge (declared deviation
  1) + `Cargo.lock` purely-additive edge (verified: +1 dependency line in
  the ariadne-pipeline block, 0 deletions, 0 version moves).
- `crates/cli/src/registry.rs`: RUST arm in the closed set; error text
  byte-checked against the handoff literal; A1/A2/A3/A4 as specified (the
  two doc comments of the two edited fns ride their behavior change:
  `parse_language_arg` lists `RUST`, `closed_set_language_by_serde_name` no
  longer claims "`--lang rust` stays the clear error").
- `crates/lang-rust/src/plugin.rs`: caps literal `discovery: true` ONLY
  (`requires_toolchain: false`), pin test renamed
  `capabilities_match_the_registration_flip_literal` (independent full-
  struct literal), new in-src detect pin
  `rust_plugin_answers_detect_language_for_rs_paths`; doc honesty updated
  for the registration + flip (modeling literals byte-identical).
- `tests/fixtures/rust-basic/` (8 files, pure ASCII, no build artifacts;
  approval rides the R-4 verdict per the fixture-approval clause - R-5's
  duty 1 carries the dated ledger note naming THIS landing SHA).
- `tests/integration/tests/rust_language_discovery_test.rs` (NEW, 7 tests).
- `tests/integration/tests/m19_scala_discovery_test.rs`: A7 ONE hunk only.
- `tests/integration/Cargo.toml`: NOT touched (the preferred m19
  delegation form held; the suite imports no lang-rust and no bincode).
- `tasks/rust-language-r4-journal.md` (this file).

## Lock proofs (path-scoped, run after commit; distiller support commands)

See "Lock proofs" section appended after the landing.


## Red protocol (a)/(b)/(c) accounting

- (a) fixture + suite authored first; gate 2 above is the Red run.
- (b) A1/A2/A7 test edits applied with NOTHING registered; the cli scoped
  run (gate 4's first invocation) captures the amendment Red:
  `default_registry_enums_cpp_then_scala` fails
  `left: [Cpp, Scala], right: [Cpp, Scala, Rust]`;
  `pin_parse_language_arg_exact_accepted_set_not_case_insensitive` fails on
  the new `rust` accepted row
  (`Err("unknown language 'rust'; ...")`). The m19 A7 red (amended vec vs
  unregistered registry) is NOT separately executed: no spare gate budget;
  its falsification is identical in kind and the same registration line
  greens it (gate 6 proves).
- (c) A3/A4 are pre-flip amendments (CW-1): edited to the `.f90` witnesses
  they compile and pass IMMEDIATELY - green-on-landing by construction.
  Their falsification is inverted: if the registration line landed WITHOUT
  them, the `src/unclaimed.rs` witness would start resolving and both tests
  would go red. Stated verbatim here per the handoff requirement.

