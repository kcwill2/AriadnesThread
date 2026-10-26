# R-1 Handoff - rust-language: spine, part repo (FileKind::RustSource + LanguageId::Rust, append-only)

Milestone: `rust-language` (UNREGISTERED post-M28 track, NO M-number). Hand 1
of 10, strictly serial: R-1 -> R-2 -> R-3 -> R-4 -> R-5 -> R-6 -> R-7 -> R-8 ->
R-9 -> R-10. R-1 edits ONLY `crates/repo`: the `FileKind` table (the one spine
edit the shape sanctions by design - shape sec. 1) and the `LanguageId`
identity consts. Start rev: your dispatch head - journal it verbatim; every
reviewer diff is against it. Author rev: the planning commit carrying
`tasks/rust-language-plan.md` + `tasks/rust-language-r{1..10}-handoff.md` on
branch `balsam-basil` (above `5aa191e`; census anchor `1853 passed / 0 failed /
13 ignored / 119 suites @ 522a770`, docs hops since keep it).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/rust-language-shape.md` sec. 2 INV-12 (enum append-only), sec. 3.1
   (the exact spine contract, verbatim literals), sec. 7 (hygiene). Kickoff
   locks L1/L11; amendment A5 (amended here, completed - see below).
3. Your files, current state: `crates/repo/src/types.rs` (304 lines; enum :32-42,
   `from_path` :45-59, `is_analyzable` :66-71, matrix pin :86-119 - rows :103
   `("a.rs", FileKind::Other)` and :114 `("a.h.rs", FileKind::Other)`, case
   matrix :125-148) and `crates/repo/src/identity.rs` (401 lines; module consts
   :55-61, associated consts :83-98, snake helpers `cpp()`/`scala()` :115-122,
   `from_file_kind` :133-141 - the ONLY exhaustive `FileKind` match in the
   workspace, planner-verified).
4. The frozen neighborhood you must not perturb: `WireLanguageForm` (:149-175),
   `language_transparent_frame` (:210-231), `legacy_rank` (:235-242) - Rust
   rides the existing tag-3 Open escape and rank 3 with NO edit there (L1).
5. `python tools/checkpoint_guard.py status` - FIRST HAND OF THE MILESTONE:
   run it as your bootstrap before any mutation. Expected: clean tree.
   Anything unexpected is a STOP-and-report.

## Duty 1
None for this hand (the planning docs land in the orchestrator's commit
beneath your dispatch head). Your bootstrap (binding input 5) stands in.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/rust-language-r1-journal.md (checkpoint journal; create it)
- crates/repo/src/types.rs (enum append + table row + predicate + matrix pins)
- crates/repo/src/identity.rs (RUST const + `Rust` const + `rust()` helper +
  `from_file_kind` arm + one new pin test)

You MUST NOT touch: `crates/lang-api/**` (R-2's crate), `crates/lang-rust/**`
(does not exist yet), `crates/cli/**`, `crates/pipeline/**`, any other file in
`crates/repo`, the frozen trio named in binding input 4, any test outside the
two matrix tests named above, `tests/**`, `tools/**`. No `git stash`, no
branch change. Cargo.lock: no new dependencies (std only); if cargo rewrites
it, revert.

## What to build (shape sec. 3.1 - the contract)

1. `types.rs` enum: APPEND `RustSource` as the LAST variant, after `Other`
   (INV-12: no pre-existing variant's serde index moves):
   `pub enum FileKind { Header, Source, ScalaSource, ScalaScript, BuildDefinition, Other, RustSource }`
2. `from_path` table gains, matching the arms' style: `Some("rs") => FileKind::RustSource,`
   (the existing match already lowercases the extension - case folding is
   inherited, no extra code).
3. `is_analyzable()` gains `FileKind::RustSource` in its `matches!` set.
   `is_source_or_header()` stays byte-identical (Scala is not in it either -
   RustSource follows the Scala precedent, no edit).
4. `identity.rs`: add module-level `pub const RUST: &str = "Rust";` (after
   `OTHER`, same doc style), associated `pub const RUST: &'static str = RUST;`,
   `#[allow(non_upper_case_globals)] pub const Rust: Self = Self(Cow::Borrowed(RUST));`,
   and guard-consistent snake helper `pub fn rust() -> Self { Self::Rust }`
   (mirrors `cpp()`/`scala()` and their doc idiom).
5. `from_file_kind` gains the arm `FileKind::RustSource => Self::Rust,` - the
   exhaustive match compiles by this same edit.
6. Amendment A5 (kickoff sec. 5) in the matrix pin
   `pin_file_kind_lowercase_extension_matrix`: row :103 becomes
   `("a.rs", FileKind::RustSource)` AND row :114 becomes
   `("a.h.rs", FileKind::RustSource)` - PLANNER FINDING: `Path::extension()`
   for `a.h.rs` is `rs`, so :114 flips too; the shape's sec. 3.1 A5 inventory
   named only :103. Add to the case matrix
   `pin_file_kind_case_matrix_is_folded_insensitive` the row
   `("a.RS", FileKind::RustSource)` (precedent rows prove folding).
7. One new in-src pin test `rust_source_kind_carries_the_rust_identity`:
   `FileKind::from_path("a.rs") == FileKind::RustSource`,
   `FileKind::RustSource.is_analyzable()`, and
   `LanguageId::from_file_kind(FileKind::RustSource) == LanguageId::Rust`,
   message naming INV-12/INV-9.

## RGB protocol

1. Red: FIRST edit only the matrix pin :103 row to a variant-free red
   assertion (you cannot name `RustSource` before it exists - a
   compile-failing test is NOT a Red): replace row :103 with a test-body
   assertion `assert_ne!(FileKind::from_path(Path::new("a.rs")), FileKind::Other, "R-1 red: .rs must not stay Other once the spine names it")`
   and capture the failure. Expected failure line: `assertion failed:
   \`(left != right)\``, `left: Other`, `right: Other` (from
   `pin_file_kind_lowercase_extension_matrix`). Journal the red capture.
2. Green: items 1-7 above (the red assertion is REPLACED by the honest
   `("a.rs", FileKind::RustSource)` row - never weakened).
3. Refactor: doc wording plain, honest names; re-run.

## Hygiene (review-enforced)
`types.rs` 304 -> ~330, `identity.rs` 401 -> ~440 (both << 1000); functions
< 100; no banned names (`tmp`/`res`/`foo`/`bar`/`test1`), no single-letter
locals; authored bytes pure ASCII. `python tools/hygiene_check.py` exit 0.

## Frozen locks (commands that prove them - kickoff sec. 5)
- L1/L11: `git diff <start>..HEAD -- crates/repo/src/identity.rs` shows ONLY
  the `RUST` const, associated `RUST`, `Rust` const, `rust()` fn,
  `from_file_kind` arm, and the appended pin test; `WireLanguageForm`,
  `language_transparent_frame`, `legacy_rank` in NO hunk.
- L2/L4/L7/L8/L9: `git diff --name-only <start>..HEAD` outside the touch-set
  is EMPTY (this alone proves goldens, m00, lang-cpp, lang-scala, tools clean;
  their suites green via gate 4's full census UNEDITED).
- L12: `python tools/encoding_guard.py --all` exit 0.

## Gates (exact commands, timeout 600000 ms each; distilling entry points only;
raw cargo is a reviewer blocker)
1. `python tools/testrun.py --distill -p ariadne-repo` - clean baseline BEFORE
   any edit; journal the canonical line verbatim.
2. `python tools/testrun.py --distill -p ariadne-repo` - Red: exit non-zero,
   the ONE expected failure named above.
3. `python tools/testrun.py --distill -p ariadne-repo` - Green: exit 0,
   passed = baseline + 1 (the new pin), all pre-existing repo tests UNEDITED
   except the three matrix rows/hunks of item 6.
4. `python tools/testrun.py --distill -p ariadne-cli` - proves amendments
   A1-A4 stay GREEN UNEDITED at this hop (no registration yet; the enum and
   identity edits must not turn any cli test red - INV-2 first-wins unchanged,
   the plugin set is untouched).
5. `python tools/testrun.py --distill` - full workspace census; journal the
   canonical line and reconcile: expect `1854..1858 passed / 0 failed / 13
   ignored / 119 suites` (delta +1..+5 attributed to the item-7 pin; suites
   flat; anchor `1853 passed / 0 failed / 13 ignored / 119 suites @ 522a770`;
   the `--baseline` FILE form stays superseded - F9 errata, `--distill` governs).
6. `python tools/hygiene_check.py` - exit 0.
7. Commit cycle: `python tools/encoding_guard.py --staged` (or `--all`
   superset if the staging seam refuses - journal the form) then
   `python tools/commit_scoped.py --scope crates/repo/src/types.rs --scope
   crates/repo/src/identity.rs --scope tasks/rust-language-r1-journal.md -m
   "feat(repo): append RustSource to FileKind and Rust to LanguageId
   (rust-language R-1 spine)"`. Never `git add .`; the journal rides inside
   the landing commit.

Declared gate budget: 6 scoped invocations + the commit cycle - inside
`<= ~10 + commit-cycle trio` (2026-10-06 M28 law); itemize per-gate counts in
the journal.

## Discipline clauses
- UTF-8: never Set-Content/Out-File/`>`/`Get-Content | -replace`; Write/Edit
  tools only; authored content pure ASCII (AGENTS.md File-Edit Discipline).
- Touch only the touch-set; out-of-scope breakage: report, do not fix.
- After EACH Green gate append a 1-3 line checkpoint to the journal.
- Context exhaustion looming: STOP, return a disk-state inventory; a dead
  session is never resumed - successor runs `python tools/checkpoint_guard.py
  resume --spec tasks/rust-language-r1-handoff.md --scope crates/repo --cmd ...`.
- Landing SHA relay: report the landing SHA; `git reflog -n 3` must show it at
  the tip; R-2's duty 1 cites this file + that SHA.
- Never pin warning counts; never state a count without the command that
  produces it (M23.5 F14 standing rules).

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence (gate 2 canonical line +
expected failure name). 3. Green evidence: distill digest header + canonical
line for gates 3-5 + capture path. 4. Any deviation + reason. 5. Out-of-scope
observations (report-only). Never paste raw tool output; cite command +
canonical line + exit code.

## Reviewer acceptance pass (reviewer's, NOT the implementer's gates)
- `python tools/distill.py -- python tools/bench_compare.py` - RUN fresh: this
  hand touches `crates/repo/src` bytes (kickoff sec. 4 fully-run stance,
  f20ad35 onward). WARNs recorded, NOT attributed (PR-2 persists).
- Per-commit `git show` oldest-first vs touch-set; the identity.rs hunk-scope
  proof (locks above); census delta attribution; L11 pure-append check
  (`RustSource` last variant - diff shows no reordered line).

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces this touch-set. Extra MUSTs:
matrix rows :103 and :114 both flipped in THIS commit (a :114 miss is a
blocker, not a nit - the reviewer re-runs the matrix green as proof); the
frozen trio in no hunk; cli green UNEDITED.

Sizing (planner, itemized): enum +2, table +1, predicate +2, consts +26,
helper +8, arm +1, matrix edits ~6/-2, new pin ~14, journal ~12 - total
~60-110 produced lines vs the ~1200 bound; 6 gates + commit trio vs ~10 +
trio; one crate dir (`crates/repo`). No split: the enum and the identity arm
are one compile-coupled append (the exhaustive match breaks otherwise) - any
seam would ship a half-built spine.
