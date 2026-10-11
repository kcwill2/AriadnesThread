# AR-10 Verdict — hoist wildcard regex in search (P7)

Commit adjudicated: `3a26a9a` (single commit; the other commits in
70e1699..HEAD belong to the scheduler/AR-16 lanes, reviewed elsewhere).

## VERDICT: APPROVED

## Gate lines (reviewer's own fresh runs, verbatim)

1. `python tools/testrun.py --distill -- -p ariadne-ui` ->
   `256 passed / 0 failed / 0 ignored / 3 suites`
   (capture: target/testrun-capture.txt; all 11 `search_characterization::`
   tests visible and `ok` in the capture at final HEAD)
2. `python tools/distill.py -- python tools/ui_smoke.py` -> `ui_smoke: PASS`
3. `python tools/encoding_guard.py --all` -> clean (no output, exit 0)
4. `python tools/hygiene_check.py` -> server.rs blocking rows are exactly the
   two AR-13-owned ones:
   - `crates/ui/src/server.rs: R2 1172 lines > 1000`
   - `crates/ui/src/server.rs:816: R1 fn `api_graph_traversal` is 161 lines (> 100)`
   No R1 row for `api_graph_search` (it spans server.rs:654-744 = 91 lines,
   under the 100 ceiling). The `m15_graph_ui_test.rs:9: R1 ... 104 lines` row
   belongs to the foreign, dirty tests/integration lane (checkpoint_guard
   confirms that file is dirty vs HEAD; it is not in this commit's paths).
5. Tools suites: `python -m unittest discover -s tools -p "*_tests.py"` ->
   `Ran 219 tests in 18.423s` / `OK`

## Commit-vs-touch-set

Exactly the 4 claimed paths: server.rs (26+/9-), ui_test/main.rs (+1),
search_characterization.rs (+279, new), journal (+137). All inside the
touch-set (see disposition 3 on the main.rs registration line). server.rs
diff hunks confined to the search-handler region (~631-694) per the
"search handler region only" constraint.

## Plan-literal / semantics verification

- Branch order intact: `/regex/` literal -> wildcard glob -> substring
  fallback (server.rs:685-693 mirrors the old closure structure exactly).
- Case-sensitive `/regex/` path untouched: lines 672-677 (`starts_with('/')
  && ends_with('/')`, `q.len() > 1` guard, raw compile, raw-target match)
  are outside the diff; the diff removes only the per-element wildcard
  branch and its two `to_lowercase()` calls.
- `build_wildcard_glob` (server.rs:641) is private (`fn`, no `pub`), called
  once per request (server.rs:683). Grep confirms the only `Regex::new`
  sites are the once-per-request regex_pattern (674) and the
  once-per-request glob (651); the only per-element `to_lowercase` left is
  the substring fallback (692) — out of the handoff's hoist scope, disclosed.
- Compile-failure parity: old per-element `if let Ok(re)` fall-through ==
  new `.ok()` -> `None` -> substring fallback stays live. Same outcomes.
- Ordering/limit locks green; 256 = 245 baseline + 11 new with zero failures
  (existing graph_api.rs search tests included).

## Numbered dispositions

1. **(?i) simple Unicode fold vs `str::to_lowercase` full-mapping
   divergence — ACCEPTED.** Journal discloses it as the only known
   divergence class. Adjudicated on merits: (a) the handoff itself directs
   the design ("case-insensitive regex replaces both `to_lowercase()`
   allocs"); (b) the handoff constraint "Pure ASCII only" plus repo-derived
   C++ paths/labels confine real data to ASCII, where every outcome is
   provably identical (both paths fold ASCII the same); the divergence
   needs exotic full-fold pairs (U+0130, ẞ-family) in a label/path, which
   the parser pipeline does not produce; (c) no characterization lock
   would have caught it — the fixture is ASCII-only by the same constraint,
   so the class is structurally out of lock reach, which is exactly why
   disclosure is the correct instrument. No required change. Carry the
   class to `tasks/milestone_ledger.md` deferred-findings if AR-13 or a
   future Unicode-identifier feature ever widens the label alphabet.
2. **`build_wildcard_glob` helper extraction (declared deviation) —
   ACCEPTED.** Adjudicated independently: inline growth would have pushed
   `api_graph_search` past the 100-line R1 ceiling, ADDING a third blocking
   hygiene row and breaking gate 4's "zero NEW violations". Extraction
   keeps the handler at 91 lines; net server.rs diff stays +26/-9, within
   the handoff's ~20-40 line band. On-merits sound.
3. **ui_test/main.rs +1 registration — WITHIN TOUCH-SET (interpretation
   note, not a violation).** The touch-set names `server_api.rs` OR "a new
   test mod"; the sanctioned new-mod option mechanically requires a `mod`
   declaration in the harness main.rs. Precedent verified in git history:
   AR-02 `b49ed06` added the identical 1-line `mod cors_same_origin;`.
4. **f11 -> f10 expectation correction — ACCEPTED as characterization done
   correctly.** The committed lock expects slot 20 = `f10`; I re-derived it
   from handler semantics: the empty query substring-matches ALL nodes
   including label-less node 9 (positions 0-8 = "1".."9"), so position 19 =
   `f10` (it would be `f11` only if node 9 were excluded — the implementer's
   initial wish). Critically, the empty-query path runs the substring
   fallback, which is byte-identical pre/post change, so green-at-final HEAD
   provably implies green pre-change for that lock too. No other expectation
   moved: all 11 locks are green at the final commit (own capture) and the
   wildcard locks' expectations are each derivable from the OLD code's
   semantics (spot-derived: `RENDER_*` anchored, `/*a*/` invalid-regex ->
   glob `^/.*a.*/$`, `/` len-guard vs `//` empty-regex).
5. **TDD evidence shape — ACCEPTED.** Journal is digest-shaped: distilling
   entry points only, capture paths, `N passed / M failed` lines, red run
   (`10 passed / 1 failed`) then 11-green pre-change, no raw dumps. The
   red run itself is unverifiable post-hoc from a single squashed commit —
   inherent to the handoff shape, not a finding.
6. **Foreign state — ATTRIBUTED, not findings.** The m15_graph_ui_test.rs
   R1 row, tests/integration + crates/pipeline dirt, and the m10 R3 warns
   all belong to the concurrent tests/integration lane (checkpoint_guard
   dirty set matches exactly). Pre-existing `crates/build` clippy
   `type_complexity` (journal-recorded, out of scope): route to
   milestone_ledger deferred-findings.

## Not run (explicit)

- `python tools/testrun.py --baseline` (workspace-wide): per dispatch
  restriction — concurrent lanes hold tests/integration and crates/graph
  dirty mid-edit; the workspace gate is the orchestrator's per the Planning
  Law. Scoped `--distill -p ariadne-ui` was the sanctioned equivalent and
  was run.
- `bench_compare.py`: explicitly out of AR-10 contract (not a pipeline
  path); journal substitutes an analytic cost argument. No bench required
  by this milestone handoff.
