# UTA2-R3 Journal - repo + lang-api (UTB-8 fix + catalog/policy matrices)

Handoff: tasks/uta2-r3-handoff.md. Binding spec: docs/specs/unit-test-audit.md
"Round 2" (UTB-8 fix; repo/lang-api pin rows; dead-arm report).
Start state: main at tip eaeaf2d (uta2 R1/R2 landed), tree carried only
reviewer artifacts outside this touch-set (tasks/uta-execution-results.md
modified, tasks/uta2-r1-verdict.md untracked) - deliberately not touched.

## Item 1 (FIX, red-first): UTB-8 symlink/junction-loop unbounded walk

Red (pre-fix, code untouched except the new test file):
- Test: crates/repo/tests/uta2_r3_symlink_loop_test.rs (built FIRST, with the
  watchdog bound baked in before any run: discovery executes on a spawned
  thread judged via `mpsc::Receiver::recv_timeout(15s)`, so the session can
  never hang on the walk; this run was additionally bounded externally
  (testrun --timeout 240 + a command timeout)).
- Loop: Windows junction `sub/loop -> sub` via `cmd /c mklink /J` (no
  elevation needed, succeeded on this host).
- Command: `python tools/testrun.py --distill --timeout 240 -- -p
  ariadne-repo --test uta2_r3_symlink_loop_test`
- Red digest (verbatim): `0 passed / 1 failed / 0 ignored / 1 suites`;
  capture: target/testrun-capture.txt.
- Red shape: the walk failed the exactly-predicted semantic reason - the
  unguarded stack re-descended the alias 70 levels deep
  ("sub/inner.cpp", "sub/loop/inner.cpp", ... one duplicate record per
  level). The theoretical infinity was truncated on this host by the OS path
  ceiling before the 15 s watchdog bound tripped; depending on base-path
  length and file fan-out the same walk does NOT finish inside the bound
  (the hang class UTB-8 names). Both red shapes - watchdog timeout or
  duplicate blow-up - fail the single assertion, and both are caused by the
  missing visited set. No fixture/harness error.
- Junction creation SUCCEEDED: the graceful-skip path was NOT taken on this
  host (skip arm exists + eprintlns loudly for environments that forbid
  loop creation, per the T7a deterministic-injection precedent).

Green fix (crates/repo/src/discovery.rs, minimal):
- Canonical-path visited set: root canonicalized at seed; every directory
  canonicalized at push time and admitted only when
  `walked_dirs.insert(canonical)` reports first sight.
- canonicalize failure falls back to the RAW path, mirroring the existing
  file-side precedent (discovery.rs file branch
  `path.canonicalize().unwrap_or_else(|_| path.clone())`) as directed.
- The RAW path is what gets pushed/walked (not the canonical one), so
  `strip_prefix(root)` - hence FileId and record paths - are unchanged for
  every dir on loop-free trees. Post-fix the first-walked instance of an
  aliased dir is the only one walked; on loop-free trees the visible record
  set is byte-for-byte what it was (existing suites confirm).
- Green scoping run: `1 passed / 0 failed / 0 ignored / 1 suites`, then full
  repo crate pre-pins: `27 passed / 0 failed / 0 ignored / 4 suites`
  (zero regressions in the existing m01-era discovery suites).

## Item 2 (PINS + REPORT): policy.rs classify_ownership_and_policy

- New inline pins (policy.rs tests): full keyword table (third_party,
  third-party, vendor, deps, external | generated, gen, build, out, target,
  $build) at leading and mid-path positions; case-insensitivity (folded
  components); whole-component-exactness anti-pins (vendorish, vendor_libs,
  targeting, outsized, third-partydeps all stay Optimizable); first-matching-
  component positional precedence (vendor/gen vs gen/vendor flip the class);
  contract arms (include/public/ prefix, mid-path /public/ contains, folded
  case) with near-miss anti-pins (public/x.h, src/publicsneak/x.h,
  include/publicx/api.h -> Optimizable) and keyword-loop-before-contract
  precedence (vendor/public/x.h -> ThirdParty).
- REPORT: `is_toolchain=true` is production-dead. The only production call
  site is discovery.rs `classify_ownership_and_policy(rel_path, false)`
  (hardcoded false); `git grep is_toolchain -- crates tools` finds only the
  policy.rs signature/arm itself. Arm behavior pinned
  (`pin_toolchain_flag_arm_is_production_dead`: flag short-circuits all path
  rules -> (Toolchain, Frozen)) so the dead arm's semantics are on record.
  Wiring or removal is an owner decision, not this hand's.

## Item 3 (PINS + REPORT): types.rs FileKind::from_path matrix

- New inline module `file_kind_matrix_tests`: lowercase extension matrix
  (cpp/cc/cxx/c -> Source; h/hpp/hxx/hh/inl/ipp -> Header; scala; sc; sbt/
  mill; other shapes: no-ext, dotfile, multi-dot, .h.rs), and the full
  upper/mixed-case matrix proving from_path case-FOLDS (.H -> Header, .CC ->
  Source, .MILL -> BuildDefinition, ...).
- REPORT (divergence pinned exactly, NOT unified): `from_path` folds case;
  `is_includable_header` is case-SENSITIVE ends_with on .h/.hpp/.hxx. So
  .H/.HH/.HPP/.HXX/.INL/.IPP are Header (analyzable) yet NOT includable;
  and the dotfile contrast: ".h" is Other for from_path (no extension per
  Path rules) but includable=true for is_includable_header (ends_with on the
  bare name). Unification = owner decision.

## Item 4 (PINS both polarities + policy proposal; NOT unified)

- Silent polarity pin (crates/repo/tests/uta2_r3_walk_polarity_test.rs,
  cross-platform): failing root `read_dir` is swallowed by the
  `Err(_) => continue` arm -> empty Ok walk. This is the already-deferred
  "unreadable repo looks empty" ledger class (reference, not re-proposed).
- Loud polarity pin (Windows, T7a share_mode(0) precedent): one locked
  analyzable file beside a readable sibling aborts the WHOLE walk with Err
  surfacing "Failed to read file for hashing" + the offending file name -
  proving the per-file `?` sites are propagate-never-skip.
- Characterization note: the deterministic injection trips the propagation
  polarity via the hashing read; `entry?`/`metadata?` are the same `?`
  polarity (adjacent sites in the same loop) but have no deterministic
  injection without a deletion race (metadata/open succeed on a share-locked
  file; dangling links fail is_file/is_dir before the `?` sites). Journaling
  the adjacency argument instead of shipping a flaky race.
- Policy proposal (owner decision, NOT applied here): unify to
  "per-entry failures collected as diagnostics + continue; root-level
  read_dir failure fatal (loud)" - both today's polarities are defensible
  only in isolation; the contradiction is the finding.

## Item 5 (PIN): Repository::locate bad-path Err context text

- lib.rs inline pin locks the chain: context text "Failed to locate
  repository path:" + the offending path echoed in its exact Debug rendering
  (`{:?}` - quoted, backslashes escaped on Windows). DEVIATION (minor,
  disclosed): first draft asserted the raw lossy string and failed - that
  failure was an assert-shape error on my side, not an unlisted behavior;
  the shipped pin asserts the exact Debug form, which is the truthful
  contract.

## Item 6 (PINS + REPORT): lang-api catalog.rs

- by_basename collision FIRST-INSERT-WINS pinned harder: both construction
  orders flip the winner (a/hash.h vs b/hash.h) - insertion-order dependent,
  NOT key-internal. REPORT (B4-class): deterministic today only because
  discovery path-sorts records before catalog construction; any unsorted
  caller inherits the UTA-B4 nondeterminism class. Key-internal tie-break
  (as H1's graph fix) = owner decision.
- Module-key collision pin: two extensions of one module name share one key;
  first inserted wins in either order (User.scala vs User.java).
- Suffix boundary pin in the handoff's own shape: `models.User` hits only
  the genuine models file, never the xmodels neighbor - on BOTH matching
  arms (file-level suffix AND package-dir parent arm), plus the converse
  query.
- Empty-query rule extended beyond the literal "": ".", "/", ".." all
  unify_segments() to empty and must match nothing.
- by_path canonical-fallback arm pin: an absolute candidate carrying `..`
  resolves through the dot-collapsing canonical_key (rel arm after
  collapsing also pinned).

## Gate record (each distilling entry point, digest verbatim)

1. repo: `python tools/testrun.py --distill --timeout 240 -- -p ariadne-repo`
   -> `38 passed / 0 failed / 0 ignored / 5 suites` (capture:
   target/testrun-capture.txt). Baseline 27 + 11 pins/tests = 38.
2. lang-api: `python tools/testrun.py --distill --timeout 240 -- -p
   ariadne-lang-api` -> `47 passed / 0 failed / 0 ignored / 5 suites`.
   Baseline 42 + 5 pins = 47.
3. hygiene: `python tools/hygiene_check.py` -> exit 0; output is R3 warn-only
   and every warn is PRE-EXISTING (none inside this hand's added lines; the
   two hits in touched files - catalog.rs:188 `c`, lib.rs:94-98 `res` - sit
   in untouched pre-existing code). R1/R2: zero findings.
   encoding: `python tools/encoding_guard.py --all` -> exit 0 (silent, full
   tracked tree witness). `--staged` post-commit reports the empty index
   (commit_scoped stages+commits atomically), so --all is the substantive
   witness, mirroring the earlier hands' sequencing.
4. commit: single scoped commit via commit_scoped.py (gates 1-2 green before
   it; hash reported in the handback, visible via `git show --stat HEAD`).

## Skips

- Symlink/junction-creation graceful skip NOT taken (mklink /J succeeded on
  this Windows host). The skip arm exists and is loud for restricted
  environments and unix CI without symlink privileges.
- The windows-only polarity mod compiles away on unix (T7a `#![cfg(windows)]`
  precedent shape, here per-mod so the cross-platform silent-polarity pin
  still runs there).

## Deliberately NOT done (with reasons)

- Did not unify the walk error polarities (handoff: characterize + propose).
- Did not touch the deferred empty-Ok root class beyond pinning its silent
  polarity (ledger already owns it).
- Did not change by_basename collision policy (report-only, B4-class).
- Did not touch any file outside crates/repo, crates/lang-api,
  tasks/uta2-r3-journal.md. No golden fixtures / byte-locked manifests were
  touched; the ar27 lock tests ran green untouched.

## Out-of-scope observations (report only)

- None new: the empty-Ok root and the alias-first-walked LIFO determinism
  (which of two aliases to the same dir wins on case-insensitive
  filesystems) are both consequences already covered by existing ledger
  entries / this hand's pins; the second is noted here for the reviewer's
  deferred-findings candidate list.
