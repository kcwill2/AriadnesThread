# R-1 Verdict - rust-language spine (FileKind::RustSource + LanguageId::Rust)

Verdict: APPROVED

Handoff of record: `tasks/rust-language-r1-handoff.md`. Landing commit:
`e862acf` (full `e862acfac8e2541d320e5a9c38cd1f81755b5caf`). Start rev:
`bc0c11c`. Reviewed fresh on 2026-10-26; the implementer relay was treated as
claims, every measurable item re-run.

## Ground truth (before judging)

- `python tools/checkpoint_guard.py status`: HEAD `e862acf`, branch
  `balsam-basil`, clean tree (no dirty, no untracked). Worktree census shows
  this worktree dirty=0.
- `git log --oneline bc0c11c..HEAD`: exactly ONE commit, `e862acf`, subject
  matching the handoff's mandated commit message verbatim.

## Per-commit review (oldest-first; the single commit)

`git show --stat "e862acf"`:
`crates/repo/src/identity.rs | 36 +++`, `crates/repo/src/types.rs | 10 ++--`,
`tasks/rust-language-r1-journal.md | 97 +++` - 140 insertions, 3 deletions.

`git show "e862acf" -- crates/repo/src/types.rs` - verified line by line:

- Enum: pure APPEND of `RustSource` after `Other` (hunk adds 2 lines, deletes
  0; no reordered line). L11 pure-append holds; all pre-existing serde indices
  unchanged (INV-12). Final file order matches the contract literal verbatim:
  `Header, Source, ScalaSource, ScalaScript, BuildDefinition, Other,
  RustSource` (types.rs:32-44).
- `from_path`: `Some("rs") => FileKind::RustSource,` verbatim; case folding
  inherited from the existing `.to_lowercase()` (types.rs:51) - no extra code,
  as contracted.
- `is_analyzable`: `FileKind::RustSource` added to the `matches!` set.
- `is_source_or_header` NOT in any hunk - byte-identical (Scala precedent
  honored; types.rs:64-66 shows only `Header | Source`).
- Matrix pin: row :103 `("a.rs", FileKind::RustSource),` AND row :114
  `("a.h.rs", FileKind::RustSource),` - BOTH flipped in THIS commit (final
  file lines 106 and 117). Reviewer re-ran the matrix green as proof (below).
- Case matrix: `("a.RS", FileKind::RustSource),` added to the
  folded-insensitive pin (final line 147).
- No red-assertion remnant: no `assert_ne` survives in types.rs (workspace
  grep); the honest row replaced the red form, never weakened.

`git show "e862acf" -- crates/repo/src/identity.rs` - hunks cover old lines
59-61, 85-90, 96-101, 121-126, 136-141, 398-401 ONLY. That is exactly the six
allowed additions and nothing else:

1. module `pub const RUST: &str = "Rust";` after `OTHER`, same doc style;
2. associated `pub const RUST: &'static str = RUST;`;
3. `#[allow(non_upper_case_globals)] pub const Rust: Self =
   Self(Cow::Borrowed(RUST));`;
4. `pub fn rust() -> Self` mirroring the `cpp()`/`scala()` doc idiom;
5. `from_file_kind` arm `FileKind::RustSource => Self::Rust,` (the exhaustive
   match compiles by this edit alone - workspace grep confirms
   `from_file_kind` is the only exhaustive `FileKind` match touched);
6. appended pin test `rust_source_kind_carries_the_rust_identity` with the
   three contracted assertions and messages naming INV-12/INV-9.

Frozen trio L1 proof: `WireLanguageForm` (old :149-175),
`language_transparent_frame` (old :210-231), `legacy_rank` (old :235-242) fall
strictly OUTSIDE every hunk range above - byte-identical, no edit. Rust rides
tag-3 Open + rank 3 with zero edit there, as contracted.

## Commit-vs-touch-set

`git diff --name-only bc0c11c..HEAD` returns exactly:

- `crates/repo/src/identity.rs`
- `crates/repo/src/types.rs`
- `tasks/rust-language-r1-journal.md`

All three inside the declared touch-set; nothing outside. L2/L4/L7/L8/L9
(goldens, m00, lang-cpp, lang-scala, tools clean) proven by this empty
outside-set. `Cargo.lock` untouched. No dual representation retained: workspace
`.rs` grep for `RustSource` / `LanguageId::Rust` / `identity::RUST` hits only
the two contracted files.

## Fresh verification (reviewer's own runs, distilling entry points)

- Scoped repo (gate-equivalent of handoff gate 3, Green):
  `python tools/testrun.py --distill -p ariadne-repo` exit 0 -
  canonical line verbatim:
  `39 passed / 0 failed / 0 ignored / 5 suites`
  Consistent with the journalled baseline `38 passed / 0 failed / 0 ignored /
  5 suites` + the one new pin.
- Matrix-green MUST re-run (the :103/:114 proof):
  `python tools/testrun.py --distill -p ariadne-repo pin_file_kind` exit 0 -
  canonical line verbatim:
  `2 passed / 0 failed / 0 ignored / 4 suites`
  Both `pin_file_kind_lowercase_extension_matrix` (carries the flipped :103
  and :114 rows) and `pin_file_kind_case_matrix_is_folded_insensitive` (carries
  the `a.RS` row) green.
- cli UNEDITED (handoff gate 4): `python tools/testrun.py --distill -p
  ariadne-cli` exit 0 - canonical line verbatim:
  `182 passed / 0 failed / 8 ignored / 8 suites`
  Matches the relay exactly; zero cli paths in the commit diff, so UNEDITED is
  proven structurally, and amendments A1-A4 stay green at this hop.
- Full census (handoff gate 5): `python tools/testrun.py --distill` exit 0 -
  canonical line verbatim:
  `1854 passed / 0 failed / 13 ignored / 119 suites`
  Reconcile: anchor `1853 passed / 0 failed / 13 ignored / 119 suites @
  522a770` -> delta +1, fully attributed to
  `identity::tests::rust_source_kind_carries_the_rust_identity`; suites flat;
  ignored unchanged. Inside the expected `1854..1858` band at the +1 end.
- Hygiene (gate 6): `python tools/distill.py -- python tools/hygiene_check.py`
  -> `### generic PASS (exit 0)` (capture 173 lines). Every printed line is
  warn-only R3; NO line names `crates/repo/src/types.rs` or
  `crates/repo/src/identity.rs`. File sizes final: types.rs 308 lines,
  identity.rs 437 lines - both << 1000.
- L12 encoding: `python tools/distill.py -- python tools/encoding_guard.py
  --all` -> `### generic PASS (exit 0)`.

## Reviewer acceptance pass (this workspace-wide set)

- `python tools/distill.py -- python tools/bench_compare.py` RUN fresh:
  `### generic PASS (exit 0)`; digest carries
  `bench compare: 3 run(s), no regressions past 1.25x baseline`. Zero WARNs
  emitted this run; PR-2 stance (WARNs recorded, never attributed) unchanged
  and moot.
- `python -m unittest discover -s tools -p "*_tests.py"`:
  `Ran 296 tests in 30.434s` / `OK` - tool suites green.
- `python tools/testrun.py --baseline` (the acceptance-pass form): refused
  with `usage error: missing value for --baseline`. This is the FILE form the
  handoff names superseded (F9 errata - `--distill` governs); the governing
  `--distill` census was run fresh and is quoted above. Not a finding.
- `ui_smoke.py` not required by this hand (no crates/ui or runtime-surface
  bytes touched) - NOT run. `golden_guard.py --base`: tool not yet landed; the
  handoff's prose byte-check stands - the frozen-trio + goldens byte-identity
  is proven by the hunk-range and name-only analyses above.

## Red evidence (historical, journal-recorded)

Red is pre-landing and cannot be reproduced without an edit (forbidden to a
reviewer). Journal gate 2 records: first mutation was the matrix pin :103 row
only; `python tools/testrun.py --distill -p ariadne-repo` exit 1, canonical
`30 passed / 1 failed / 0 ignored / 1 suites`; the ONE failure named
`types::file_kind_matrix_tests::pin_file_kind_lowercase_extension_matrix` with
`assertion \`left != right\` failed`, `left: Other`, right Other at
`crates\repo\src\types.rs:87:9`; capture `target/testrun-capture.txt`. The
shape matches the handoff's expected red exactly. The Green state carries the
honest row (verified above), so the red was never laundered.

## Refusals encountered (verbatim) and substitutions

Two reviewer commands were refused by the catch-all deny rule; refusal text in
both cases was verbatim:
`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`

1. `python -c "print(sum(...))"` (line counts) - refused. Substituted: Read
   tool tail reads; line counts reported above.
2. `python tools/hygiene_check.py; echo "HYGIENE_EXIT=$LASTEXITCODE"`
   (chained exit-code probe) - refused. Substituted: `distill.py` wrapper,
   which reports `### generic PASS (exit 0)` explicitly.

Every expected gate ran; nothing was skipped on account of a refusal.

## Findings

1. MINOR - `tasks/rust-language-r1-journal.md:93-97`: the committed journal
   ends with a stray placeholder `(to be appended per Green gate)` followed by
   a truncated duplicate section header `## Gate inventory (itemized per-hand
   counts, M28 law)` and a single orphan line `- Gate 1 baseline: 1
   invocation.` - an incomplete second copy of the gate-inventory section
   already complete at lines 67-80. Documentation defect only (journal is in
   the touch-set; zero code/gate impact). Clear via a docs-only rider or as
   part of R-2 duty 1's landing commit.
2. MINOR - journal/relay consistency: journal line 79 states "Deviations:
   none." while the implementer relay declared two deviations (a) and (b).
   Adjudication on the merits (below) shows the journal is substantively
   correct and the relay over-declared; no commit change required. Recorded so
   the ledger does not inherit phantom deviations.

No blockers. No undeclared deviations found: the diff matches the contract
item-for-item, plan literals verified verbatim, and nothing outside the
touch-set was touched.

## Deviation dispositions

- Deviation (a) - "`encoding_guard.py --staged` vacuous at the staging seam;
  `--all` run post-commit as the L12 check": ACCEPTED. Handoff gate 7
  pre-authorizes exactly this fallback ("`--all` superset if the staging seam
  refuses - journal the form") and the handoff's own L12 lock is defined as
  `--all` exit 0. The journal records the form in its Lock proofs section, and
  the reviewer re-ran `--all` fresh: `### generic PASS (exit 0)`.
- Deviation (b) - "red assertion placed at test-body top with row :103 removed
  in the same edit": ACCEPTED, reclassified as NOT a deviation. The handoff's
  RGB protocol itself prescribes replacing the vec row with a test-body
  `assert_ne!` ("you cannot name `RustSource` before it exists - a
  compile-failing test is NOT a Red"); a macro cannot sit inside the
  `[("name", FileKind)]` cases vec. The journalled red (exit 1, the ONE named
  matrix failure) and the Green honest rows (:103 + :114, re-run green above)
  prove faithful execution.

## Follow-up dispositions

- Finding 1 goes to R-2 duty 1 (or a docs-only rider commit): strip the
  duplicated trailing journal section.
- Finding 2: no artifact change; noted here for the ledger row R-2 lands.
- No out-of-scope breakage was reported or found; nothing for
  `tasks/milestone_ledger.md` deferred-findings beyond the above.
