# AR-27b Handoff - Registry-derived `--lang` open set + FileKind role refactor + third-language E2E (A2, hop 2/2)

## DISPATCH (read first)

DISPATCH ONLY after `tasks/ar-27-ta-handoff.md` is reviewer APPROVED and
LANDED. The hops are SERIAL and share `crates/repo/src/types.rs`,
`crates/repo/src/identity.rs`, `crates/lang-api/src/identity.rs`, and
`crates/cli/src/registry.rs` - never parallel. This hop's RED-first E2E lock
depends on T-a's landed closed-set `parse_language_arg("Fortran") -> Err`
and on T-a's escape-arm serde. Sole cargo-heavy lane (STRICT COMPILE-GREEN,
ruling section 5 dispatch discipline). Chain origin: ruling
`tasks/ar-27-shape-ruling.md` (Option B as amended); stop evidence
`tasks/ar-27-handoff-journal.md`; superseded contract
`tasks/ar-27-handoff.md` (bannered). Sizing: ~300-380 produced lines, 9
gated invocations (Planning Law compliant).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/ar-27-shape-ruling.md` sections 5.6-5.7 and 7 (quoted VERBATIM
   below; authority).
3. `tasks/ar-27-ta-handoff.md` + its landed journal `tasks/ar-27-ta-handoff-
   journal.md` (rider ledger, landed suite counts - your baselines come
   FROM the T-a journal, re-measured at YOUR dispatch HEAD).
4. `crates/cli/src/lib.rs` :146-148 and :179-181 (the hardcoded
   `(cpp|scala|other)` help text you make registry-derived);
   `crates/cli/src/registry.rs` POST-T-A (parse closed set +
   `ensure_language_registered` :24-32 with
   `language plugin not registered: {language:?}`).
5. `crates/lang/src/registry.rs` :122-125 (`languages()` - registration
   order is the contract).
6. `crates/repo/src/types.rs` (FileKind :31-72 post-T-a; `from_file_kind`
   now on the newtype in `crates/repo/src/identity.rs`);
   `crates/repo/src/discovery.rs` :89 (tags `language` from `FileKind`).
7. `crates/lang-api/src/identity.rs` (`language_of_file_kind` + exhaustive
   unit test).
8. `tests/integration/tests/m17_plugin_seam_test.rs` :326-328 (R2 spine
   carve-out making path-taxonomy tagging legal in `repo`).
9. AR-18/AR-19 mock-plugin precedents in `tests/integration/tests/` (the
   E2E mock registry shape).

## Objective

Make `--lang` an OPEN set derived from the plugin registry (byte-identical
rendering at the default registry), refactor `FileKind` to language-agnostic
roles with compile-compatibility bridges (or the ruled fallback), and land
the RED-first third-language E2E lock proving: detect -> tag -> persist ->
`--lang <id>` select with ZERO core edits. Invariants: zero committed-byte
movement (guard-proven), all authority locks UNEDITED both sides, epoch pin
stays 2.2.0, registry stays the language authority (D-A6), NO `repo ->
registry` dependency introduced.

## Baselines (re-measure and journal BEFORE any mutation at YOUR dispatch
HEAD; the workspace census vs anchor `1400/0/13/105` is the reviewer's
acceptance gate, not yours)

- Authority locks ALL UNEDITED, both hops (ruling section 1): m23 bincode
  file `5/0/0/1` (in lang-api), m23 symbol-isolation `7/0/0/1` (pin the
  split!), m00 `4/0/0/2`, m24 `8/0/0/1`, m24a `2/0/0/1`, m18 `23/0/0/1`,
  repo `15/0/0/2`-as-rebaselined-by-T-a, cli `118/0/8/7`-as-rebaselined,
  lang-api `41/0/0/4`-as-rebaselined, ui `272/0/0/3`.
- `python tools/distill.py -- python tools/ui_smoke.py` -> `ui_smoke: PASS`
  REQUIRED this hop (payload serde surfaces).
- FileKind consumers that must compile UNEDITED via the bridges (measured
  green, suite UNEDITED): m19 (`m19:114` site), m23 symbol
  (`m23:145` site), `crates/lang-scala/src/shaper.rs:45`; plus m22 manifest
  byte lock and m24a (ruling: keep m00/m19/m22/m24a green). Exact suite
  names by glob of `tests/integration/tests/`, journalled.
- `python tools/hygiene_check.py` ZERO blocking rows; `encoding_guard
  --all` clean.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/ar-27-tb-handoff-journal.md` (checkpoint journal; always in-set)
- `crates/repo/src/types.rs` (FileKind role refactor per the ruling quote
  below: variant rename + associated-const bridges + `#[serde(alias = ...)]`
  + `from_path` retagging + `from_file_kind` role table)
- `crates/repo/src/discovery.rs` (:89 - tags `language` from PATH taxonomy
  per ruling; named in-crate extension; NO registry dependency)
- `crates/lang-api/src/identity.rs` (`language_of_file_kind` rewritten to
  match on the NEW variant names - the bridge consts compile as
  EXPRESSIONS but are not legal `match` PATTERNS; its exhaustive unit test
  rewritten through the bridge names with UNCHANGED expectations:
  `language_of_file_kind` per-extension behavior must be observable-
  identical to today)
- `crates/cli/src/lib.rs` (`--lang` help: doc-comment closed list replaced
  by a clap `help = <expr>` evaluated at Command-build from the registry -
  feasibility banked in journal section 7; rendered text at the default
  registry byte-identical to today)
- `crates/cli/src/registry.rs` (parse path becomes registry-candidate
  matching; closed-set authority moves to `ensure_language_registered`;
  existing cli locks UNEDITED unless the lock file is one YOU add below)
- NEW test files only (existing files byte-frozen):
  - `tests/integration/tests/ar27_open_language_e2e_test.rs` (E2E lock +
    registry-candidate help locks; single new file preferred)
  - `crates/cli/tests/ar27_lang_help_byte_lock_test.rs` (default-registry
    help + unknown-id Err string byte-locks)

You MUST NOT touch: `crates/pipeline/**`, `crates/worker-core/**`,
`crates/index/**`, `crates/build/**`, `crates/artifact/**`,
`crates/report/**`, `crates/ui/**`, `crates/lang/src/**` (read-only
reference), `crates/lang-scala/**` (shaper:45 compiles UNEDITED via the
bridge - that is the proof, not an edit target), `crates/repo/src/identity.rs`
(newtype/serde stays as T-a landed it), EVERY existing file under `tests/**`
and `crates/*/tests/**` (suites and goldens UNEDITED), all
`tasks/ar-27-*` contract/ruling/journal files except your own journal. No
`git stash`, no branch change.

## Step 0 - FileKind byte-guard pre-grep (HARD, before any FileKind edit;
journal the raw hits or their absence)

The journal proved zero committed JSON carries `"ScalaSource"`/
`"ScalaScript"`/`"BuildDefinition"`; T-b RE-PROVES, binary fixtures
included (ruling section 5.6):
1. Text sweep: `python tools/distill.py -- git grep -I -l -e ScalaSource
   -e ScalaScript -e BuildDefinition -- tests/ crates/` and classify hits:
   source/test CODE mentions are legal; committed FIXTURE/manifest/golden
   mentions are guard-dirty.
2. Binary sweep: enumerate tracked `*.bin`/golden fixture files under
   `tests/` and `tests/fixtures/` and scan for FileKind variant strings AND
   for `files.bin`-shaped records whose `kind` byte index region would
   shift; journal the enumeration and result.
Clean -> GUARD-CLEAN shape below. Any committed-byte hit -> GUARD-DIRTY
fallback shape below (ruled now so no one improvises). Hits ambiguous ->
STOP-AND-REPORT with the file list.

## The FileKind ruling (section 5.6, VERBATIM - binding)

> FileKind role refactor is RULED INTO T-B (the journal proposed T-a;
> adjusted). Rationale: T-a's value is pure byte-stability and FileKind is
> inert there; the refactor's justification and its E2E lock are T-b's; the
> files.bin variant-index shift is contained by T-b's byte-guard: pre-grep
> ALL tracked fixtures, binary included, for committed FileKind bytes
> ("ScalaSource"/"ScalaScript"/"BuildDefinition") - the journal proved zero
> for committed JSON; repeat for `.bin` goldens + keep m00/m19/m22/m24a
> green.
> - Guard clean: rename to language-agnostic roles (Header, Source, Script,
>   BuildDefinition, Other) + associated-const bridges (`FileKind::ScalaSource
>   = Source`, `FileKind::ScalaScript = Script`) so m19:114, m23:145,
>   lang-scala shaper:45 compile UNEDITED + `#[serde(alias = "ScalaSource")]`
>   / `#[serde(alias = "ScalaScript")]` for warm JSON manifest reads (JSON
>   only - aliases are self-describing-format mechanisms; the journal's own
>   finding). New manifests write the role names for Scala files - ruled
>   consequence, zero committed-byte impact (grep-proven class).
> - Guard dirty (any committed byte golden found): FALLBACK SHAPE, ruled now
>   so no one improvises: FileKind byte shape frozen (5 variants stay),
>   `FileKind::role()` added, per-language variants treated as spine
>   taxonomy data, open-set language authority stays
>   `registry.detect_language` (D-A6, identity.rs:133-141). Same E2E lock,
>   zero byte movement.
> - Touch-set extension (named, in-crate): `crates/repo/src/discovery.rs`
>   (:89) tags language from PATH taxonomy (m17 carve-out makes the spine
>   legal); NO repo->registry dependency may be introduced. repo suite
>   rebaselines in the T-b re-brief.

Mechanical consequences the guard-clean shape forces (implementer notes, not
redesign): under the rename, `.scala`/`.sc` tag as `Script` and `.sbt`/
`.mill` as `BuildDefinition`, so `from_file_kind` yields Scala through the
roles exactly as today through the per-language variants, and
`language_of_file_kind` stays observable-identical per extension. The
variant-index shift is `BuildDefinition` 4->3 (and `ScalaScript` 3->2
collapsing into `Script`): exactly the region your binary sweep guards.

## The `--lang` open-set ruling (section 5.7, VERBATIM - binding)

> `--lang` open set: candidates from `registry.languages()`
> (lang/src/registry.rs:123, registration order). At the DEFAULT registry the
> rendered text stays byte-identical to today's "(cpp|scala|other)"; newly
> registered plugin ids append in registration order (new lock: third
> language appears when a mock plugin registers it). Closed-set authority
> becomes `ensure_language_registered` (already bails with
> `language plugin not registered: {language:?}`; wording may name the
> registry candidates in T-b, in-set). RED-FIRST E2E lock: mock third
> language flows detect -> tag (registry path) -> persist (bincode escape
> round-trip + JSON bare name) -> `--lang <id>` select with ZERO core edits;
> RED today = the `parse_language_arg("Fortran")` Err plus absence of an
> open-id persist path.

Lock shapes (exact expectations):
- HELP BYTE-LOCK (characterization, GREEN-FIRST like T-a's oracles): the
  rendered `analyze --lang` and `index --lang` help lines captured from the
  UNTOUCHED cli and embedded as inline constants; committed GREEN in its
  own commit BEFORE the refactor; the same assertions stay green after.
  At the default registry the unknown-id `Err` string also stays
  byte-identical to T-a's wording (lock it in the same file).
- OPEN-SET LOCKS (new registry, mock third plugin "Fortran"): rendered
  candidates include `fortran` appended in registration order;
  `--lang Fortran`/`--lang fortran` parse to the registered id; unknown id
  still `Err` naming the extended candidate set (wording may extend per
  ruling).
- E2E LOCK (RED-FIRST, the ruling's chain): mock plugin registered onto a
  registry built through the existing public seams (AR-18/19 mock-plugin
  precedent) - a `.f95` file detects to `Fortran` via
  `registry.detect_language` (tag via the registry path), a record tagged
  `Fortran` bincode round-trips through the T-a escape (`03 00 00 00` +
  u64 LE len + UTF-8) and JSON bare-name `"Fortran"`, and `--lang fortran`
  selection accepts the id with ZERO core edits. Capture the RED run first
  (expect: today's `parse_language_arg("Fortran")` Err plus a compile-level
  absence of the open-id persist path - journal the failing-test list
  verbatim BEFORE implementing).

## RGB protocol

1. Bootstrap + journal (dispatch HEAD, baselines above, T-a landed hashes).
2. BYTE-GUARD sweep (Step 0) - journal evidence; route guard-clean/dirty.
3. HELP BYTE-LOCK committed GREEN pre-refactor (own scoped commit).
4. RED: E2E open-set lock authored, capture failing-test list (the
   predicted RED named above), journal verbatim.
5. GREEN: FileKind roles (or ruled fallback), discovery tagging,
   `language_of_file_kind` rewrite, registry-derived help + parse; full
   gate set.
6. Refactor: honest names, doc comments stating bridges are JSON-alias +
   const-expression only (NOT bincode-safe: bincode is positional - cite
   the journal's finding). Re-run.

## Hygiene (review-enforced)

Functions < 100 lines; the help-expression must not park a paragraph
(doc/args stay tight); no banned names. `encoding_guard --all` clean;
`hygiene_check.py` exit 0 ZERO blocking rows.

## Frozen locks (LOCK AMENDMENTS: NONE - ruling section 7 VERBATIM)

> ZERO. This ruling sanctions no edit to any tracked golden, lock, or byte-
> authority file. The FileKind serde aliases touch no committed bytes (journal
> grep proof; T-b re-proves including binary fixtures). The epoch pin stays
> 2.2.0. `m24:549`'s stale comment stays unedited (journal section 7).

Machine checks: `python tools/distill.py -- git diff <dispatch-HEAD> --
tests` shows ONLY your one new integration file (additions, never edits);
`python tools/distill.py -- git status --porcelain -- tests/golden` EMPTY;
m19/m22/m24a/m18/m23/m24/m00 suite FILES byte-identical (same diff check
covers them). New manifests writing role names is a ruled consequence with
zero committed-byte impact - the guard sweep is its proof, not a golden
edit.

## Gates (exact commands, timeout 600000 ms each; scoped only)

1. `python tools/distill.py -- cargo check --workspace --all-targets`
   (baseline pre-mutation; STRICT COMPILE-GREEN at every commit).
2. `python tools/testrun.py --distill -p ariadne-cli` - T-a-rebaselined
   count + your new byte-lock file's tests (conservation equation
   journalled); help byte-lock GREEN pre and post the refactor.
3. `python tools/testrun.py --distill -p ariadne-repo` - T-a-rebaselined
   count +D (role rename retires ZERO committed bytes; from_path/
   from_file_kind unit updates), suite UNEDITED baseline recorded.
4. `python tools/testrun.py --distill -p ariadne-lang-api` -
   T-a-rebaselined count EXACT (identity test rewritten through bridges,
   same test count); m23-bincode `5/0/0/1` UNEDITED inside.
5. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   ar27_open_language_e2e_test --test m23_symbol_isolation_test --test
   m00_golden_baseline_test --test m00_facts_path_parity_test` - your E2E
   RED-captured then GREEN, plus `7/0/0/1` + `4/0/0/2` UNEDITED.
6. Same-package UNEDITED re-runs the FileKind surface feeds: the m19, m22,
   m24 (`8/0/0/1`), m24a (`2/0/0/1`), m18 (`23/0/0/1`) suites (names by
   glob; one combined `--test` invocation; journal each canonical line).
7. `python tools/testrun.py --distill -p ariadne-ui` - `272/0/0/3` UNEDITED
   AND `python tools/distill.py -- python tools/ui_smoke.py` ->
   `ui_smoke: PASS` (REQUIRED this hop).
8. `python tools/hygiene_check.py` - exit 0, zero blocking rows.
9. `python tools/encoding_guard.py --staged` before each commit +
   `python tools/encoding_guard.py --all` after - both exit 0.

Commits (commit_scoped, never `git add .`):
- Byte-lock commit:
  `python tools/commit_scoped.py --scope "crates/cli/tests/ar27_lang_help_byte_lock_test.rs" --scope "tasks/ar-27-tb-handoff-journal.md" -m "test(cli): ar27 lang-help + err-string byte locks green PRE-refactor (AR-27b)"`
- Implementation commit:
  `python tools/commit_scoped.py --scope "crates/repo/src/**" --scope "crates/lang-api/src/identity.rs" --scope "crates/cli/src/**" --scope "tests/integration/tests/ar27_open_language_e2e_test.rs" --scope "crates/cli/tests/ar27_lang_help_byte_lock_test.rs" --scope "tasks/ar-27-tb-handoff-journal.md" -m "feat(repo,lang-api,cli): FileKind role refactor + registry-derived --lang open set + third-language E2E (AR-27b)"`
  (guard-dirty fallback: drop nothing from the scopes; the FileKind edit
  inside `types.rs` takes the ruled fallback shape instead).

## Rename-completeness proof clause (AR-20a D7 - conditional)

No `git mv` planned. IF any path is `git mv`'d: `git show --stat -M HEAD` +
`git ls-files <old path>` via distiller immediately after; old path must be
gone, no stray paths in the stat; stray delete -> immediate second scoped
commit + journal.

## Discipline clauses

- UTF-8: Write/Edit tools only; authored content pure ASCII (AGENTS.md).
- Touch only the touch-set. Guard-dirty -> the ruled fallback, NOT an
  improvised third shape. Any hit you cannot classify, any out-of-set break,
  any bridge that fails to compile UNEDITED at m19:114/m23:145/shaper:45 ->
  STOP-AND-REPORT (zero lock edits sanctioned; a contemplated lock edit =
  ruling-level event, ruling section 6's token-class history is NOT a
  license for T-b).
- Distilling entry points only; quote canonical lines from OWN stdout
  (capture-race ruling).
- Refused-command substitutions: raw `git status`/`git grep`-chained forms
  and `python -c` are allowlist-refused - wrap single commands as
  `python tools/distill.py -- git grep ...`; `testrun.py` takes `--test
  <suite-name>` (no `-n`, no `--help`); adjudicate exit codes from printed
  contracts (journal section 10 precedent).
- Journal-incremental SMALL steps (1-3 lines per gate); context looming ->
  STOP with disk-state inventory; a dead session is NEVER resumed -
  successor runs `python tools/checkpoint_guard.py resume --spec
  tasks/ar-27-tb-handoff.md --scope ... --cmd ...`.

## Report back (final message)

1. Both commit hashes + `git show --stat` (distilled). 2. Guard sweep
   evidence (both sweeps, hits or their absence). 3. Byte-lock GREEN-PRE
  evidence (canonical line beside the pre-refactor commit). 4. RED capture
   (failing-test list) then GREEN canonical lines, gates 2-7 verbatim from
   own stdout. 5. Bridge-proof line: m19/m23/shaper:45 UNEDITED green. 6.
   Deviations + reason. 7. Out-of-scope observations (report-only). Never
   paste raw tool output.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill (fresh full verification, per-commit `git show` oldest-first,
commit-vs-touch-set enforcement; workspace census vs the `1400/0/13/105`
anchor + journal-recorded deltas is the reviewer's acceptance pass). On
APPROVAL of this hop, AR-27 (A2) closes: ledger census rows update per the
landed suite deltas.
