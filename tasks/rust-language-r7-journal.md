# R-7 Journal - rust-language: load_metric flip + language-is-data proofs

Milestone: `rust-language` (unregistered post-M28 track). Hand 7 of 10.
Implementer: SUCCESSOR session (dispatched 2026-10-26 after the first R-7
session died mid-hop). Landing scope per the handoff touch-set:
`crates/lang-rust/src/plugin.rs` (caps flip + pin rename),
`tests/integration/tests/rust_language_load_analysis_test.rs` (new suite),
this journal.

## SUCCESSOR-hand fact (resume-checkpoint law)

The predecessor R-7 session (`Execute R-7 load metric hop`, board
participant ses_e88217601ffeYMMeNLjY4752qj) hit its context wall with an
EMPTY compaction summary AFTER landing duty 1 (`5492372`: R-6 verdict +
ledger relay) and its board ASK post, but BEFORE its landing commit and
this journal. Per standing law the dead session was NEVER resumed; this
session bootstrapped from disk only, via
`python tools/checkpoint_guard.py status` then
`python tools/checkpoint_guard.py resume --spec tasks/rust-language-r7-handoff.md
--scope crates/lang-rust --cmd <three scoped distilling gates>`.
The resume report `tasks/rust-language-r7-handoff-resume.md` was regenerated
by the guard this session; it is a guard artifact, outside the touch-set,
deliberately NOT committed.

## Predecessor's landed scope (verified on disk, not trusted from relay)

- Duty 1 landed: commit `5492372` "review(rust-language): land R-6 verdict +
  ledger flip (R-7 duty 1)" - confirmed in `git log` via the guard status.
- Working tree in the handoff's Green state: `crates/lang-rust/src/plugin.rs`
  dirty +16/-13 (load_metric true in the impl literal AND the pin literal;
  doc-comment flip narrative; pin renamed
  `capabilities_match_the_load_metric_flip_literal`, byte-exact per the
  handoff's flip contract), untracked
  `tests/integration/tests/rust_language_load_analysis_test.rs` (5 tests,
  500 lines).
- Caps literal verified against the handoff contract:
  `discovery + semantic_index + dependency_graph + load_metric` true;
  `planning/extraction/rewriting/validation` false; `requires_toolchain:
  false` - exactly four stage bits true (plugin.rs lines 150-162).
- Nothing else dirty or untracked beyond the guard's resume report:
  `git status --short` shows exactly ` M crates/lang-rust/src/plugin.rs` +
  the two untracked files above. No stash used by this session (R-6 D6
  ruling: recurrence = blocker).

## Verification runs (this session's fresh Green - the landing basis)

Guard resume verdicts (fresh runs through the distilling entry points):
- lang-rust: green; re-run standalone: `56 passed / 0 failed / 0 ignored /
  2 suites`.
- new suite scoped: green; re-run standalone: `5 passed / 0 failed /
  0 ignored / 1 suites` (the handoff's expected 5/0).
- m22 model suite scoped UNEDITED: `6 passed / 0 failed / 0 ignored /
  1 suites` (file zero-diff per git status).
- `python tools/distill.py -- python tools/ui_smoke.py`:
  `ui_smoke: PASS` (zero-edit boot still serves routes + no-store).
- `python tools/hygiene_check.py` via distiller: `### generic PASS (exit 0)`;
  only pre-existing repo-wide R3 warns, zero R1/R2, none of this hand's
  two code files appear in the list.
- Captures: each distilling run overwrote
  `target/testrun-capture.txt` / `target/distill-capture.txt` (paths printed
  by each digest line above the fold of this journal's writing).

## Red evidence (relay basis + structural review - stated honestly)

This session did NOT re-author the Red: reproducing it would mean reverting
the plugin.rs flip in the landing tree purely to re-observe, which the
dispatch bars unless cheap and safe. Basis recorded instead:

1. Predecessor's board-relay (post board_8e711406-507c-4c58-a0bb-
   0ebba6088e90, quoted as relay evidence, marked as such): the suite ran
   RED first at `4/1` - the failing test being the caps pin, failing
   `left: load_metric: false, right: true`, exactly the handoff's predicted
   Red for the pre-hand caps.
2. This session's structural falsifiability review of the pins: the in-src
   pin (`capabilities_match_the_load_metric_flip_literal`) and the
   integration lock (`capabilities_lock_the_load_metric_flip_literal`) each
   assert against an INDEPENDENT full-struct literal with `load_metric:
   true` plus an exactly-four-true bit count; the pre-hand caps struct
   (`load_metric: false`) cannot pass either assert, so both are genuinely
   falsifiable, not tautological.
3. Independent corroborating signal from THIS session's fresh full sweep:
   the three superseded caps locks fail with `left:` showing the flipped
   struct (`load_metric: true`) - direct mechanical proof that the flipped
   value differs from the pre-hand literals, i.e. what the pre-hand tree
   would have shown mirrored.

## Full census (gate 6) and attribution

Fail-fast run: `700 passed / 1 failed / 8 ignored / 67 suites` - cargo
stopped at the first red suite (not the census). The documented seam (R-5
relay, R-6 D3) is the no-fail-fast capture + replay:
- `python tools/distill.py -- cargo test --workspace --no-fail-fast` ->
  exit 1, capture `target/distill-capture.txt` (3412 lines), the failed
  targets being EXACTLY `rust_language_dependency_graph_test`,
  `rust_language_discovery_test`, `rust_language_semantic_index_test`.
- `python tools/testrun.py --distill --replay target/distill-capture.txt` ->
  `1940 passed / 3 failed / 13 ignored / 125 suites` - byte-equal to the
  predecessor's pre-death sweep; the 3 failures are exactly the superseded
  caps locks at `rust_language_dependency_graph_test.rs:130`,
  `rust_language_discovery_test.rs:301`,
  `rust_language_semantic_index_test.rs:170`, each `left:` showing the
  flipped struct; zero collateral.
- Attribution: anchor `1938 / 0 / 13 / 124` at `470ae22` (R-6 rider), +5
  this suite's tests, -3 superseded locks flipping red = `1940 passed /
  3 failed / 13 ignored / 125 suites` exact; suites +1 = 125.
- The three red files are OUTSIDE the touch-set; report-don't-fix held.
  The expected post-rider clean line is `1943 passed / 0 failed /
  13 ignored / 125 suites`.

## R-7 ASK + orchestrator disposition (recorded per dispatch)

The predecessor posted the ASK (board_8e711406...): the mandated flip
supersedes the three caps locks listed above, all outside the R-7
touch-set, with options (a) rider commit / (b) fold into R-8. The
orchestrator answered OUTSIDE this journal's readable scope: option (a),
a rider the orchestrator lands AFTER this landing commit, amending the
three one-literals (`load_metric: false -> true` in each stale lock).
This hand does NOT touch those three files; its scoped gates and this
journal are the rider's evidence base.

## INV-14 three-crate zero-diff proof (journal-borne per the handoff)

- Pre-commit working-tree proof (recorded here before committing):
  `python tools/distill.py -- git diff --name-only HEAD -- crates/ui
  crates/report crates/artifact` -> EMPTY (digest capture 0 lines, exit 0).
- Post-commit range proof (`5492372..HEAD`, the dispatch head) is re-run by
  the reviewer; this commit's own diff contains only the three touch-set
  files, so the three crates carry zero bytes across the whole R-7 hop.
- The suite asserts the DATA answers (manifest `languages` = ["Rust"],
  D26 vocabulary through the registry-selected `*_with` formatters, bare-
  name `Debug` label = "Rust") with NO stdout scraping and NO edit to
  `crates/cli/src/commands/ui/node_maps.rs` (read-only citation only).

## L-locks

- L5/L6/L7/L8/L9: no hunk touches m18-m23 suites, lang-cpp, lang-scala,
  tools (the only dirty tracked file is plugin.rs; its hunk is the caps
  literal, doc-comment, and pin rename).
- L2/L3: goldens + fixtures zero-diff (git status above; fixture copies are
  TempDir-only in the suite).
- L4: m00 goldens green UNEDITED (present in the 1940-pass census).
- L1/L11: repo absent from every hunk.
- L12: encoding guard exit 0 (`--all` runs recorded in the report-back;
  `--staged` not runnable between commit_scoped's self-staging and commit -
  the handoff's "(or --all)" alternative used, run post-landing covers the
  committed blobs including this suite).

## Deviations and notes

1. Suite is 500 lines vs the planner's ~250 estimate - still < 1000, R1/R2
   clean; every handoff-mandated proof is present. Reason: the load-lane
   shape-class helper and the collateral witness are asserted as named
   invariants rather than inline lumps.
2. The guard's handoff touch-set parse picked up the MUST NOT globs instead
   of the touch-set (known best-effort limit); explicit `--scope`/`--cmd`
   were passed per the skill, so the resume verdicts are trustworthy; the
   "stray edit" label on the in-touch-set untracked suite is that parse
   artifact, triaged here.
3. No `git stash` used (R-6 D6 standing ruling).
4. SHA relay for R-8 duty 1: this hand's duty-1 = `5492372`; landing SHA is
   the commit carrying this journal (self-referential SHA impossible; the
   report-back and reflog carry it - R-8's duty 1 cites both per the
   handoff's discipline clause).
