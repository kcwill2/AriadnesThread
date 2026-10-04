#!/usr/bin/env python3
"""Scope-resolved commits: commit ONLY the working-tree content matching the
declared scopes, immune to foreign files a concurrent session may have staged
into the shared index (repo incident E5: sibling implementers polluted each
other's commits on one shared worktree).

Mechanism: resolve globs to explicit paths, `git add` them, then
`git commit --only -- <paths>` so the commit carries ONLY those paths even
with foreign staged content. This tool is ADVISORY: plain `git commit` still
bypasses it; the enforcement net is the review-gate skill's commit-vs-touch-set
check and CI. Structural fix: run sibling implementers in separate Agent
Manager worktrees; this is the fallback for shared runs.

Usage:
  python tools/commit_scoped.py --scope <glob> [--scope <glob>...] -m <msg>
      [--lock-retry-delay <seconds>]
Exit codes: 0 committed, 1 rejected scope, 2 internal error (incl. lock
exhaustion, git failure, usage).
"""

import argparse
import fnmatch
import subprocess
import sys
import time
from pathlib import Path

ROOT_LIKE_SCOPES = frozenset({".", "./", "*", "/", "", ".\\"})
LOCK_ERROR_NEEDLE = "index.lock"
MAX_LOCK_ATTEMPTS = 3
DEFAULT_LOCK_RETRY_DELAY = 10.0


def run_git(root, *arguments):
    finished = subprocess.run(
        ["git", *arguments],
        cwd=str(root),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return finished.returncode, finished.stdout, finished.stderr


class GitLockContention(Exception):
    pass


def describe_failure(arguments, status, stdout, stderr):
    """Build a failure message that names a cause.

    `git commit` reports some refusals on stdout, not stderr: "nothing to
    commit, working tree clean" exits 1 with an empty stderr. Surfacing stderr
    alone turns those into a bare "failed:" with nothing after it, which is the
    one case an operator most needs explained. Fall back to stdout, then to the
    exit status, so a failure is never silent.
    """
    detail = stderr.strip() or stdout.strip() or f"no output; exit {status}"
    return f"git {' '.join(arguments[:2])} failed: {detail}"


def run_git_with_lock_retry(root, delay, *arguments):
    last_stderr = ""
    for attempt in range(1, MAX_LOCK_ATTEMPTS + 1):
        status, stdout, stderr = run_git(root, *arguments)
        if status == 0:
            return stdout
        last_stderr = stderr
        if LOCK_ERROR_NEEDLE not in stderr:
            raise RuntimeError(describe_failure(arguments, status, stdout, stderr))
        if attempt < MAX_LOCK_ATTEMPTS:
            time.sleep(delay)
    raise GitLockContention(
        f"index.lock still held after {MAX_LOCK_ATTEMPTS} attempts: "
        f"{last_stderr.strip()}"
    )


def list_repo_paths(root):
    tracked = run_git_with_lock_retry(root, 0, "ls-files")
    untracked = run_git_with_lock_retry(
        root, 0, "ls-files", "--others", "--exclude-standard"
    )
    listing = f"{tracked}\n{untracked}"
    return sorted(
        {entry.replace("\\", "/") for entry in listing.splitlines() if entry}
    )


def scope_matches(scope, relative_path):
    if scope in ROOT_LIKE_SCOPES:
        raise ValueError(f"scope {scope!r} is a repo-root scope; refusing")
    if fnmatch.fnmatchcase(relative_path, scope):
        return True
    is_plain = not any(marker in scope for marker in "*?[")
    return is_plain and relative_path.startswith(scope.rstrip("/") + "/")


def resolve_scopes(root, scopes):
    candidates = list_repo_paths(root)
    resolved = set()
    for scope in scopes:
        matched = {
            relative_path
            for relative_path in candidates
            if scope_matches(scope, relative_path)
        }
        if not matched:
            raise SystemExitWithCode(
                1, f"scope {scope!r} resolved to nothing"
            )
        resolved |= matched
    return sorted(resolved)


class SystemExitWithCode(Exception):
    def __init__(self, code, message):
        super().__init__(message)
        self.code = code


def build_parser():
    parser = argparse.ArgumentParser(
        prog="commit_scoped.py",
        description="Commit only the working-tree content inside --scope globs.",
    )
    parser.add_argument(
        "--scope", action="append", required=True, metavar="GLOB",
        help="path glob to commit; repeatable (repo-root scopes rejected)",
    )
    parser.add_argument(
        "-m", "--message", required=True, help="commit message"
    )
    parser.add_argument(
        "--lock-retry-delay", type=float, default=DEFAULT_LOCK_RETRY_DELAY,
        help="seconds between index.lock retries (default 10)",
    )
    return parser


def commit_paths(root, paths, message, delay):
    run_git_with_lock_retry(
        root, delay, "add", "--", *paths
    )
    run_git_with_lock_retry(
        root, delay, "commit", "--only", "-m", message, "--", *paths
    )


def main(argv=None):
    parser = build_parser()
    arguments = parser.parse_args(
        list(sys.argv[1:] if argv is None else argv)
    )
    root = Path.cwd()
    try:
        paths = resolve_scopes(root, arguments.scope)
    except SystemExitWithCode as rejection:
        print(str(rejection), file=sys.stderr)
        return rejection.code
    except ValueError as bad_scope:
        print(str(bad_scope), file=sys.stderr)
        return 1
    except (GitLockContention, RuntimeError) as git_failure:
        print(f"ERROR {git_failure}", file=sys.stderr)
        return 2
    try:
        commit_paths(root, paths, arguments.message, arguments.lock_retry_delay)
    except GitLockContention as lock_failure:
        print(f"ERROR {lock_failure}", file=sys.stderr)
        return 2
    except RuntimeError as git_failure:
        print(f"ERROR {git_failure}", file=sys.stderr)
        return 2
    status, numstat, _ = run_git(
        root, "show", "--numstat", "--format=commit %h %s", "HEAD"
    )
    if status == 0:
        print(numstat)
    return 0


if __name__ == "__main__":
    sys.exit(main())
