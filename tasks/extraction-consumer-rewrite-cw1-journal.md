# CW-1 Journal - extraction-consumer-rewrite

Implementer hand 1 of 3. Contract:
`tasks/extraction-consumer-rewrite-cw1-handoff.md`.

## Bootstrap (checkpoint_guard)

`python tools/checkpoint_guard.py status` at dispatch:

- HEAD `2e5b86f` on `main` (matches the dispatch-required start rev).
- Dirty: only the tolerated pre-existing
  `extensions/ariadne-extension/ariadne-extension-0.1.0.vsix`. No untracked
  files. No STOP condition.
- Start rev of record: `2e5b86f` (every reviewer diff is against this).

## UTA intake (docs/specs/unit-test-audit.md, Deferred findings 2026-10-19)

Read in full before touching lang front-ends. The deferred lang-cpp items
name detector/planning/shaper case divergence, backend missing-source,
`#else` double-masking, ParserCtx self-edge, and the `A a();` phantom -
zero items name `dependency_rewriter.rs` or the `consumer_rewrite_spec`
seam. Matches the planner intake result. No new finding carried: nothing
discovered while reading that belongs in the deferred ledger.

## Gate inventory (itemized, per the 2026-10-06 M28 law)

10 scoped invocations + commit cycle. Gate 9 (bench_compare) status is a
declared deviation - see the Deviations section.

## Gate 1 - clean baseline BEFORE any edit

`python tools/testrun.py --distill -p ariadne-lang-cpp` - exit 0:

```
201 passed / 0 failed / 0 ignored / 2 suites
```

Capture: `target/testrun-capture.txt`. Census reconciliation baseline:
this lang-cpp line plus the anchor of record
`1830 passed / 0 failed / 13 ignored / 119 suites @ 765df7b`.

## Red capture (all three files authored before the hook existed)

Gate 2 - `python tools/testrun.py --distill -p ariadne-lang-cpp` - exit 1:

```
201 passed / 4 failed / 0 ignored / 1 suites
```

Exactly the 4 named unit tests, each panicking with the expect message
"C++ always has directive grammar" (the trait default returns `None`), as
the handoff predicted:

- `consumer_rewrite_spec_maps_bare_name_paths_identity`
- `consumer_rewrite_spec_maps_paths_to_bare_file_names`
- `consumer_rewrite_spec_fallback_keeps_display_string_without_file_name`
- `discovery_hit_iff_rewrite_changes_text_over_the_spelling_classes`

Note: the distiller reports `1 suites` at FAIL because cargo skips the
doctest suite when the unit-test binary fails; the lang-cpp suite count
returns to 2 at Green (201 pre-existing + 4 new = 205).

Gate 3 - `python tools/testrun.py --distill -p ariadne-integration-tests
--test m20_apply_migrate_characterization_test --test
m19_optimize_characterization_test` - exit 1:

```
4 passed / 1 failed / 0 ignored / 1 suites
```

The amended m19 test failed for the documented reason (pre-hook optimize
is vacuous; the file still reads the bare include). Verbatim first-assert
panic line: `assertion 'left == right' failed: extraction-consumer-rewrite:
the committed optimize rewrites the consumer's bare include into the four
extracted includes in place`, left = `#include "thing.h"...` (verbatim
capture at `target/testrun-capture.txt`).

Gate 3a (supplement, itemized) - cargo fail-fast ran the m19 binary first
and stopped before the m20 binary, so gate 3 alone could not surface the
m20 panic. Single-test capture: `python tools/testrun.py --distill -p
ariadne-integration-tests --test m20_apply_migrate_characterization_test` -
exit 1:

```
5 passed / 1 failed / 0 ignored / 1 suites
```

The amended m20 test failed the include-flip equality for the documented
reason; verbatim first-assert message: `assertion 'left == right' failed:
extraction-consumer-rewrite: apply rewrites tests/consumer.cpp - the bare
old include is replaced in place by the four extracted includes`, left
(`tests/consumer.cpp` on disk) = `#include "thing.h"...`. No assertion
was weakened anywhere for this capture; the on-disk CRLF is exactly why
the amendments use replace-clones instead of hardcoded file bytes.

## Green

Hook + `dependency_spelling` implemented in
`crates/lang-cpp/src/dependency_rewriter.rs` (module + working tree,
pre-commit rev `2e5b86f`).

Gate 4 - `python tools/testrun.py --distill -p ariadne-lang-cpp` - exit 0:

```
205 passed / 0 failed / 0 ignored / 2 suites
```

(201 baseline + 4 new; suite count back to 2, confirming the Red note.)

Gate 5 - same command as gate 3 - exit 0:

```
11 passed / 0 failed / 0 ignored / 2 suites
```

Checkpoint: hook green; the amended riders green on the first run, which
also confirms the optimize path's `generated/`-prefixed payload maps to
the same BTreeSet-ordered block (R5 intended symmetry).

Gate 6 - frozen m10/m14/m18/m17/m23 UNEDITED - exit 0:

```
48 passed / 0 failed / 0 ignored / 5 suites
```

X2/X3 hold; the m18 parity suite (preview JSON + the `- #include
"include/thing.h"` path-space diff-text display) green with its file in
no hunk - the fix lives below the payload layer (X3).

Gate 7 - `python tools/testrun.py --distill -p ariadne-lang-scala` - exit 0:

```
257 passed / 0 failed / 0 ignored / 2 suites
```

X4/R6: `crates/lang-scala/**` appears in no diff (see zero-diff proofs).

Gate 8 - `python tools/testrun.py --distill` (full census) - exit 0:

```
1834 passed / 0 failed / 13 ignored / 119 suites
```

Reconciliation vs anchor `1830 passed / 0 failed / 13 ignored / 119
suites @ 765df7b`: +4 passed exactly (the 4 new lang-cpp unit tests);
failed, ignored, and suite counts held (the m19 rename is a rename, not a
count change; census-by-name reconciliation stays mechanical). Capture
lines proving the frozen locks green (capture
`target/testrun-capture.txt`, 3192 lines):

- line 1388: `test rewriter::tests::pin_neutral_consumer_rewrite_spec_default_is_none ... ok`
  (the pin locks the trait DEFAULT, which this hand does not change)
- line 2639: `test test_transaction_apply_and_rollback ... ok` (the
  `crates/validate` byte-parity suite; `transaction_apply_test.rs`
  appears in no diff; its bare-`old.h` payload survives because the hook
  spec equals the path fallback there - C1 unit test 1)

## Refactor

The expected-block constant is already deduped locally in each rider file
(one `expected_block` binding per file, used by both flip asserts; no
cross-file extraction - separate test binaries) - it was authored that
way, so no code changed between authoring and gates 4/5; those runs are
the refactor re-run. Names are intention-revealing
(`dependency_spelling`, `expected_block`, `discovery_hits`, the renamed
m19 test).

## Zero-diff proofs (git status at gate time; raw git denied, MCP status cited)

Changed: exactly the four touch-set files
(`crates/lang-cpp/src/dependency_rewriter.rs`, the two rider tests, this
journal) + the tolerated pre-existing `extensions/.../*.vsix` stray.
Unchanged: `Cargo.lock` (no dependency added - std::path only),
`crates/cli/tests` (standing producer proof for the timing gate),
`crates/cli/**`, `crates/transform/**`, `crates/validate/**`,
`crates/lang/**`, `crates/lang-scala/**`, `tests/fixtures/**`,
`tools/**`, every other test.

## Deviations (each with reason)

1. Gate 9 (bench_compare fresh recording) NOT run: the dispatch
   instruction for this session explicitly assigned `bench_compare` (and
   the other workspace-wide gates) to the reviewer hop. The handoff's
   gate-9 slot was instead used for the gate 3a Red supplement (cargo
   fail-fast forced a separate m20 capture). R9 fresh recording therefore
   belongs to the reviewer hop; producer zero-diff (`crates/cli/tests`
   in no diff) is recorded above so the recording stays attributable.
2. Gate 3 split into 3 + 3a: `cargo test` fail-fast ran the m19 binary
   first and stopped before the m20 binary, so the two-test command could
   not surface both panics in one capture. 3a captured m20 alone. Total
   scoped invocations unchanged at 10 (1,2,3,3a,4,5,6,7,8,10).
3. Census used the `--distill` substitute per the handoff (the
   `--baseline` FILE form stays superseded); the workspace-wide
   `--baseline` census, full tools discover, and `ui_smoke` were not run
   per dispatch (reviewer hop).
4. encoding_guard form used: see the commit-cycle section below.

## Gate inventory (itemized) + commit cycle

| Gate | Command | Exit |
|---|---|---|
| 1 | testrun --distill -p ariadne-lang-cpp (baseline) | 0 |
| 2 | testrun --distill -p ariadne-lang-cpp (Red) | 1 |
| 3 | testrun --distill -p ariadne-integration-tests --test m20... --test m19... (Red) | 1 |
| 3a | testrun --distill -p ariadne-integration-tests --test m20... (Red supplement) | 1 |
| 4 | testrun --distill -p ariadne-lang-cpp (Green) | 0 |
| 5 | testrun (same as 3, Green) | 0 |
| 6 | testrun -p ariadne-integration-tests --test m10/m14/m18/m17/m23 | 0 |
| 7 | testrun --distill -p ariadne-lang-scala | 0 |
| 8 | testrun --distill (full census) | 0 |
| 9 | bench_compare - NOT run (deviation 1; reviewer hop) | n/a |
| 10 | hygiene_check.py | 0 |
| commit | encoding_guard + commit_scoped | see below |

`python tools/hygiene_check.py` exit 0: R1/R2 silent; only pre-existing
R3 warn-only rows in files this hand did not touch (flips are governed
by dated ledger rows; none of my files appears).


