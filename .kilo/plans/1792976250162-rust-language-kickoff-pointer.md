# rust-language milestone kickoff - plan pointer (2026-10-25)

The authoritative artifacts for this milestone are the two task docs (the
architect deliverables of record; this file is a review pointer only, per
the client plan-view convention - the tasks/ docs govern):

- `tasks/rust-language-kickoff.md` - objective, the three user decisions
  of record (2026-10-25), acceptance commands, frozen locks L1-L12, named
  test amendments A1-A8, inherited carried-not-now rows, graduation-backlog
  audit (D4), landing duty, scope-out.
- `tasks/rust-language-shape.md` - invariants INV-1..INV-15 with named
  locks, exact contracts (spine appends, registration, modeling decision
  table, capability literals, detector/shaper/hooks/validator), module map,
  R-1..R-6 handoff-arc sketch within the sizing law, risk register with
  oracles, hygiene notes, docs riders.
- Ledger row appended to `tasks/milestone_ledger.md` (live `rust-language`
  row, UNREGISTERED post-M28 track, no M-number, "kickoff 2026-10-25",
  mirroring the extraction-symbol-move registration wording).
- Kilo-memory mirror saved: key `ledger.milestone.rust-language`
  (memory-mirrors-ledger law; the memory tool writes outside the workspace,
  the ledger row is the authority).

Shape highlights (for review):

1. Third plugin via the existing seams: open `LanguageId` needs no
   vocabulary enum edit; registration appends LAST in
   `crates/pipeline/src/registry.rs`; C++ stays first (order is a contract).
2. Spine edits are exactly: `FileKind::RustSource` (APPENDED after `Other`
   to keep serde indices), `.rs` in `from_path`, `is_analyzable` arm,
   `from_file_kind` arm, `LanguageId::Rust`/`rust()` consts, and one
   appended `AggregationKind::ModDeclaration` variant (no production
   `match` exists on it - grep-verified).
3. Two enumeration sites were MISSING from the dispatch brief's list and
   trip at the registration hop: `crates/cli/src/registry.rs` uses
   `src/unclaimed.rs` as its "no plugin claims this" witness twice
   (`pin_plan_backend_honest_errors_...` :411/:419 and
   `planning_hooks_answer_honestly_...` :743) - amendments A3/A4 move the
   witness to `.f90` in the SAME commit that registers Rust.
4. Modeling: unit=SourceFile, aggregation=ModDeclaration, guard=None,
   placement=SingleFile, reexports=false, cycle_policy=
   TolerateWithinPackage (use-edge cycles legal in a module tree; the
   stance is planner-informational + report-side via the existing
   registry lookup - ZERO core edits), markers target/vendor/.cargo,
   vocabulary Use/used/mod aggregation, caveat = macro-unexpanded.
5. syn + proc-macro2(span-locations) + unicode-ident land as
   workspace-pinned deps - a deliberate, user-ordered deviation from the
   guide's "three deps and no more" line; the m17 guards cannot be
   violated by them; a dated guide rider records the deviation.
6. The three CLI `CyclePolicy::Reject` literals stay: since AR-19 all
   three planner sites dispatch hooks through the registry; the literals
   are informational stances. Rust enrolls in planning by overriding the
   two trait hooks.
7. Biggest semantic risk (RISK-MODTREE): an extracted Rust module is
   invisible to its crate until the parent declares `mod x;` - the R-5
   rewriter/extractor must land that declaration in the same transaction
   and the apply suite asserts the parent file is modified.

Landing duty (the kickoff doc section 8 carries the exact command): the
docs + ledger row commit as ONE scoped commit; this architect session has
no shell, so `hygiene_check.py` and `commit_scoped.py` run at the landing
hop - stated explicitly, no phantom SHA.
