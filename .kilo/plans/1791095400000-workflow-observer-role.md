# Workflow Observer - Spec & Plan

Status: READY TO EXECUTE - Prepared 2026-10-04 - Base: `main` @ `e276910`

## Goal

A seventh, non-gating role plus a deterministic audit tool that measures whether the workflow is
actually working, aggregates failures into countable classes so they can be ranked, and proposes
improvements it is not authorized to apply itself.

## Why now (evidence)

- The dominant failure class is silent absence, not wrong work.
  E10: the M19 branch committed its walkthrough, the UI-theming deck is absent from every branch.
  E16: the three-theme implementation existed in a worktree and appears in ZERO commits on any
  branch, while kilo-memory recorded it as decided and shipped.
  E1: a dead implementer left an empty compaction summary and the successor reverse-engineered
  intent from disk.
- The harvesting step already happened once, by hand, and stopped.
  `.kilo/plans/1790831557705-process-tooling-codification.md:9-26` is a table of incidents E1-E16
  turned into `tools/` scripts. Nothing observes for E17 onward.
- Live debt read directly while planning this (2026-10-04), all still on disk:
  7 undischarged rows in the deferred-findings table of `tasks/milestone_ledger.md`, two targeting
  nominally-complete milestones and three carrying unresolvable targets such as `"operator"` and
  `"next hygiene pass"`.
  4 census rows stale since 2026-10-01.
  Only 2 of 11 journals declare a commit SHA, and no handoff anywhere records a reviewer verdict.
  `plan-repair-t2/t3/t5-journal.md` each record gates "discharged by inspection (no shell in this
  session)".
  M21 has handoffs but no row in the milestone table at all.

## Core idea

Two layers. The tool emits countable, machine-checkable instances grouped into declared classes.
The agent reads that bounded set, ranks it, names any class the tool does not yet have, and
proposes changes.

Classes span the whole chain, because a landing-only observer can never see that the plans were
wrong: this repo has a milestone named `plan-repair` with T1-T5 plus a rejected commit (`bc6efa0`)
requiring resubmission, and that is a planning-quality signal with no landing-stage analogue.

## Decisions

- **D1 - Tool is the checklist, agent is the diagnosis.**
  Absence needs a prior expectation and only a parser can hold one reliably, so presence/landing
  assertions are deterministic. Novel shape is judgment, so the agent owns it.

- **D2 - Advisory only.**
  The observer may not touch plans, handoffs, journals, skills, agent files,
  `tasks/milestone_ledger.md`, or any reviewer verdict. Findings reach the workflow only when a
  human or the orchestrator routes them through the normal architect/planner path.
  A role that can amend a verdict is not a reviewer of the workflow, it is a second, unreviewed
  gate.

- **D3 - Tool is always-on, agent is deliberate.**
  `tools/workflow_audit.py` runs freely, including as a non-blocking CI step, so detection cannot
  be forgotten. The agent runs on request or once at milestone closeout.
  No scheduler, daemon, or background execution. Every role denies `background_process` and that is
  not relaxed here.

- **D4 - Counting widens the observable surface, not the implementation scope.**
  A class such as "reviewer verdict not persisted" can be COUNTED without implementing
  persistence. So the full class inventory ships in v1 while B1 (verdict persistence), B2 (journal
  SHA convention), and B3 (substituted-gate detection) stay out of scope as before. The class
  exists with a count, the agent proposes the fix, the operator decides.

- **D5 - Regressions and standing debt are reported separately.**
  REGRESSIONS is debt at or after the adoption cutoff; it is the part that should trend to zero and
  the part CI surfaces. STANDING DEBT is the pre-cutoff set, reported as a count that can only
  shrink. A permanently red digest stops being information.

- **D6 - Rates need denominators, and two of them.**
  Every class reports current-regime rate (post-cutoff, the actionable number, since it moves as
  work is fixed forward) plus all-time rate as context. Any rate whose denominator is below a
  declared threshold is marked LOW CONFIDENCE, so `1/1` never reads as a trend.

- **D7 - Counts must not rank on count alone.**
  One instance of declared-not-produced is irrecoverable lost work (E16); eleven stale census rows
  are bookkeeping. So each class declares a boolean `irrecoverable` flag, which exists purely to
  stop a 1-instance catastrophe from being buried under a large cosmetic class. All actual severity
  and priority ordering is the agent's judgment, grounded in the numbers. No severity ladder lives
  in code, because policy in code rots and gets argued about instead of used.

- **D8 - Classes whose rate is near 100% are constants, not measures.**
  Each class declares an `expected_rate`. A class sitting at its expected value is labeled a KNOWN
  GAP AWAITING ITS B-DECISION and is not scored as a finding. Two of the five classes are constants
  today (verdicts 0/26, SHAs 9/11). They stay visible because they are the standing argument for
  B1/B2, but the digest must not pretend they discriminate.

- **D9 - The tool is stateless; the archive remembers.**
  Trend needs history, but no state file is needed: the tool reports current counts only, and the
  agent computes trend by comparing against `tasks/workflow-observer-archive.md`, which it appends
  to each run. The tool never remembers, so it cannot drift out of sync with a baseline file.

- **D10 - The ledger is the expectation registry and holds the cutoff.**
  Milestone status drives which tasks are expected to have produced artifacts, and the adoption
  cutoff is a dated ledger row, consistent with D4's existing rule that the ledger owns flips.
  Advancing the cutoff is part of closeout. Consequence: the audit inherits ledger inaccuracy.

- **D11 - Search all refs, and bucket what cannot be resolved.**
  `git log --all`, because E16's own ledger wording defines the loss as existing in zero commits on
  ANY branch; current-branch scope would misreport off-branch work as lost. A handoff whose
  milestone has no ledger row (M21) goes to a distinct UNRESOLVED bucket, never to CLEAN, because
  fail-soft-to-clean is the worst possible outcome.

- **D12 - The first run is a triage run and counts nothing.**
  `--triage` emits every instance with no counts, rates, or regression judgement. The agent reviews
  the list, discards false positives, promotes classes that proved real, and only then does run 2
  become the first authoritative baseline. Run 1's numbers would conflate real standing debt with
  tool bugs, and the tool cannot tell those apart.

- **D13 - Reuse the handoff parser; never write a second one.**
  `tools/checkpoint_guard.py` already extracts everything needed: `parse_handoff()` returns
  `HandoffParse(globs, commands)` at line 310, and the module also provides
  `_extract_touch_set_globs`, `_extract_command_lines`, `path_in_scope`, `collect_dirty`,
  `collect_untracked`, `kilo_worktree_dirs`. `workflow_audit.py` imports these.

- **D14 - Naming.**
  Tool `tools/workflow_audit.py`, suite `tools/workflow_audit_tests.py`.
  Agent `~/.config/kilo/agents/workflow-observer.md`.
  Contract `docs/specs/workflow-observer.md`.
  Report `tasks/workflow-observer.md`, superseded runs appended to
  `tasks/workflow-observer-archive.md`.

## Class inventory

Rates are denominators, not raw counts. `irrecoverable` is a per-class boolean per D7.

| Class | Instance | Eligible denominator | Notes |
|---|---|---|---|
| `declared-not-produced` | handoff declares touch-set globs, no commit on any ref touches them | handoffs with parsable non-empty globs | `irrecoverable: true` |
| `unrecorded-evidence` | journal lacks a commit SHA, or handoff has no linked reviewer verdict | handoffs | constant until B1/B2, `expected_rate: high` |
| `undischarged-ledger-row` | deferred or census row with no archive entry, split into two sub-counts | rows ever added | the 7 + 4 already found |
| `unresolved-provenance` | handoff whose milestone has no ledger row, or whose Where column names a different ref | all handoffs | M21 lands here |
| `rework-signal` | more than one journal per handoff id, REJECTED verdict strings, milestone or plan titles matching repair/redo/fix | handoffs, and milestones respectively | HEURISTIC: title patterns and counts, not prose semantics; label as approximate |

Known gap: no v1 class covers the implementation stage of the chain.
`claimed-but-not-executed` gates would need B3 prose parsing and is deliberately deferred.

## Tasks

Four implementation units sized to the handoff sizing law (one module dir, <=10 gates,
<=~1200 lines), plus one live step H5 that is a run rather than a commit.
Each in-repo unit ends with its own scoped commit via `python tools/commit_scoped.py`.

1. **H1 - Class framework plus ledger-derived classes.**
   - Instance records with stable content-derived IDs, so the same finding is recognisable across
     runs without stored state.
   - Class registry: name, denominator definition, `irrecoverable`, `expected_rate`.
   - Parse `tasks/milestone_ledger.md` into milestone, deferred, and census rows; parse
     `tasks/ledger_archive.md` to learn which rows were discharged; parse the cutoff from its dated
     row.
   - Dual-rate computation per D6, LOW CONFIDENCE marking, regressions/standing split per D5.
   - `--triage` mode per D12.
   - Classes: `undischarged-ledger-row`.
   - Gate: `python tools/workflow_audit_tests.py`, `python tools/hygiene_check.py`,
     `python tools/encoding_guard.py --all`.

2. **H2 - Handoff-derived classes over git.**
   - Scan `tasks/*-handoff.md`, reuse `parse_handoff` for globs, resolve each handoff's milestone id
     from its title.
   - Classes: `declared-not-produced` (with `irrecoverable` flag), `unrecorded-evidence`,
     `unresolved-provenance`, `rework-signal`.
   - `git log --all --name-only` bounded per handoff; bucket unresolved provenance per D11 rather
     than reporting clean.
   - Every class ships a positive fixture and a negative fixture on synthetic temp git repos. No
     test reads the real repo.
   - Gate: `python tools/workflow_audit_tests.py`, `python tools/hygiene_check.py`,
     `python tools/encoding_guard.py --all`.

3. **H3 - Digest rendering, CI wiring, and the standing contract.**
   - Line-oriented digest, stable enough for the agent to cite by line number.
   - Per-class sections ordered by rate, with any `irrecoverable` instance surfaced regardless of
     rank. KNOWN GAP labels per D8. Exit codes mirror the repo convention: 0 clean, 1 findings,
     2 internal error; a CI step that never blocks.
   - `--json` for the agent.
   - `docs/specs/workflow-observer.md`: entry points, digest shape, the advisory limit, the
     regressions/standing split, cutoff semantics, the constants caveat, and the observer's own
     graduation rule, which is advisory-only until a dated ledger row says otherwise.
   - Gate: `python -m unittest discover -s tools -p "*_tests.py"`,
     `python tools/encoding_guard.py --all`.

4. **H4 - The observer agent. Out of repo, needs an implementation-capable agent.**
   - `~/.config/kilo/agents/workflow-observer.md`, `mode: subagent`, inheriting the caller model
     because it is a judgment role that should be escalable by switching the parent model.
   - Permissions, last-match-wins with the catch-all `"*": "deny"` first, matching the existing
     roster: `task` deny, `background_process` deny, mutating `git_git_*` deny, edit limited to its
     own report and archive, bash allowlist limited to the audit tool and read-only git.
   - Mandatory precondition check: if `tools/workflow_audit.py` or `tasks/milestone_ledger.md` is
     absent, report NOT APPLICABLE and stop. The roster is global and affects every Kilo project,
     but this tool is specific to this repo.
   - Prompt constraints: consume the digest rather than rescanning the repo; cite a digest line for
     every finding; never adjudicate or amend a reviewer verdict; never edit governance files;
     compute trend against the archive; propose promotions and route them via the orchestrator.
   - Add the role to the orchestrator's task allowlist, and add a milestone ledger row recording the
     global artifact plus a config backup, as the five-role row did.

5. **H5 - Triage run, then first authoritative baseline.**
   - Run `--triage`, review every instance, discard false positives, and promote the classes that
     proved real.
   - Write the dated adoption-cutoff row to the ledger.
   - Run counting for the first time; that is the first baseline. Record it as run 1 in
     `tasks/workflow-observer.md`, with triage output archived.
   - Expected: standing set non-empty and consistent with the debt enumerated above; regressions
     empty.

## Risks

- **Prose false positives in `declared-not-produced`.** The class cannot distinguish "declared and
  never done" from "declared as deferred": `tool-distill-a-handoff.md:20` says "T4 governance
  deferred to handoff B", so some handoffs will correctly have globs with zero commits. This is an
  irreducible false-positive rate from prose. Keep the class advisory-only, record the limitation in
  the spec, and do not attempt to parse deferral in v1.
- **Ledger prose brittleness.** These are markdown tables, not a schema. Parsing must tolerate
  column drift and fail soft, and a parse failure must never render as clean.
- **Constants presented as measures.** Mitigated by D8, but only if the label is actually
  implemented; otherwise the digest trains you to skim.
- **Greenwashing.** A tool that counts what it can count will look healthier than the workflow is.
  The agent layer covers part of the remainder; the spec must state this plainly rather than imply
  full coverage.
- **Triage never happens.** If the triage run is skipped, every later trend is built on
  unvalidated counts. This is why D12 makes the triage run an explicit mode rather than a habit.
- **H4 is outside repo review.** The agent file and orchestrator allowlist live under
  `~/.config/kilo/` and are not covered by repo gates, so they need manual verification after
  writing.

## Validation

- Unit: `python tools/workflow_audit_tests.py`, every class with a positive and a negative fixture.
- Repo gates per unit: `python -m unittest discover -s tools -p "*_tests.py"`,
  `python tools/hygiene_check.py` with no R1/R2, `python tools/encoding_guard.py --all`.
- Live: triage output reviewed instance by instance; first counting run's standing set consistent
  with the enumerated debt; regressions empty on day one.
- Observer: one real invocation produces a report whose every finding cites a digest line.

## Out of scope

- B1 verdict persistence, B2 journal SHA convention, B3 substituted-gate detection.
- Narratively reading session transcripts as a primary evidence source. Transcripts remain
  available to the agent as supporting evidence but are never required.
- Any gate power. The observer never blocks a commit, a verdict, or a closeout.
- Discharging the flagged debt itself. The tool reports; humans discharge.
- Letting the observer edit `AGENTS.md`, the `idd-workflow` skill, or any agent prompt.
- A severity ladder, trend arithmetic, or a baseline state file in the tool.

## Open questions

- Milestone assignment is the planner's call at kickoff. This is milestone-independent tooling, so
  the D5 precedent in the process-tooling plan suggests `main`, but M23.5 is live in its own
  worktree and must not be disturbed.
- B1: where verdicts live, and does persisting them change the reviewer's report shape?
- B2: which milestone introduces the SHA convention, and what is its warn-to-blocking date?
- B3: should substituted-gate detection live in the observer's prompt instead of a tool check?
- Does closeout invoke the observer, or does the orchestrator offer it and you decide?
- At what standing-debt count does the standing section stop being worth printing?
- Is the implementation-chain-stage gap worth a structural class later, or is rework-signal enough?
- Would a project-local agent avoid the NOT APPLICABLE guard, or is global uniformity with the
  existing roster worth that cost?