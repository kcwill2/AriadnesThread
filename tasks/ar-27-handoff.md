> SUPERSEDED 2026-10-12: the implementer STOPPED at dispatch (journal-only landing, zero code: `tasks/ar-27-handoff-journal.md`) - the contract's core byte-stability mechanism ("serde aliases keep artifact bytes stable") is empirically unreachable: old-enum bincode bytes fail to load through the transparent alias and the alias moves `context_hash` +7 bytes (journal section 5). Architect ruling `tasks/ar-27-shape-ruling.md` ADOPTED Option B as amended (dual-format serde: pinned u32 LE tags + string escape positionally, transparent bare-name human-readable, `IncludeEdge.language` string-frame exception; custom legacy-index Ord; value-From deleted, `&`-From kept; clone riders). Option A's epoch bump REJECTED as primary path. Re-planned by the planner as a 2-hop serial chain: `tasks/ar-27-ta-handoff.md` (byte-oracle fixtures green PRE-mutation + home move + dual-format serde; ~450-520 ln) -> `tasks/ar-27-tb-handoff.md` (registry-derived `--lang` + FileKind roles + third-language E2E; ~300-380 ln). LOCK AMENDMENTS: NONE (ruling section 7). Retained as the original wave-6 contract - do not dispatch.

# AR-27 Handoff - SourceLanguage -> LanguageId, open `--lang` (A2, HIGH-risk)

Wave 6. Crates (`repo` + `lang-api` + `cli` + consumers). Dispatch ONLY
after AR-19 AND AR-24 approved AND lang-api churn settled (AR-20 landed;
AR-26 preferred landed). This is the highest-risk handoff in the plan —
escalation model allowed.

## Objective

Migrate `TuAnalysis.language`, `FileRecord`, and `--lang` acceptance to
`LanguageId` (serde-transparent; precedent exists in-tree — locate it).
Derive `--lang` candidates from `registry.languages()`; replace `FileKind`
per-language variant sections with language-id + kind-role. Serde aliases
keep artifact bytes stable across the bump.

## Impact

Open-set plugin end to end: a new language plugin is detected, TAGGED,
persisted, and CLI-selected with ZERO edits to repo/lang-api/cli — the
single largest enabler of the Scala-then-more intent. After this, a third
language is plugin-only work.

## STOP-AND-REPORT conditions (any = halt, no partial landings)

- Any m23/m00 bincode lock fails under serde aliases (do NOT bump versions
  unilaterally; if a bump is unavoidable, journal the justification and
  report back BEFORE proceeding).
- Diff growth beyond ~800 lines or scope crossing beyond repo/lang-api/cli
  + golden locks — report for split (T-a: LanguageId serde migration +
  aliases; T-b: --lang open-set + FileKind role refactor).

## Touch-set (hard)

- `crates/repo/src/types.rs`
- `crates/lang-api/src/types.rs` (+ serde alias module)
- `crates/cli/src/registry.rs` + `--lang` acceptance paths + consumers
  surfaced by compiler
- golden/serde locks: review only; UNEDITED unless T-b declared amendments
- `tasks/ar-27-handoff-journal.md`

## Constraints

- Serde alias table is the byte-stability mechanism (old `Cpp`/`Scala`
  strings still deserialize; serialized output matches registry ids —
  journal exact alias list).
- TDD: new lock — third "language" via test-plugin registry flows through
  detect -> tag -> persist -> `--lang` select with ZERO core edits (uses
  AR-18/AR-19 mock-plugin precedent); RED demonstrates today's hardcoded
  failure.
- m23 bincode locks UNEDITED (authority). m00 parity UNEDITED.
- STRICT COMPILE-GREEN across workspace (types on every lane's path —
  coordinate: no other cargo-heavy lane may be mid-flight at dispatch).
- Pure ASCII; commit_scoped per crate + journal.

## Gate commands

1. `-p ariadne-repo`, `-p ariadne-lang-api`, `-p ariadne-cli` (exact
   reconciled counts)
2. m00 golden + facts-path; m23 bincode lock; m24/m24a identity + Scala plan
   path (AR-19 lock) — ALL UNEDITED
3. `-p ariadne-ui` UNEDITED (payload serde surfaces)
4. `python tools/distill.py -- python tools/ui_smoke.py` (PASS)
5. `encoding_guard --all`; `hygiene_check.py` (zero-blocking baseline)

## Invariants / locks

- Artifact bytes written by the OLD enum still load (alias lock with a
  pre-generated old-bytes fixture).
- `--lang` help lists registry-derived candidates incl. scala (new lock).
