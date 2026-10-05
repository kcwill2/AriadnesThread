# T2 verdict - M27 tu-index-hang-throughput: both flush sites + audit + DP-6 rider

Verdict: **APPROVED (FINAL, 2026-10-05)** - landed chain `ea21687` +
`59f0512` + `bcc3bb0` + riders `72252a4` + journal `9ceb8e3`, parent
`de5622f`. LANDING RE-CHECK section at bottom; rider-hop errata (amended
package MUST lines + GO) immediately above it.

Hop: handoff `tasks/tu-index-hang-throughput-t2-handoff.md`, commits
`ea21687` (fix + tests) / `59f0512` (DP-6 bump + m12 pin + m27 un-ignore) /
`bcc3bb0` (journal completion), parent `de5622f`, journal
`tasks/tu-index-hang-throughput-t2-journal.md`. Reviewer: fresh full
re-verification (all scoped re-runs, both profiles, component proof, full
census, hygiene/encoding, ui_smoke). Gates + canonical lines:
`tasks/tu-index-hang-throughput-execution-results.md` (addendum 3).

The three commits are clean and land inside the touch-set. The landing state
is RED on TWO machine-pinned DP-6 dependents: one declared and stop-reported
(m21), one the handoff's gate plan could not see (report byte-lock). Both
have exact precedent and a pinned one-line fix each. Land the riders below,
then the milestone census reconciles clean.

## Findings

1. **BLOCKER (landing state, NOT the implementer's error) - second
   undeclared DP-6 dependent: report manifest byte-lock.**
   `crates/report/src/manager/tests.rs:599`
   `const LOCK_MANIFEST_JSON_HASH: u64 = 0xddc3_f9ee_566b_5ea6;` hard-locks
   FNV-1a hashes over the saved load-manifest bytes, which embed
   `analysis_version`. The bump shifts it. Scoped proof:
   `34 passed / 1 failed / 0 ignored / 1 suites`; census (no-fail-fast):
   `1217 passed / 2 failed / 4 ignored / 86 suites`, failing line
   `manager::tests::load_artifact_bytes_are_byte_locked ... I2 violation:
   artifact bytes changed (json=0x70c91d079946f7d9 bin=0x7a121e910cbf2e00
   manifest=0xbee91e361bbe1e4b)` - json/bin still match their locks exactly
   (report payload carries no version, as the test's own comment at :596-598
   documents). NONE of the handoff's ten gates runs the report crate, and
   the handoff's stop-and-report clause was triggered by m21's fail-fast
   abort before its full `-p` run could reveal this. The M18.10 bump
   resynced this same constant (comment at :596-598 is that precedent).
   Required: rider R2 below.
2. **BLOCKER (landing state, sanctioned-rider) - m21 witness literal.**
   `tests/integration/tests/m21_scala_semantic_index_test.rs:499` hard-pins
   `assert_eq!(CURRENT_ANALYSIS_VERSION, "2.0.0", "M18.10 cache epoch
   witness");` in the RUNNABLE `scala_fingerprint_moves_only_the_scala_digest`.
   Reviewer's own grep census confirms it is the ONLY surviving `"2.0.0"`
   literal pin besides the ones the handoff enumerated; every other usage is
   the symbol (version-neutral). Implementer's stop-and-report was correct
   per the touch-set rule. RULING: **sanctioned as rider R1** below.
3. **Minor - handoff under-enumerated the DP-6 dependents twice.** The lock
   row's proof instruction ("grep the constant across `crates/` - one
   definition site changed") finds only literal DEFINITIONS; m21 (a literal
   pin in `tests/`) and the report byte-lock (a HASH pin, references no
   version string at all) were outside the enumeration. Ledger errata for
   the M27 row: the DP-6 dependent set is FIVE sites - cache.rs const +
   cache.rs assert, m12 pin, m27 un-ignore, m21 witness, report manifest
   byte-lock hash. Route: `tasks/milestone_ledger.md` dated row (orchestrator
   duty; verdict records it, does not write it).
4. **Minor - out-of-scope tool defect discovered.** Plain (non-distill)
   `python tools/testrun.py` (no args) crashes in its echo path on a cp1252
   console: `UnicodeEncodeError` at `tools/testrun.py:262` (`_echo`) - the
   m21 assert message contains an em-dash, which cp1252 cannot encode.
   Pre-existing (testrun predates this hop; the `--distill` route was
   unaffected and is the standing contract path). Route to ledger
   deferred-findings; not this hop's scope.

## Plan-literal verification (all PASS)

- Both flush sites fixed in ONE commit: `git show "ea21687" --
  crates/lang-cpp/src/tokenizer.rs` carries BOTH `- word.len()` -> `- len`
  and `- punct.len()` -> `- len` hunks, each reusing the existing
  `let len = <run>.chars().count();` - live code confirms `column` and `end`
  now share one unit (`tokenizer.rs:307-313`, `:327-333`). One-site landing
  rule satisfied.
- `parser.rs` byte-identical: absent from `git diff --name-only
  de5622f..HEAD`; M25 floor guard `:61-69` intact, unweakened.
- Audit independent re-run (reviewer grep `\w+ - \w+|-= |\.len\(\) -` over
  both files @ HEAD, non-hits are additions/comparisons): exactly the 12
  audited sites. Every SAFE-BY-INVARIANT proof re-verified against live
  code: A1 short-circuit `quote_at == 0 ||` (:233); A2 `content_start`
  starts AT `delimiter_start` (:260) increments only (:264); A6/A7 guard
  `self.i >= 2` (:281); A8 inner `self.i >= 3` (:286); A9 loop condition
  `paren_depth > 0` (:301); A10 `if self.i > 0` (:367); A11 (:390); A12
  (:482). Zero second bugs - audit sound.
- Test shapes + names verbatim (4/4); release ran the SAME module (I14):
  both profiles `152 passed / 0 failed / 0 ignored / 2 suites` re-run fresh.
  Bounds are real external properties (`[1, line_chars + 1]` computed from
  the source, `end - start == chars().count()` per token), not tautologies.
- RED-evidence credibility (cannot re-run RED post-landing; adjudicated on
  internal arithmetic + T0b corroboration): debug panics quoted at the exact
  `:311`/`:331:21` sites with E3's message; release wrap values
  `18446744073709551615` = `2-3` (euro punct at `col+1`=2 via the single-char
  flush path `:107-113`, `col += 1` per CHAR at :115) and `10-11` (word) -
  both reproduce exactly on the pinned test sources; the m27 corpus debug RED
  was independently REPRODUCED by the approved T0b hop. Credible.
- Punct-test interpretation (deviation D3): a bare U+00AB cannot panic
  (floors at 0; T0b Gate-1 empirical proof); the combined U+20AC driver +
  U+00AB shape satisfies BOTH clauses of the handoff's own spec ("containing
  U+00AB at/near line start" AND "Pre-fix debug: panic"). ACCEPTED.
- ASCII parity: for ASCII `run.len() == run.chars().count()` - the fix is
  bit-identical on ASCII paths by construction; `test_ascii_path_
  tokenization_unchanged` green in BOTH profiles; m00 goldens + m18 parity
  UNEDITED (absent from diff) and green in the component proof.
- Rider discipline: cache.rs = const + doc sentence + assert literal ONLY
  (envelope struct fields untouched, I7; `§9` context line byte-intact);
  m12 = literal + comments only; m27 = exactly the `#[ignore]` line deleted
  (`0 insertions / 1 deletion`), anti-vacuity specimen-bytes test green and
  the corpus test COMPLETES (`test_m27_nonascii_corpus_cold_index_completes
  ... ok` in the component capture).
- Constant moves EXACTLY 2.0.0 -> 2.1.0 at the single definition site
  (`cache.rs:21`); assert `:336` = `"2.1.0"`.
- Touch-set: `git diff --name-only de5622f..HEAD` = 6 paths, ALL declared.
  Journal-only completion commit `bcc3bb0` in-set (journal always in-set).
- Data-integrity disposition text present in the journal (:286-299) and
  routed to the user: re-index every affected non-ASCII corpus after landing
  (the version bump is the ONLY automatic invalidation; fingerprint hashes
  content, not analyzer code); ASCII corpora need no re-index (parity
  proves byte-identical output); shard scan stays a candidate detector.

## Deviation dispositions (declared D1-D8)

- D1 direct `commit_scoped.py` (distill-wrapped form refused): ACCEPTED -
  sanctioned precedent T0b `4ef8412`; ledger dated row 60 records the
  allowlist gap; scopes/messages verbatim as declared.
- D2 distill-wrapped `git add` staging: ACCEPTED - same sanctioned route
  (ledger row 60).
- D3 punct-test combined driver: ACCEPTED - see plan-literal section; strict
  superset of the spec, handoff wording internally unsatisfiable.
- D4 ASCII baselines transcribed from the pre-fix RED capture: ACCEPTED -
  transcription risk is nil (ASCII path bit-invariant under the fix); lock
  passes both profiles.
- D5 Gate-1 digest wording `### cargo build FAIL (exit 101)`: ACCEPTED -
  non-zero as required; distiller labeling gap, not substantive.
- D6 m21 stop-and-report: ACCEPTED + rider sanctioned (finding 2). The
  touch-set rule was honored exactly as written.
- D7 m12 comment ASCII normalization (`->`/`-` inside rewritten lines):
  ACCEPTED - pure-ASCII authoring rule mandates it on lines being (re)written;
  diff shows untouched lines keep original bytes; encoding guard clean.
- D8 journal completion as follow-up commit: ACCEPTED - journal always
  in-set; keeps both landing commits at their declared scopes; T0b precedent.

No undeclared deviations found (findings 1-4 are plan/tooling defects, not
implementer edits outside the contract).

## RIDERS - exact instruction to relay to the implementer

Land as ONE follow-up commit on `main` (parent `bcc3bb0`), two files, both
edits machine-pinned. Use Write/Edit tools only (never PowerShell text
pipes). Route:

1. `tests/integration/tests/m21_scala_semantic_index_test.rs` - the line
   currently reading (at :499):

       assert_eq!(CURRENT_ANALYSIS_VERSION, "2.0.0", "M18.10 cache epoch witness");

   becomes, literal-only, witness message string UNCHANGED:

       assert_eq!(CURRENT_ANALYSIS_VERSION, "2.1.0", "M18.10 cache epoch witness");

   No other line in the file changes.
2. `crates/report/src/manager/tests.rs` - the constant at :599 becomes:

       const LOCK_MANIFEST_JSON_HASH: u64 = 0xbee9_1e36_1bbe_1e4b;

   and the comment block at :596-598 is replaced (pure ASCII) with:

       // Resynced for the M27 T2 DP-6 epoch bump ("2.0.0" -> "2.1.0"): the load
       // manifest embeds `analysis_version`, so its bytes legitimately shift
       // while the report json/bin locks stay pinned (they carry no version).
       // Prior value 0xddc3_f9ee_566b_5ea6 recorded the M18.10 bump
       // ("1.0.0" -> "2.0.0").

   `LOCK_REPORT_JSON_HASH` and `LOCK_REPORT_BIN_HASH` stay EXACTLY as they
   are (they match their locks still - the report payload carries no
   version). No other line in the file changes. The new manifest hash is the
   reviewer-observed actual, identical across two independent runs
   (`0xbee91e361bbe1e4b`).
3. Gates before commit (AMENDED 2026-10-05 rider-hop errata - see
   ERRATA ADDENDUM below; the original 100/0/0/24 and 35/0/0/1 lines were
   extrapolated from fail-fast-truncated RED-tree runs and were WRONG):
   - `python tools/testrun.py --distill -p ariadne-integration-tests`
     -> MUST print `141 passed / 0 failed / 0 ignored / 35 suites`
   - `python tools/testrun.py --distill -p ariadne-report`
     -> MUST print `36 passed / 0 failed / 0 ignored / 3 suites`
   BINDING gate remains the workspace census line below.
   - staging: `python tools/distill.py -- git add --
     tests/integration/tests/m21_scala_semantic_index_test.rs
     crates/report/src/manager/tests.rs` (sanctioned route), then
     `python tools/encoding_guard.py --staged` exit 0.
4. Commit (exact command, direct `commit_scoped.py` entry point - same
   sanctioned route as D1):

       python tools/commit_scoped.py --scope tests/integration/tests/m21_scala_semantic_index_test.rs --scope crates/report/src/manager/tests.rs -m "test(epoch): resync bump dependents - m21 witness literal + report manifest byte-lock 0xddc3f9ee -> 0xbee91e36 (M27 T2 DP-6 riders)"

5. Journal: append the rider checkpoint to
   `tasks/tu-index-hang-throughput-t2-journal.md` (in-set, follow-up commit
   or the rider commit - implementer's call, declared).

## Landing re-check the reviewer/orchestrator owes (machine-pinned)

- `git show "<rider>"` = exactly 2 files; both diffs literal-only.
- `python tools/testrun.py --distill -- --workspace --no-fail-fast`
  -> MUST print exactly `1219 passed / 0 failed / 4 ignored / 86 suites`
  (anchor `1214/0/5/86` + 4 new underflow tests + 1 m27 un-ignore; ignored
  5 -> 4; suites 86 held). Any other total = stop and re-adjudicate.
- Record the DP-6 bump commit pair + the re-index disposition routing to the
  user (G2-3) in the ledger M27 row, plus finding 3's five-site dependent
  census errata and finding 4's deferred-findings row.

## Gate denial log (attempt discipline)

Every expected gate ran; denials hit only non-gate conveniences (all with the
boilerplate catch-all `{"permission":"bash","pattern":"*","action":"deny",
"source":"global"}`): direct `rg` (substituted: grep tool), compound
conditional/redirect forms for exit-code capture (substituted: distill route
which prints `### generic PASS (exit 0)`), `python -c` one-liners
(substituted: reading test sources directly). `python tools/testrun.py
--baseline` is a testrun usage error, not a permission denial - see execution
results (known ledger row 59 errata; `--distill` census substitute run, plus
`-- --workspace --no-fail-fast` to defeat the rider-pending fail-fast abort).
`bench_compare` NOT run - declared exemption: the handoff declares no bench
gate; M27's bench truth is bound to T1a/T1b (PR-2 live gate); the ledger's
dated bench-baseline staleness row would make a hop-1 bench flaky.

---

# ERRATA ADDENDUM - rider-hop re-adjudication, 2026-10-05

Tree: `bcc3bb0` + the two sanctioned rider edits (verified diff-exact: m21:499
literal-only, witness message unchanged; `LOCK_MANIFEST_JSON_HASH` =
`0xbee9_1e36_1bbe_1e4b`, lineage comment pure ASCII - line-wrap differs
trivially from dictated wording, content-equivalent, ACCEPTED; json/bin
constants untouched; no other lines in either file).

## My rider-MUST arithmetic error, adjudicated

The implementer stopped correctly: the gates printed `141 passed / 0 failed /
0 ignored / 35 suites` (integration) and `36 passed / 0 failed / 0 ignored /
3 suites` (report), not my MUSTs. Reviewer re-ran both packages AND the
binding workspace gate on the armed tree - all three reproduce the
implementer's lines; the implementer's reconciliation is SOUND:

- Pre-rider RED tree: my `99/1/0/24` and `34/1/0/1` runs were DEFAULT
  fail-fast runs that ABORTED AT THE FAILING TARGET - m22+ integration
  targets and the report `extraction_artifacts_test`/doc-test suites never
  started. My MUSTs extrapolated as if every target had run.
- Green tree: all targets start - integration gains the ~11 never-started
  suites (24 -> 35, +41 passed incl. the m21 flip: 100+41=141); report
  lib-35 + extraction_artifacts-1 + 0-test doc block = 36/3.
- Corroboration independent of package arithmetic: same session, default
  census printed `388 passed / 1 failed / 1 ignored / 38 suites` vs the
  no-fail-fast `1217 passed / 2 failed / 4 ignored / 86 suites` on the same
  tree - fail-fast truncation is real and large in this workspace.

## AMENDED rider-hop MUST lines (verified on the armed tree)

- `python tools/testrun.py --distill -p ariadne-integration-tests` ->
  `141 passed / 0 failed / 0 ignored / 35 suites` (exit 0, reproduced).
- `python tools/testrun.py --distill -p ariadne-report` ->
  `36 passed / 0 failed / 0 ignored / 3 suites` (exit 0, reproduced).
- BINDING landing gate (unchanged, pinned, reproduced verbatim on the armed
  tree): `python tools/testrun.py --distill -- --workspace --no-fail-fast`
  -> `1219 passed / 0 failed / 4 ignored / 86 suites`, exit 0.

## RELEASE - GO for the armed rider commit

Verdict upgraded toward landing: the three fix commits + the verified armed
rider edits, committed, clear both findings 1-2. Instruct the implementer
(relay) to land the armed `commit_scoped.py` command UNCHANGED (the exact
command in RIDERS step 4), plus the journal rider checkpoint (rider hashes,
amended package lines, finding-3 five-site DP-6 dependent census, finding-4
`testrun.py:262` cp1252 defect reported-not-fixed) per the T0b-precedent
journal-completion route. Post-landing reviewer re-check (unchanged):
`git show <rider>` = exactly 2 literal-only files; the binding workspace
line prints exactly `1219 passed / 0 failed / 4 ignored / 86 suites`.

## Deferred-findings proposal (ledger + templates, one line)

> Rider/MUST count lines must be quoted from a GREEN-tree run or a
> `--no-fail-fast` run: default cargo runs ABORT AT THE FIRST FAILING TARGET,
> hiding never-started targets and making truncated counts look like package
> totals (M27 T2 rider hop: reviewer MUSTs `100/24` and `35/1` vs green truth
> `141/35` and `36/3`).

Route: `tasks/milestone_ledger.md` dated row + handoff-spec/review-gate
template notes (orchestrator duty; recorded here, not written here).
Findings 3 (five-site DP-6 dependent census errata) and 4 (testrun cp1252
echo defect at `tools/testrun.py:262`) remain reported-not-fixed as stated
above.

---

# LANDING RE-CHECK (FINAL) - verdict CLOSED APPROVED, 2026-10-05

All three pre-registered re-check duties PASS on the committed tree
(`9ceb8e3`, chain `de5622f -> ea21687 -> 59f0512 -> bcc3bb0 -> 72252a4 ->
9ceb8e3`):

1. `git show "72252a4"` -> exactly 2 files, both literal-only:
   `tests/integration/tests/m21_scala_semantic_index_test.rs` 1/1 (witness
   message string byte-unchanged, only the `"2.0.0"` -> `"2.1.0"` literal)
   and `crates/report/src/manager/tests.rs` 6/4 (`LOCK_MANIFEST_JSON_HASH`
   `0xddc3_f9ee_566b_5ea6` -> `0xbee9_1e36_1bbe_1e4b` + pure-ASCII lineage
   comment; json/bin lock constants appear as unchanged context). Blob
   hashes `49c2080..e1986e2` / `9e90e5b..e0549f9` are BYTE-IDENTICAL to the
   armed state the reviewer verified pre-GO. Message verbatim as required.
2. `git show "9ceb8e3"` -> `1 file changed, 47 insertions(+)`, journal only,
   carrying the required checkpoint: rider hashes R1/R2, the amended package
   lines (`141 passed / 0 failed / 0 ignored / 35 suites`; `36 passed /
   0 failed / 0 ignored / 3 suites`), the complete DP-6 dependent census
   (five literal sites + the one hash pin - framing-equivalent to
   finding 3's set, complete enumeration), and finding 4
   (`tools/testrun.py:262` cp1252 echo defect) reported-NOT-fixed.
3. Binding landing gate on the committed tree
   (`python tools/testrun.py --distill -- --workspace --no-fail-fast`):
   `### cargo test PASS (exit 0)` +
   `1219 passed / 0 failed / 4 ignored / 86 suites` - the pinned line,
   printed exactly. Third consecutive reproduction (armed tree x2, committed
   tree x1).

## Dispatch-door statement (M27 sequencing)

T0-report + T0b + T2 are all landed and gated. **The T1 dispatch door is
OPEN**: T1a (RELEASE `boot_stages` bench + launch-profile record +
sufficiency-gate verdict) is dispatch-ready now; T1b (`bench_compare
--bench boot-stages` gate mode + merge-baseline) follows T1a. **The PR-2
operator precondition STILL STANDS before T1b's live bench gate**: refresh
`bench_baseline.json` via `--update-baseline` on a VERIFIED-IDLE machine
first (ledger dated bench-staleness row; T1b is PR-2-bound). T3 stays
DISPATCH-BLOCKED: written T1 sufficiency-gate FAIL AND M26 V3 landing are
both required (T2's landing now satisfies the "fix precedes T3 closeout"
half; the FAIL verdict + V3 remain). Reminder for the orchestrator, per the
mandatory data-integrity disposition: re-index every affected non-ASCII
corpus - the 2.1.0 bump is the automatic invalidation; nothing else
self-invalidates.

## Orchestrator landing duties (recorded, not written here)

Ledger M27 row: T2 + riders landed at `72252a4`/`9ceb8e3`; the
`CURRENT_ANALYSIS_VERSION` 2.1.0 freeze resumes; finding-3 dated errata row
(dependent set = five literal sites + one hash pin); finding-4
deferred-findings row; the ERRATA template lesson row ("rider MUST lines
from a green-tree or --no-fail-fast run"); census anchor advances to
`1219 passed / 0 failed / 4 ignored / 86 suites`.
