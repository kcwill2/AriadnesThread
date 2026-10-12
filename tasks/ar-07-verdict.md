# AR-07 Verdict - Symbol by-id load API additive seam (P1 part 1)

Date: 2026-10-10. Reviewer: Kilo (review-gate). Commits:
`8ea8d0d` (feature) + `602c176` (disclosure addendum), branch
`shining-enthusiasm`. Reviewed jointly with AR-06
(`70e1699`) - shared gates, see `tasks/ar-06-verdict.md`; shared gate
evidence cross-referenced below.

## VERDICT: APPROVED

## Ground truth

- `checkpoint_guard status`: HEAD `70e1699`. The wave range also contains the
  foreign `b49ed06` (AR-02, separately APPROVED) between the two AR-07
  commits - not part of this adjudication.
- Per-commit review oldest-first: `8ea8d0d`, `602c176`.

## Commit-vs-touch-set (step 4)

`602c176`: journal only - EXACT.
`8ea8d0d` touches: `crates/graph/src/load.rs` (touch-set),
`tasks/ar-07-handoff-journal.md` (touch-set), and
`crates/graph/tests/ar06_edge_dedup_test.rs` - FOREIGN content owned by the
AR-06 handoff, swept by the handoff-mandated glob `crates/graph/tests/**`.
One scope leak, disclosed; adjudicated below as finding 1.

## Scope-leak deviation - ACCEPTED as disclosed deviation (finding 1)

Disclosure is COMPLETE and pre-review: journal section "Commit-scope
incident", dedicated addendum commit `602c176` (~54 min after the feature
commit), board INFO to the AR-06 implementer + RESULT to main, all landed
before this first review of `8ea8d0d`. Containment verified independently:
the swept file is a behavior-characterization suite of the then-current
legacy `add_edge` (AR-06's RED baseline ran 7p/0f against pre-change code),
the addendum's retained capture shows it green at `8ea8d0d`, and the identical
bytes (empty `git diff 8ea8d0d..70e1699` on that path) are green in my fresh
graph-suite run. Inert and green at the introducing commit - the acceptance
condition holds. Recorded as a finding per the review instruction.
Remediation reasoning accepted: history rewrite permission-denied by design
in a shared worktree, `commit_scoped.py` has no unstage mode, deleting the
file would destroy another worker's WIP.

## Plan-literal / additive-purity checks

- `analyze_symbol_load_by_id(&SymbolId)` holds the metric body; the name entry
  `analyze_symbol_load` resolves via `symbol_id_by_name` and delegates
  (load.rs diff) - body exists in exactly one place, no dual representation
  that can drift. Mirrors the `analyze_file_load` / `analyze_file_load_by_id`
  precedent named in the handoff. CONFIRMED at source.
- `symbol_id_by_name` is public and carries the original scan body verbatim
  (matches `id.0` OR record `name`, BTreeMap ascending first-match preserved).
- `find_symbol_id` kept as one-line private alias because the out-of-touch-set
  child module calls it: `crates/graph/src/load_explain.rs:146` CONFIRMED.
- Dead-branch error-string change (old `{query}` -> new `{symbol_id.0}` on
  "Symbol record not found for"): verified unreachable before AND after
  (the id is taken by iterating the same BTreeMap that `get` queries);
  disclosed in the journal; no existing test locks it. Not a behavior change.
- The 5 locks are genuine, read in full: full `LoadMetric` parity between
  entries; dual-match helper lock with fixture `ns::Thing` / record name
  `Thing` (id != name proves the id-string and record-name arms are distinct
  and agree); miss -> `None`; the three error strings
  (`"Symbol not found: Absent"`, `"Symbol record not found for: Absent"`,
  `"No defining file for symbol"`) each locked, with name/by-id agreement on
  the missing-defining-file miss (Ghost). CONFIRMED in the inline
  `symbol_load_by_id` mod.

## Map-redundancy deviation - ACCEPTED on merits (finding 2 records the mis-citation)

`ReducedSemanticModel::symbols` is `BTreeMap<SymbolId, ReducedSymbolRecord>`
(`crates/graph/src/reduced.rs:86` CONFIRMED) - the by-id entry's record lookup
is O(log S) for free; a side map would be a second representation of the same
fact. A name->id map could not be built honestly in this touch-set (insertion
lives in the frozen, out-of-touch-set `reduced.rs`; memoization lives in
`load_caches.rs`; interior mutability would break `Send + Sync`), and the
plan's AR-08 consumer iterates ids and bypasses name resolution
(plan:139-140). Stated impact - "no runtime change yet; creates the seam for
AR-08" - is fully delivered. Accepted per ponytail.

## Gates (fresh, reviewer stdout) - see `tasks/ar-07-execution-results.md`

- graph scoped: `116 passed / 0 failed / 0 ignored / 5 suites` (matches
  implementer's GREEN line)
- workspace census: `1326 passed / 0 failed / 13 ignored / 94 suites`
- encoding `--all` silent exit 0; hygiene = exactly the 2 pre-existing
  `server.rs` R1/R2 rows; zero rows on `load.rs`.

## Findings

1. (minor, accepted deviation) `8ea8d0d` carries the foreign
   `crates/graph/tests/ar06_edge_dedup_test.rs` - OUTSIDE AR-07's content
   ownership though inside its mandated glob. Disclosure complete before any
   review (journal + `602c176` + board to main and AR-06); file verified
   inert green at `8ea8d0d` and byte-identical at `70e1699`. No rework
   (history rewrite denied by design in the shared worktree). Wave
   reconciliation should treat `8ea8d0d` as the file's introduction commit
   and AR-06 as its content owner.
2. (minor, documentation accuracy) The journal states the no-map fallback is
   "per the handoff's stated fallback" - neither `tasks/ar-07-handoff.md` nor
   the plan anchor (`.kilo/plans/...-audit-remediation.md:129-136`) contains
   any fallback clause. The deviation itself is sound and accepted on merits
   (BTreeMap verification above); the citation is inaccurate and must not be
   relied on by AR-08 or future readers. No content change required to land;
   correct the wording whenever the journal is next touched.
3. (note, no action) Name-entry error string on the unreachable
   "Symbol record not found" branch now formats `symbol_id.0` instead of the
   query; unreachable before and after (same-map invariant), disclosed in the
   journal, verified by me at source.

## Deviation dispositions

- Side map NOT built (handoff objective names it literally): ACCEPTED -
  redundant against `reduced.rs:86`; insertion site frozen out-of-touch-set;
  impact criterion (AR-08 seam) delivered. (See finding 2 for the
  mis-citation.)
- Tests inline in `load.rs` rather than a sibling file: ACCEPTED - stays
  inside the commit scopes; `load.rs` remains under 1000 lines (zero hygiene
  rows).
- RED presented as `error[E0599]` compile failure (`testrun` DRIFT exit 2):
  ACCEPTED - canonical Rust RED for a missing API, capture-cited before any
  implementation.
- Scope leak into `8ea8d0d`: ACCEPTED as disclosed deviation (finding 1).
