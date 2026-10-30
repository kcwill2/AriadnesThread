# R-9b3 verdict - rust-language (reviewer gate, skill `review-gate`, 2026-10-27)

Hand: R-9b3 single landing `c93bada` (parent `aa67ff2` = AMENDMENT-3 docs
commit; `8c1ac46` = R-9b2 STOP journal). Duty-less hand, no duty-1.
Governing contract: `tasks/rust-language-r9b3-amendment.md` (AMENDMENT-3),
read complete; also journal secs. 10-12, AMENDMENT-2 sec. 4, AMENDMENT-1
secs. 1a/7, ledger row 62.

## Verdict: APPROVED

Zero blockers. Findings 1-5 are minor/informational dispositions; none
requires re-work. The fix is what the amendment ruled (option (a)), the
landing is what the hand said, every named lock holds, and the fresh
evidence is exact at every gate.

## 0. Ground truth

`python tools/checkpoint_guard.py status`: HEAD `c93bada`, branch
`add-rust-language-support`, no dirty tracked files, one untracked file
(`tasks/rust-language-r9b3-amendment-resume.md`, sec. 6 D2). Post-gate
re-check (`git status`): tree still clean bar that artifact - the bench
changed nothing (D-3 held).

## 1. Per-commit review (oldest-first) + commit-vs-touch-set

Range `aa67ff2..c93bada` = ONE commit. `git show --stat`:
`Cargo.lock` +1, `crates/lang-rust/src/dependency_rewriter.rs` 69/6,
`crates/lang-rust/src/discovery_fidelity_tests.rs` +56 (new),
`crates/lang-rust/src/lib.rs` +2, `tasks/rust-language-r9b-journal.md`
+131/-0, `tests/integration/Cargo.toml` +2,
`tests/integration/tests/rust_language_symbol_isolation_test.rs` +992 (new).

`git diff --name-only aa67ff2..c93bada` = exactly those 7 paths, every one
inside AMENDMENT-3 sec. 4b: production zone + witnesses, the CONDITIONAL
new-file allowance (sec. 4b, taken), its declaration host (finding 1),
the journal append-only (single hunk `@@ -391,5 +391,136 @@`, 0 deleted
lines, appended at sec. 12), the two granted manifest paths, the suite
landing with the sanctioned re-pin. Commit message matches the amendment's
prescribed `commit_scoped` message VERBATIM. No path outside the touch-set.

## 2. Gate-C diff audit (RULING 1 fix-correctness)

`git diff aa67ff2..c93bada -- crates/lang-rust/src/dependency_rewriter.rs`
shows exactly three hunk zones: the module-header bullet (doc `//!` lines,
`-4/+8`), the `consumer_discovery_targets` doc comment + body (`-2/+18`),
and a pure APPEND of W1 + W3 inside `mod tests` (zero `-` lines). All 6
deleted lines are: 4 header-bullet doc lines, 1 doc-comment line, 1 body
line (`let mut probes: Vec<String> = moved_module_ids.to_vec();`).
ZERO modified/deleted lines in `leaf_affected_by`, `find_dependency_targets`,
`consumer_rewrite_spec`, `derive_flat_id_pairs`, `retarget_id`,
`insert_declared_mods`, and every existing test fn - witnesses
`discovery_targets_are_the_moved_paths_plus_the_source_module_stem` (:644),
`commented_quoted_and_sibling_spellings_are_never_hits` (:687),
`derived_spec_retargets_the_flat_id_from_a_mod_rs_parent` (:739),
`derived_spelling_never_touches_a_colliding_flat_name` (:770), the inert
`lib`/`mod` anchor witness (:898 region) all present UNEDITED (numbers =
post-landing, +18 shift from the two pre-test hunks).

Grammar law (AMENDMENT-1 sec. 1a(2) + CW-1 agree-by-construction): the new
leg uses the SPEC LEG'S OWN inputs and functions - `flat_id_leaf`
(:221-224, the same single-segment `crate::` eligibility grammar
`derive_flat_id_pairs` gates on) and `module_path_of_src_file`
(`mod_tree.rs:139-173`, frozen file), composing `format!("{module}::{flat_leaf}")`
- byte-identical composition to the spec leg's old-spelling `format!("{old_module}::{flat_leaf}")`.
Therefore for the wired route (`target_file == old_includes[i]`) every added
probe IS the spec's old spelling: agree-by-construction, the C++ CW-1 law
(`crates/lang-cpp/src/dependency_rewriter.rs:96-113` - "the SAME ... rule
... so discovery's predicate and the rewrite's matcher agree by
construction"), Rust analog by derivation as RULING 1 requires. The
dedupe-by-equality (`if !probes.contains(&honest)`) implements the
amendment's own parenthetical ("spelled `crate::X` when `M == "crate"`,
which is then deduped by equality with the id leg"): with M == "crate",
`honest == id` always, so the observable list is identical to an explicit
`M != "crate"` guard. Multi-segment/foreign ids: `flat_id_leaf` = None ->
contribute nothing; non-src target: M = None -> contribute nothing
(conservative "never invent a probe" stance). Order: id, honest, ... , stem
last - pinned by W1/W3 and the suite. No retained dual probe-list
representation: the old `to_vec()` leg was replaced, not parallel-kept.

INV resolution (amendment secs. 1c/1d):
- INV-R9B3-1 -> W1 `discovery_probes_the_honest_module_spelling_for_a_nested_parent`
  (verbatim plan-2 list pin) + W2 `discovery_and_rewrite_hit_iff_over_the_honest_spellings`
  (the Rust X1(a) fidelity pair, relocated per sec. 4b allowance; five
  classes exactly as named in sec. 1d, positive class asserted
  discovered-AND-rewritten, glob class = the aggregate-narrowing symmetric
  class with the R-8 asymmetry CITED not re-proved, doc cites :643-659/
  :523-534 as required). Both green in my fresh crate run.
- INV-R9B3-2 -> W3 `crate_root_discovery_probes_stay_byte_identical` +
  UNEDITED :626-639 witness (zero hunk) + UNEDITED suite plan-1 pin
  :527-531 (outside the single suite hunk).
- INV-R9B3-3 -> the zero-diff audit above; :669-686, :752-767, :721-746
  green UNEDITED inside my fresh `105/0/0/2`.
- INV-R9B3-4 -> the two named suite reds green with bytes UNTOUCHED
  (proof sec. 3), turning green by mechanism, my fresh `11/0/0/1`.

## 3. Suite disk-identity + re-pin adjudication

The suite was UNTRACKED at `8c1ac46`/`aa67ff2`, so the STOP disk state is
the implementer's byte copy `r9b3_suite_stop_bytes.rs` (temp). Independent
re-derivation (not trusting the capture):
`git diff --no-index <stop-copy> <worktree suite>` = EXACTLY one hunk
`@@ -532,8 +532,10 @@` (+4/-2) in
`preview_lists_discovery_targets_and_the_parent_oracle_names_the_mod_line`,
index line `c7276ab..ecc84ad`, and `git rev-parse "c93bada:tests/.../rust_language_symbol_isolation_test.rs"`
= `ecc84adb68c5e65be443f8fb9502a858c3d1dd0b` - the committed file IS the diff's b-side.
Byte-identity of everything else is machine-proven; the two regression
regions (`apply_moves_models_marker_and_rewrites_the_consumer_import` now
:670, `apply_isolates_both_functions_and_rewrites_the_consumer_verbatim`
now :712 - consistent +2 shift from :668/:710) and the plan-1 pin
(:527-531, read at source: `vec!["crate::fixture_marker".to_string(),
"lib".to_string()]`) are OUTSIDE the hunk, unchanged.

Re-pin vs the sanctioned replacement list (sec. 4b): landed list
`["crate::models_marker", "crate::models::models_marker", "mod"]` =
element-for-byte match to the amendment's list, dated comment cites
"AMENDMENT-3 ruling 4b, dated 2026-10-27". STRICTLY STRENGTHENING
confirmed: the assertion form is unchanged `assert_eq!` full-Vec equality,
the new pinned list is a superset (3 > 2, contains the old list in order),
and the re-pinned middle probe equals the spec leg's pinned old spelling
(`:739` witness) - INV-R9B3-1/4 exactly. The hunk also evolved the assert
MESSAGE line (finding 3, accepted): the old message described the
two-probe list and would have lied about the new one; the message line
sits inside the same sanctioned assert of the named test.

`Cargo.toml` hunk = the two granted lines VERBATIM (comment text byte-matches
the AMENDMENT-2 sec. 4 granted comment; row `ariadne-lang-rust = { path =
"../../crates/lang-rust" }`). `Cargo.lock` hunk = +1 line
`"ariadne-lang-rust",` inserted alphabetically in the
`ariadne-integration-tests` deps list - the minimal machine form cargo
emits. Byte-identity to the untracked STOP disk state is not re-derivable
at landing (finding 4 note); content verification passes on both files.

## 4. RGB protocol adjudication

Leg 1: the two suite reds are the named regression witnesses, unedited to
this landing (sec. 3 proves bytes). Leg 2 (RED): journal gate 2 `103 passed
/ 2 failed / 0 ignored / 1 suites` with the two named reds and verbatim
`left:` lines - W1 `left: ["crate::models_marker", "mod"]` vs right = the
sanctioned three-list (exactly the predicted missing-middle shape); W2
`left: false / right: true` for class `honest M::X leaf` - the D6
disagreement red for the predicted semantic reason; the `1 suites` shape
is the amendment's own standing distiller observation. W3 green-at-birth
attributed, never silent. Leg 3: GREEN `105 passed / 0 failed / 0 ignored /
2 suites`, refactor + post-split re-runs same line. Red evidence precedes
the production edit (gate 2 before gate 3); the W2 relocation happened
AFTER red was captured (deviation 3, sec. 6). The intermediate RED tree was
uncommitted - structurally unreproducible by a reviewer at landing, same
adjudication basis as R-9a criterion 2: verbatim `left:` values are true
against the start-rev production blob (`git show "aa67ff2:crates/lang-rust/src/dependency_rewriter.rs"`
shows the `to_vec()` body producing exactly `["crate::models_marker",
"mod"]`), and W2's journalled assert message string
(`fidelity pair broken for class {class}`) is byte-present in the relocated
file - the inline and relocated W2 are the same witness.

Green-at-birth adjudication on the merits (not rubber-stamped): W3 pins
that crate-root/main.rs probe lists stay `[id, stem]` - observable shipped
behavior BEFORE the fix (the `to_vec()` body answered exactly that) that
the derivation must not perturb. It claims no unshipped mechanism; it is
the lock INV-R9B3-2 names, and sec. 1d pre-sanctions it by name under the
SM-3 clause (the usage AMENDMENT-2 sec. 2 already employed). The W2
negative classes are the same class of justified characterizations:
conservative non-hits already pinned at :669-686/:752-767 (green UNEDITED
in my runs), and the glob class asserts the post-fix symmetric
aggregate-narrowing pair while citing the R-8 asymmetry as landed law.
ACCEPTED on the merits.

## 5. Fresh gates (all reviewer-run, distilling entry points, 2026-10-27)

| Gate | Expected | Fresh result |
|---|---|---|
| `-p ariadne-lang-rust` | `105 / 0 / 0 / 2` | `105 passed / 0 failed / 0 ignored / 2 suites` (exit 0) - exact; 29 R-8 + 11 R-9a + R-9b1 pins green UNEDITED inside (zero hunks on them, sec. 2) |
| `--test rust_language_symbol_isolation_test` | `11 / 0 / 0 / 1` | `11 passed / 0 failed / 0 ignored / 1 suites` (exit 0) - exact |
| `--test m23_symbol_isolation_test` | `7 / 0 / 0 / 1` | `7 passed / 0 failed / 0 ignored / 1 suites` (exit 0) - exact |
| `-p ariadne-cli` | `182 / 0 / 8 / 8` | `182 passed / 0 failed / 8 ignored / 8 suites` (exit 0) - exact; registry/discovery tests green inside |
| `-p ariadne-cluster --test planner_injection` | `6 / 0 / 0 / 1` | `6 passed / 0 failed / 0 ignored / 1 suites` (exit 0) - exact; cluster zero-diff |
| full census (bare, `--distill -- --no-fail-fast`) | `2003 / 0 / 13 / 126` | `2003 passed / 0 failed / 13 ignored / 126 suites` (exit 0, capture 3432 lines) - EXACT at `c93bada`. Deltas by name over `1989/0/13/125 @ 588bb86`: +11 named suite tests + 3 named witnesses (W1/W2/W3 names at source); suites +1 = the one new integration binary; ignored flat 13. Floor of record = AMENDMENT-3 ruling 3 (measured), not the superseded 2023 |
| `hygiene_check.py` | exit 0 | exit 0 (no `### FAIL`/exit-2; warn-only R3 rows, all pre-existing in untouched zones; zero rows name a touched file). `dependency_rewriter.rs` 955 lines (<995 budget, <1000 R2); `discovery_fidelity_tests.rs` 56; suite 992 (<1000); `consumer_discovery_targets` ~20 lines (<100) |
| `encoding_guard.py --all` | exit 0 | exit 0 (silent) - authored bytes pure ASCII |
| tools `unittest discover` | `Ran ... OK` | `Ran 296 tests in 22.059s` / `OK` |
| bench (REQUIRED - `crates/lang-rust/src` touched) | PASS, D-3 | `python tools/distill.py -- python tools/bench_compare.py` -> `### generic PASS (exit 0)`; `bench compare: 3 run(s), no regressions past 1.25x baseline`; ZERO WARNs (record stance moot); tree clean after - no baseline/corpus change (D-3) |
| ui_smoke (kickoff sec. 4 surface) | PASS | `### generic PASS (exit 0)` / `boot+route checks clean (cold fixture, attempt 1, 1.1s)` / `ui_smoke: PASS` |

Zero-diff proofs by name (`git diff --name-only aa67ff2..c93bada -- ...` ->
EMPTY): the engine/lang crates batch `crates/lang-cpp crates/lang-scala
crates/lang crates/cluster crates/cli crates/transform crates/validate
crates/index crates/graph crates/repo crates/artifact crates/report
crates/pipeline crates/ui crates/build crates/scheduler tools`; the lang-rust
forbidden files `plugin.rs constraints.rs planning.rs validator.rs
mod_tree.rs move_retention.rs extractor.rs` (plus analyzer/resolver/
parser_*/shaper/detector/boundary - absent from the whole-range name-only
list, which contains only the three named lang-rust files); the four caps
locks (`rust_language_discovery_test.rs`,
`rust_language_semantic_index_test.rs`, `rust_language_dependency_graph_test.rs`,
`rust_language_load_analysis_test.rs`); `m23_symbol_isolation_test.rs`;
m18-m22 + m24 suite files; `m00_facts_path_parity_test.rs`,
`m00_golden_baseline_test.rs` (goldens), `ar27_field_probe_test.rs`,
`tests/fixtures` (L3 read-only). `CURRENT_ANALYSIS_VERSION` (crates/index)
zero bytes; fingerprint `"rust-1"` + `CyclePolicy::Reject` trivially held
(no file carrying them has a hunk). `refs/stash` absent:
`git rev-parse --verify refs/stash` -> `fatal: Needed a single revision`.

Report-evidence check (review-gate step 8): journal sec. 12 is
digest-shaped - distilling entry points only, canonical lines + capture
paths (`target/testrun-capture.txt`), zero raw dumps, red `left:` lines
verbatim, F14-clean (no self SHA).

## 6. Deviation dispositions (numbered, on the merits)

- D1 (journal deviation 2+3) CONDITIONAL split escape TAKEN: ACCEPTED.
  Trigger real (inline growth hit 1002 > 995 budget and > R2 1000); the
  escape names exactly the mod-declaration line + `crates/lang-rust/src/
  discovery_fidelity_tests.rs` (the only new-file allowance) and requires
  journal declaration - all present. lib.rs hunk enforced: exactly
  `#[cfg(test)]` + `mod discovery_fidelity_tests;`, NOTHING else (finding 1
  host adjudication). W2 content matches sec. 1d exactly (sec. 2). Red was
  captured inline BEFORE relocation and the relocated bytes re-gated at
  gate 3 with the unchanged `105/0/0/2` line - the escape's letter and
  purpose (byte-equivalent witness, hygiene ceiling) honored; residual
  byte-level non-rederivability recorded as finding 2.
- D2 (journal deviation 1) bootstrap resume glitch + untracked
  `tasks/rust-language-r9b3-amendment-resume.md`: correct disposition
  verified - the file is a `checkpoint_guard resume` report (read at disk;
  holds only facts already in journal sec. 12), outside the touch-set,
  deliberately NOT committed. ROUTED: DELETE at R-10 duty-1 (R-7 precedent,
  R-9b1 verdict criterion 11 band; reviewer is barred from deleting). Sec. 9.
- D3 (journal deviation 4) catch-all-deny boilerplate on bootstrap probes,
  retried via `tools/distill.py`: CONFIRMED no gate skipped - the hand's
  full gate ladder (1-8 + hygiene) is journalled with digest evidence, and
  my own fresh acceptance pass re-ran every gate at landing independently
  (sec. 5); my own refusals are listed in sec. 7 with substitutions.
- D4 (implicit) base-rev note: journal honestly states gate-C audit vs
  `8c1ac46` is not applicable - base is `aa67ff2` (the amendment's relay
  commit). This verdict audits `aa67ff2..c93bada` per sec. 4e; suite
  disk-identity per sec. 3 above. No finding.

## 7. Findings

1. MINOR/ACCEPTED - `crates/lang-rust/src/lib.rs` is not enumerated in the
   sec. 4b touch-set host list. The landing's +2 lines are EXACTLY the
   escape's named `#[cfg(test)] mod discovery_fidelity_tests;` declaration
   (cfg attribute + mod line), and lib.rs is the only host where the
   amendment's named path `crates/lang-rust/src/discovery_fidelity_tests.rs`
   resolves without a `#[path]` attribute or relocation (both alternatives
   would violate the amendment's own naming). `lib.rs` is also absent from
   the MUST-NOT enumeration (which names the other seven lang-rust files).
   Disposition: within the declared allowance; the host is forced by Rust
   module law. No change required.
2. MINOR/RECORD - byte-level pre/post-relocation equivalence of W2 is not
   independently re-derivable by a reviewer (the inline version existed
   only in the uncommitted intermediate tree). Adjudicated on: journalled
   gate-2 failure message byte-present in the relocated file, identical
   semantics (same five classes, same asserts per sec. 1d), re-gate at
   gate 3 with the unchanged count line, and the escape's text ("carrying
   W2"). Acceptable; a future escape-taken hand should snapshot the inline
   witness bytes to temp before relocating (record for closeout practice,
   not a defect here).
3. MINOR/ACCEPTED - the single sanctioned suite hunk also replaced the
   assert MESSAGE line (`"plan 2 probes: ..."`) plus two comment lines. The
   comment lines ARE the amendment-required dated citation; the message
   line sits inside the same `assert_eq!` of the named test and the old
   text would have described the pre-fix list falsely. Within the named
   re-pin; strictly-strengthening claim unaffected.
4. INFORMATIONAL - "Cargo rows landed byte-identical to the R-9b2 disk
   state" is not reviewer-re-derivable byte-wise (that disk state was never
   tracked). Verified instead: landed `Cargo.toml` hunk = the AMENDMENT-2
   sec. 4 granted row + comment VERBATIM (2 lines, `+2/-0`); landed
   `Cargo.lock` delta = the minimal machine insertion (+1 line, sorted);
   both compile-consistent (suite green at landing); tree was dirty with
   exactly these files at bootstrap per the resume report + journal.
5. INFORMATIONAL/ROUTED - untracked `tasks/rust-language-r9b3-amendment-resume.md`
   must be DELETED at R-10 duty-1 (R-7 precedent). Reviewer edit grant bars
   me from deleting; obligation restated verbatim in sec. 9.

## 8. Out-of-scope observations

None new beyond the amendment's own recorded rows: DEFER-ZERO-CONSUMER
(ruling 2, dated engine deferral), the flat-id-sharing honest limit row
(`constraints.rs:29-33`), the R-9a clippy-baseline informational (R-9a
addendum) - all already ledger/verdict-owned; nothing dropped.

## 9. R-10 duty-1 obligations (VERBATIM-ready)

Ledger relay (AMENDMENT-3 sec. 6 Draft B with SHAs filled - land in the
R-10 duty-1 commit, ledger is orchestrator-owned):

`R-9b3 LANDED c93bada APPROVED 2026-10-27 (nested-module consumer-discovery fix option (a) - discovery and rewrite agree by construction; symbol-isolation suite 11 tests green incl. the byte-untouched regression pair; granted lang-rust dev-dep row landed), census anchor 2003 passed / 0 failed / 13 ignored / 126 suites @ c93bada, R-10 dispatched (docs hop + parity suite + dated riders incl. the DEFER-ZERO-CONSUMER row).`

R-10 duty-1 commit ALSO carries:
- DELETE the untracked `tasks/rust-language-r9b3-amendment-resume.md`
  (tool artifact; R-7 precedent cleanup row; do not commit it first).
- Land `tasks/rust-language-r9b3-verdict.md` and the R-9b3 row appended to
  `tasks/rust-language-execution-results.md` (both authored 2026-10-27 by
  this review).
- Memory mirror `ledger.milestone.rust-language` updated in the same commit
  (anchor `2003 passed / 0 failed / 13 ignored / 126 suites @ c93bada`).
- Do NOT re-land any existing SHA (`c52817c` / `f2387f1` / `fcb078c` /
  `cef3b0a` / `eb1e0c9` / `588bb86` / `8c1ac46` / `aa67ff2` / `c93bada`).

R-10 hand scope (per AMENDMENT-3 + kickoff closeout lists): docs hop +
parity suite + dated riders: DEFER-ZERO-CONSUMER row (engine zero-consumer-
commit honesty, ruling 2); the five docs/adding-a-language.md riders (R-6
verdict finding 4 relay); the stale registry.rs header correction (R-9b1
findings 1-2: `.f90` precedent is R-4-era `ec552bd`, m24 uses `.mock`);
`syn::__private::ToTokens` -> direct `quote` dep deferral (R-8 D1 dated
row, first manifest-capable hop); the flat-id-sharing honest-limit dated
row (`constraints.rs:29-33`). Census posture: docs-only hand = bench
EXEMPT with the zero-`crates/*/src`-diff proof (kickoff sec. 4); parity
suite adds named tests, every delta attributed by name.
