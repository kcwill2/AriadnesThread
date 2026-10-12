# AR-26a Verdict - APPROVED

Handoff: `tasks/ar-26a-handoff.md` (chain hop 1/3, ruling board_12d48349
option A + riders). Landing commit `4794467`, journal `4794467` +
`5b7c53d`/`2fb9b21`/`075e3bf` (journal-only). Full gate evidence:
`tasks/ar-26a-execution-results.md`.

**APPROVED.** Land stands; AR-26a2 may dispatch only after finding 1 is
folded into its contract text (or its dispatch quotes the journal RIDER).

## Central adjudications (explicit verdicts)

1. **D2 widening scope - VERIFIED EXACTLY ONE HUNK.** Independent blob diff
   `c8ae8ea:crates/report/src/manifests.rs` vs
   `HEAD:crates/artifact/src/manifests.rs` = one hunk, one line:
   `pub(crate) fn collect_stage_languages` -> `pub fn ...` at file line 22.
   A 1-line blob diff is exhaustive - no other item's visibility changed,
   nothing else differs. `git show --name-status -M 4794467` -> R099;
   `--stat -M` -> `crates/{report => artifact}/src/manifests.rs | 2 +-`
   reproduced independently.
2. **D3 exceeds ruling letter - NECESSITY CONFIRMED, ACCEPTED on merits.**
   Proven without touching the tree: `--force-warn=hidden_glob_reexports
   --force-warn=unused_imports` via `cargo --config build.rustflags`
   (force-warn beats any allow) against the committed code emitted exactly
   2 warnings, both at `crates/report/src/lib.rs:16:16` - the rider line
   itself (shadowing of the glob re-export at :8:9 + the binding referenced
   only as that mechanism). Without the allow pair the hop adds 2 diff-
   attributable warnings, breaking zero-warning gate 1; with it, zero. The
   mandated binding is byte-exact as ruled. The journal RIDER section
   ENUMERATES all three components (rider line + allow pair + comment) for
   26a2 removal, alongside item 1's `pub(crate)` revert - no finding for the
   journal; see finding 1 for the 26a2 contract text.
3. **Mutation trail - REVERT COMPLETE.** Tree byte-clean vs HEAD
   (checkpoint_guard numstat "(none)"); committed lib.rs carries both shim
   `pub use` lines active; Glob census: exactly one `manifests.rs` on disk
   (artifact). The 9-site E0432 breadth is mechanically consistent:
   commenting both shim lines kills the crate-root `manifests` binding, so
   all 9 `use crate::manifests::{...}` sites fail - wider than the 3
   fn-importers, as journalled.
4. **Report conservation - VERIFIED.** Own run `47 passed / 0 failed /
   0 ignored / 4 suites`; decomposition 40 (lib) + 6
   (bincode_bounded_decode) + 1 (extraction_artifacts) + 0 (Doc-tests) = 47
   reproduced from own capture. `discovery_io.rs:13`, `metrics_io.rs:13`,
   `graph_io.rs:13` byte-UNEDITED - all of `manager/**`, `manager.rs`,
   `report/tests/**`, `tests/**` inside the EMPTY frozen diff
   (`git diff c8ae8ea HEAD -- <frozen paths>` -> no output, rev-vs-rev;
   the foreign report.json CRLF is worktree-side only, tree now clean).
5. **Artifact hop-shape - VERIFIED.** `-p ariadne-artifact` =
   `0 passed / 0 failed / 0 ignored / 2 suites` (contract expectation);
   m00 pair `4 passed / 0 failed / 0 ignored / 2 suites` UNEDITED;
   m23 `7 passed / 0 failed / 0 ignored / 1 suites` = dispatch-HEAD
   baseline UNEDITED.

## Deviation dispositions

- **D1** (temp-dir probe instead of worktree mutation at blocker phase):
  ACCEPTED - same evidence class, zero blast radius; already ruled through
  (option A arose from it).
- **D2** (single visibility hunk, R099 not R100): ACCEPTED - pre-approved by
  ruling rider 1; independently verified per adjudication 1 (exactly the
  ruled one-line scope).
- **D3** (allow pair + rationale comment on the rider line): ACCEPTED - the
  only deviation exceeding the ruling's letter; necessity independently
  confirmed (adjudication 2); terminal-state cleanup fully enumerated in the
  journal RIDER.
- **D4** (`mkdir crates/artifact/src` via one-shot distiller python script):
  ACCEPTED - allowlist plumbing substitution; `git mv` needs the destination
  dir; contract mandates the dir.

## Findings

1. **(minor - follow-up disposition, blocks 26a2 DISPATCH not this land)**
   `tasks/ar-26a2-handoff.md` (planner-authored @ `957c391`, PREDATES the
   ruling) does not carry ruling rider 3: grep finds ZERO mentions of
   `collect_stage_languages`, the rider line, or the `allow` pair in it, and
   its touch-set says `crates/report/src/manifests.rs: UNTOUCHED` (a path
   that no longer exists) with no line for re-privatizing
   `crates/artifact/src/manifests.rs:22`. Its final-lib.rs pin happens to
   drop the rider lines, but the `pub` -> `pub(crate)` revert would land
   nowhere, permanently violating terminal-surface = 12 DTOs. The 26a
   journal RIDER (the ruling's banked mandate, and complete per adjudication
   2) must be folded into the 26a2 handoff text - definition revert +
   enumerated drop of rider line + allow pair + comment + touch-set line for
   `crates/artifact/src/manifests.rs` - BEFORE 26a2 dispatchs. Ledger row
   candidate; owner: orchestrator/planner, not this implementer.
2. **(minor - observation, superseded)** Handoff baseline "workspace census
   projection at current HEAD `1397/0/13/103`" is stale by one test
   (omits AR-31's +1): ground truth pre-hop was `1398/0/13/103`. Replaced by
   the new anchor row `1398/0/13/105` (see execution results).
3. **(informational - configuration, not the implementer)** Reviewer bash
   refusals recorded verbatim in execution results: one compound `git diff`
   chain, `git ls-files`, and `git ls-tree` hit the catch-all deny; all
   substituted (plain per-path `git diff`, Glob census + R099 rename status
   + clean tree for the D7 stale-copy proof). No gate skipped.

## Acceptance pass - canonical lines (own stdout)

- compile-green gate 1: `ok (exit 0, 31 warnings emitted)`; zero warning
  attribution to `ariadne-report`/`ariadne-artifact` in own capture.
- `-p ariadne-artifact`: `0 passed / 0 failed / 0 ignored / 2 suites`
- `-p ariadne-report`: `47 passed / 0 failed / 0 ignored / 4 suites`
- m00 pair: `4 passed / 0 failed / 0 ignored / 2 suites`
- m23: `7 passed / 0 failed / 0 ignored / 1 suites`
- hygiene: R3 warn rows only, ZERO blocking, `crates/artifact` absent
- encoding: `encoding_guard.py --all` silent (exit 0)
- tools suites: `Ran 219 tests in 20.633s` / `OK`
- CENSUS (new anchor): `1398 passed / 0 failed / 13 ignored / 105 suites`
  reconciled: 1393/0/13/102 @AR-24 -> AR-25 +4/+1 -> AR-31 +1 -> AR-26a
  +0 tests/+2 suites = measured. Fully attributed.
- NOT run: `--baseline` (errata, file-mode/absent), `ui_smoke`,
  `bench_compare` (out of contract; closeout duty).

## Rename-completeness (D7)

`R099` old->new in the single landing commit; deletion+addition together (no
stray-deletion follow-up needed); no stale tracked/on-disk copy at the old
path; all paths inside the touch-set. Contract's supersession of the R100
expectation (by ruling) honored.
