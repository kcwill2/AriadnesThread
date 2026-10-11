# T-B Handoff - branding-assets: extension banner + icon (extensions/ariadne-extension)

Authored 2026-10-11 by the planner role. Unregistered post-M28 track per
kickoff: NO M-number; `tasks/milestone_ledger.md` stays numbering authority
(ledger registration row exists; the hotspot-canvas live-row pattern is the
precedent).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract; zero verbal briefing needed).
2. `tasks/branding-assets-kickoff.md` (intent, invariants 3/5/7, inherited
   frozen locks, track acceptance).
3. `tasks/branding-assets-shape.md` (decision D2 - the banner now, the PNG
   icon conditional with its deferral path; D3 ownership; invariant I7
   packaging validity; risk R3 rasterizer availability).
4. Content anchors (verbatim, verified against the working tree 2026-10-11):
   `extensions/ariadne-extension/README.md` first three lines (`# Ariadne
   Extension for VS Code`, blank, `The Ariadne Extension allows you to:`);
   `extensions/ariadne-extension/package.json` (`"license": "MIT",` at line 7;
   NO `icon` field today); `extensions/ariadne-extension/.vscodeignore`
   (verified 2026-10-11: contains NO media-excluding entry, so `media/**`
   packages; read-only relay - do not edit).
5. Literal source: `media/AT.jpg` (photographic wide banner ~16:9, the
   wordmark "ARIADNE'S THREAD - CODE DEPENDENCY ANALYZER"). `media/` is the
   read-only source of truth.

## Dispatch preconditions (blocking)

- If the working tree is dirty on any touch-set file, STOP and report.
- This handoff is file-disjoint from handoff T-A (`crates/ui` + 2 tools/
  riders) and from the in-flight server.rs split (AR-13): parallel-safe. It
  MUST NOT touch anything outside `extensions/ariadne-extension/` - a byte
  copied here has no coupling to the app-side copy (D3 ownership rule).

## Objective and invariants

Put the canonical banner art into the VS Code extension: a byte-identical
`media/banner.jpg` copy referenced as the README banner, and - only if a
rasterizer is available - a derived square `media/icon.png` wired to the
marketplace `"icon"` field. Preserve, falsifiably:

- I1 (kickoff invariant 3) `extensions/ariadne-extension/media/banner.jpg` is
  byte-identical to `media/AT.jpg`; `media/**` never edited, zero-diff.
- I2 (kickoff invariant 5) `encoding_guard.py --staged` exit 0 with the new
  blobs staged (see Risk R1 clause below - this is the one gate that can
  legitimately refuse to pass; the response is STOP-report, never a guard
  edit).
- I3 (kickoff invariant 7) every NEW string pure ASCII; the README alt text
  and any new manifest string spell `Ariadne's Thread` (possessive -
  correction key `product.name.possessive`). Pre-existing README lines -
  including the emoji section headers (valid UTF-8) - survive byte-identical:
  this handoff ADDS lines, it does not rewrite the README.
- I4 packaging validity (shape invariant 7): `package.json` parses as JSON;
  IF `"icon"` is set, the path exists and the file is a PNG >=128x128
  (magic-bytes + IHDR machine check below); `"icon"` is UNSET iff no PNG is
  committed - a dangling `"icon"` path breaks packaging and is a blocker.
- I5 zero collateral diff: every file outside the Touch-set below -
  `src/extension.ts` (no webview; verified), `.vscodeignore`,
  `package-lock.json`, the stale `ariadne-extension-0.1.0.vsix`,
  `media/**` (source of truth), `crates/**`, `tools/**` - byte-identical to
  `TRACK_BASE`.

## Scope boundaries (explicit out-of-scope)

- `.vsix` rebuild/publish: out (kickoff scope-out; packaging is an operator
  duty; the tracked `ariadne-extension-0.1.0.vsix` is a stale artifact and
  must stay untouched - its presence is NOT a defect this handoff fixes).
- `src/extension.ts` / any TS/JS change: out (no webview surface exists;
  verified). `npm run compile` is therefore NOT a gate of this handoff:
  declared exemption - zero TypeScript diff, the manifest addition is data
  only; a node/npm availability probe is irrelevant.
- `"icon"` pointing at a `.jpg`: IMPOSSIBLE by contract (vsce requires a
  square PNG >=128px; shape D2) - the JPG banner NEVER satisfies the icon
  field.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/branding-assets-tb-journal.md` (checkpoint journal; create it;
  always in-set)
- `extensions/ariadne-extension/media/banner.jpg` (ADD - byte-level copy of
  `media/AT.jpg`; procedure C0; never a PowerShell text pipe)
- `extensions/ariadne-extension/README.md` (EXACTLY one banner paragraph
  inserted after the H1 - content C1; Edit tool only)
- `extensions/ariadne-extension/media/icon.png` (ADD - CONDITIONAL, only on
  the rasterizer-yes path; content C2)
- `extensions/ariadne-extension/package.json` (CONDITIONAL, only on the
  rasterizer-yes path: +1 line `"icon": "media/icon.png",` after the
  `"license": "MIT",` line - content C3; no other manifest change, no
  reordering, no reformatting)

You MUST NOT touch: `media/**` (source of truth), `extensions/ariadne-extension/src/**`,
`extensions/ariadne-extension/.vscodeignore`, `package-lock.json`, any
`*.vsix`, `themes.css`, anything under `crates/`, `tools/`, `tests/`,
`tasks/milestone_ledger.md`, any `Cargo.toml`/`Cargo.lock`. No `git stash`,
no branch change.

## What to build (contracts C0-C3; before-strings verbatim from the tree 2026-10-11)

C0 Byte-level banner copy (binary-safe, AGENTS.md File-Edit Discipline):

```
python tools/distill.py -- python -c "from pathlib import Path; Path('extensions/ariadne-extension/media/banner.jpg').write_bytes(Path('media/AT.jpg').read_bytes())"
```

Prove byte identity in the journal:
`python tools/distill.py -- git hash-object media/AT.jpg extensions/ariadne-extension/media/banner.jpg`
- the two printed hashes MUST be equal. If your session's permissions deny
even via the distiller, record the denial + both file byte sizes in the
journal and report it (the reviewer hop runs the hash check fresh).

C1 README banner. Before (exact, lines 1-3):

```markdown
# Ariadne Extension for VS Code

The Ariadne Extension allows you to:
```

after (exact; pure ASCII; possessive product name; the rest of the file,
emoji headers included, untouched):

```markdown
# Ariadne Extension for VS Code

![Ariadne's Thread](media/banner.jpg)

The Ariadne Extension allows you to:
```

C2 PNG icon derivation - RASTERIZER-YES path only. Probe first:

```
python -c "import PIL"
```

- ImportError (or any other rasterizer absent): take the DEFERRAL path (see
  below). Do NOT hand-author a PNG, do NOT install packages, do NOT commit a
  placeholder.
- PIL present: derive the 128x128 icon from the jpg's square logo band (the
  shape names the left band; contract = the left square, resized with
  LANCZOS):

```
python -c "from pathlib import Path; from PIL import Image; im = Image.open('media/AT.jpg').convert('RGB'); w, h = im.size; im.crop((0, 0, h, h)).resize((128, 128), Image.LANCZOS).save('extensions/ariadne-extension/media/icon.png')"
```

Then machine-validate (pure-ASCII byte-escapes, no literals of the binary):

```
python tools/distill.py -- python -c "from pathlib import Path; b = Path('extensions/ariadne-extension/media/icon.png').read_bytes(); assert b[:8] == b'\x89PNG\r\n\x1a\n' and b[12:16] == b'IHDR', 'not a PNG'; w = int.from_bytes(b[16:20], 'big'); hgt = int.from_bytes(b[20:24], 'big'); assert w >= 128 and hgt >= 128, 'below 128x128'; print('PNG OK', w, hgt)"
```

Then EYEBALL it: open the produced PNG with the Read tool (it attaches
images) and confirm the frame shows the brand mark, not a clipped wordmark or
edge band. Garbage/mis-crop = DEFERRAL path: delete the candidate file, ship
banner-only, journal the reason. This eyeball line is required in the journal
either way.

DEFERRAL path (kickoff-sanctioned, shape D2): ship C0 + C1 only. `"icon"`
stays ABSENT from `package.json` (a dangling icon path breaks packaging -
invariant I4). Journal a dated line: `PNG deferred - reason: <no rasterizer |
crop failed eyeball>; banner ships without icon field; follow-up candidate
on closeout`. Handoff B is COMPLETE on the deferral path; the deferral is a
recorded outcome, not a failure.

C3 package.json icon field - ONLY when `media/icon.png` is committed. Before
(exact):

```json
  "license": "MIT",
```

after (exact; nothing else changes):

```json
  "license": "MIT",
  "icon": "media/icon.png",
```

Machine validity after the edit (gate 4): `package.json` parses as JSON:

```
python tools/distill.py -- python -c "import json; json.load(open('extensions/ariadne-extension/package.json', encoding='utf-8')); print('package.json parses')"
```

## RGB protocol (data handoff - honest shell form)

1. Red: BEFORE any mutation, run the identity assertion and capture its
   failure - the destination must not exist yet:
   `python -c "from pathlib import Path; assert Path('extensions/ariadne-extension/media/banner.jpg').exists(), 'banner copy missing'"`
   - expected: AssertionError. Journal it. (A check that cannot fail before
   the work is not Red; there is no test framework to bolt on here and none
   belongs in this touch-set.)
2. Green: C0; re-run the same assertion - passes; then the hash-object
   equality (gate 2). For the conditional path: C2 probe + derivation +
   validation (or deferral); C3 + JSON parse (gate 4); C1 last, via Edit.
   NEVER weaken an assertion to pass.
3. Refactor: nothing to dedupe in a byte copy; the README wording is the
   contract. Proceed to the remaining gates.

## Required tests

None added to any test suite (scope contains no test project). The machine
falsifiers are the gate commands themselves + the reviewer's zero-diff set
equality. Do NOT create test files, tools/ scripts, or npm test hooks for
this - the validity checks above are the whole oracle (YAGNI, shape hygiene
note 3: any NEW tools gate script would need its own conventions + touch-set
declaration; none is needed here).

## Hygiene (review-enforced)

No functions, no source files: the added text is 1 README paragraph + 1
JSON line; both under every ceiling. Authored text pure ASCII, each line
< 2000 chars. The binary additions are declared in the journal with byte
sizes (kickoff sizing note: `banner.jpg` size was not measurable from the
architect seat - record the ACTUAL size you copy; `icon.png` if shipped, same
line). `python tools/hygiene_check.py` is not a gate of this handoff (no
scanned-suffix file changes; `.md`/`.json`/`.jpg`/`.png` are outside
R2_SCANNED_SUFFIXES) - the reviewer hop runs the workspace-wide hygiene
sweep and finds this handoff's diff set hygiene-neutral.

## Frozen locks (inherited; proof commands scoped)

| Lock | Proof |
|---|---|
| `media/**` zero-diff (source of truth, kickoff invariant 3 / D3) | `git diff TRACK_BASE -- media` EMPTY; C0 hash-object equality |
| `package.json` structural freeze: all pre-existing fields survive byte-identical; the ONLY diff is the +1 `"icon"` line - or the whole file is diff-EMPTY on the deferral path | reviewer `git show` of the landing commit: single-line hunk `+  "icon": "media/icon.png",` or no hunk at all |
| README pre-existing lines byte-identical (emoji headers are valid UTF-8 and STAY; Edit tool only, no whole-file rewrite) | reviewer `git show` hunk review: additions only, zero `-` lines outside the inserted block's blank-line context |
| The stale `ariadne-extension-0.1.0.vsix` untouched (kickoff scope-out; R8) | `git diff TRACK_BASE -- extensions/ariadne-extension` filtered: no `.vsix` path |
| `src/extension.ts`, `.vscodeignore`, `package-lock.json`, `crates/**`, `tools/**`, `themes.css` zero-diff (I5) | reviewer `git diff --name-only TRACK_BASE..HEAD` set equality against the Touch-set minus journal |
| No mojibake; new blobs pass the guard | `python tools/encoding_guard.py --staged` exit 0 before committing (`--all` is the reviewer hop) |
| I4 dangling-icon impossibility | gate 4 JSON parse + (if shipped) gate 3 PNG validation; the two condition branches are journal-recorded and mutually exclusive |

## Risk R1 clause (encoding guard vs binary blobs - read before committing)

`encoding_guard.py` classifies "binary" as "contains a NUL byte"
(`looks_binary`). A realistic JPEG/PNG entropy stream contains 0x00 bytes and
is then skipped cleanly. IF either new blob is NUL-free, the guard fails it
as "not utf-8 decodable". That outcome is a STOP-REPORT, not a fix target:
never edit `encoding_guard.py`, never force-add an exclusion; a guard change
requires its own dated ledger row (operator decision). Record the exact guard
output line in the journal and report; the orchestrator dispositions (e.g. a
re-save of the asset in `media/` is source-of-truth territory - NOT yours).

## Gates (exact commands; implementer hop; timeout 600000 ms each)

1. Step 0: journal `TRACK_BASE <rev>`; record byte sizes of
   `media/AT.jpg` (source) and the Red assertion (RGB step 1); journal the
   PIL probe result (`import PIL` -> OK or ImportError, verbatim module
   error line).
2. Banner: copy (C0) + `python tools/distill.py -- git hash-object media/AT.jpg extensions/ariadne-extension/media/banner.jpg`
   - hashes equal (journal).
3. Conditional PNG (rasterizer-yes path only): derivation +
   `python tools/distill.py -- python -c "..."` magic/IHDR validation (C2 -
   the full command there) + Read-tool eyeball line. Deferral path: skip,
   journal the dated deferral line instead.
4. Manifest (conditional): C3 edit (only if PNG committed) +
   `python tools/distill.py -- python -c "import json; ..."` parse check.
5. C1 README edit (Edit tool; before-string must match exactly - if not,
   STOP per dispatch precondition).
6. `python tools/encoding_guard.py --staged` - exit 0 (stage via commit_scoped's
   staging or the orchestrator; see Risk R1 clause on failure).
7. Commit with `python tools/commit_scoped.py --scope extensions/ariadne-extension/media
   --scope extensions/ariadne-extension/README.md
   --scope extensions/ariadne-extension/package.json
   --scope tasks/branding-assets-tb-journal.md
   -m "feat(extension): README banner from canonical media + brand icon"`
   (one landing commit; on the deferral path the package.json scope stages
   nothing - harmless and honest; never `git add .`; if your session's
   permissions deny `commit_scoped.py`/`git`, STOP and report - commit duty
   routes to the orchestrator per the standing allowlist errata row).

Journal duty: itemize per-gate invocation counts (expect ~6-8 on the
rasterizer-yes path, ~5 on deferral); DECLARE any overshoot beyond 10 as a
dated D-row. Workspace-wide gates (`encoding_guard --all`, census, hygiene
sweep) belong to the reviewer hop.

## Reviewer hop (fresh session; adjudicate via `review-gate`; re-run FRESH)

1. `git diff --name-only TRACK_BASE..HEAD` set-equality against the Touch-set
   minus the journal - EXACT (deferral path: `icon.png` + `package.json` must
   be ABSENT from the set).
2. Fresh hash-object equality `media/AT.jpg` vs
   `extensions/ariadne-extension/media/banner.jpg` - equal.
3. `python tools/encoding_guard.py --all` exit 0 with the new blobs tracked
   (this is also R1's final arbiter; a failure there routes the same
   STOP-report disposition).
4. I4 branch check: which branch shipped? If `icon` set: path exists, gate 3
   PNG validation re-run fresh (magic + IHDR >=128). If unset: no `icon.png`
   in HEAD, no `"icon"` in `package.json`, journal deferral line present.
   Mixed state (icon set + no file, or file + field absent) = blocker.
5. `python tools/hygiene_check.py` exit code identical to the pre-landing
   baseline (this handoff adds no scanned-suffix file).
6. `git show` per-commit: README additions-only hunk (emoji lines survive
   byte-identical), `package.json` single-line hunk or none; possessive
   product name in every new string; ASCII.
7. Eyeball relay: the journal's PNG eyeball line (or deferral line) is
   present; the reviewer re-attaches `icon.png` if shipped.

## Discipline clauses

- UTF-8 / binary safety (kickoff E2 lock; AGENTS.md File-Edit Discipline):
  never `Set-Content`/`Out-File`/`>`/`Get-Content -replace` - the README's
  existing emoji lines are destroyed by PowerShell text pipes; Read/Edit
  tools only for text, byte-level Python only for binaries. Authored text
  pure ASCII; binary byte-escapes in validation commands, never literals.
- Touch only the touch-set; on discovering out-of-scope breakage (e.g. the
  guard rejecting a blob): report, do not fix, do not "help" by re-saving
  source media.
- Bare high-volume commands (raw `cargo`, full `unittest discover`,
  `ui_smoke`, `bench_compare`, raw census) are irrelevant to this scope and
  still reviewer blockers if invoked; use the distilling entry points for
  everything with bulk output (`docs/specs/tool-output-distillation.md`).
- After EACH Green step, append a 1-3 line checkpoint to
  `tasks/branding-assets-tb-journal.md`: step + commit hash, the rasterizer
  branch taken, byte sizes, any deviation, any lock/contract finding. The
  journal is the dead session's decision log; without it, recovery restores
  code (git) but loses the branch choice - which is exactly the reasoning
  that cannot be recovered from the tree.
- If context exhaustion looms: STOP, return a disk-state inventory; a dead
  session is NEVER resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/branding-assets-tb-handoff.md --scope extensions/ariadne-extension --cmd "python tools/encoding_guard.py --all"`.

## Report back (final message)

1. Commit hash + `git show --stat`. 2. Red evidence (the pre-copy assertion
failure line). 3. Green evidence: the two `git hash-object` hashes, the JSON
parse line, the PNG validation line or the dated deferral line, README hunk
summary (`+N/-0`), byte sizes of every new blob. 4. Branch taken (rasterizer
yes/no) + reason. 5. Any deviation + reason (declare gate-count overshoot as
a D-row). 6. Out-of-scope observations (report-only). Never paste raw tool
output: cite command + output line + exit code; the reviewer re-runs against
the commits.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces the Touch-set above EXACT.
This handoff has NO tools/ or crates/ riders - any file outside
`extensions/ariadne-extension/` plus the journal in the diff is a blocker.
