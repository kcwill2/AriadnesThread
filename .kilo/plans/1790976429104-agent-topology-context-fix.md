# Agent Topology & Context Protection Fix

## Context

Observed: implementer/orchestrator sessions hit a hard 262k context wall; the
"tool-caller" in the drop-down is never used by orchestrated work; plans and
skills describe a delegation topology that Kilo cannot execute.

Evidence gathered 2026-10-02 (all verified, not inferred):

1. `~/.config/kilo/kilo.jsonc` `provider.spark` declares context/input/output
   numbers only inside `options.headers` -- HTTP headers, not model limits. No
   model `limit` is declared anywhere, so compaction has no correct base and
   sessions die at the server-side 262144 cap instead of compacting (matches
   memory `m23_subagent_context_wall`, ledger M18.11 row).
2. `compaction.threshold_percent: 70` is NOT a documented field (documented:
   `compaction.auto`, `compaction.prune`). It may be inert. Verify against
   the `$schema` URL (`https://app.kilo.ai/config.json`) before keeping it.
3. All four role agents (`architect-orchestrator`, `implementer`,
   `tool-caller`, `review-guide`) are `mode: "primary"` in `kilo.jsonc`
   (migrated from `.roomodes`). The Task tool can only spawn `subagent` mode
   agents, so `implementer`'s "run the test suite via the tool-caller" prompt
   is mechanically unsatisfiable -> implementers run tools raw (confirmed
   empirically in `docs/specs/tool-output-distillation.md` I8 rationale).
4. `tool-caller`'s prompt mandates raw log returns ("Do not summarize away
   critical stack traces") -- directly against the standing contract
   `docs/specs/tool-output-distillation.md` (I8/I9, digest + capture shape).
5. `indexing.openai-compatible.baseUrl` is `http:127.0.0.1:8086/v1` (missing
   `//`); a live `semantic_search` probe returned a wrong-target hit at score
   0.41 -- index degraded or dead.

## Decisions (ratified with this plan; dissent = refine)

- **D1 - Hybrid topology**: `architect-orchestrator` stays the default primary.
  `implementer`, `tool-caller`, `reviewer` become project subagents in
  `.kilo/agent/` (frontmatter `mode: subagent`; confirmed schema in
  kilo-config: `description`, `mode`, `model`, `steps`, `permission`).
  Implementer protection is the I8 distill tools (Kilo subagents do not nest,
  so tool-caller is NOT the implementer's shield -- it is an optional one-shot
  bulk-command absorber spawnable by primary agents).
- **D2 - Project scope for role agents**: contract is repo law; AGENTS.md
  mandates `.kilo/` for new agents. Global `kilo.jsonc` keeps only
  `architect-orchestrator`, `code-reviewer`, `review-guide`; the global
  primary `implementer` and `tool-caller` entries are DELETED (no drift-prone
  duplicates).
- **D3 - Compaction**: declare model `limit` correctly; keep only documented
  compaction keys after schema verification. `threshold_percent` removed if
  the schema rejects it.
- **D4 - Pilot task**: the ledger deferred row "delete UTF-16 debris
  `test_output.txt`, `test_results.txt`, `parse.py`" runs through the fixed
  chain end to end.

## Guardrails (repo standing law, binding on every task)

- Pure-ASCII authored content; Read/Edit/Write tools only, never PowerShell
  content pipes; `python tools/encoding_guard.py --all` stays clean.
- Scoped commits via `python tools/commit_scoped.py` (never `git add .`); plan
  file committed together with the changes it describes (repo convention).
- Every test/build/lint command in any session here goes through the distill
  entry points (`tools/testrun.py --distill`, `tools/distill.py`).
- Ledger (`tasks/milestone_ledger.md`) + kilo-memory mirror updated in the
  same commit; this work gets one ledger row (global config + global skill
  edits are out-of-band and MUST be recorded there -- precedent: tool-runner
  SKILL.md amendment row).

## Phase 1 - Config hygiene (global, no repo dependency)

**T1. Declare model limits; fix compaction base.**
- Backup `~/.config/kilo/kilo.jsonc` -> `kilo.jsonc.bak` first.
- Fetch `https://app.kilo.ai/config.json` ($schema) and confirm the model
  `limit` field shape (expected: `limit: { "context": N, "output": N }` on the
  model entry).
- `spark/models/qwen3.8-flash-next`: add `limit.context: 262144`,
  `limit.output: 32000`. Same for both `kjkjkjk` models (context per header
  values, output 31072). Keep the `headers` blocks (server hints, inert for
  Kilo but harmless).
- Check `threshold_percent` against the schema: if undocumented, remove it and
  rely on `compaction.auto` (default true).
- Gate: restart Kilo; `/status` (or sidebar token accounting) reports a
  ~262k context base for a spark session; sessions compact before dying
  (verify once with any long session or by inspecting `/status` numbers).

**T2. Repair the embedding endpoint.**
- Line ~64: `http:127.0.0.1:8086/v1` -> `http://127.0.0.1:8086/v1`; confirm
  reachability (`curl http://127.0.0.1:8086/v1/models` or equivalent); rebuild
  the index.
- Gate: `semantic_search("cargo test canonical summary line parsing",
  path=tools)` returns `tools/testrun.py` as top hit (baseline today: wrong
  target at 0.41).

## Phase 2 - Real subagents (project `.kilo/agent/`)

**T3. `.kilo/agent/tool-caller.md`** (`mode: subagent`).
- Prompt = the digest contract already written in
  `~/.agents/skills/tool-runner/SKILL.md` (routing table; report shape
  `Command/Exit Code/Verdict/Digest/Capture`; never paste raw; targeted capture
  slices only). Do NOT carry over the "return raw traces" section.
- Permission: `read: allow`, `bash: allow`, `edit: deny`, `task: deny`.
- Delete the global primary `tool-caller` from `kilo.jsonc` (drops it from the
  drop-down; becomes Task-spawnable).
- Gate: from a primary session, Task-spawn `tool-caller` with
  `python tools/distill.py -- cargo check`; reply is digest-shaped only.

**T4. `.kilo/agent/implementer.md`** (`mode: subagent`).
- Base on the global `implementer` prompt; DELETE every "via the tool-caller"
  phrase; ADD the handoff-spec disciplines (read handoff + journal; touch-set;
  distill entry points for all gates; journal checkpoint per module green; on
  looming exhaustion STOP and return disk-state inventory, successor uses
  `checkpoint_guard resume`).
- Permission: `read/edit/grep/glob/list: allow`, `bash: allow`, `skill: allow`
  (to load `tdd`), `external_directory: ask`. `steps:` raise above the default
  if a full RGB cycle trips it (tune during pilot).
- Delete the global primary `implementer` from `kilo.jsonc`.
- Gate: Task-spawn `implementer` on a toy task; it completes one Red-Green
  cycle using only `tools/testrun.py --distill`; final report contains digest
  header + canonical line, zero raw output.

**T5. `.kilo/agent/reviewer.md`** (`mode: subagent`).
- Thin prompt: "Adjudicate the named handoff via the project `review-gate`
  skill; report APPROVED/REJECTED + numbered findings; read-only."
- Permission: `read: allow`, `edit: deny`, `bash: allow` (gate re-runs),
  `skill: allow`.
- Gate: spawnable via Task from an orchestrator session against an existing
  handoff (e.g. `tasks/tool-distill-c-handoff.md` if still open, else pilot).

**T6. Update `architect-orchestrator` prompt in `kilo.jsonc`.**
- Replace generic "delegate to the Implementer/Reviewer" with mechanical
  instructions: delegate by Task-spawning `implementer` / `reviewer` subagents
  with a `tasks/<task>-handoff.md` authored via the `handoff-spec` skill;
  adjudicate returns as digest evidence only; never inline full-phase work.
- Keep `default_agent: "architect-orchestrator"` (D1); if orchestration
  overhead on trivial asks annoys, switch default to built-in `code` later --
  not this plan.

## Phase 3 - Skill + doc prose alignment

**T7. `~/.agents/skills/implementer-rgb-tdd/SKILL.md`**: replace the
"Never run tools directly. Always delegate to /tool-runner" block with the I8
rule (distill entry points; report = digest + canonical line + capture path).

**T8. `~/.agents/skills/idd-workflow/`**: fix `COORDINATION.md` diagram to the
D1 topology (no implementer->tool-runner nesting); point `SKILL.md` state
management at the real artifacts (`tasks/<x>-handoff.md`, journals, ledger,
`review-gate`, `checkpoint_guard`) instead of `tasks/plan.md`/`todo.md`.
- Gate for T7/T8: grep `~/.agents/skills/{idd-workflow,implementer-rgb-tdd}`
  for "delegate to /tool-runner" and "tool-runner (subagent)" -> zero hits.
- Both edits are out-of-band; record in the ledger row (precedent followed).

**T9. Delete `.roomodes`; reconcile `AGENTS.md`** ("Custom Modes" section and
Quick Mode prose) to describe the `.kilo/agent/` topology. Commit
`.kilo/plans/1790976429104-...md` + AGENTS.md + `.roomodes` deletion in one
scoped commit.
- Gate: `git grep -l roomodes` -> only historical plans; `encoding_guard.py
  --staged` exit 0; `checkpoint_guard.py status` census unchanged.

## Phase 4 - Context-budget sizing rule (planning law)

**T10. Add sizing rule to `AGENTS.md`** (overrides dev-planning's file-count
table): a handoff unit = <= 1 crate/scoped area, <= ~10 scoped gate commands,
<= ~1200 expected produced lines; bigger splits. Workspace-wide final gates
(`--baseline` run, full `unittest discover`, `ui_smoke`, `bench_compare`) run
only in a fresh session or the orchestrator's bounded own run. Sizing axis =
gate count x output volume.
- One matching line in `.kilo/skills/handoff-spec/SKILL.md` template (gates
  section already caps "scoped only until final gate" -- make the budget
  explicit there too).

## Phase 5 - End-to-end pilot (D4)

**T11. Pilot the debris-deletion deferred row** (UTF-16 `test_output.txt`,
`test_results.txt`, `parse.py`) orchestrator -> implementer(handoff-spec) ->
review-gate, with a one-module handoff + journal.
- Evidence: spawns visible in UI; digest-only reports; journal written;
  orchestrator peak context well under the compaction point; ledger row for
  this whole plan flips its pilot cell; memory mirrored same commit; the
  deferred row moved to `tasks/ledger_archive.md` per the archive rule.

## Sequencing

- T1, T2 independent, first (config-only; everything else validates better
  under a sane compaction base).
- T3 -> T4 -> T5 -> T6 (topology; each gated before the next).
- T7-T10 after T3-T6 (prose follows mechanism, never the reverse).
- T11 last, after all.

## Risks

- `limit` field name/location for custom-provider models needs schema
  verification (T1 step 1); if the shape differs, adapt -- the finding is
  "no limit declared", the field spelling is an implementation detail.
- Subagent step caps: `steps` frontmatter is the knob if the pilot trips it.
- Global skills under `~/.agents/skills/` are shared across projects; the I8
  wording is repo-specific -- phrase edits to stay tool-agnostic ("route
  through the project's distilling entry points") so other projects don't
  inherit AriadnesThread law.

## Out of scope

- Handoff C (ANSI-strip) adjudication -- separate in-flight work.
- `default_agent` change; model/provider tuning; compaction threshold tuning
  beyond key validity; `.kilo/skills/deslop` and other operator-decision
  strays (separate ledger rows).

## Validation summary

Per-task gates above; final acceptance = T11 evidence bundle +
`python -m unittest discover -s tools -p "*_tests.py"` digest green +
`encoding_guard.py --all` clean + ledger/memory committed together.
