# Verdict - ui-extraction-card-playbook T-A: APPROVED

Commit under review: `df1e21e` (parent / journal start rev `4a65b6f`).
Adjudicated via `review-gate` with the reduced, by-design gate list for this
hard-chain hand. Reduced list per the orchestrator's hop instruction; the
workspace is EXPECTED red in `crates/cli` until T-B lands (designed cost, see
O1 disposition), so the workspace gates are deferred, not skipped silently.

## Gate results (this hop, fresh, verbatim)

| # | Gate | Result |
|---|---|---|
| 1 | `python tools/testrun.py --distill -p ariadne-ui` | `### cargo test PASS (exit 0)` / `200 passed / 0 failed / 0 ignored / 3 suites` -- reproduces implementer final digest exactly |
| 2 | per-commit `git show df1e21e` + touch-set | single commit; every path in-set (below) |
| 3 | `hygiene_check.py` | R3 warn-only legacy rows; NONE from T-A lines; exit 0 (violation contract -> 1 not triggered) |
| 4 | `encoding_guard.py --all` | no output, exit 0 |
| 5 | P3 key-vec literal | old 11 keys + exactly `target_files`, `target_files_truncated`, correctly sorted (proposals_payload.rs:86-100 read whole) |

Implementer digests re-verified: Green `198 passed / 0 failed / 0 ignored /
3 suites` is the pre-typed-locks intermediate (not reproducible post-commit;
totals reconcile: 197 prior scoped + 1 Red-phase test + 2 typed locks = 200);
Red `196 passed / 2 failed / 0 ignored / 2 suites` exit 1 is journal-attested
(historical state, unreproducible); failing list = the two named tests as
mandated. Journal is digest-shaped per the distillation contract (canonical
lines, exits, capture paths, zero raw dumps).

## Commit-vs-touch-set

`git diff --name-only 4a65b6f..df1e21e` = `crates/ui/src/dto.rs`,
`crates/ui/tests/ui_test/dto_builders.rs`,
`crates/ui/tests/ui_test/proposals_payload.rs`,
`tasks/ui-extraction-card-playbook-ta-journal.md`. All in-set. Nothing else:
no ledger ride this time. `git diff 4a65b6f..df1e21e -- crates/cli
crates/ui/assets tests/fixtures tools crates/ui/src/lib.rs
crates/ui/tests/ui_test/export_order.rs` -> EMPTY (explicit proof run).
`export_order.rs` byte-identical confirms D1 mechanically: the scoped suite
compiles green, so no exhaustive literal there could have been missed.

## Lock audit

- dto.rs diff purely additive: two insertion hunks, +27/-0; both new fields
  carry `#[serde(default)]` (C4 verbatim); C1 block verbatim -- const value 8,
  `ProposedTargetFileDto { path: String, symbol_names: Vec<String>,
  symbol_count: usize }`, derives `Debug, Clone, PartialEq, Eq, Serialize,
  Deserialize` per shape C1; no cluster type crosses (Strings/usize/Vec only,
  dto.rs:182-184 rule respected).
- Outer `skip_serializing_if` region byte-identical: dto.rs:250-253 intact,
  outside every hunk; its two key-stability tests green UNEDITED (absent from
  diff; scoped suite green).
- `dto_builders.rs` = exactly the 2 mechanical fill-in lines, no assertion
  edits. `proposals_payload.rs` = compelled `use` widening (declared), the
  sanctioned P3 amendment, 2 fill-ins, 3 appended tests; all pre-existing test
  bodies byte-identical.
- dto.rs lands 907 of 1000 (R2 clean); module extraction stays deferred per
  shape 9.

## Deviation dispositions

1. D1 (`export_order.rs` no-op): ACCEPTED. File left byte-identical; green
   compile is the machine proof that no `ExtractionProposalDto` literal exists
   there; the unchanged file legitimately cannot appear in `git show --stat`.
2. D2 (`git add` allowlist-denied; commit_scoped `--only` + set-equality
   instead): ACCEPTED. Reviewer's independent name-only diff confirms exact
   set equality; the vacuous pre-commit `--staged` on a clean index is
   compensated by the reviewer's `--all` (exit 0).
3. D3 (6 gate invocations vs planned 5): ACCEPTED. The handoff's own step-3
   appends + step-4 mandated re-run make 3 testruns unavoidable; 6 <= 10.
4. D4 (doc comments authored inside Green, step 4 as verification re-run):
   ACCEPTED. Same commit, no assertion weakened, re-run confirms clean.

## Out-of-scope observation disposition

- O1 (`crates/cli/src/commands/ui.rs:524-536`): VERIFIED by read -- exhaustive
  struct literal, no `..` rest, no new fields -> `crates/cli` cannot compile
  between this landing and T-B. Ruled a DESIGNED hard-chain cost, not a T-A
  violation: the handoff names the HARD CHAIN up front, forbids
  `crates/cli/**` in its touch-set (so T-A could not fix it without a
  blocker), the shape module map assigns `crates/cli/src/commands/ui.rs` to
  T-B's touch-set, and the ledger PLANNED row records the chain ("T-B compiles
  only after T-A's DTO fields land", none parallel-safe). Bookkeeping for the
  orchestrator: any workspace census / `ui_smoke` / bench between T-A and
  T-B's landing will fail on cli compile -- run them only from T-B onward.

## Findings

1. (minor, non-blocking) `crates/ui/src/dto.rs:187` -- the new doc comment
   uses a U+2014 em dash ("plain display data only -- the cluster plan type
   never crosses into ui"). The handoff's hygiene line says "every authored
   line pure ASCII". Mitigating: it exactly matches the file's established
   house style (10 pre-existing em-dash lines including the adjacent comment
   at :201), `encoding_guard.py --all` (the design enforcement, which targets
   double-encoded mojibake, not legitimate single-encoded characters) is
   silent, and the rule's purpose is intact. Disposition: accepted as-is;
   T-B/T-C should use ASCII `--` in new doc comments to keep the letter of
   the rule for fresh lines.

## Gate-coverage statement (deferred, not skipped)

Not runnable this hop BY DESIGN (workspace red in cli until T-B): full
workspace census (`testrun.py --distill` unscoped), `ui_smoke.py`,
`bench_compare.py` (also moot for T-A: no pipeline/asset change; T-B is the
non-assets Rust hand that owes the bench per shape risk 8). Deferred to:
census + smoke at T-B's reviewer hop onward (and mandatorily at milestone
closeout); bench at T-B's hop. Tools-suite discover: unaffected by the chain,
not requested in this reduced hop; run at the first hop that can take the
full workspace. Scope of proof for T-A is the scoped `-p ariadne-ui` suite
plus the static locks above -- all green, all verified fresh.

APPROVED. T-B may start; it owns the `ui.rs:524` field fills in its own
touch-set.
