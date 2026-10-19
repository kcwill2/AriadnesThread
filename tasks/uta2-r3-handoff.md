# UTA2-R3 Handoff - repo + lang-api: symlink guard + catalog/policy matrices

Binding inputs:
1. This file. 2. docs/specs/unit-test-audit.md "Round 2" (UTB-8 fix; repo/lang-api rows 3/5/6/7/15; dead-arm report). 3. crates/repo/src/discovery.rs (walk), policy.rs, types.rs, lib.rs (locate); crates/lang-api/src/catalog.rs. 4. Model: T7a's share_mode deterministic-injection precedent (tests/uta_t7a_discovery_error_test.rs).

## Touch-set (hard)
- tasks/uta2-r3-journal.md
- crates/repo/src/**, crates/lang-api/src/** (+ their tests dirs)
You MUST NOT touch: other crates, tools. No stash/branch change.

## What to build
FIX (red test first):
1. UTB-8 symlink-loop infinite walk: canonical-path visited set in
   discover_repository_files_with_progress (dirs canonicalized at push time;
   canonicalize-fail falls back to the raw path per existing file-side
   precedent). Regression test builds a directory loop (Windows: `cmd /c
   mklink /J`; unix: std::os::unix symlink_dir), runs discovery with a
   watchdog bound (spawn thread + timeout join, or the walk in a child
   thread), asserts the walk terminates and the loop dir appears once;
   environment-forbids-creation -> graceful skip with journal note (T7a
   precedent). Red = walk hangs without the fix; demonstrate via the bound
   failing on pre-fix code (journal capture).

REQUIRED PINS (no behavior change):
2. policy.rs classify_ownership_and_policy: full keyword table
   (third-party, vendor, deps, external, gen, build, out, target, $build),
   mixed case, mid-path positions; + REPORT (no test fix) that
   is_toolchain=true is never passed by production callers (dead arm).
3. types.rs FileKind::from_path extension matrix (cc,cxx,hh,inl,ipp,mill,c,
   sc, upper/lower case incl. .H -> Header vs is_includable_header
   case-sensitivity divergence - pin exactly, report).
4. walk error-polarity characterization: pin entry?/metadata? hard-abort on
   an unreadable FILE (locked-file injection or deletion race via
   share_mode) alongside the :43-45 silently-continuing read_dir arm -
   journal the policy-proposal (do not unify).
5. Repository::locate bad-path Err context text lock.
6. catalog.rs by_basename collision first-insert-wins pin (report:
   B4-class; deterministic-by-insertion-order); by_module_path_suffix
   component-boundary (`models.User` vs `xmodels.User`) + empty-query rule;
   by_path canonical-path fallback arm.

## RGB, hygiene, locks
Red-first for the fix; pins that expose unlisted wrong behavior: pin + report, no fix.
Hygiene/encoding exit 0; existing repo integration suites (m01 etc.) green.

## Gates (timeout 600000 ms each)
1. `python tools/testrun.py --distill -- -p ariadne-repo`
2. `python tools/testrun.py --distill -- -p ariadne-lang-api`
3. hygiene, encoding (all/staged witness)
4. Commit: `python tools/commit_scoped.py --scope "crates/repo/**" --scope "crates/lang-api/**" --scope "tasks/uta2-r3-journal.md" -m "fix(repo): uta2-r3 canonical visited set bounds symlink-loop walks; test(repo,lang-api): policy/kind/catalog matrices"` (truth-adjust).

## Discipline
Write/Edit only, pure ASCII; distilling entry points; journal per gate; STOP on context wall.

## Report back
Commit hash + stat; red evidence (pre-fix hang proof); per-crate canonical lines; deviations; skips (symlink-creation-forbidden path if taken); out-of-scope observations.
