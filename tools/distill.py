#!/usr/bin/env python3
"""Distilling runner for build/clippy/generic commands (spec T3).

Usage:
    python tools/distill.py [--kind build|generic|auto] [--capture PATH]
        [--timeout SECONDS] [--input-file F --child-exit N] -- <child argv...>

Runs the child with subprocess argv (never shell=True) from the repo root,
writes the full combined output to a capture file, and prints ONLY a bounded
digest. Verdict derives from the child exit code alone (I2); log text can
never flip PASS/FAIL. --kind auto: first child token `cargo` -> build,
otherwise generic; offline (--input-file) defaults to generic.
--input-file + --child-exit parse a captured file without running a child
(this is the documented test-support seam).

Exit codes: 0 child clean, 1 child failed or timeout, 2 usage error.
Default capture: target/distill-capture.txt.
"""

import re
import subprocess
import sys
from collections import namedtuple
from pathlib import Path

import distilled

REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_TIMEOUT_SECONDS = 900
DEFAULT_CAPTURE_NAME = "distill-capture.txt"
EXIT_CLEAN = 0
EXIT_VERDICT_FAILED = 1
EXIT_USAGE_OR_DRIFT = 2

KIND_AUTO = "auto"
KIND_BUILD = "build"
KIND_GENERIC = "generic"
VALID_KINDS = frozenset({KIND_AUTO, KIND_BUILD, KIND_GENERIC})

DistillOptions = namedtuple(
    "DistillOptions",
    "kind timeout_seconds capture_path input_file child_exit child_argv",
)
ChildResult = namedtuple("ChildResult", "text exit_code timed_out")


def _split_option(arg):
    if arg.startswith("--") and "=" in arg:
        head, _, tail = arg.partition("=")
        return head, tail
    return arg, None


def _take_value(argv, index, inline_value):
    if inline_value is not None:
        return inline_value, index
    if index >= len(argv) or argv[index].startswith("-"):
        return None, index
    return argv[index], index + 1


def _positive_int(text):
    try:
        value = int(text)
    except ValueError:
        return None
    return value if value >= 0 else None


def parse_arguments(argv):
    """Returns DistillOptions, or a usage-error message string."""
    kind = KIND_AUTO
    timeout_seconds = DEFAULT_TIMEOUT_SECONDS
    capture_path = None
    input_file = None
    child_exit = None
    child_argv = []
    index = 0
    while index < len(argv):
        arg = argv[index]
        index += 1
        if arg == "--":
            child_argv = list(argv[index:])
            break
        option, inline_value = _split_option(arg)
        if not arg.startswith("-"):
            child_argv = list(argv[index - 1:])
            break
        if option == "--kind":
            kind, index = _take_value(argv, index, inline_value)
            if kind not in VALID_KINDS:
                return "invalid or missing --kind value (auto/build/generic)"
        elif option == "--timeout":
            value, index = _take_value(argv, index, inline_value)
            parsed = _positive_int(value) if value is not None else None
            if parsed is None or parsed < 1:
                return f"invalid or missing --timeout value: {value}"
            timeout_seconds = parsed
        elif option == "--capture":
            capture_path, index = _take_value(argv, index, inline_value)
            if capture_path is None:
                return "missing value for --capture"
        elif option == "--input-file":
            input_file, index = _take_value(argv, index, inline_value)
            if input_file is None:
                return "missing value for --input-file"
        elif option == "--child-exit":
            value, index = _take_value(argv, index, inline_value)
            child_exit = _positive_int(value) if value is not None else None
            if child_exit is None:
                return "missing or invalid --child-exit value"
        else:
            return f"unknown flag before child command: {arg}"
    if child_argv and input_file is not None:
        return "cannot combine --input-file with a child command"
    if not child_argv and input_file is None:
        return "usage: distill.py [flags] -- <child argv...> (or --input-file)"
    return DistillOptions(
        kind=kind,
        timeout_seconds=timeout_seconds,
        capture_path=capture_path,
        input_file=input_file,
        child_exit=child_exit,
        child_argv=child_argv,
    )


def resolve_kind(options):
    if options.kind != KIND_AUTO:
        return options.kind
    if options.child_argv and options.child_argv[0] == "cargo":
        return KIND_BUILD
    return KIND_GENERIC


def _decode_partial(partial):
    if partial is None:
        return ""
    if isinstance(partial, bytes):
        return partial.decode("utf-8", errors="replace")
    return partial


def run_child(child_argv, timeout_seconds):
    """Runs the child once, capturing combined output (test seam mirror)."""
    try:
        finished = subprocess.run(
            [str(token) for token in child_argv],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired as expired:
        partial = _decode_partial(expired.stdout) + _decode_partial(expired.stderr)
        return ChildResult(text=partial, exit_code=None, timed_out=True)
    except OSError as missing_executable:
        print(f"cannot run child: {missing_executable}", file=sys.stderr)
        return ChildResult(text="", exit_code=None, timed_out=False)
    return ChildResult(
        text=finished.stdout + finished.stderr,
        exit_code=finished.returncode,
        timed_out=False,
    )


def _read_input_file(path):
    try:
        return Path(path).read_text(encoding="utf-8", errors="replace")
    except OSError as failure:
        print(f"usage error: cannot read --input-file: {failure}",
              file=sys.stderr)
        return None


def _digest(text, exit_code, kind, capture_path):
    if kind == KIND_BUILD:
        return distilled.digest_cargo_build(text, exit_code,
                                            capture_path=capture_path)
    return distilled.digest_generic(text, exit_code, capture_path=capture_path)


def main(argv=None) -> int:
    arguments = list(sys.argv[1:] if argv is None else argv)
    options = parse_arguments(arguments)
    if isinstance(options, str):
        print(f"usage error: {options}", file=sys.stderr)
        return EXIT_USAGE_OR_DRIFT
    capture_path = options.capture_path or str(
        REPO_ROOT / distilled.DEFAULT_CAPTURE_DIR / DEFAULT_CAPTURE_NAME
    )
    if options.input_file is not None:
        text = _read_input_file(options.input_file)
        if text is None:
            return EXIT_USAGE_OR_DRIFT
        exit_code = options.child_exit or EXIT_CLEAN
    else:
        result = run_child(options.child_argv, options.timeout_seconds)
        if result.timed_out:
            distilled.write_capture(result.text, capture_path)
            print("TIMEOUT after %ds -> capture: %s"
                  % (options.timeout_seconds, capture_path))
            return EXIT_VERDICT_FAILED
        if result.exit_code is None:
            return EXIT_USAGE_OR_DRIFT
        text = result.text
        exit_code = result.exit_code
    distilled.write_capture(text, capture_path)
    digest = _digest(text, exit_code, resolve_kind(options), capture_path)
    for line in digest.lines:
        print(line)
    return EXIT_CLEAN if exit_code == 0 else EXIT_VERDICT_FAILED


if __name__ == "__main__":
    sys.exit(main())
