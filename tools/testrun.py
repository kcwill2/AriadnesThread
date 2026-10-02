#!/usr/bin/env python3
"""Canonical cargo test reporting (incident E4: no more hand-parsing output).

Usage:
    python tools/testrun.py [scoped cargo test args] [--timeout SECONDS]
        [--retry-on-timeout] [--baseline FILE] [--replay CAPTURED_FILE]

Runs `cargo test --workspace` by default, or `cargo test <scoped args>` when
the user passes scoped args (e.g. -p ariadne-lang). Echoes the captured cargo
output, then prints EXACTLY ONE canonical summary line as the last stdout
line: `N passed / M failed / K ignored / S suites`.

Pinned parse spec: sum every line starting `test result: ok. N passed;
M failed; K ignored` or `test result: FAILED. N passed; M failed; K ignored`
(all test binaries plus doc-test groups); suites are the `Running ...` and
`Doc-tests ...` section header lines. Zero result lines, or any result line
deviating from the pinned shape, means cargo version drift: print
`parse format drift` and exit 2 rather than a silently wrong number.

--baseline FILE holds one line `N/M/K/S`; a mismatch prints both lines and
exits 1 (intentional deltas are accepted by editing the baseline in the same
commit). --replay reads captured cargo output from a file instead of running
cargo; this is the documented test-support hook.

--distill replaces the raw cargo echo with the bounded digest from
tools/distilled.py and preserves the full output in a capture file
(default target/testrun-capture.txt, override with --capture PATH). The
canonical line stays the last stdout line either way (I1); exit codes and
drift behavior are identical in both modes.

Exit codes: 0 clean, 1 failed tests / baseline mismatch / timeout,
2 parse format drift or usage error.
"""

import re
import subprocess
import sys
from collections import namedtuple
from pathlib import Path

import distilled

DEFAULT_TIMEOUT_SECONDS = 600
EXIT_CLEAN = 0
EXIT_VERDICT_FAILED = 1
EXIT_USAGE_OR_DRIFT = 2

REPO_ROOT = Path(__file__).resolve().parents[1]
DRIFT_MESSAGE = "parse format drift"

TEST_RESULT_LINE = re.compile(
    r"^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored"
)
ANY_RESULT_MARKER = "test result:"
SUITE_HEADER_LINE = re.compile(r"^\s*(?:Running|Doc-tests)\s")
BASELINE_SHAPE = re.compile(r"^(\d+)/(\d+)/(\d+)/(\d+)$")

OWN_SWITCH_FLAGS = frozenset({"--retry-on-timeout", "--distill"})
OWN_VALUE_FLAGS = frozenset(
    {"--timeout", "--baseline", "--replay", "--capture"}
)
CARGO_VALUE_FLAGS = frozenset({
    "-j", "--exclude", "--features", "--jobs", "--manifest-path", "-p",
    "--package", "--target", "--test",
})
CARGO_SWITCH_FLAGS = frozenset({
    "--all-features", "--benches", "--bins", "--doc", "--frozen", "--lib",
    "--locked", "--no-default-features", "--offline", "--release",
    "--tests", "--workspace",
})

TestCounts = namedtuple("TestCounts", "passed failed ignored suites")
RunOutcome = namedtuple("RunOutcome", "text timed_out")
ToolOptions = namedtuple(
    "ToolOptions",
    "cargo_args timeout_seconds baseline_path replay_path retry_on_timeout"
    " distill capture_path",
    defaults=(False, None),
)


def parse_cargo_output(text):
    """Pure parser: pinned cargo shape -> TestCounts, or None on drift."""
    passed = 0
    failed = 0
    ignored = 0
    suites = 0
    result_groups = 0
    for line in text.splitlines():
        if SUITE_HEADER_LINE.match(line):
            suites += 1
        if ANY_RESULT_MARKER not in line:
            continue
        match = TEST_RESULT_LINE.match(line)
        if match is None:
            return None
        passed += int(match.group(1))
        failed += int(match.group(2))
        ignored += int(match.group(3))
        result_groups += 1
    if result_groups == 0:
        return None
    return TestCounts(passed, failed, ignored, suites)


def canonical_line(counts) -> str:
    return (
        f"{counts.passed} passed / {counts.failed} failed / "
        f"{counts.ignored} ignored / {counts.suites} suites"
    )


def load_baseline(path):
    """Returns (counts, error_message); exactly one element is None."""
    try:
        content = Path(path).read_text(encoding="utf-8").strip()
    except OSError as failure:
        return None, f"cannot read baseline file: {failure}"
    match = BASELINE_SHAPE.match(content)
    if match is None:
        return None, "baseline file must contain one line in N/M/K/S form"
    return TestCounts(*(int(group) for group in match.groups())), None


def run_cargo_command(cargo_args, timeout_seconds):
    """Runs cargo test once, capturing combined output (test-support seam)."""
    command = ["cargo", "test", *cargo_args]
    try:
        finished = subprocess.run(
            command,
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired:
        return RunOutcome(text="", timed_out=True)
    except OSError as missing_cargo:
        print(f"cannot run cargo: {missing_cargo}", file=sys.stderr)
        return RunOutcome(text=None, timed_out=False)
    return RunOutcome(text=finished.stdout + finished.stderr, timed_out=False)


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
    return value if value >= 1 else None


def parse_arguments(argv):
    """Returns ToolOptions, or a usage-error message string."""
    cargo_args = []
    timeout_seconds = DEFAULT_TIMEOUT_SECONDS
    baseline_path = None
    replay_path = None
    retry_on_timeout = False
    distill = False
    capture_path = None
    index = 0
    while index < len(argv):
        arg = argv[index]
        index += 1
        if arg == "--":
            cargo_args.extend(argv[index:])
            break
        option, inline_value = _split_option(arg)
        if option in OWN_SWITCH_FLAGS:
            if option == "--distill":
                distill = True
            else:
                retry_on_timeout = True
        elif option in OWN_VALUE_FLAGS:
            value, index = _take_value(argv, index, inline_value)
            if value is None:
                return f"missing value for {arg}"
            if option == "--timeout":
                parsed_timeout = _positive_int(value)
                if parsed_timeout is None:
                    return f"invalid --timeout value: {value}"
                timeout_seconds = parsed_timeout
            elif option == "--baseline":
                baseline_path = value
            elif option == "--capture":
                capture_path = value
            else:
                replay_path = value
        elif option in CARGO_VALUE_FLAGS:
            value, index = _take_value(argv, index, inline_value)
            if value is None:
                return f"missing value for {arg}"
            cargo_args.append(arg if inline_value is not None else option)
            if inline_value is None:
                cargo_args.append(value)
        elif option in CARGO_SWITCH_FLAGS:
            cargo_args.append(arg)
        elif arg.startswith("-") and len(arg) > 1:
            return f"unknown flag: {arg}"
        else:
            cargo_args.append(arg)
    if not cargo_args:
        cargo_args = ["--workspace"]
    return ToolOptions(
        cargo_args=cargo_args,
        timeout_seconds=timeout_seconds,
        baseline_path=baseline_path,
        replay_path=replay_path,
        retry_on_timeout=retry_on_timeout,
        distill=distill,
        capture_path=capture_path,
    )


def _read_replay(replay_path):
    try:
        return Path(replay_path).read_text(encoding="utf-8", errors="replace")
    except OSError as failure:
        print(f"usage error: cannot read --replay file: {failure}",
              file=sys.stderr)
        return None


def _collect_cargo_output(options):
    outcome = run_cargo_command(options.cargo_args, options.timeout_seconds)
    if outcome.timed_out and options.retry_on_timeout:
        print(f"TIMEOUT after {options.timeout_seconds}s")
        outcome = run_cargo_command(
            options.cargo_args, options.timeout_seconds
        )
    if outcome.timed_out:
        print(f"TIMEOUT after {options.timeout_seconds}s")
    return outcome


def _echo(text):
    if not text:
        return
    sys.stdout.write(text)
    if not text.endswith(("\n", "\r")):
        sys.stdout.write("\n")


def _emit_distilled_output(cargo_text, options, counts):
    """Distill mode: bounded digest on stdout, full text in the capture (I5)."""
    capture_path = options.capture_path or str(
        REPO_ROOT / distilled.DEFAULT_CAPTURE_DIR / "testrun-capture.txt"
    )
    distilled.write_capture(cargo_text, capture_path)
    if counts is None:
        exit_for_digest = EXIT_USAGE_OR_DRIFT
    elif counts.failed == 0:
        exit_for_digest = EXIT_CLEAN
    else:
        exit_for_digest = EXIT_VERDICT_FAILED
    for line in distilled.digest_cargo_test(
        cargo_text, counts, exit_for_digest, capture_path=capture_path
    ):
        print(line)


def main(argv=None) -> int:
    arguments = list(sys.argv[1:] if argv is None else argv)
    options = parse_arguments(arguments)
    if isinstance(options, str):
        print(f"usage error: {options}", file=sys.stderr)
        return EXIT_USAGE_OR_DRIFT
    if options.replay_path is not None:
        cargo_text = _read_replay(options.replay_path)
        if cargo_text is None:
            return EXIT_USAGE_OR_DRIFT
    else:
        outcome = _collect_cargo_output(options)
        if outcome.timed_out:
            return EXIT_VERDICT_FAILED
        if outcome.text is None:
            return EXIT_USAGE_OR_DRIFT
        cargo_text = outcome.text
    counts = parse_cargo_output(cargo_text)
    if options.distill:
        _emit_distilled_output(cargo_text, options, counts)
    else:
        _echo(cargo_text)
    if counts is None:
        print(DRIFT_MESSAGE)
        return EXIT_USAGE_OR_DRIFT
    summary = canonical_line(counts)
    if options.baseline_path is not None:
        baseline_counts, baseline_error = load_baseline(options.baseline_path)
        if baseline_counts is None:
            print(f"usage error: {baseline_error}", file=sys.stderr)
            return EXIT_USAGE_OR_DRIFT
        if baseline_counts != counts:
            print("baseline mismatch")
            print(f"baseline {canonical_line(baseline_counts)}")
            print(f"actual {summary}")
            return EXIT_VERDICT_FAILED
    print(summary)
    return EXIT_CLEAN if counts.failed == 0 else EXIT_VERDICT_FAILED


if __name__ == "__main__":
    sys.exit(main())
