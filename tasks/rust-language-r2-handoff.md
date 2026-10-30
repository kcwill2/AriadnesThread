# R-2 Handoff - rust-language: spine, part lang-api (AggregationKind::ModDeclaration + advisory row A6)

Milestone: `rust-language` (UNREGISTERED post-M28 track, NO M-number). Hand 2
of 10, serial. Edits ONLY `crates/lang-api`: the planning-vocabulary append
(shape sec. 1: "one appended planning-vocabulary variant in `lang-api`") and
amendment A6. Start rev: your dispatch head (R-1's landing + its verdict,
your duty 1). Author rev: the planning commit above `5aa191e`.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/rust-language-shape.md` sec. 2 INV-12, sec. 3.1 (the `AggregationKind`
   append line), sec. 3.3 (why `ModDeclaration` is the honest value), kickoff
   L5/L11, amendment A6.
3. Your files, current state: `crates/lang-api/src/modeling.rs` (204 lines;
   `AggregationKind` enum at :16) and `crates/lang-api/src/identity.rs` (52
   lines; A6 target `language_of_file_kind_maps_every_file_kind_exhaustively`
   at :34-51, cases array :36-43).
4. Precedent for the serde-index doc idiom: `crates/repo/src/identity.rs`
   `WireLanguageForm` doc (frozen-index prose; READ ONLY - do not edit repo).
5. `python tools/checkpoint_guard.py resume --spec
   tasks/rust-language-r2-handoff.md --scope crates/lang-api --cmd ...` boot
   discipline if you are a successor session; otherwise `status` at bootstrap.

## Duty 1 (lands BEFORE any R-2 mutation)
Land R-1's verdict + ledger flip as your first commit: stage ONLY
`tasks/rust-language-r1-verdict.md` (plus the one-line ledger edit) via
`python tools/distill.py -- git add -- tasks/rust-language-r1-verdict.md
tasks/milestone_ledger.md`, `python tools/encoding_guard.py --staged` (silent
exit 0), then `python tools/commit_scoped.py --scope
tasks/rust-language-r1-verdict.md --scope tasks/milestone_ledger.md -m
"review(rust-language): R-1 APPROVED verdict + ledger flip (duty 1)"`. The
ledger edit: in the `Rust language plugin` row append a dated clause:
`R-1 LANDED <SHA> APPROVED <date>, census anchor <canonical line> @ <SHA>,
R-2 dispatched.` Journal both SHAs. Out of your review scope as a hand - the
verdict you land must already be adjudicated APPROVED by the reviewer hop.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/rust-language-r2-journal.md (create)
- tasks/rust-language-r1-verdict.md + tasks/milestone_ledger.md (duty 1 only,
  nothing else)
- crates/lang-api/src/modeling.rs (enum append + index-stability pin)
- crates/lang-api/src/identity.rs (the A6 row ONLY - the `cases` array gains
  one tuple; the :15 re-export list stays byte-identical)

You MUST NOT touch: anything under `crates/repo/`, `crates/lang/**`,
`crates/lang-rust/**`, `crates/cli/**`, `tests/**`, `tools/**`. No
Cargo.lock delta (std-only, no new deps - if cargo rewrites it, revert).
No `git stash`, no branch change.

## What to build (shape sec. 3.1 - the contract)

1. `modeling.rs`: APPEND the new variant LAST after `None` (INV-12):
   `pub enum AggregationKind { Header, PackageObject, None, ModDeclaration }`
   plus a short doc on the new variant: Rust's `mod x;` + sibling-file
   mechanism (shape sec. 3.3 reason column), and a frozen-indices note:
   pre-existing variants' positional (bincode) indices 0/1/2 are unmoved;
   `ModDeclaration` takes index 3 at the END. Planner-verified: NO production
   `match` on `AggregationKind` exists anywhere (all occurrences are
   construction sites or `#[cfg(test)]` modules) - if your build discovers a
   surprise exhaustive site, that is a STOP-and-report (shape sec. 3.1
   STOP-clause; RISK-AGGKIND), not a silent arm.
2. New in-src pin test
   `aggregation_kind_append_is_serde_index_stable`: serde_json serializes the
   four unit variants to their names (`"Header"`, `"PackageObject"`, `"None"`,
   `"ModDeclaration"`) and the first three keep their historical names -
   message naming INV-12/L11.
3. Amendment A6 in `identity.rs` (:34-51): add the tuple
   `(FileKind::RustSource, LanguageId::Rust),` to the `cases` array (both
   items exist post-R-1; the re-export at :15 needs no `RUST` symbol - the
   expression spelling `LanguageId::Rust` resolves without it, mirroring how
   the array already uses `LanguageId::Cpp`).

## RGB protocol

1. Red - JUSTIFIED RED-SKIP, declared per the SM-1 / ui-extraction-wizard W-1 /
   extraction-consumer-rewrite CW-2 precedent trio (cite all three): no
   assertion can fail against the pre-hand tree without the new variant
   existing, and the A6 row is green-on-landing by construction (the advisory
   arm already maps RustSource -> Rust since R-1 - the row PINS a truth the
   spine shipped, which is exactly what an amendment is for, per the CW-1
   amendment-discipline precedent). Journal the justification plus the
   falsifiability statement: `aggregation_kind_append_is_serde_index_stable`
   fails if the variant is renamed (JSON name changes) or inserted not-last
   (the historical names/order pins catch the reorder through the positional
   contract); the A6 row fails if `from_file_kind`'s arm were later regressed.
2. Green: items 1-3; re-run gates until green.
3. Refactor: doc wording plain; re-run.

## Hygiene (review-enforced)
`modeling.rs` 204 -> ~235, `identity.rs` 52 -> ~54; functions < 100; no banned
names; authored bytes pure ASCII; `python tools/hygiene_check.py` exit 0.

## Frozen locks (commands that prove them)
- L11: the enum diff is a pure append (one variant line after `None` + doc;
  no reordered line in the diff).
- L1-L9: `git diff --name-only <start>..HEAD` (excluding your duty-1 commit)
  is EMPTY outside the touch-set; goldens, m00, m17, lang-cpp, lang-scala,
  tools green UNEDITED via gate 3's census.
- L12: `python tools/encoding_guard.py --all` exit 0.

## Gates (exact commands, timeout 600000 ms each; distilling entry points only)
1. `python tools/testrun.py --distill -p ariadne-lang-api` - clean baseline
   before any mutation; journal the canonical line.
2. `python tools/testrun.py --distill -p ariadne-lang-api` - Green: exit 0,
   passed = baseline + 1 (the new pin), the advisory exhaustive test green
   with its +1 row UNEDITED otherwise.
3. `python tools/testrun.py --distill` - full census; reconcile from the
   journal: expect passed >= R-1 hop + 1, failed 0, ignored 13, suites flat;
   delta attributed by name; anchor carried from your duty-1 ledger clause.
4. `python tools/hygiene_check.py` - exit 0.
5. Commit cycle: `python tools/encoding_guard.py --staged` (or `--all`
   fallback, journal the form) then `python tools/commit_scoped.py --scope
   crates/lang-api/src/modeling.rs --scope crates/lang-api/src/identity.rs
   --scope tasks/rust-language-r2-journal.md -m "feat(lang-api): append
   ModDeclaration to AggregationKind and the A6 advisory row
   (rust-language R-2 spine)"`. Journal rides inside.

Declared gate budget: 4 scoped invocations + commit cycle; well inside the law.

## Discipline clauses
- UTF-8: Write/Edit tools only; pure ASCII; never PowerShell redirection
  writes (AGENTS.md File-Edit Discipline).
- Touch only the touch-set; out-of-scope breakage: report, don't fix.
- Journal checkpoint after each Green gate.
- STOP-on-context-exhaustion; successor runs `checkpoint_guard.py resume
  --spec tasks/rust-language-r2-handoff.md --scope crates/lang-api --cmd ...`.
- Landing SHA relay: report the landing SHA; reflog tip check; R-3's duty 1
  cites this file + that SHA.
- Never pin warning counts; never state a count without its command (F14).

## Report back (final message)
1. Duty-1 SHA + landing SHA + `git show --stat` of the landing. 2. The
  justified Red-skip declaration with its three citations. 3. Green evidence:
  distill digest header + canonical lines for gates 2-3 + capture path. 4. Any
  deviation + reason. 5. Out-of-scope observations. No raw output pasted.

## Reviewer acceptance pass
- `python tools/distill.py -- python tools/bench_compare.py` - RUN fresh
  (`crates/lang-api/src` touched; kickoff sec. 4 fully-run stance; WARNs
  recorded, not attributed - PR-2).
- Per-commit `git show` oldest-first; duty-1 commit exactly the verdict +
  ledger; landing diff exactly the two lib files + journal; enum pure-append
  check; A6 diff is exactly one added array tuple.

Reviewer-gate reference: `review-gate` skill adjudicates. Extra MUSTs: the
:15 re-export list byte-identical; no `crates/repo/**` hunk in the landing
commit; if ANY new exhaustive `AggregationKind` site surfaced, the verdict is
REJECTED pending a planner amendment (RISK-AGGKIND STOP).

Sizing (planner, itemized): variant + doc ~8, pin test ~18, A6 row ~1,
journal ~10 - total ~35-60 produced lines; 4 gates + commit trio; one crate
dir (`crates/lang-api`). No split: the variant and the advisory row are one
vocabulary hop; a hand carrying only one half ships a vocabulary that lies.
