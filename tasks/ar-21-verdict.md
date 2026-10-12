# AR-21 Verdict - Dual resolver -> gated test oracle (A5 MED)

Reviewer session ses_ed4bdfc9cffeBJHlaBVbLNXEaZ. Contract `tasks/ar-21-handoff.md`,
journal `tasks/ar-21-handoff-journal.md`. Commits reviewed: 7797edb (journal
baseline), 3e874ab (atomic landing), 9fa9a1c (journal addendum). HEAD at review:
b7136db (foreign AR-25 landing above the review range). Gate evidence with all
verbatim lines: `tasks/ar-21-execution-results.md`.

## VERDICT: APPROVED

No blockers. All implementer claims independently reproduced from own stdout.

## Per-commit review (oldest-first)

- **7797edb** - adds `tasks/ar-21-handoff-journal.md` only (in touch-set).
  Mechanism ruling + D1 scope-gap pre-disclosed pre-code: correct discipline.
- **3e874ab** - 5 paths: `crates/lang-cpp/src/resolver.rs`,
  `crates/lang-cpp/src/lib.rs`, `crates/lang-cpp/Cargo.toml` (all in touch-set),
  `tasks/ar-21-handoff-journal.md` (in touch-set), and
  `tests/integration/Cargo.toml` (OUT of touch-set = declared deviation D1,
  adjudicated below). Cargo.lock untouched. No dependency edge added (feature
  only; m17 cone intact).
- **9fa9a1c** - journal addendum only (in touch-set). Records landing hashes.

## Commit-vs-touch-set

`git diff --name-status` per commit: exactly ONE out-of-set path across the
whole review range - `tests/integration/Cargo.toml` (D1), as declared. The
touch-set-permitted oracle wiring site (`resolver_parity.rs`) is ZERO-edited:
git diff over 7797edb..9fa9a1c for `tests/integration/tests` shows only the
FOREIGN `m24_language_identity_test.rs` (touched by foreign AR-25 commit
2f3b5e7, not an AR-21 commit). m18/m00/m17 oracle files byte-identical across
the range.

## Central adjudication 1 - RED form admissibility (compile-failure RED)

ADMITTED. This handoff gates a surface out of a build; there is no behavior
delta to assert red - the invariant IS "the off-state does not compile the
legacy resolver", which is a build-gate property, so an off-state build
failure is the correct red. The forcing proof is mechanically sound and I
verified its premises independently:

1. The only workspace consumers of the legacy symbols are the 4 oracle sites
   (`resolver_parity.rs:215/:290`, `external_boundary_parity.rs:228/:399` -
   grep-verified + my own compile capture places warnings at exactly those
   coordinates) and lang-cpp's own `#[cfg(test)]` unit tests (which compile
   WITH `cfg(test)`, so the gate's `any(test, ...)` arm never leaves them
   stranded).
2. A Cargo feature cannot be enabled from test source - only from a manifest
   edge. Integration tests are external consumers compiled WITHOUT
   `cfg(test)`, so the `#[cfg(test)]`-only alternative genuinely cannot serve
   the oracle (journal ruling 1 confirmed structurally).
3. Therefore wiring absent + gating present => E0433 at exactly those 4
   sites => the `testrun` DRIFT (exit 2) is the forced consequence, proving
   D1 wiring is forced, not chosen.

The RED trail itself was transient (never committable by design); its claims
re-derive from the committed state as above. No re-run attempted (would
require mutating the wired manifest - outside reviewer grant).

## Central adjudication 2 - D1 (tests/integration/Cargo.toml, OUT of touch-set)

ACCEPTED on merits. Verified:
- Single functional line: `features = ["test-resolver"]` appended to the
  PRE-EXISTING `ariadne-lang-cpp` dev-dep edge (lines 31-36 comment + line 36);
  the rest is explanatory comment. `[dev-dependencies]` section of a tests-only
  package - production manifests zero-diff (only lang-cpp's own manifest
  changes, adding `[features] test-resolver = []` with no `default` key).
- `cargo tree -e features -i ariadne-lang-cpp` (own run): the feature
  `test-resolver` is enabled ONLY under `[dev-dependencies] ->
  ariadne-integration-tests`; every production edge (pipeline, clang-worker,
  cli, tool) reaches lang-cpp through `feature "default"` only. Structural
  proof the production graph never gets the feature.
- Consequences confirmed: `resolver_parity.rs` zero edits (the touch-set's
  FEATURE-WIRING-ONLY allowance is left unused, stricter than required);
  default `cargo test --workspace` compiles the m18 binary (my m18 run passed
  on default feature flags with no manual feature flags); production consumers
  never enable it (grep: the string `test-resolver` appears in exactly one
  manifest edge, plus source cfg-gates and docs).
- Precedents (AR-03 D1 mod-declaration line; AR-24 D2 forced signature lines)
  are on point: mechanically forced wiring, tests-only blast radius,
  pre-disclosed in the journal baseline commit BEFORE the code landed.

## Central adjudication 3 - D2 (#[allow(deprecated)] confinement)

ACCEPTED. `#[allow(deprecated)]` appears at exactly three sites, all
self-use/implementation-level of the gated surface: `find_in_discovery`
(resolver.rs:140, internal call into gated `normalize_path`), the
`IncludeResolver` impl block (resolver.rs:174, self-reference), and the
lib.rs re-export (lib.rs:42, deliberate re-export). Consumer sides are NOT
suppressed - verified from my own compile captures: 4 dated warnings at the
oracle sites (`resolver_parity.rs:215/:290`,
`external_boundary_parity.rs:228/:399`) and 10 dated warnings at the
lang-cpp unit-test use sites (`resolver.rs:400/406/412/418/424/430/497/587/654/858`,
target `ariadne-lang-cpp (lib test) generated 10 warnings`). The dated
removal clock is audible for every consumer; feature-on lib build emits zero
warnings (own check, exit 0, `0 warnings emitted`).

## Central adjudication 4 - deprecation surface + plan-literal checks

- `#[deprecated]` note format per contract: both gated surfaces carry
  "remove when the m18_cpp_plugin_parity oracles retire - dated 2026-10-11"
  (resolver.rs:76-78 free fn; resolver.rs:163-165 struct). These two
  attributes are the ONLY deprecation-claiming surface in the crate.
- Gating is item-by-item per the journal's landed shape: gated imports
  (resolver.rs:56-72), `normalize_path` (:79), `collect_include_candidates`
  (:86), `find_in_discovery` (:139), struct (:166), impl (:173), lib re-export
  (lib.rs:41). Live `CppDependencyResolver` keeps only ungated imports. No
  retained live dual representation in the production build (see production
  claim below). Module doc intra-doc link plain-coded to avoid
  unresolved-link warning in feature-off doc builds - in-set doc edit, sound.
- Contract literal "153/0/0/2": STALE at dispatch (AR-24 landed +2 kind tests
  before AR-21). The review brief re-briefs the anchor to 155; I measured
  `155 passed / 0 failed / 0 ignored / 2 suites` byte-equal in BOTH
  configurations (default and `--features test-resolver`) - count-parity
  across configs held. Compliant; not an implementer finding.

## Production compiles-it-out claim (load-bearing) - VERIFIED

Three independent confirmations, all own-stdout:
1. `cargo check -p ariadne-pipeline` -> `ok (exit 0, 0 warnings emitted)` -
   production graph (pipeline -> lang-cpp, feature-off, no cfg(test)) zero
   warnings; no deprecation surface reachable. (clang-worker/tool: zero
   `IncludeResolver` references grep-verified in `crates/`+`tools/` outside
   lang-cpp internals and the m18 oracles; `cargo check --workspace
   --all-targets` exit 0 with no deprecation warning from ANY production
   target.)
2. `cargo tree -e features -i` attribution (D1 section): feature enabled only
   via the dev-dep edge.
3. Source-level: every legacy item behind
   `#[cfg(any(test, feature = "test-resolver"))]` - the code is not compiled,
   not merely hidden.

## Hygiene / encoding / report-evidence

- `hygiene_check.py` exit 0, ZERO blocking rows (every printed row R3 warn;
  `resolver.rs` absent from findings; new-regime "zero blocking" pinned, no
  R3 count cited).
- `encoding_guard.py --all` exit 0, silent.
- Journal report-evidence is digest-shaped: distilling entry points throughout,
  digest lines + exit codes + capture paths, zero raw dumps; own-stdout quoting
  per the capture-race rule. Foreign-tree handling (report.json CRLF churn,
  m24 foreign WIP) correct - left unstaged, excluded from scopes.

## Numbered findings (all minor; no rework required)

1. (minor) Stale oracle doc comments: `m18_cpp_plugin_parity_test.rs:11` ("remains
   public and non-deprecated") and `m18_cpp_plugin_parity/resolver_parity.rs:270`
   ("public, non-deprecated after ...") are now false wording - files are
   uneditable under the contract (FEATURE-WIRING-ONLY forbids doc edits), so
   this is contract-forced, correctly disclosed and banked in the journal's
   "Oracle doc-comment note". DISPOSITION: record as a deferred-findings row
   in `tasks/milestone_ledger.md` for the oracle-retirement handoff (the dated
   `#[deprecated]` notes already point there). Orchestrator ledger write -
   recorded here so it is not dropped.
2. (informational) Contract gate literal 153/0/0/2 stale at dispatch vs the
   re-anchored 155 (AR-24 +2); closeout should treat 155 as the AR-21-era
   lang-cpp anchor and update the contract wording if it is ever re-cited.

## Deviation dispositions

- D1 (tests/integration/Cargo.toml feature wiring, outside literal touch-set):
  ACCEPTED - mechanically forced, tests-only blast radius, pre-disclosed,
  precedents consistent.
- D2 (#[allow(deprecated)] on self-use/impl sites, in touch-set): ACCEPTED -
  confinement independently verified; consumer clock live.
- Mechanism choice (feature route over cfg(test)/re-export-only/build-cfg
  alternatives): ACCEPTED - rejected alternatives shown machine-unsatisfiable
  against the contract invariants.

## Gates NOT run (by orchestrator constraint, recorded not hidden)

Workspace-wide acceptance gates (testrun --baseline per errata, tools unittest
discover, ui_smoke, bench_compare) NOT run: AR-25 review lane live in this
shared worktree; brief restricts to scoped gates; AR-21 touches zero
tools/, zero UI, zero hot-path surface (all three commits path-listed above).
Census anchor 1393/0/13/102 not re-measured for the same reason; the
orchestrator's fresh-session workspace run owns it.

## Refused-command record

One command refused: `git show "3e874ab" -- crates/lang-cpp/src/resolver.rs |
Select-Object -Skip 130` -> boilerplate
`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`
(pipe/Select-Object variant, not a gate denial). Worked around by reading the
working-tree file (path byte-clean vs HEAD per checkpoint_guard clean tree).
All other gates ran; none refused.
