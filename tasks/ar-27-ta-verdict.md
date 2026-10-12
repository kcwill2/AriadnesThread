# AR-27 T-a Verdict - APPROVED

Session: reviewer (review-gate skill). Contract `tasks/ar-27-ta-handoff.md`
(amended zero-rider version); authority `tasks/ar-27-shape-ruling.md` (section 2
= law; RE-RULING R1-R6); journal sections 12-15; range `046e38c..4732cd1`
(`85179b0` ORACLE -> `f0e1cd1` IMPL -> `4732cd1` journal close-out).
Evidence: `tasks/ar-27-ta-execution-results.md`.

## Verdict: APPROVED (5 minor/informational findings, no rework required)

## First-order scope adjudications (decided before the byte work)

### 1. index crate `15/0/0/2 -> 17/0/0/3` - LEGITIMATE, not a scope breach

What landed in `crates/index`: exactly ONE new file,
`crates/index/tests/ar27_context_bytes_lock_test.rs` (92 lines, commit
`85179b0` only). Whole-range proof: `git diff 046e38c..HEAD -- crates/index`
shows ONLY that new file; `crates/index/src` has ZERO changes (epoch pin
`cache.rs:30`, its pin test, and the digest locks untouched).
Is it named in the contract? YES, three times:
- Touch-set "NEW test files only" bullet:
  `crates/index/tests/ar27_context_bytes_lock_test.rs`.
- RGB step 2 "index file: `CompilationContext` bincode bytes + digest hex"
  (the ADAPT-8 digest seam `TuCacheFingerprint::compute` lives in index;
  nowhere else could this oracle go).
- Gate 4: "`-p ariadne-index` - pre-existing digest locks UNEDITED; +1 suite
  (probe fixture green pre and post)" - the contract PINS the +1 suite.
The T-a0-era `15/0/0/2 UNEDITED` pin proves index COMPILE survival of the T-a0
tokens in `crates/index/src`; it never claimed the index test dir was closed -
the T-a contract explicitly opens it new-file-only. The +2 tests are the two
oracle tests (`old_context_blob_loads_and_rewrites_byte_identically`,
`context_hash_digest_is_byte_frozen_adapt8`); the +1 suite is that one file.
R3 ledger check: R3's "NOT touched in T-a0" list and the R6 planner-change
list never close `crates/index/tests/**` to T-a - the T-a0 probe's
`repo/types.rs`-only note is a T-a0 fact, not a T-a constraint. The zero-rider
MUST-NOT list names pipeline/build/artifact/worker-core - index is NOT on it;
the actual index prohibition is `crates/index/src/**` (honored, verified).

### 2. `probe 3/0/0/1` gate line - LEGITIMATE committed lock suite, not residue

It is `-p ariadne-integration-tests --test ar27_field_probe_test` =
`tests/integration/tests/ar27_field_probe_test.rs` (191 lines), a committed
new file NAMED in the touch-set ("NEW test files only" bullet 4) and mandated
by ruling section 2.8 (the declared-enumeration probe lock). Its 3 tests are
real locks: (a) census of every `pub language: LanguageId` src field ==
declared 5-site set with a panic-on-surprise classifier, (b) the
bincode-persisted set EXACTLY {IncludeEdge.language}, (c) IncludeEdge JSON
bare-name locks incl. the open id and the absent-key default. The
deleted-before-commit attestation discipline governed the COPY-STRIP probe
(T-a0) and the temporary byte printer (`ar27_tmp_capture.rs`) - the printer
left NO residue: absent from the tree, absent from every commit in range
(stat lists exact file sets), and `git log --all --
crates/lang-api/tests/ar27_tmp_capture.rs` is EMPTY.

## The byte contract (blob/fixture-level verification)

### 3. Fixture truthfulness - VERIFIED

Design of the independent check: every byte fixture is asserted in its own
committed test against LIVE re-serialization with strict full-array equality
(`bincode::serialize(&expected).unwrap() == FIXTURE`, no prefix/subset
weakening), plus load equality (`deserialize(fixture) == expected`). Running
the suites green at HEAD therefore proves fixture == current-code bytes; the
oracle-commit green proves fixture == pre-mutation bytes; the same-array proof
between commits (below) glues the two worlds. My own fresh runs are in the
execution-results addendum. Spot arithmetic on disclosed corrections: DEFAULT
array = 6x12+4 = 76 bytes, EXTENDS = 8x12+7 = 103 bytes - matching the
re-captured truth values (EXTENDS 103, not the formula's 111; SEALED 185
green in-suite). `OLD_INCLUDE_VEC_BIN` language regions hand-decoded to
`u64 LE len + UTF-8` for Cpp/Scala/Fortran (genuinely string-framed, not
u32); `OLD_CTX_BYTES` ends `[0,0,0,0]` (u32 LE Cpp) with the digest triple-pin
(hex == SHA256(bytes) == `TuCacheFingerprint::compute().context_hash`).
No transcription error survives.

### 4. Green-first ordering - VERIFIED by commit math + byte-identity; literal
re-run at `85179b0` BLOCKED by read-only grants (stated honestly)

`85179b0` parent is `046e38c` (pre-mutation); the oracle commit contains ZERO
`src/` files (stat: 4 new test files + `repo/Cargo.toml` +2 dev-dep lines +
journal 140-line append). The impl commit `f0e1cd1` touches the lang-api oracle
file as a PURE APPEND (`+67` lines after line 198: the discriminator-pin,
escape-arm, and bad-tag tests) - every fixture byte array at `85179b0` is
byte-identical to the one green in my HEAD runs; repo/index/probe fixture
files are absent from `f0e1cd1`'s stat, so identical too. LIMITATION, verbatim:
`git worktree add C:\Users\Kurt\AppData\Local\Temp\kilo\ar27-oracle-checkout
85179b0` -> `The user has specified a rule which prevents you from using this
specific tool call. Here are some of the relevant rules
{"permission":"bash","pattern":"*","action":"deny","source":"global"}`; write
git is denied, so I did not re-execute the suites AT `85179b0`. The pre-run is
established by: no-src-commit + journal digests (19/44/17 + 3 probe) +
identical arrays green at HEAD under the new code + the old-enum byte shapes
being the trivial derive case the discriminator pin now proves by bytes.

### 5. Encoding contract section 2 - CONFORMANT (all eight clauses)

1. `WireLanguageForm` private (no `pub`), derive-only, arms `Cpp, Scala,
   Other, Open(String)` in FROZEN order (repo/src/identity.rs:149-155).
2. Serialize: legacy ids -> index (pin test asserts `00/01/02 00 00 00`);
   open -> tag 3 + u64 LE len + UTF-8 (escape test asserts the 19-byte Fortran
   stream). Legacy always written as index even by new code (`from_id`).
3. Deserialize positional via `WireLanguageForm::deserialize`: out-of-range
   tag is a serde error (bad-tag test: tag 7, tag 255, truncated tag-3 all
   `is_err()`), never a panic.
4. Human-readable arm: `is_human_readable()` in BOTH directions
   (identity.rs:179, :189); transparent bare name for ALL ids; no
   externally-tagged `{"Open":...}` anywhere (probe asserts its absence).
5. Collision: legacy tag set {0,1,2}; tag 3 unreachable pre-AR-27a (old
   3-variant enum bytes are the fixtures).
6. Round-trips: old u32 blobs load (`85179b0` fixtures), escape blobs load,
   JSON identical pre/post (m23 family + m00 + ui green UNEDITED).
7. String-frame exception: `language_transparent_frame` in `ariadne-repo`
   (pub module, bare name in both formats), attached field-level with
   `#[serde(default, with = "...")]` KEEPING `default`
   (lang-api/src/types.rs:311); `Vec<IncludeEdge>` warm bytes green UNCHANGED
   through the frame.
8. Probe lock: declared set EXACTLY {IncludeEdge.language}; 5-site census
   matches the ruling's clean list + the journal's re-derivation
   (types.rs/cache.rs/modeling.rs/unit.rs/planner.rs); surprise = panic.
`is_human_readable` discriminator pin committed post-impl (ruling section 8).
Custom legacy-rank `Ord` (rank then lexicographic; total, consistent with
string-equality `Eq`; `Cpp < Scala < Other < open-lex`; sort-order unit lock
green). Manifest-sort consumer proven by artifact 40/0/0/2 + m22/m00 UNEDITED.
Coherence: value `From` DELETED; by-ref `From<&LanguageId>` KEPT hand-written
(body `source.clone()`); the std-blanket-absence refinement is correct (std
offers only reflexive `From<T> for T`) - `artifact/manager/metrics_io.rs:241`
compiles UNTOUCHED (file absent from the whole-range footprint; artifact suite
green; workspace `cargo check --workspace --all-targets` exit 0; no E0119).
T-a0 clone tokens: whole-range registry diff is exactly 3 hunks confined to
`parse_language_arg` + the new helper; tokens at current :28/:190/:208/:250/
:386/:464/:536 sit OUTSIDE every hunk in ledger clone form; `{language:?}`
bail wording byte-stable (Debug = bare name). 39 tokens survive.

### 6. Identity pins - ALL REPRODUCED (own stdout, verbatim lines in addendum)

cli `118 passed / 0 failed / 8 ignored / 7 suites`; ui `272 passed / 0 failed /
0 ignored / 3 suites`; build `23 passed / 0 failed / 0 ignored / 2 suites`;
artifact `40 passed / 0 failed / 0 ignored / 2 suites`; pipeline
`40 passed / 0 failed / 4 ignored / 9 suites`; m23-bincode
`5 passed / 0 failed / 0 ignored / 1 suites`; m23-symbol+m00
`11 passed / 0 failed / 0 ignored / 3 suites` (7+4 split); m24+m24a+m18
`33 passed / 0 failed / 0 ignored / 3 suites` (8+2+23 split); ui_smoke PASS;
hygiene zero blocking rows; encoding_guard --all silent.
repo `15/0/0/2 -> 26/0/0/3`: +6 moved units +4 oracle fixtures +1 new
legacy-rank lock; conservation equation holds (lang-api lost exactly the 6
moved unit tests, assertion bodies intact - two comment texts updated only
where the old wording would lie post-move). lang-api
`41/0/0/4 -> 41/0/0/5`: -6 moved +3 oracle +3 post-impl locks, +1 suite - the
oracle additions are ALL of it. index `15/0/0/2 -> 17/0/0/3`: adjudicated in
verdict 1.

## Census and attribution (state-before-compare honored)

Pre-declared projection: anchor `1400/0/13/105` + repo +11 + lang-api 0 +
index +2 + probe +3 = `1416/0/13/109`. Measured:
`1416 passed / 0 failed / 13 ignored / 109 suites` (exit 0) - EXACT. ZERO
unattributed movement. Tree quiet: `git status --porcelain` = the same 7
pre-existing foreign entries as at dispatch (2 M foreign churn + 5 ?? foreign
docs); worktree census: main dirty=6 (foreign), shining-enthusiasm dirty=7
(foreign churn, none authored by this hop), tarjan-s-scc dirty=2 (foreign).
tools: `Ran 219 tests in 24.127s / OK`.

## Deviation dispositions (independently adjudicated, not rubber-stamped)

- D1 (probe bincode bytes placed in lang-api file): ACCEPTED. The contract's
  own machine check ("git diff dispatch-HEAD -- tests EMPTY except the ONE new
  file") forbids the alternative (`tests/integration/Cargo.toml` bincode
  dev-dep); the internal contradiction is real, coverage preserved, placement
  reason recorded pre-commit, both machine checks green (verified myself).
- D2 (transcription incident; formula-circular first derivation caught,
  re-captured from live `bincode::serialize`, printer deleted pre-commit,
  `FAILURES: 0` cross-check): ACCEPTED on merits - independently confirmed the
  committed arrays are live-serialize-equal at HEAD and array-identical
  between oracle and impl commits. The honest record is exactly what the
  discipline asks for.
- D3 (python -c through distiller; Remove-Item policy-denied -> scripted
  scratch cleanup): ACCEPTED. Scratch-only, zero tracked-byte rewriting.
- D4 (Cargo.lock rode IMPL commit): dispositioned as finding 3 (minor).
- D5 (help-line byte-lock correction): ACCEPTED - gate 7 is ui + ui_smoke only;
  the HELP lock is ruled into T-b (ruling section 5.7). No help text changed.
- Coherence refinement (value-From deletion justified via std reflexive
  blanket, not "E0119 vs blanket" ambiguity): ACCEPTED - code-level verified
  (verdict 5).
- Journal append-only discipline: HONORED - `85179b0` +140/-0, `f0e1cd1`
  +82/-0, `4732cd1` +10/-0 on the journal; hop-1 STOP record untouched.

## Findings (numbered; all minor/informational; no rework on `85179b0/f0e1cd1/4732cd1`)

1. MINOR - Non-ASCII authored content vs the pure-ASCII clause: 7 characters
   on 7 added lines: `crates/repo/src/identity.rs:1,14,20,41` (U+2014),
   `crates/cli/src/registry.rs:57` (U+00A7), `crates/lang-api/src/types.rs:305`
   (U+2014, mirrors the deleted line's style) and :307 (U+2014). All validly
   single-encoded (encoding_guard --all silent - the mojibake hazard is
   provably absent; fixtures are numeric-only arrays). Fix: replace with
   `--`/`sec` on the next in-scope touch of these files; ledger candidate.
2. MINOR - Commit messages deviate from the contract's verbatim `-m` strings
   (`f0e1cd1` typed `feat(ar-27a)` vs commanded `refactor(repo,lang-api,cli)`;
   `85179b0` paraphrased), and a third journal-only commit `4732cd1` lands
   beyond the prescribed two-commit ledger. No byte/lock/scope impact (touch-set
   exact in both commits); record for the ledger.
3. MINOR - `Cargo.lock` dev-dep entry rode `f0e1cd1` (oracle `commit_scoped`
   scope named no lock), so the lock was momentarily stale between
   `85179b0..f0e1cd1`. Correct commit_scoped behavior; HEAD lock verified
   consistent (my cargo runs produced zero lock churn).
4. INFORMATIONAL - green-first literal re-run at `85179b0` not executable under
   reviewer read-only git (verbatim refusal recorded in verdict 4); equivalent
   evidence chain substituted and stated.
5. INFORMATIONAL - `python tools/testrun.py --baseline` is structurally
   unrunnable: it takes a FILE and no census-baseline file exists in the repo
   (`usage error: missing value for --baseline`; AR-03 finding 3 precedent,
   still open). The workspace census line is quoted instead. NOT RUN per
   dispatch: `bench_compare.py` (OUT of contract).

## Pin-updates for the bank (final adopted values)

- `ariadne-repo`: `15/0/0/2 -> 26/0/0/3`
- `ariadne-lang-api`: `41/0/0/4 -> 41/0/0/5`
- `ariadne-index`: `15/0/0/2 -> 17/0/0/3`
- `ar27_field_probe_test` (integration): NEW suite `3/0/0/1`
- workspace census anchor: `1400/0/13/105 -> 1416/0/13/109`
- UNCHANGED (re-pinned green UNEDITED): cli `118/0/8/7`, ui `272/0/0/3`,
  build `23/0/0/2`, artifact `40/0/0/2`, pipeline `40/0/4/9`, m23-bincode
  `5/0/0/1`, m23-symbol `7/0/0/1`, m00 pair `4/0/0/2`, m24 `8/0/0/1`,
  m24a `2/0/0/1`, m18 `23/0/0/1`.

T-b may dispatch on this HEAD (`4732cd1`); no rework gates.
