#!/usr/bin/env python3
"""Checkpoint guard: deterministic recovery from a dead implementer session
(repo incidents E1/E16). Prints canonical repo state and bootstraps a fresh
session from a handoff file WITHOUT ever guessing intent.

Usage:
  python tools/checkpoint_guard.py status
  python tools/checkpoint_guard.py resume --spec <handoff.md>
      [--scope <glob>]... [--cmd <command line>]...
Exit codes: 0 clean/success, 1 violation or failing verdict found,
2 internal error (bad args, git failure).
"""

import fnmatch
import os
import re
import shlex
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import List, Optional

NEVER_RESUME_RULE = (
    "RULE: Sessions whose compaction summary was empty are NEVER resumed. "
    "This report is the fresh-session bootstrap."
)
MAIN_REF = "main"
RECENT_COMMIT_COUNT = 5
COMMAND_TIMEOUT_SECONDS = 600
TAIL_LINES = 5
USAGE = (
    "usage: python tools/checkpoint_guard.py status | resume --spec "
    "<handoff.md> [--scope <glob>]... [--cmd <line>]..."
)
COMMAND_PREFIXES = ("cargo ", "python ", "python3 ")
TOUCH_SET_HEADING = re.compile(r"^\s*##\s+Touch-set\b")
LEVEL_TWO_HEADING = re.compile(r"^\s*##\s")
BACKTICK_TOKEN = re.compile(r"`([^`\s]+)`")


class GuardError(Exception):
    pass


@dataclass
class WorktreeEntry:
    path: str
    detached: bool = False
    branch: Optional[str] = None


@dataclass
class DirProbe:
    ref: str = "n/a"
    ahead: str = "n/a"
    behind: str = "n/a"
    dirty: str = "n/a"
    is_git: bool = False


@dataclass
class HandoffParse:
    globs: List[str] = field(default_factory=list)
    commands: List[str] = field(default_factory=list)
    error: Optional[str] = None


@dataclass
class ResumeArguments:
    spec: Optional[str] = None
    scopes: List[str] = field(default_factory=list)
    commands: List[str] = field(default_factory=list)


@dataclass
class CommandVerdict:
    command_line: str
    verdict: str
    tail: str


def git_run(root, arguments, cwd=None, allow_failure=False):
    finished = subprocess.run(
        ["git", *[str(argument) for argument in arguments]],
        cwd=str(cwd if cwd is not None else root),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if finished.returncode != 0 and not allow_failure:
        joined = " ".join(str(argument) for argument in arguments)
        raise GuardError(f"git {joined} failed: {finished.stderr.strip()}")
    return finished


def git_text(root, *arguments):
    return git_run(root, arguments).stdout


def head_state(root):
    short_sha = git_text(root, "rev-parse", "--short", "HEAD").strip()
    symbolic = git_run(root, ["symbolic-ref", "--short", "-q", "HEAD"],
                       allow_failure=True)
    branch = symbolic.stdout.strip() if symbolic.returncode == 0 else "DETACHED"
    return short_sha, branch


def collect_dirty(root):
    entries = []
    for line in git_text(root, "diff", "HEAD", "--numstat").splitlines():
        fields = line.split("\t")
        if len(fields) >= 3:
            entries.append((fields[0], fields[1], fields[2]))
    return entries


def collect_untracked(root):
    listing = git_text(root, "ls-files", "--others", "--exclude-standard")
    return [line for line in listing.splitlines() if line.strip()]


def collect_recent_commits(root):
    return git_text(root, "log", "--oneline", f"-n{RECENT_COMMIT_COUNT}").splitlines()


def group_key(relative_path):
    parts = relative_path.split("/")
    if len(parts) > 1 and parts[0] == "crates" and len(parts) > 2:
        return "crates/" + parts[1]
    return parts[0] if len(parts) > 1 else "(repo root)"


def print_dirty_section(entries):
    print("== DIRTY FILES (git diff HEAD --numstat: worktree + index vs HEAD) ==")
    if not entries:
        print("(none)")
        return
    grouped = {}
    for added, deleted, path in entries:
        grouped.setdefault(group_key(path), []).append(f"{added}\t{deleted}\t{path}")
    for key in sorted(grouped):
        print(f"[{key}]")
        for line in sorted(grouped[key]):
            print(f"  {line}")


def print_untracked_section(untracked):
    print("== UNTRACKED FILES ==")
    if not untracked:
        print("(none)")
        return
    for path in sorted(untracked):
        print(path)


def print_commits_section():
    print(f"== RECENT COMMITS (last {RECENT_COMMIT_COUNT}) ==")


def parse_worktree_porcelain(text):
    entries = []
    current = None
    for line in text.splitlines():
        if line.startswith("worktree "):
            current = WorktreeEntry(path=line[len("worktree "):])
            entries.append(current)
        elif line == "detached" and current is not None:
            current.detached = True
        elif line.startswith("branch ") and current is not None:
            current.branch = line[len("branch "):].replace("refs/heads/", "")
    return entries


def _normalized(path) -> str:
    return os.path.normcase(str(Path(str(path)).resolve()))


def kilo_worktree_dirs(root, registered_norms):
    base = Path(root) / ".kilo" / "worktrees"
    if not base.is_dir():
        return []
    candidates = []
    for child in sorted(base.iterdir()):
        if child.is_dir() and _normalized(child) not in registered_norms:
            candidates.append(child)
    return candidates


def probe_directory(path):
    probe = DirProbe()
    rev_parse = git_run(path, ["rev-parse", "--git-dir"], allow_failure=True)
    if rev_parse.returncode != 0:
        return probe
    probe.is_git = True
    symbolic = git_run(path, ["symbolic-ref", "--short", "-q", "HEAD"],
                       allow_failure=True)
    probe.ref = symbolic.stdout.strip() if symbolic.returncode == 0 else "DETACHED"
    counts = git_run(path, ["rev-list", "--left-right", "--count",
                            f"{MAIN_REF}...HEAD"], allow_failure=True)
    if counts.returncode == 0:
        fields = counts.stdout.split()
        if len(fields) == 2:
            probe.behind, probe.ahead = fields
    status = git_run(path, ["status", "--porcelain", "--untracked-files=all"],
                     allow_failure=True)
    if status.returncode == 0:
        probe.dirty = str(len([line for line in status.stdout.splitlines()
                               if line.strip()]))
    return probe


def print_census_line(display_path, ref, probe, flag):
    suffix = f" {flag}" if flag else ""
    print(f"* {display_path} branch={ref} ahead={probe.ahead} "
          f"behind={probe.behind} dirty={probe.dirty}{suffix}")


def print_worktree_census(root):
    print("== WORKTREE CENSUS (git worktree list + .kilo/worktrees union) ==")
    listed = parse_worktree_porcelain(git_text(root, "worktree", "list",
                                               "--porcelain"))
    registered_norms = {_normalized(entry.path) for entry in listed}
    for entry in listed:
        probe = probe_directory(entry.path)
        ref = "DETACHED" if entry.detached else (entry.branch or probe.ref)
        print_census_line(entry.path, ref, probe, "")
    for directory in kilo_worktree_dirs(root, registered_norms):
        probe = probe_directory(directory)
        flag = "ORPHAN-UNREGISTERED" if probe.is_git else "ORPHAN-NO-GIT"
        print_census_line(str(directory), "n/a" if not probe.is_git else probe.ref,
                          probe, flag)


def run_status(root):
    print(NEVER_RESUME_RULE)
    short_sha, branch = head_state(root)
    print("== HEAD ==")
    print(f"sha: {short_sha}")
    print(f"branch: {branch}")
    print_dirty_section(collect_dirty(root))
    print_untracked_section(collect_untracked(root))
    print_commits_section()
    for line in collect_recent_commits(root):
        print(line)
    print_worktree_census(root)
    return 0


def _dedupe(sequence):
    seen = set()
    kept = []
    for item in sequence:
        if item not in seen:
            seen.add(item)
            kept.append(item)
    return kept


def _touch_set_lines(lines):
    collecting = False
    for line in lines:
        if not collecting and TOUCH_SET_HEADING.match(line):
            collecting = True
            continue
        if collecting and LEVEL_TWO_HEADING.match(line):
            break
        if collecting:
            yield line


def _extract_touch_set_globs(lines):
    globs = []
    for line in _touch_set_lines(lines):
        for token in BACKTICK_TOKEN.findall(line):
            if "/" in token or "*" in token:
                globs.append(token.replace("\\", "/"))
    return _dedupe(globs)


def _command_candidate(text):
    candidate = text.strip()
    for marker in ("-", "*"):
        if candidate.startswith(marker + " "):
            candidate = candidate[len(marker) + 1:].strip()
    if len(candidate) > 1 and candidate.startswith("`") and candidate.endswith("`"):
        candidate = candidate[1:-1].strip()
    return candidate if candidate.startswith(COMMAND_PREFIXES) else None


def _extract_command_lines(lines):
    commands = []
    inside_fence = False
    for line in lines:
        if line.strip().startswith("```"):
            inside_fence = not inside_fence
            continue
        candidate = None
        if inside_fence:
            if line.strip().startswith(COMMAND_PREFIXES):
                candidate = line.strip()
        elif line.strip().startswith(("-", "*")):
            candidate = _command_candidate(line)
        if candidate is not None:
            commands.append(candidate)
    return _dedupe(commands)


def parse_handoff(spec_path):
    try:
        text = Path(str(spec_path)).read_text(encoding="utf-8", errors="replace")
    except OSError as read_failure:
        return HandoffParse(error=f"spec unreadable: {read_failure}")
    lines = text.splitlines()
    return HandoffParse(globs=_extract_touch_set_globs(lines),
                        commands=_extract_command_lines(lines))


def parse_resume_arguments(arguments):
    parsed = ResumeArguments()
    index = 0
    while index < len(arguments):
        option = arguments[index]
        if option not in ("--spec", "--scope", "--cmd"):
            raise GuardError(f"unknown resume option: {option}. {USAGE}")
        if index + 1 >= len(arguments):
            raise GuardError(f"{option} requires a value. {USAGE}")
        value = arguments[index + 1]
        if option == "--spec":
            parsed.spec = value
        elif option == "--scope":
            parsed.scopes.append(value.replace("\\", "/"))
        else:
            parsed.commands.append(value)
        index += 2
    return parsed


def split_command_line(command_line):
    raw_tokens = shlex.split(command_line, posix=False)
    tokens = []
    for token in raw_tokens:
        quoted = len(token) > 1 and token.startswith('"') and token.endswith('"')
        tokens.append(token[1:-1] if quoted else token)
    return tokens


def _tail_of(*streams):
    combined = [line for stream in streams for line in stream.splitlines()
                if line.strip()]
    return " / ".join(combined[-TAIL_LINES:])[:160]


def run_scoped_command(command_line, root):
    try:
        tokens = split_command_line(command_line)
    except ValueError as split_failure:
        return CommandVerdict(command_line, "red ERROR", str(split_failure))
    if not tokens:
        return CommandVerdict(command_line, "red ERROR", "empty command line")
    try:
        finished = subprocess.run(tokens, cwd=str(root), capture_output=True,
                                  text=True, errors="replace",
                                  timeout=COMMAND_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        return CommandVerdict(command_line, "red TIMEOUT",
                              f"no result after {COMMAND_TIMEOUT_SECONDS}s")
    except OSError as exec_failure:
        return CommandVerdict(command_line, "red ERROR", str(exec_failure))
    verdict = "green" if finished.returncode == 0 else f"red (exit {finished.returncode})"
    return CommandVerdict(command_line, verdict,
                          _tail_of(finished.stdout, finished.stderr))


def path_in_scope(relative_path, scope_globs):
    posix_path = relative_path.replace("\\", "/")
    for pattern in scope_globs:
        normalized = pattern.replace("\\", "/")
        if fnmatch.fnmatchcase(posix_path, normalized):
            return True
        if posix_path.startswith(normalized.rstrip("/") + "/"):
            return True
    return False


def classify_edits(edits, scope_globs):
    in_scope = [path for path in edits if path_in_scope(path, scope_globs)]
    strays = [path for path in edits if not path_in_scope(path, scope_globs)]
    return in_scope, strays


def resume_report_name(spec):
    if not spec:
        return "manual-resume.md"
    base = Path(spec).name
    if base.lower().endswith(".md"):
        base = base[: -len(".md")]
    return f"{base}-resume.md"


def _report_verdict_rows(verdicts):
    rows = ["| command | verdict | tail |", "| --- | --- | --- |"]
    for verdict in verdicts:
        command = verdict.command_line.replace("|", "\\|")
        tail = verdict.tail.replace("|", "\\|")
        rows.append(f"| {command} | {verdict.verdict} | {tail} |")
    return rows


def write_resume_report(root, spec_label, short_sha, branch, dirty, untracked,
                        verdicts, in_scope, strays, spec_path):
    report_dir = Path(root) / "tasks"
    report_dir.mkdir(parents=True, exist_ok=True)
    lines = [f"# Resume bootstrap for {spec_label}", ""]
    lines.append("## Rule")
    lines.append(NEVER_RESUME_RULE)
    lines.append("")
    lines.append("Run the guard suite first, before any mutation: "
                 "tools/testrun.py scoped gates, tools/encoding_guard.py "
                 "--staged, tools/hygiene_check.py.")
    lines.append("")
    lines.append("## Disk state")
    lines.append(f"- HEAD: {short_sha} ({branch})")
    lines.append("- Dirty (numstat vs HEAD): "
                 + (", ".join(path for _, _, path in dirty) or "(none)"))
    lines.append("- Untracked: " + (", ".join(untracked) or "(none)"))
    lines.append("")
    lines.append("## Command verdicts")
    lines.extend(_report_verdict_rows(verdicts))
    lines.append("")
    lines.append("## In-scope edits")
    lines.extend([f"- {path}" for path in in_scope] or ["- (none)"])
    lines.append("")
    lines.append("## Stray edits (out of scope)")
    lines.extend([f"- {path}" for path in strays] or ["- (none)"])
    lines.append("")
    lines.append("## Remaining handoff steps")
    lines.append(f"- [ ] TODO: copy remaining steps from {spec_path or 'the handoff'}")
    lines.append("")
    report_name = resume_report_name(spec_path)
    (report_dir / report_name).write_text("\n".join(lines), encoding="utf-8")
    return f"tasks/{report_name}"


def print_parse_summary(parsed, scopes, commands):
    if parsed.error:
        print(f"HANDOFF PARSE: {parsed.error}")
    print(f"parsed touch-set globs: {', '.join(parsed.globs) or '(none)'}")
    print(f"parsed commands: {', '.join(parsed.commands) or '(none)'}")
    print(f"effective scope globs: {', '.join(scopes) or '(none)'}")
    print(f"effective commands: {', '.join(commands) or '(none)'}")
    if not scopes or not commands:
        print("PARSE AMBIGUOUS: the guard will not reverse-engineer intent. "
              "Re-run with explicit --scope <glob> and --cmd '<command line>' "
              "args.")


def run_resume(root, arguments):
    requested = parse_resume_arguments(arguments)
    parsed = (parse_handoff(requested.spec) if requested.spec
              else HandoffParse(error="no --spec provided"))
    scopes = requested.scopes + [glob for glob in parsed.globs
                                 if glob not in requested.scopes]
    commands = requested.commands + [command for command in parsed.commands
                                     if command not in requested.commands]
    print(NEVER_RESUME_RULE)
    print_parse_summary(parsed, scopes, commands)
    ambiguous = not scopes or not commands
    short_sha, branch = head_state(root)
    dirty = collect_dirty(root)
    untracked = collect_untracked(root)
    edits = _dedupe([path for _, _, path in dirty] + list(untracked))
    in_scope, strays = classify_edits(edits, scopes)
    verdicts = [run_scoped_command(command, root) for command in commands]
    for verdict in verdicts:
        print(f"verdict: {verdict.command_line} -> {verdict.verdict}")
    report_relative = write_resume_report(
        root, Path(requested.spec).name if requested.spec else "(no spec)",
        short_sha, branch, dirty, untracked, verdicts, in_scope, strays,
        requested.spec)
    print(f"in-scope edits: {len(in_scope)} | stray edits: {len(strays)}")
    print(f"resume report written: {report_relative}")
    failed = any(not verdict.verdict.startswith("green") for verdict in verdicts)
    if failed or strays or ambiguous:
        return 1
    return 0


def main(argv=None):
    arguments = list(sys.argv[1:] if argv is None else argv)
    if not arguments:
        print(USAGE)
        return 2
    subcommand, rest = arguments[0], arguments[1:]
    root = Path.cwd()
    try:
        if subcommand == "status":
            return run_status(root)
        if subcommand == "resume":
            return run_resume(root, rest)
    except GuardError as failure:
        print(f"ERROR: {failure}")
        return 2
    print(USAGE)
    return 2


if __name__ == "__main__":
    sys.exit(main())
