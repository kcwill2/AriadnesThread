# AR-27a0 Handoff - Inert clone-token pre-land: complete copy-semantics ledger (A2, hop 0/3)

## DISPATCH ORDER - AR-27 re-shape v2 (read first)

Three SERIAL hops, created by the RE-RULING in `tasks/ar-27-shape-ruling.md`
(sections R1-R6, appended 2026-10-12 after the T-a stop journal
`tasks/ar-27-ta-handoff-journal.md`, commit `747a266`). Each hop dispatches
only after the previous hop's reviewer APPROVED and landed. NOT parallel-safe:
T-a0 and T-a share `crates/cli/src/registry.rs` (T-a0's clone tokens must
survive T-a's parse rework untouched) and both compile-green the whole
workspace; T-a and T-b share `crates/repo/src/types.rs`,
`crates/repo/src/identity.rs`, `crates/lang-api/src/identity.rs`,
`crates/cli/src/registry.rs`. Sole cargo-heavy lane (STRICT COMPILE-GREEN,
ruling R3) - no other lane may hold in-flight edits in the ledger crates
while T-a0 mutates.

| hop | file | scope | produced lines | gates |
|---|---|---|---|---|
| 0 (this file) | `tasks/ar-27-ta0-handoff.md` | ALL copy-semantics clone tokens, landed INERTLY while `SourceLanguage` is still derived-`Copy`; completeness probe converts the ledger from lower-bound to COMPLETE | ~90-140 | 9 |
| 1 | `tasks/ar-27-ta-handoff.md` | byte-oracle fixtures + home move + dual-format serde (AMENDED: zero riders) | ~430-500 | 10 |
| 2 | `tasks/ar-27-tb-handoff.md` | registry-derived `--lang` + FileKind roles + third-language E2E | ~300-380 | 9 |

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/ar-27-shape-ruling.md` RE-RULING sections R1-R5 (VERBATIM below,
   authority; R6.4 embeds them here) and its section 2 (the positional
   encoding contract - untouched by this hop, cited for context only).
3. `tasks/ar-27-ta-handoff-journal.md` - the T-a STOP journal: its measured
   ledger items (24 production sites / 9 files / 6 frozen-test sites), its
   reverted-probe protocol (method below), and its cli `registry.rs` ledger
   items 2-6 that R3 cites for exact token forms.
4. Each ledger file at its listed lines (below): read the exact expression
   before inserting each token.

## Objective

Pre-land EVERY copy-semantics token that the AR-27a `SourceLanguage =
LanguageId` alias will force, while `SourceLanguage` is still the
derived-`Copy` enum - so every insertion is provably value-inert
(clone-on-`Copy` is identity), the reviewer adjudicates a MECHANICAL
token-only commit, and the ledger's completeness is closed BEFORE the
semantic move lands. Suite test counts and every assertion, literal,
fixture, and byte are IDENTICAL before/after this hop. That identity IS the
acceptance shape.

## The authority (RE-RULING R1-R5, VERBATIM - binding)

> ## R1. Chosen option: B (T-a0 consumer-adaptation hop FIRST), with A's
> enumerated precision embedded as T-a0's ledger.
>
> Land every copy-semantics token FIRST, in `tasks/ar-27-ta0-handoff.md`,
> WHILE `SourceLanguage` is still the derived-`Copy` enum. Justification:
> - Provably-inert riders: while the type is `Copy`, `.clone()` IS the copy
>   (definitionally value-identical). The reviewer adjudicates a mechanical
>   token-only commit instead of reasoning per-site "this clone restores prior
>   semantics" inside a ~500-line semantic move. This is the AR-31
>   characterization-first / AR-16 inert-landing precedent.
> - The ledger lower-bound closes BEFORE the move lands: T-a0's completeness
>   probe (R3 protocol below) surfaces the hidden integration/frozen-test
>   remainder in a throwaway Copy-strip world where they compile. Under option
>   A those sites surface MID-T-a (the integration package only compiles once
>   the cli lib is fixed - i.e. after the move), guaranteeing mid-hop STOPs.
> - Planning-Law sizing (axis = gate count x output volume): T-a0 is ~35-45
>   single-token edits + ~9 tiny gates (worst under budget); T-a sheds its
>   rider section and returns to a pure move+serde+oracles unit; T-b untouched.
>   Option A stayed within the 1200-line bound too, but it fused two review
>   objects (semantic move + frozen-territory token class) into one commit set,
>   and it kept the unbounded ledger live during the semantic move.
>
> ## R2. FROZEN-TEST-TOKEN CLASS RULING (the sharp question), binding class law:
>
> Compiler-forced copy-semantics TOKEN insertions into byte-frozen gate-lock
> test files are COMPILE-ADAPTATION RIDERS, NOT lock amendments. The "LOCK
> AMENDMENTS: NONE" clause governs what a lock ASSERTS (bytes, literals,
> expectations, ordering, wording, counts); it is not violated by a value-
> FETCH token that cannot change the fetched value - provable, because the
> tokens land while the type is derived-`Copy` (clone-on-`Copy` is identity).
> This generalizes the accepted AR-24-D2 (mock-signature token) / AR-25-D1
> (render token) precedents from precedent to class.
>
> Hard constraints (all reviewer-blockable):
> 1. Token forms ONLY: `x` -> `x.clone()`; `*x` -> `x.clone()`;
>    `x.into()` -> `x.clone().into()`; `From::from(x)` -> `From::from(x.clone())`
>    (likewise `LanguageId::from(x)`). NO `&`-iteration rewrites, NO signature
>    or pattern surgery, NO reordering, NO logic. Uniform clone form -
>    per-site `&`-cleverness is explicitly ruled OUT for reviewability.
> 2. Zero edits to any literal, assertion, expectation, fixture, count, or
>    ordering in any file; suite test counts identical before/after.
> 3. Sites must appear in the contract's site ledger; probe-surfaced additions
>    are appended to the ledger in the journal BEFORE commit, same class.
> 4. Reviewer verification recipe (token-diff class check): for each named
>    frozen file, `git show <T-a0-commit> -- <file>` - every changed +/- pair
>    must be token-normalizable: after deleting inserted `.clone()`s and
>    restoring `*x` for `x.clone()` rewrites, the pair is byte-equal. Any pair
>    that is not = REJECT the hop. Quote per-file outputs in the verdict.
> 5. Affected suites run UNEDITED-assertions at identical counts: pipeline
>    `40/0/4/9` (contains `for_incremental_parity_test` `5/0/0/1`),
>    m23-symbol `7/0/0/1`, the m22 suite, cli `118/0/8/7` - plus
>    `-p ariadne-build` and `-p ariadne-artifact` counts MEASURED pre-mutation
>    and pinned by the reviewer at T-a0 (never pinned before).
>
> Amended section 7 (restated): still ZERO edits to committed BYTES and ZERO
> changes to what any lock asserts; the only sanctioned tracked-file edits are
> the ledger-enumerated value-invariant copy tokens of constraint 1, per R2.
>
> ## R3. T-a0 contract shape (planner authors `tasks/ar-27-ta0-handoff.md`):
>
> Objective: pre-land every copy-semantics site the AR-27a alias will force,
> inertly, before any type change. Sole cargo-heavy lane; STRICT COMPILE-GREEN.
>
> Touch-set = EXACTLY this ledger (12 files; token-class only) - this list
> SUPERSEDES the 6-site cap of section 4 and is the named out-of-set boundary:
> - `crates/build/src/discovery.rs` :253 `ctx.language = record.language;` ->
>   `record.language.clone()`; :272 `language: record.language,` ->
>   `record.language.clone()`. (Verified expression-level reads; the ADAPT-8
>   comment block :245-252 stays BYTE-UNEDITED and its claim remains true
>   through the alias - the clone moves neither type nor bytes.)
> - `crates/artifact/src/manager/discovery_io.rs` :51, :130, :131 ->
>   `.map(|f| f.language.clone())` / `.map(|c| c.language.clone())` /
>   `.map(|tu| tu.language.clone())` (clone form, NOT `&`-rewrite - R2.1).
> - `crates/artifact/src/manager/m22_manifest_languages_tests.rs` :39, :154,
>   :169 -> `*lang` -> `lang.clone()` (src test-mod in a MUST-NOT crate; enters
>   only as a named T-a0 token file).
> - `crates/pipeline/src/stage_index.rs` :110, :122 ->
>   `self.language_filter.clone()`.
> - `crates/pipeline/src/stage_load.rs` :183 -> `language.clone()`.
> - `crates/pipeline/src/for_incremental.rs` :66 `LanguageId::from(language
>   .clone())` (unblocks the :67 `{language:?}`), :85 clone inside the retain
>   closure, :121 `ensure_language_registered(&registry, language.clone())`,
>   plus one clone among the remaining uses (:125/:141/:143) so every use
>   except the last clones; journal the exact token set (compiler-verified in
>   the probe).
> - `crates/pipeline/src/lib.rs` :681 -> clone.
> - `crates/cli/src/registry.rs` :28 (`from(language.clone())`, fixes the :29
>   bail E0382), :174-175, :192, :448 `*language` -> `language.clone()`, :520
>   - exact tokens per the T-a journal ledger items 2-6.
> - `crates/cli/src/commands/analyze.rs` :93, :143, :148, :154, :169, :262;
>   `crates/cli/src/commands/index.rs` :77; `crates/cli/src/commands/ui/
>   node_maps.rs` :59 `file.language.clone()` (entry key), :62 `*language` ->
>   `language.clone()`.
> - FROZEN existing test files, R2-class tokens:
>   `crates/pipeline/tests/for_incremental_parity_test.rs` :98, :114, :153,
>   :163; `tests/integration/tests/m22_scala_load_analysis_test.rs` :280
>   `record.language.clone().into()`; `tests/integration/tests/
>   m23_symbol_isolation_test.rs` :406 `LanguageId::from(record.language
>   .clone())`.
> - Probe-surfaced remainder (journal's lower-bound delta): same class, same
>   forms, appended to the ledger.
> - NOT touched in T-a0: `crates/repo/**`, `crates/lang-api/**`
>   (identity.rs's `&`-From body is part of T-a's move/rewrite),
>   `crates/worker-core/**` (tu_store:210 no-op CONFIRMED by probe - ruling
>   claim upheld), every other file, all goldens.
>
> Method (three-step, the journal's own reverted-probe protocol as the oracle):
> 1. Land the ledger tokens (they compile inertly at today's Copy type).
> 2. COMPLETENESS PROBE: strip `Copy` from the `SourceLanguage` derive list
>    (throwaway, NEVER committed; throwaway tokens allowed ONLY in
>    repo/lang-api/cli-registry-owned files that T-a will own - those sites are
>    committed by T-a, not T-a0); run `cargo check --workspace --all-targets`;
>    iterate until probe-world reports ZERO copy-semantics errors; revert the
>    probe byte-clean (`git status --porcelain -- crates` empty apart from the
>    committed tokens). This converts the ledger from lower-bound to complete.
> 3. Real-world gate set (<=9 invocations): all-targets workspace check green;
>    affected-crate suites at exact counts (pipeline 40/0/4/9, cli 118/0/8/7,
>    build/artifact MEASURED+pin, index 15/0/0/2 UNEDITED digest proof,
>    m22+m23+for_incremental_parity identical counts); encoding_guard
>    --all/--staged; hygiene zero-blocking; the R2.4 token-diff outputs for
>    the three frozen test files quoted in the journal. Single scoped commit
>    (commit_scoped, explicit --scope per file).
>
> Sequencing: T-a0 APPROVED -> T-a (move+serde; MUST compile-green with ZERO
> rider edits; newly surfaced SAME-class copy site = clone-token addition +
> journal, no re-ruling; any site needing non-token adaptation = STOP-AND-
> REPORT) -> T-b unchanged. Clippy note: pre-landed clones add transient
> `clone_on_copy` warnings while the enum is Copy; CI does not gate warnings
> (ci.yml:28-30, the `-D warnings` flip is a tracked future milestone) and
> AR-28 owns the clone-on-Copy sweep; warnings vanish for these sites when the
> alias lands.
>
> ## R4. Option sanity-checks (reaffirmed):
>
> - Option C (Copy-able/interned identity) REJECTION STANDS, now at measured
>   cost ~28-45 inert tokens: an interned/Copy representation needs a process-
>   global interner, changes `LanguageId::new(impl Into<Cow<'static, str>>)`
>   semantics that T-b's `--lang` user-token depends on, and fights the
>   load-bearing owned-parse lock (`language_id_round_trips_..._json` parses
>   `"Fortran"` to an owned id). Redesign-at-last-mile is strictly riskier than
>   inert tokens; do not re-litigate.
> - Option D (epoch bump) remains NOT AN OPTION: copy-loss is a compile-time
>   class; bytes and epochs are untouched by this problem.
>
> ## R5. Erratum into the record (T-a journal section 6, upheld):
>
> Section 4 of this ruling cited stage_index ":103/:110" as the by-value
> borrow sites. Corrected: the E0507 sites are `:110` AND `:122`; `:103`
> (match on `self.language_filter` with discriminant-only `Some(_)` pattern)
> does NOT move and needs no token. Blessing a non-site would have made the
> reviewer's token-diff check fail. (`worker-core tu_store:210` no-op and
> `metrics_io:241` riding the kept `From<&T>` were both CONFIRMED by the
> probe - those ruling claims stand.)

Ledger count note (disclosure): R3 labels the ledger "12 files"; the
enumerated paths above are FOURTEEN (11 source + 1 artifact src test-mod +
2 frozen integration + 1 frozen pipeline test file counts as the
`m22_manifest_languages_tests.rs` inclusion). The reviewer enforces the
EXPLICIT PER-PATH list, not the count. If the probe appends further
same-class sites (R3 bullet "Probe-surfaced remainder"), they enter the
ledger in the journal BEFORE commit and join the commit scope - same class,
same forms, still token-only.

## Baselines (re-measure and journal BEFORE any mutation at YOUR dispatch
HEAD; the workspace census vs anchor `1400/0/13/105` is the reviewer's
acceptance gate, not yours)

- `-p ariadne-pipeline`: `40 passed / 0 failed / 4 ignored / 9 suites`
  (R2.5), containing `for_incremental_parity_test` `5/0/0/1`.
- `-p ariadne-cli`: `118 passed / 0 failed / 8 ignored / 7 suites`.
- m23-symbol-isolation `7/0/0/1`; the m22 suite (resolve exact file/count by
  run, pin it); index `15/0/0/2` UNEDITED (this is ALSO the digest proof that
  the ADAPT-8 territory stayed byte-clean - `cache.rs` is not touched at all
  this hop).
- `-p ariadne-build` and `-p ariadne-artifact`: NEVER PINNED before -
  MEASURE and pin in the journal pre-mutation (R2.5); the reviewer re-pins
  them at T-a0.
- `python tools/hygiene_check.py` ZERO blocking rows;
  `python tools/encoding_guard.py --all` clean.

## Touch-set (hard) = the R3 ledger, EXACTLY

Plus `tasks/ar-27-ta0-handoff-journal.md` (checkpoint journal; always
in-set). The full per-path list for commit_scoped enforcement:
- `crates/build/src/discovery.rs` (tokens :253, :272 ONLY; ADAPT-8 comment
  block :245-252 BYTE-UNEDITED - gate 5's index digest suite proves the
  digest territory stayed clean and the `git show` token-diff proves the
  comment)
- `crates/artifact/src/manager/discovery_io.rs` (:51, :130, :131)
- `crates/artifact/src/manager/m22_manifest_languages_tests.rs` (:39, :154,
  :169 - named inclusion in a MUST-NOT crate, R2-class only)
- `crates/pipeline/src/stage_index.rs` (:110, :122 - NOT :103, R5 erratum)
- `crates/pipeline/src/stage_load.rs` (:183)
- `crates/pipeline/src/for_incremental.rs` (:66, :85, :121, + one among
  :125/:141/:143 so every use except the last clones; journal the set)
- `crates/pipeline/src/lib.rs` (:681)
- `crates/cli/src/registry.rs` (:28, :174-175, :192, :448, :520 - exact
  tokens per T-a journal ledger items 2-6)
- `crates/cli/src/commands/analyze.rs` (:93, :143, :148, :154, :169, :262)
- `crates/cli/src/commands/index.rs` (:77)
- `crates/cli/src/commands/ui/node_maps.rs` (:59, :62)
- FROZEN (R2-class, token-normalizability enforced by the reviewer's R2.4
  recipe): `crates/pipeline/tests/for_incremental_parity_test.rs` (:98,
  :114, :153, :163), `tests/integration/tests/m22_scala_load_analysis_test.rs`
  (:280), `tests/integration/tests/m23_symbol_isolation_test.rs` (:406)
- probe-appended same-class sites (journal BEFORE commit)

You MUST NOT touch: `crates/repo/**` (the probe's throwaway `Copy`-strip on
the derive list at `crates/repo/src/types.rs:141` is the ONE probe-world
edit, reverted before commit and attested - it is NEVER committed),
`crates/lang-api/**` (same throwaway-token allowance as repo per R3; T-a
owns those files), `crates/worker-core/**` (tu_store:210 no-op CONFIRMED),
`crates/index/**` (digest territory), `crates/report/**`, `crates/ui/**`,
`crates/graph/**`, `crates/cluster/**`, `crates/transform/**`,
`crates/validate/**`, `crates/lang/**`, `crates/lang-scala/**`,
`crates/lang-cpp/**`, every other existing file under `tests/**` and
`crates/*/tests/**` beyond the three named frozen files, all goldens, all
`tasks/` files except your journal.

## Method (RGB shape of this hop: the probe IS the completeness oracle)

1. Journal bootstrap: dispatch HEAD + all Baselines lines (build/artifact
   PIN lines included) BEFORE touching anything.
2. Land the ledger tokens per the quoted R3 list: read each expression
   first, insert ONLY the ruled token form (R2.1). All-targets check stays
   GREEN (inert: the type is `Copy` today, clones are identity).
   `clone_on_copy`-style warnings are EXPECTED and transient (R3 Sequencing
   note; AR-28 owns the sweep) - do NOT silence them, do NOT gate on them.
3. COMPLETENESS PROBE (throwaway, per the quoted R3 method): strip `Copy`
   from the `SourceLanguage` derive list; `cargo check --workspace
   --all-targets` via distiller; enumerate EVERY copy-semantics error site
   (E0507/E0382 class); ledger-completing tokens go into the real files
   (append to the journal ledger BEFORE commit, R2.3); throwaway tokens are
   allowed ONLY in repo/lang-api/cli-registry files T-a owns (never
   committed); iterate until probe-world reports ZERO copy-semantics
   errors.
4. REVERT + ATTEST: restore `crates/repo/src/types.rs` byte-clean
   (`python tools/distill.py -- git diff -- crates/repo crates/lang-api` ->
   EMPTY; lang-api only if you used a throwaway token there);
   `python tools/distill.py -- git status --porcelain -- crates` must list
   ONLY the ledger files, nothing else; quote both outputs in the journal
   (this is the deleted-before-commit attestation for the probe).
5. Real-world gates below; single scoped commit.

## Gates (exact commands, timeout 600000 ms each; <=9 invocations; scoped
only - the workspace census vs `1400/0/13/105` is the reviewer's)

1. `python tools/distill.py -- cargo check --workspace --all-targets` -
   GREEN (real world, post-probe-revert).
2. `python tools/testrun.py --distill -p ariadne-pipeline` -
   `40/0/4/9` IDENTICAL (incl. `for_incremental_parity_test` `5/0/0/1`).
3. `python tools/testrun.py --distill -p ariadne-cli` - `118/0/8/7`
   IDENTICAL.
4. `python tools/testrun.py --distill -p ariadne-build` and
   `python tools/testrun.py --distill -p ariadne-artifact` - IDENTICAL to
   your pinned pre-mutation lines.
5. `python tools/testrun.py --distill -p ariadne-index` - `15/0/0/2`
   UNEDITED (ADAPT-8 digest proof).
6. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m23_symbol_isolation_test --test m22_scala_load_analysis_test` -
   `7/0/0/1` + your pinned m22 line, counts IDENTICAL (assertions UNEDITED
   except the two ruled tokens).
7. `python tools/hygiene_check.py` - exit 0, zero blocking rows;
   `python tools/encoding_guard.py --staged` - exit 0 before the commit.
8. `python tools/encoding_guard.py --all` - exit 0, journal the run.
9. Token-diff evidence (R2.4, journal-quoted): for the three frozen test
   files run `python tools/distill.py -- git diff -- <file>` BEFORE commit
   (working-tree form of the same check the reviewer will run post-commit
   with `git show`) and journal that every +/- pair is token-normalizable
   (delete `.clone()` / restore `*x` -> byte-equal).

Commit (ONE, commit_scoped with EXPLICIT per-file --scope - never `git add .`,
no globs):
`python tools/commit_scoped.py --scope crates/build/src/discovery.rs --scope crates/artifact/src/manager/discovery_io.rs --scope crates/artifact/src/manager/m22_manifest_languages_tests.rs --scope crates/pipeline/src/stage_index.rs --scope crates/pipeline/src/stage_load.rs --scope crates/pipeline/src/for_incremental.rs --scope crates/pipeline/src/lib.rs --scope crates/cli/src/registry.rs --scope crates/cli/src/commands/analyze.rs --scope crates/cli/src/commands/index.rs --scope crates/cli/src/commands/ui/node_maps.rs --scope crates/pipeline/tests/for_incremental_parity_test.rs --scope tests/integration/tests/m22_scala_load_analysis_test.rs --scope tests/integration/tests/m23_symbol_isolation_test.rs --scope tasks/ar-27-ta0-handoff-journal.md -m "refactor: AR-27a0 inert clone-token pre-land - complete ledger via Copy-strip probe (T-a0)"`
(plus one `--scope <path>` per probe-appended ledger file, journal-listed).

## Hygiene (review-enforced)

Token-only diff: no function crosses the 100-line ceiling from a `.clone()`
(single-expression inserts); no new files except the journal;
`hygiene_check.py` ZERO blocking rows unchanged.

## Frozen locks (LOCK AMENDMENTS: NONE for what locks ASSERT - amended
section 7, R2)

The three frozen test files enter ONLY as R2-class token files; every other
authority lock (m23-bincode `5/0/0/1`, m00 `4/0/0/2`, m24 `8/0/0/1`,
m24a `2/0/0/1`, m18 `23/0/0/1`, ui `272/0/0/3`, lang-api `41/0/0/4`, repo
`15/0/0/2`) stays UNTOUCHED this hop - not even tokens (their sites are not
in the ledger; the probe certifies zero). No git mv anywhere (D7 clause
trivially satisfied; `git show --stat` must show ONLY ledger paths).

## Discipline clauses

- UTF-8: Write/Edit tools only; pure ASCII.
- Token forms ONLY (R2.1 verbatim above). Any site that seems to need more
  than a token: STOP-AND-REPORT (that is a T-a-class adaptation, not T-a0).
- The probe NEVER lands: `crates/repo/src/types.rs` derive line restored,
  attestation quoted in the journal (gate 4 of the Method).
- Distilling entry points only; canonical lines from OWN stdout (capture-
  race ruling). Refused-command substitutions on this lane: raw `git
  status`/`git grep`-chains/`python -c`/chained-echo are allowlist-refused -
  wrap single commands via `python tools/distill.py -- ...`; `testrun.py`
  takes `--test <suite-name>` (no `-n`, no `--help` - read its docstring);
  adjudicate exit codes from printed contracts (T-a journal section 10
  precedent).
- Journal-incremental SMALL steps (1-3 lines per gate: probe iteration
  counts included). Context looming -> STOP with disk-state inventory; a
  dead session is NEVER resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/ar-27-ta0-handoff.md
  --scope ... --cmd ...`.
- Sole cargo-heavy lane until the single commit lands.

## Report back (final message)

1. The ONE commit hash + `git show --stat` (distilled) - must list ONLY
   ledger paths (+journal). 2. Ledger delta: probe-appended sites vs the
   R3 list (or "NONE - ledger was complete"). 3. Probe attestation outputs
   (Method step 4, both quoted). 4. Gate 2-6 canonical lines IDENTICAL to
   baselines, verbatim from own stdout. 5. Token-diff normalizability lines
   for the three frozen files (gate 9). 6. Any deviation + reason. 7.
   Out-of-scope observations (report-only). Never paste raw tool output.

Reviewer-gate reference: acceptance via the `review-gate` skill PLUS the
R2.4 token-diff class check run per named frozen file (`git show
<T-a0-commit> -- <file>`; every +/- pair token-normalizable or REJECT the
hop; quote per-file outputs in the verdict), commit-vs-touch-set enforcement
against the explicit per-path ledger list above, and the build/artifact
re-pin. On APPROVAL: T-a dispatches (amended contract, zero riders).
