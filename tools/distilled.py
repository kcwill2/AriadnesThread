#!/usr/bin/env python3
"""Bounded tool-output digests (spec 1791044160000: verdict + excerpt + capture).

Pure digest functions for the three pinned output families plus the one
capture writer:
  - digest_cargo_test: cargo test output, counts owned by testrun's pinned
    parser (never re-implemented here);
  - digest_cargo_build: cargo build/clippy `error[E...]:` blocks;
  - digest_generic: deterministic fallback for commands without a pinned
    parser (verdict from exit code only, I2);
  - write_capture: byte-safe full-output preservation under target/.

Verdict honesty (I2): pass/fail derives ONLY from exit codes and pinned-shape
parses, never from substring matching of log text. Truncation always
self-declares `... and K more ...` (I9). Every digest is bounded (I4).
"""

import re
from collections import namedtuple
from pathlib import Path

import testrun

ANSI_ESCAPE = re.compile(r"\x1b\[[0-9;]*m")

FAILURE_BLOCK_START = re.compile(r"^---- (.+) stdout ----\s*$")
FAILURE_BLOCK_END = re.compile(r"^(?:---- |failures:$|test result:)")
PANIC_MARKER = "panicked at"

BUILD_ERROR_OPEN = re.compile(r"^error(?:\[E\d+\])?: ")
BUILD_ERROR_CODE = re.compile(r"^error\[E(\d+)\]: (.*)$")
BUILD_ERROR_PLAIN = re.compile(r"^error: (.*)$")
BUILD_LOCATION = re.compile(r"^\s*--> ")
BUILD_NOTE = re.compile(r"^\s*= ")
BUILD_WARNING_SUMMARY = re.compile(r"^warning: .* generated (\d+) warnings?$")

GENERIC_NAME = re.compile(r"^(?:FAIL|ERROR): ")
TRACEBACK_START = "Traceback (most recent call last):"
GENERIC_EXCEPTION = re.compile(
    r"^(?:[A-Za-z_.]*(?:Error|Exception)[: ]|error:|ERROR)"
)

MAX_TEST_BLOCKS = 5
MAX_TEST_BLOCK_LINES = 8
PANIC_TAIL_LINES = 2
MAX_BUILD_BLOCKS = 10
GENERIC_LINE_BUDGET = 50
MAX_GENERIC_NAMES = 5
MAX_TRACEBACK_BLOCKS = 2
MAX_TRACEBACK_LINES = 10
MAX_EXCEPTION_LINES = 5
FALLBACK_HEAD_LINES = 3
FALLBACK_TAIL_LINES = 12

DEFAULT_CAPTURE_DIR = Path("target")

FailureExcerpt = namedtuple("FailureExcerpt", "name panic_line lines")
BuildErrorBlock = namedtuple("BuildErrorBlock", "code message location lines")
Digest = namedtuple("Digest", "verdict lines capture_path")


def plain_lines(text):
    """Strip ANSI per line before any matching or emission."""
    return [ANSI_ESCAPE.sub("", line) for line in text.splitlines()]


def _capture_tag(capture_path, line_count):
    if capture_path is None:
        return ""
    return " [capture: %s, %d lines]" % (capture_path, line_count)


def _extract_failure_excerpts(lines, max_block_lines):
    """Pinned grammar: `---- name stdout ----` up to the next boundary."""
    excerpts = []
    index = 0
    while index < len(lines):
        start = FAILURE_BLOCK_START.match(lines[index])
        if start is None:
            index += 1
            continue
        name = start.group(1)
        body = []
        index += 1
        while index < len(lines) and not FAILURE_BLOCK_END.match(lines[index]):
            body.append(lines[index])
            index += 1
        excerpts.append(_one_excerpt(name, body, max_block_lines))
    return excerpts


def _one_excerpt(name, body, max_block_lines):
    limit = max(1, max_block_lines - 1)
    panic_index = None
    for position, line in enumerate(body):
        if PANIC_MARKER in line:
            panic_index = position
            break
    if panic_index is None:
        return FailureExcerpt(name, "", body[:limit])
    tail_end = min(len(body), panic_index + 1 + PANIC_TAIL_LINES)
    return FailureExcerpt(name, body[panic_index], body[panic_index:tail_end])


def digest_cargo_test(text, counts, exit_code,
                      max_blocks=MAX_TEST_BLOCKS,
                      max_block_lines=MAX_TEST_BLOCK_LINES,
                      capture_path=None):
    """Digest cargo test output; counts from testrun.parse_cargo_output.

    counts None means pinned-parse drift: verdict DRIFT, never PASS (I3).
    """
    body = plain_lines(text)
    tag = _capture_tag(capture_path, len(body))
    if counts is None:
        return [
            "### cargo test DRIFT (exit %d)%s" % (exit_code, tag),
            "%s: no test result line matched the pinned parse spec"
            % testrun.DRIFT_MESSAGE,
        ]
    verdict = "PASS" if exit_code == 0 and counts.failed == 0 else "FAIL"
    lines = [
        "### cargo test %s (exit %d)%s" % (verdict, exit_code, tag),
        testrun.canonical_line(counts),
    ]
    if counts.failed == 0:
        lines.append("no failures")
        return lines
    excerpts = _extract_failure_excerpts(body, max_block_lines)
    shown = 0
    for excerpt in excerpts[:max_blocks]:
        lines.append("---- %s ----" % excerpt.name)
        lines.extend(excerpt.lines)
        shown += 1
    if shown == 0:
        lines.append("no failure detail blocks in output (see capture)")
    hidden = counts.failed - shown
    if hidden > 0:
        lines.append("... and %d more failures" % hidden)
    return lines


def _build_block_lines(code, message, location, note):
    block_lines = [
        "error[%s]: %s" % (code, message) if code else "error: %s" % message
    ]
    if location:
        block_lines.append(location)
    if note:
        block_lines.append(note)
    return block_lines


def _extract_build_error_blocks(lines):
    """Pinned shape: `error[..]: ` opener, `-->` line, first `=`-note line.

    Dedupes on (code, message); returns (unique_blocks, unique_total).
    """
    blocks = []
    seen = set()
    current = None
    for line in lines:
        if BUILD_ERROR_OPEN.match(line):
            if current is not None:
                _finish_build_block(blocks, seen, current)
            code, message = _split_build_error(line)
            current = {"code": code, "message": message,
                       "location": "", "note": ""}
        elif current is not None:
            if not current["location"] and BUILD_LOCATION.match(line):
                current["location"] = line.rstrip()
            elif not current["note"] and BUILD_NOTE.match(line):
                current["note"] = line.rstrip()
    if current is not None:
        _finish_build_block(blocks, seen, current)
    return blocks, len(seen)


def _split_build_error(line):
    match = BUILD_ERROR_CODE.match(line)
    if match is not None:
        return match.group(1), match.group(2)
    match = BUILD_ERROR_PLAIN.match(line)
    if match is not None:
        return "", match.group(1)
    return "", line


def _finish_build_block(blocks, seen, current):
    key = (current["code"], current["message"])
    if key in seen:
        return
    seen.add(key)
    blocks.append(BuildErrorBlock(
        key[0], key[1], current["location"],
        _build_block_lines(
            key[0], key[1], current["location"], current["note"]
        ),
    ))


def _warning_summary_total(lines):
    total = 0
    for line in lines:
        match = BUILD_WARNING_SUMMARY.match(line)
        if match is not None:
            total += int(match.group(1))
    return total


def digest_cargo_build(text, exit_code, max_blocks=MAX_BUILD_BLOCKS,
                       capture_path=None):
    """Digest cargo build/clippy output; verdict from exit code only (I2)."""
    lines = plain_lines(text)
    warnings = _warning_summary_total(lines)
    if exit_code == 0:
        suffix = " -> capture: %s" % capture_path if capture_path else ""
        return Digest(
            "PASS",
            ["ok (exit 0, %d warnings emitted)%s" % (warnings, suffix)],
            capture_path,
        )
    blocks, unique_total = _extract_build_error_blocks(lines)
    digest_lines = ["### cargo build FAIL (exit %d)" % exit_code]
    for block in blocks[:max_blocks]:
        digest_lines.extend(block.lines)
    hidden = unique_total - min(unique_total, max_blocks)
    if hidden > 0:
        digest_lines.append("... and %d more errors" % hidden)
    if unique_total == 0:
        digest_lines.extend(_head_tail_fallback(lines))
    if capture_path is not None:
        digest_lines.append(
            "capture: %s (%d lines)" % (capture_path, len(lines))
        )
    return Digest("FAIL", digest_lines, capture_path)


def _head_tail_fallback(lines):
    if len(lines) <= FALLBACK_HEAD_LINES + FALLBACK_TAIL_LINES:
        return list(lines)
    omitted = len(lines) - FALLBACK_HEAD_LINES - FALLBACK_TAIL_LINES
    return (
        lines[:FALLBACK_HEAD_LINES]
        + ["... (%d lines omitted)" % omitted]
        + lines[-FALLBACK_TAIL_LINES:]
    )


def _traceback_blocks(lines):
    excerpt = []
    blocks = 0
    index = 0
    while index < len(lines) and blocks < MAX_TRACEBACK_BLOCKS:
        if not lines[index].startswith(TRACEBACK_START):
            index += 1
            continue
        block = [lines[index]]
        index += 1
        while index < len(lines) and (
            lines[index].startswith((" ", "\t")) or lines[index] == ""
        ):
            block.append(lines[index])
            index += 1
        if index < len(lines) and lines[index] != "":
            block.append(lines[index])
            index += 1
        excerpt.extend(block[:MAX_TRACEBACK_LINES])
        blocks += 1
    return excerpt


def digest_generic(text, exit_code, max_lines=GENERIC_LINE_BUDGET,
                   capture_path=None):
    """Fallback digest: verdict from exit code ONLY, text never flips it.

    Excerpt priority: unittest FAIL/ERROR names, then Traceback blocks,
    then exception summary lines, then anchored head/tail. Budget bounded.
    """
    lines = plain_lines(text)
    verdict = "PASS" if exit_code == 0 else "FAIL"
    excerpt = (
        _generic_name_lines(lines)
        or _traceback_blocks(lines)
        or _generic_exception_lines(lines)
        or _head_tail_fallback(lines)
    )
    reserve = 2 if capture_path is not None else 1
    digest_lines = ["### generic %s (exit %d)" % (verdict, exit_code)]
    digest_lines.extend(excerpt[:max(0, max_lines - reserve)])
    if capture_path is not None:
        digest_lines.append(
            "capture: %s (%d lines)" % (capture_path, len(lines))
        )
    return Digest(verdict, digest_lines, capture_path)


def _generic_name_lines(lines):
    return [
        line for line in lines if GENERIC_NAME.match(line)
    ][:MAX_GENERIC_NAMES]


def _generic_exception_lines(lines):
    return [
        line for line in lines if GENERIC_EXCEPTION.match(line)
    ][:MAX_EXCEPTION_LINES]


def write_capture(text, capture_path):
    """Byte-safe capture write (binary mode, parents created); returns Path."""
    path = Path(capture_path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "wb") as handle:
        handle.write(text.encode("utf-8"))
    return path
