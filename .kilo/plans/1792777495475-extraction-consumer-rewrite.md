# Plan digest - extraction-consumer-rewrite (architect hop, 2026-10-23)

Artifacts of record (this hop): `tasks/extraction-consumer-rewrite-kickoff.md`
+ `tasks/extraction-consumer-rewrite-shape.md`. This file is the planner's
digest; the planner authors handoffs (`handoff-spec`) and performs the
ledger kickoff flip touching only `tasks/milestone_ledger.md`.

## The change in one paragraph

An apply's consumer rewrite is a silent no-op on C++ because discovery
probes the bare spelling (`thing.h`) while the rewrite spec carries the
plan target's path (`include/thing.h`). The fix is C++ implementing the
`DependencyRewriter::consumer_rewrite_spec` hook (already reserved for
exactly this, Scala already implements it) with the same bare-file-name
rule its `consumer_discovery_targets` uses. Discovery sets, path payloads,
preview bytes, the transaction/gate machinery, and Scala stay byte-frozen.
The two characterization tests that lock the bug (m20 apply, m19 optimize -
the m19 twin was found in the survey; the ledger row names only m20) are
amended under a closed amendment policy, the A-4 follow-up faithfulness
oracle lands as an equality in the cli apply proof, and the two copy
relays (`hotspots.js:521-522` comment, the wizard S4 truncation note) are
flipped in the same milestone so honesty returns when the fix lands.

## Move verdict (the user-told-architect decision)

Symbol move DEFERRED; follow-on key `extraction-symbol-move` registered in
the kickoff. Rationale (full evidence, shape section 4): C++ move = a
lang-cpp parser-span workstream (M25-class hazards) plus a user-owned
copy-vs-move default product decision plus a second amendment table;
move already ships honestly for move-capable plugins (Scala) with the
dry-run `move_removals`/`copy_only` spine in place; D6 is move's hard
prerequisite anyway, so the ordering physics puts it after this milestone.
Copy-only stays available and truthfully labeled meanwhile.

## Hands (3, strictly serial, all inside the sizing law)

1. CW-1 crates/lang-cpp: the hook + fidelity/unit tests; declared riders
   = the m20 and m19 characterization amendments (closed policy, captured
   Red, reviewer per-commit diff; everything else in both files
   byte-identical). ~250-400 lines, ~9 gates.
2. CW-2 crates/cli: A-4 test 1 upgraded to expected-modified equality
   (the deferred D6 oracle), no-deletions assertion, `ascii_only(&plan_id)`,
   cosmetic fix, plan_id ASCII unit test. ~200-330 lines, ~9 gates.
3. CW-3 crates/ui: hotspots comment flip, S4 note reword, a new lock so
   "never runs anything" cannot rot back; fetch-site bound `<= 2` survives
   untouched. ~50-110 lines, ~6-7 gates.

## Key invariants (falsifiable; oracles in the shape)

X1 discovery-hit iff rewrite-changes-text (plugin unit pair + end-to-end
equality); X2 consumer-set equality UNEDITED; X3 preview/dry-run bytes
UNEDITED (incl. m18 parity diff text - the fix is below the payload
layer); X4 plugin neutrality (m17 guard, Scala untouched); X5 copy-only
preserved (no C++ removal hook; declarations survive); X6 gate/transaction
untouched (caller census 3, token/mutex tests UNEDITED); X7 ASCII/hygiene;
X8 a closed amendment list for reviewer diff enforcement.

## Flagged for the user (not silently decided by this shape)

The copy-vs-move default (move-by-default vs opt-in for C++) is registered
as the `extraction-symbol-move` kickoff's product decision. Also registered
as follow-on candidates (not keys yet): include-closure/compilability of
rewritten consumers (successor to D-M23-12), and directive-spelling display
in the dry-run rows.

## Gate anchors

Census anchor `1830 passed / 0 failed / 13 ignored / 119 suites @ 765df7b`;
live main at `a38c7aaa` (git MCP; only stray: the pre-existing dirty
`.vsix`). `checkpoint_guard status` not run by this hop (architect has no
bash) - first hand re-runs it as bootstrap. Bench timing gate is runnable
(`f20ad35` label fix); record, attribute only if clean (PR-2 persists).
UTA deferred-findings intake (docs/specs/unit-test-audit.md) is a mandatory
pre-authoring read for CW-1/CW-2 handoffs (lang front-ends are touched).
