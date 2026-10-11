#!/usr/bin/env python3
"""Theme token drift guard.

Originally (T13) this shipped as a stopgap: the three-theme system (plan
1790656207440) was specced but never landed (E16), so it only froze counts
while the plan's own Rust tests carried the parity assertions. That plan has
since executed -- themes.css/theme.js landed and swept every literal out of
styles.css, main.js, preview.js and index.html -- so the split is now:

  1. --check-added --base <rev>: flag NEW hardcoded hex literals in
     crates/ui/assets/** (a literal is new if it is absent from the file at
     <rev>), EXCEPT in TOKEN_FILES, whose literals ARE the sanctioned palette
     and are instead pinned by the count freeze; and
  2. freeze the per-file hex COUNT recorded in the committed baseline, so new
     hardcoded colors cannot hide in styles.css (invisible to the R2 size
     gate: .css is not in R2_SCANNED_SUFFIXES) and cannot inflate the token
     file unnoticed.

Rule files must reach their colors through var(--token); the token file is the
one place a hex literal is expected. tests/ui_test/theme_contract.rs asserts
both halves (styles.css carries no literals; tokens live only in themes.css).

Usage:
  python tools/theme_token_check.py --check-added --base <rev>
  python tools/theme_token_check.py --update-baseline
Exit codes: 0 clean, 1 drift violation, 2 internal/usage error.
"""

import json
import re
import subprocess
import sys
from pathlib import Path

ASSETS_DIR = "crates/ui/assets"
BASELINE_PATH = "tools/theme_token_baseline.json"
# The token file is where hardcoded colors are SUPPOSED to live: its literals
# are the palette, not drift. Exempt it from the added-literal scan and let the
# frozen count below pin it instead.
# icon.svg is sanctioned brand art (source of truth: media/AT-svg.svg): its
# literals ARE the fixed brand palette, exempt from BOTH scans exactly like
# the token file; a re-brand edits media/ then re-syncs the copy (shape D3).
TOKEN_FILES = ("themes.css", "icon.svg")
HEX_LENGTHS = (8, 6, 4, 3)
HEX_LITERAL = re.compile(
    r"#[0-9a-fA-F]{8}(?![0-9a-fA-F])"
    r"|#[0-9a-fA-F]{6}(?![0-9a-fA-F])"
    r"|#[0-9a-fA-F]{4}(?![0-9a-fA-F])"
    r"|#[0-9a-fA-F]{3}(?![0-9a-fA-F])"
)


def find_hex_literals(text):
    """All #rgb/#rgba/#rrggbb/#rrggbbaa literals in order (longest match)."""
    return [match.group(0) for match in HEX_LITERAL.finditer(text)]


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


class GitError(Exception):
    pass


def git_output(root, *arguments):
    status, stdout, stderr = run_git(root, *arguments)
    if status != 0:
        raise GitError(f"git {' '.join(arguments)}: {stderr.strip()}")
    return stdout


def file_text_at_rev(root, revision, relative_path):
    finished = subprocess.run(
        ["git", "show", f"{revision}:{relative_path}"],
        cwd=str(root),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if finished.returncode != 0:
        return None
    return finished.stdout


def iter_asset_files(root):
    assets = Path(root) / ASSETS_DIR
    if not assets.is_dir():
        return
    for path in sorted(assets.rglob("*")):
        if not path.is_file():
            continue
        payload = path.read_bytes()
        if b"\x00" in payload:
            continue
        yield path.relative_to(root).as_posix(), payload.decode(
            "utf-8", errors="replace"
        )


def scan_added_literals(root, base_rev):
    """Added-asset literals that were absent from the file at base_rev."""
    diff = git_output(root, "diff", base_rev, "--", ASSETS_DIR)
    base_cache = {}
    violations = []
    current_file = None
    for line in diff.splitlines():
        if line.startswith("+++ b/"):
            current_file = line[len("+++ b/"):]
            continue
        if not line.startswith("+") or line.startswith("+++"):
            continue
        if current_file is None:
            continue
        if current_file.rsplit("/", 1)[-1] in TOKEN_FILES:
            continue
        if current_file not in base_cache:
            base_cache[current_file] = (
                file_text_at_rev(root, base_rev, current_file) or ""
            )
        for literal in find_hex_literals(line):
            if literal not in base_cache[current_file]:
                violations.append(
                    f"{current_file}: new hex literal {literal} added since "
                    f"{base_rev}"
                )
    return violations


def scan_frozen_counts(root, baseline):
    violations = []
    for relative_path, text in iter_asset_files(root):
        literals = find_hex_literals(text)
        allowed = baseline.get("counts", {}).get(relative_path, 0)
        if len(literals) > allowed:
            violations.append(
                f"{relative_path}: {len(literals)} hex literals exceeds "
                f"frozen baseline count {allowed} (update "
                f"{BASELINE_PATH} deliberately in this commit)"
            )
    return violations


def load_baseline(root):
    path = Path(root) / BASELINE_PATH
    if not path.is_file():
        raise GitError(
            f"baseline file {BASELINE_PATH} missing; run --update-baseline "
            "and commit it"
        )
    return json.loads(path.read_text(encoding="utf-8"))


def update_baseline(root):
    counts = {}
    locations = set()
    for relative_path, text in iter_asset_files(root):
        found = find_hex_literals(text)
        counts[relative_path] = len(found)
        for line_number, line in enumerate(text.splitlines(), start=1):
            for literal in find_hex_literals(line):
                locations.add(f"{relative_path}:{line_number}:{literal}")
    status, head, _ = run_git(root, "rev-parse", "--short", "HEAD")
    payload = {
        "generated_from_head": head.strip() if status == 0 else "unborn",
        "counts": dict(sorted(counts.items())),
        "locations": sorted(locations),
    }
    target = Path(root) / BASELINE_PATH
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(f"baseline written: {len(counts)} files, "
          f"{sum(counts.values())} literals, {len(locations)} locations")
    return 0


def parse_mode(argv):
    if argv == ["--update-baseline"]:
        return "update", None
    if argv[:1] == ["--check-added"]:
        if len(argv) == 3 and argv[1] == "--base" and argv[2]:
            return "check", argv[2]
        return None, None
    return None, None


def main(argv=None):
    arguments = list(sys.argv[1:] if argv is None else argv)
    mode, base_rev = parse_mode(arguments)
    if mode is None:
        print(__doc__.split("Usage:")[1].split("Exit codes:")[0].strip())
        return 2
    root = Path.cwd()
    try:
        if mode == "update":
            return update_baseline(root)
        baseline = load_baseline(root)
        violations = scan_added_literals(root, base_rev)
        violations += scan_frozen_counts(root, baseline)
    except (GitError, json.JSONDecodeError) as failure:
        print(f"ERROR {failure}", file=sys.stderr)
        return 2
    for violation in sorted(set(violations)):
        print(violation)
    return 1 if violations else 0


if __name__ == "__main__":
    sys.exit(main())
