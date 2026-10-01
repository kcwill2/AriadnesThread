#!/usr/bin/env python3
"""Encoding guard: reject double-encoded-UTF-8 mojibake and non-UTF-8 bytes in
tracked/staged git content (repo incident E2: PowerShell file writes corrupted
UTF-8 punctuation in tracked files, silently, across multiple sessions).

Detection is byte-level on git plumbing output, never through PowerShell text
pipelines. Mojibake signature sequences are CONSTRUCTED FROM BYTES, never
written as source literals, so this file and its tests cannot trip the guard.

Usage: python tools/encoding_guard.py [--all | --staged] [--root <path>]
Exit codes: 0 clean, 1 violation found, 2 internal/usage error.
"""

import subprocess
import sys
from pathlib import Path

# Double-encoded UTF-8 lead signatures. When a real UTF-8 byte >= 0x80 is
# misdecoded as latin-1/cp1252 and re-encoded as UTF-8, the corruption always
# starts with one of these byte sequences in the damaged text:
#   U+00C2 ("A with circumflex")   -> c3 82   (mojibake lead for 0xC2 xx)
#   U+00C3 ("A with tilde")        -> c3 83   (mojibake lead for 0xC3 xx)
#   U+00E2 U+20AC ("euro cluster") -> c3 a2 e2 82 ac   (mojibake of E2 80 xx)
#   U+00EF U+00BF U+00BD ("replacement ghost") -> c3 af c2 bf c2 bd
# Correct UTF-8 of common punctuation (section sign, arrow, em dash) never
# contains these sequences, which is what makes them reliable tripwires.
MOJIBAKE_SIGNATURES = (
    b"\xc3\x82",           # double-encoded lead 0xC2 xx
    b"\xc3\x83",           # double-encoded lead 0xC3 xx
    b"\xc3\xa2\xe2\x82\xac",  # cp1253-style euro cluster
    b"\xc3\xaf\xc2\xbf\xc2\xbd",  # double-encoded U+FFFD
)

# Documentation paths legitimately quote corruption signatures (specs, plans)
# and are therefore exempt. Entries are forward-slash path prefixes.
ALLOW_LIST_PREFIXES = (".kilo/plans/", "plans/")

# Path segments whose subtrees are third-party vendored content: not authored
# here, may carry any bytes, and out of the corruption policy's reach.
SKIP_PATH_SEGMENTS = frozenset({"node_modules", "target", ".git"})


def git_text(root, *arguments):
    finished = subprocess.run(
        ["git", *arguments],
        cwd=str(root),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if finished.returncode != 0:
        raise RuntimeError(
            f"git {' '.join(arguments)} failed: {finished.stderr.strip()}"
        )
    return finished.stdout


def git_bytes(root, *arguments):
    finished = subprocess.run(
        ["git", *arguments],
        cwd=str(root),
        capture_output=True,
    )
    if finished.returncode != 0:
        raise RuntimeError(
            f"git {' '.join(arguments)} failed: {finished.stderr.decode(errors='replace')}"
        )
    return finished.stdout


def is_allow_listed(relative_path):
    return relative_path.startswith(ALLOW_LIST_PREFIXES)


def is_vendored(relative_path):
    return bool(SKIP_PATH_SEGMENTS.intersection(relative_path.split("/")))


def looks_binary(payload):
    return b"\x00" in payload


def scan_blob_bytes(payload):
    """Return a list of human-readable hit descriptions for one blob."""
    if looks_binary(payload):
        return []
    hits = []
    for signature in MOJIBAKE_SIGNATURES:
        if signature in payload:
            label = signature.hex(" ")
            hits.append(f"mojibake signature {label}")
    try:
        payload.decode("utf-8")
    except UnicodeDecodeError:
        hits.append("not utf-8 decodable")
    return hits


def tracked_paths(root):
    listing = git_text(root, "ls-files", "-z")
    return [entry for entry in listing.split("\0") if entry]


def staged_paths(root):
    listing = git_text(root, "diff", "--cached", "--name-only", "-z",
                       "--diff-filter=ACMR")
    return [entry for entry in listing.split("\0") if entry]


def read_working_bytes(root, relative_path):
    return (Path(root) / relative_path).read_bytes()


def read_staged_bytes(root, relative_path):
    return git_bytes(root, "cat-file", "blob", f":{relative_path}")


def collect_violations(root, paths, reader):
    violations = []
    for relative_path in paths:
        relative = relative_path.replace("\\", "/")
        if is_allow_listed(relative) or is_vendored(relative):
            continue
        try:
            payload = reader(root, relative_path)
        except FileNotFoundError:
            continue
        except (OSError, RuntimeError) as read_failure:
            violations.append(f"{relative}: ERROR cannot read blob: {read_failure}")
            continue
        for hit in scan_blob_bytes(payload):
            violations.append(f"{relative}: {hit}")
    return violations


def main(argv=None):
    arguments = list(sys.argv[1:] if argv is None else argv)
    root = Path.cwd()
    mode = "tracked"
    remaining = []
    index = 0
    while index < len(arguments):
        argument = arguments[index]
        if argument == "--staged":
            mode = "staged"
        elif argument == "--all":
            mode = "tracked"
        elif argument == "--root":
            index += 1
            if index >= len(arguments):
                print("ERROR --root requires a path")
                return 2
            root = Path(arguments[index])
        else:
            remaining.append(argument)
        index += 1
    if remaining or not root.is_dir():
        print("usage: encoding_guard.py [--all|--staged] [--root <path>]")
        return 2
    try:
        if mode == "staged":
            violations = collect_violations(root, staged_paths(root),
                                            read_staged_bytes)
        else:
            violations = collect_violations(root, tracked_paths(root),
                                            read_working_bytes)
    except RuntimeError as git_failure:
        print(f"ERROR git plumbing failed: {git_failure}")
        return 2
    for violation in sorted(violations):
        print(violation)
    return 1 if violations else 0


if __name__ == "__main__":
    sys.exit(main())
