# UTA-T6 Verdict - cluster + artifact + build + pipeline pins (H6)

Handoff: `tasks/uta-t6-handoff.md`. Base: `34f3f87`. Implementation: single commit
`e95a6d5 test(cluster,artifact,build,pipeline): uta-t6 security/manifest/frontier/reuse pins`,
shipped by the SUCCESSOR session (`ses_eaf688cb...`) after the original implementer session
(`ses_eaf9e1cb...`) died uncommitted with an empty compaction summary (never resumed, per
repo law). Recovery chain: `tasks/uta-t6-handoff-resume.md` + `tasks/uta-t6-journal.md`
"Successor checkpoint" + `checkpoint_guard resume` bootstrap.

## VERDICT: APPROVED

4 numbered findings, all minor/non-blocking; no required changes on the implementer.
Gate evidence: `tasks/uta-execution-results.md` H6 addendum.

Canonical lines observed fresh (this review):
- `57 passed / 0 failed / 0 ignored / 2 suites` (artifact)
- `67 passed / 0 failed / 0 ignored / 8 suites` (cluster)
- `28 passed / 0 failed / 0 ignored / 2 suites` (build)
- `46 passed / 0 failed / 4 ignored / 9 suites` (pipeline; the 4 ignores are the
  pre-existing bench fixtures `boot_stages_debug_context_timing`,
  `boot_stages_release_timing`, `collateral_load_stage_cold_timing`,
  `justification_payload_and_route_cold_timing` - verified in the capture)
- tools suite: `Ran 232 tests in 19.753s` / `OK`
- hygiene + encoding: `### generic PASS (exit 0)` x2 (via `tools/distill.py`)

## 1. THE RECOVERY - verified complete, nothing lost, no half-edited source

The resume report's in-scope disk state at `34f3f87` (8 dirty + 4 untracked pin
modules + 1 untracked journal = 13 files) maps 1:1 onto `e95a6d5`'s 13 paths:

| resume-reported (dead session) | in `e95a6d5` |
|---|---|
| artifact manager.rs / bincode_options.rs / metrics_io.rs (dirty) | +2 / +84 / +76 |
| new manager/uta_t6_pin_tests.rs | +428 (new) |
| build lib.rs (dirty) / new uta_t6_pins.rs | +3 / +267 (new) |
| cluster frontier.rs / planner.rs / scc.rs (dirty) | +184 / +2 / +88 |
| new cluster planner/uta_t6_pins.rs | +208 (new) |
| pipeline lib.rs (dirty) / new uta_t6_pins.rs | +3 / +275 (new) |
| tasks/uta-t6-journal.md (untracked) | +98 (new) |

The 6 "stray" files the resume report flagged as out of scope
(`uta-execution-results.md` + the five `uta-t*-verdict.md`) were NOT committed -
they remain untracked (checkpoint_guard census). Purely-additive claim verified at
`git diff --numstat 34f3f87..HEAD`: **zero deletions in every one of the 13 files**;
all four suites green twice in this session, so no half-edited source survives.

## 2. Pins-only discipline - verified hunk-by-hunk, zero production-behavior change

Every `+N` line outside `#[cfg(test)]` in the eight modified src files is a mod
declaration or a blank line: manager.rs (`#[cfg(test)]` + `mod uta_t6_pin_tests;`),
build/lib.rs (blank + cfg/test + mod), planner.rs (cfg/test + mod), pipeline/lib.rs
(blank + cfg/test + mod). The other four modified files (bincode_options.rs,
metrics_io.rs, frontier.rs, scc.rs) received only appended
`#[cfg(test)] mod ... { }` blocks after the closing brace of the last production
item. No production function, const, or signature touched. Frozen locks: zero
deletions means every pre-existing test is byte-untouched; the pre-existing
`manager::tests::*`, `m22_manifest_languages_tests::*`, `payload_reshape_tests::*`
all pass unchanged in the fresh capture.

## 3. Security row (item 1) - genuine with_limit rejection on both variants

Hand-built hostile blobs declare `u64::MAX` lengths (endian-agnostic by value)
against the real production path
`DefaultOptions::new().with_fixint_encoding().allow_trailing_bytes().with_limit(16 GiB)`
(bincode_options.rs:52-56). Adjudicated against the vendored bincode 1.3.3 source
(`registry/.../bincode-1.3.3/src/de/mod.rs`), NOT against the implementer's comment:
- **String path**: `read_string` -> `read_vec` (:93-96) calls
  `read_bytes(len)` -> `options.limit().add(count)` (:84-85) BEFORE
  `get_byte_buffer(len)` (the allocation/abort vector). The two string-rejection
  tests therefore reject on the with_limit path exactly as claimed, and would
  abort (capacity overflow), not merely fail an assert, if the limit were removed.
  Tested through BOTH public entry points (`deserialize_bounded` slice +
  `deserialize_bounded_reader` Cursor). Honest pass-through control present.
- **Seq path**: see finding 2 (EOF-driven rejection; truthful but not
  limit-specific). The module's actual allocation-blowup vector is the string/bytes
  path, and it is pinned. The test names ("rejects huge declared ... length") do
  not overclaim an internal path; no mislabeled happy path exists - all four
  rejection tests use genuinely hostile inputs and all five ran green (capture
  lines 3-8).

## 4. Adjudicated-pin honesty (per spec rows in docs/specs/unit-test-audit.md)

All six directed pin names shipped VERBATIM and each assertion is at least as
strong as the spec row requires:
- `pin_frontier_breaks_at_first_frozen` - audit:73 row (`frontier.rs:71-75`), start->frozen->opt->collateral
  pinned to exactly `[start.h]`; hand-verified against frontier.rs:64-76 (break at first non-Optimizable).
- `pin_frontier_ignores_ownership` - audit:74 row (`frontier.rs:20` literally `_file_ownerships`);
  byte-equal answers with empty vs all-foreign (ThirdParty/Toolchain) ownership maps.
- `pin_manifest_without_root_key_skips_root_check` - audit:75 row; production `if let
  Some(root)` gate at manager.rs:172 skipped when key absent; asserted Ok + fresh.
- `pin_corrupt_bin_falls_back_to_json` - audit:76 row (apply_io.rs:94-109); corrupt
  bin + honest json still loads with the json timestamp asserted.
- `pin_normalize_drops_duplicate_named_context` - audit:77 row (types.rs:97
  `dedup_by(|a, b| a.name == b.name)`); loser dropped WHOLE - survivor's compiler/
  defines/include_dirs asserted first-wins PLUS explicit absence of the loser's data
  (no field-merge escape).
- `pin_extraction_manifest_fingerprint_empty` - fingerprint asserted present AND `""`
  (the key-present-but-empty case, explicitly distinguished from the skip-key escape).
All six executed green by name in the fresh captures, including the two
`#[cfg(windows)]` artifact pins (`pin_same_path_table...` capture line 43,
`pin_verbatim_manager_root...` line 59).

## 5. Item-by-item plan-literal audit

- Item 2: root-mismatch->stale (error names the root gate), key-absent skip,
  `"{not json"`->stale (verbatim blob from the handoff), same_path table = case +
  `/` vs `\` + trailing-slash (both directions) + `\\?\` verbatim + symmetry +
  sibling control. The Windows-gated table pins ran (host is Windows, capture-confirmed).
- Item 3: fallback ladder 3 arms (json fallback / both-corrupt names path /
  unknown-extension rejected); streaming emit-Err arm (prior bytes intact + tmp
  removed) and rename-fail arm - the latter pins an ADMITTED WART (staged tmp left
  behind) with a loud comment "pin of an admitted wart, not an endorsement". Honest.
- Item 4: non-empty TU round-trip + `Some(IncrementalStats)` manifest fields
  (11 exact assertions) + reload; extraction fingerprint pin.
- Item 5: all five path shapes present (`shape_no_path_returns_empty`,
  `shape_start_equals_collateral_returns_empty`, `shape_all_optimizable_...`,
  `shape_missing_policy_entry_treated_as_optimizable` + the two smell pins) - 6 tests.
- Item 6: merge anchor-position/reverse-append + transitive-join + non-intersecting
  no-op asserted as EXACT full-vec matches (hand-traced against planner.rs:696-717:
  anchor = `intersecting.split_first()`, `rest.iter().rev()` - pin narrative is the
  code's own doc); screening duplicate later-wins asserted with the FULL winner
  struct; check_candidate_safety accept + `FrozenFile`-with-target pair. 4 tests.
- Item 7: cross-SCC order byte-genuine - exact `[C],[B],[A]` / `[A,B,C],[D]` vectors
  PLUS 4x repeated-run equality on both `PartialEq` and Debug-rendered bytes;
  duplicate-seed no-double-emit pinned. 3 tests.
- Item 8: normalize whole-context drop; discover precedence as a three-tier deletion
  walk (root>build>out - a superset of the handoff's "two compile_commands.json");
  plain-relative branch (cites discovery.rs:284-286 exactly) + verbatim-canonical
  string-fallback branch with counterfactual guard; format_num boundaries 0/1,000/
  100,000/1,000,000 both report sides; format_number divergence (cluster plan.rs:73-83
  string-reverse vs build measure.rs:78-94 arithmetic) verified to exist as journal-recorded,
  NOT unified per instruction; arrow char referenced by code point, file stays ASCII.
- Item 9: all five rows (9a swallow-contract with premise assertions, 9b no-poison,
  9c named-error bail with all-claimed control, 9d replace + both summary branches
  byte-exact, 9e unclaimed->Reject with claimed-stance control) - 6 tests, all
  green in capture (first-ever run of the dead session's item-9 module).
- Item 10: REPORT ONLY - journal records it; independently confirmed:
  planner.rs:477 `scc::compute_symbol_sccs(&file_symbols, mapped_edges)` inside the
  per-candidate loop, and the reuse helpers `build_symbol_adjacency` (scc.rs:21) +
  `compute_symbol_sccs_from_adj` (scc.rs:38) exist and are unused by that hot loop.
- No retained dual representation is possible (pins-only hand; no algorithm replaced).
- Commit message matches the handoff's gate-7 literal verbatim; commit made via
  `commit_scoped.py` over the five declared globs; touch-set clean (13/13 paths
  inside `crates/{cluster,artifact,build,pipeline}/src/**` + journal).

## 6. Deviation dispositions (independent adjudication)

- **D1 - first same_path attempt went RED at manager level; table pinned at the
  function instead** - ACCEPTED on merits. The red was a fixture artifact, not a
  behavior conflict: `Repository::locate` canonicalizes, so a manager-built root IS
  `\\?\`-verbatim on Windows and the slash-rewritten spelling is not constructible
  through it; the handoff's adjudicated table was pinned at `same_path` itself
  (the level the handoff's item-2 text names), fully and symmetrically, and the
  real-world consequence became a second honest test
  (`pin_verbatim_manager_root_rejects_the_slash_rewritten_spelling`) with a premise
  assertion, pass-side forms, and an expect_err + error-message match. Nothing was
  dropped, nothing production changed, the smell is REPORTED for owner decision
  (journal + pin comment). This honors the RGB STOP clause (no unadjudicated row
  was pinned as if adjudicated; the discovered behavior got its own named lock).
- **D2 - scratch file `crates/artifact/same_path_probe.py` claimed absent** -
  ACCEPTED, independently corroborated: no `.py` under `crates/artifact/` (glob),
  `git log --oneline --all -- crates/artifact/same_path_probe.py` -> empty (never
  committed on any ref), checkpoint_guard untracked census clean. The successor
  reported the absence rather than silently dropping the brief item or fabricating
  a deletion. Honest.
- **D3 - superseded pin-count estimate** - ACCEPTED. The committed journal carries
  only final counts, and they match the executed test-name inventory in this
  review's fresh captures exactly: artifact 5 bounded_decode + 2 streaming_write +
  10 uta_t6_pin_tests = 17; cluster 6 frontier + 4 planner + 3 scc = 13; build 5;
  pipeline 6 -> 41 total. Suite arithmetic consistent
  (40+17=57, 54+13=67, 23+5=28, 40+6=46). No stale estimate survives anywhere in
  the committed tree; the dead session's intermediate estimate, whatever it was,
  is not in the journal and the successor did not paper over it.
- **D4 - rename-failure pin asserting the tmp-left-behind wart** - ACCEPTED.
  Handoff asked for "rename-fail pinned"; the pin does so with real assertions
  (Err surfaces, target dir untouched, staged tmp count == 1) and self-labels the
  wart. Journal's "Pin honesty note" discloses it.

## 7. Findings (all minor; none block the merge)

1. **Journal gate record omits distill capture paths** (uta-t6-journal.md
   "Successor checkpoint" gate record: canonical lines + exit witnesses present,
   capture paths not recorded; review-gate step-8 capture-evidence clause).
   Closed reviewer-side: fresh re-runs this session produced the capture headers
   quoted in the H6 execution-results addendum. Route: journal habit, closeout note.
2. **Seq-side bincode pins do not isolate the with_limit path** (ledger candidate).
   bincode 1.3.3 gates a DECLARED length through the limit only on the str/bytes
   paths (`de/mod.rs:94-96` `read_vec` -> `read_bytes` -> `limit().add` before any
   allocation); `deserialize_seq` (:348-350) routes to `deserialize_tuple` with
   per-element accounting, so the shipped `*_rejects_huge_declared_seq_length`
   tests reject via element-read EOF. The pins are truthful (hostile input, fast
   Err, both variants, no blowup) and the security-critical string path IS
   limit-verified, so the module's stated purpose is locked; but a seq lock that
   would still fail with `.with_limit` removed needs a just-over-limit blob with
   real bytes present. Owner decision, deferred-findings row.
3. **Reviewer allowlist shape refusals (configuration note, not skipped gates)**:
   the compound `> $env:TEMP` + `$LASTEXITCODE`-echo shape and `Get-ChildItem` hit
   the catch-all deny boilerplate
   (`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`).
   All expected gates ran anyway (plain commands + `tools/distill.py` exit-code
   witnesses); same shape family recorded by H1-H5.
4. **Spec citation drift (owner-side)**: `docs/specs/unit-test-audit.md:75` cites
   `manager.rs:170` for the root-key skip; the live `if let Some(root)` gate is at
   manager.rs:172 (2-line drift, substance identical). Refresh the spec row at closeout.

## 8. Follow-up dispositions (routed, not dropped)

- Findings 2 and 4 -> `tasks/milestone_ledger.md` deferred-findings at UTA closeout
  (seq-limit blob lock owner decision; spec citation drift).
- Owner-decision backlog already correctly journal-recorded by this hand and to be
  carried at closeout: same_path verbatim-combination wart (slash-rewritten
  verbatim root mismatch), rename-fail tmp-leak, normalize whole-context drop,
  root-key-absent skip, corrupt-bin silent fallback, extraction "" fingerprint,
  frontier break-at-first-frozen + ownership-ignored, planner:477 SCC-rebuild perf
  (REPORT ONLY), format_number divergence (no unify).
- `--baseline` / `ui_smoke` / `bench_compare` remain the milestone-closeout gates
  (H6 of 8+ hands; per AGENTS.md sizing rule and T1-T5 precedent).
