#!/usr/bin/env python3
"""Repo hygiene gate: R1 (no fn/def over 100 physical lines) and R2 (no source
file over 1000 physical lines).

Usage: python tools/hygiene_check.py [root]   (root defaults to repo root)
Exit codes: 0 clean, 1 violations found, 2 internal/parse error.
"""

import os
import re
import sys
from collections import namedtuple
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

R1_LIMIT_LINES = 100
R2_LIMIT_LINES = 1000
R2_SCANNED_SUFFIXES = frozenset({".rs", ".py", ".html", ".js"})
PRUNED_DIR_NAMES = frozenset({"target", "node_modules", "__pycache__"})

# R2 allow list — the ONLY entry: repo-root report.html is a generated pipeline
# report artifact whose size varies between pipeline runs, explicitly out of
# scope per the code-hygiene refactor plan. Do not add entries without
# updating that plan.
ALLOWED_LONG_FILES = frozenset({"report.html"})

FunctionSpan = namedtuple("FunctionSpan", "line name length")

RUST_CHAR_LITERAL = re.compile(
    r"'(?:\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F]{1,6}\}|[nrt0'\\])|[^\\'\n])'"
)
BYTE_CHAR_LITERAL = re.compile(
    r"b'(?:\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F]{1,6}\}|[nrt0'\\])|[^\\'\n])'"
)
RUST_FUNCTION_NAME = re.compile(r"\s*(r#)?([A-Za-z_][A-Za-z0-9_]*)")
RUST_SIGNATURE_START = re.compile(r"\s*[<(]")
PYTHON_STRING_OPEN = re.compile(r"[rbufRBUF]{0,2}(?P<quote>'''|\"\"\"|\"|')")
PYTHON_FUNCTION_NAME = re.compile(r"\s*([A-Za-z_][A-Za-z0-9_]*)")


class ParseError(Exception):
    def __init__(self, message: str, line: int):
        super().__init__(f"line {line}: {message}")
        self.message = message
        self.line = line


def find_rust_functions(text: str):
    return _RustFunctionFinder().find(split_physical_lines(text))


def find_python_functions(text: str):
    return _PythonFunctionFinder().find(split_physical_lines(text))


PHYSICAL_LINE_BREAK = re.compile(r"\r\n|\r|\n")


def split_physical_lines(text: str):
    lines = PHYSICAL_LINE_BREAK.split(text)
    if lines and lines[-1] == "":
        lines.pop()
    return lines


def physical_line_count(text: str) -> int:
    return len(split_physical_lines(text))


def _is_word_character(char: str) -> bool:
    return char == "_" or char.isalnum()


def _rust_fn_keyword_at(line: str, position: int) -> bool:
    if not line.startswith("fn", position):
        return False
    before = line[position - 1] if position > 0 else " "
    after = line[position + 2] if position + 2 < len(line) else " "
    return not _is_word_character(before) and not _is_word_character(after)


def _rust_definition_name_at(line: str, position: int) -> Optional[str]:
    if not _rust_fn_keyword_at(line, position):
        return None
    name = RUST_FUNCTION_NAME.match(line, position + 2)
    if name is None or RUST_SIGNATURE_START.match(line, name.end()) is None:
        return None
    return (name.group(1) or "") + name.group(2)


def _raw_string_length(line: str, position: int):
    scan = position + (1 if line[position] in "bc" else 0)
    if line[scan:scan + 1] != "r":
        return None
    scan += 1
    hashes = 0
    while line[scan:scan + 1] == "#":
        hashes += 1
        scan += 1
    if line[scan:scan + 1] != '"':
        return None
    return scan + 1 - position, hashes


def _lifetime_end(line: str, position: int) -> Optional[int]:
    end = position + 1
    while end < len(line) and _is_word_character(line[end]):
        end += 1
    return end if end > position + 1 else None


def _rust_char_or_lifetime_end(line: str, position: int) -> Optional[int]:
    if line[position] == "b" and line.startswith("'", position + 1):
        match = BYTE_CHAR_LITERAL.match(line, position)
        if match:
            return match.end()
    if line[position] != "'":
        return None
    match = RUST_CHAR_LITERAL.match(line, position)
    if match:
        return match.end()
    return _lifetime_end(line, position)


def _advance_rust_code(line: str, position: int, state, events) -> int:
    if _rust_definition_name_at(line, position) is not None:
        events.append(("fn", position))
    char = line[position]
    following = line[position + 1] if position + 1 < len(line) else ""
    if char == "/" and following == "/":
        return len(line)
    if char == "/" and following == "*":
        state.comment_depth = 1
        return position + 2
    if char == "{":
        events.append(("open", position))
        return position + 1
    if char == "}":
        events.append(("close", position))
        return position + 1
    raw = _raw_string_length(line, position)
    if raw is not None:
        consumed, hashes = raw
        state.raw_hashes = hashes
        return position + consumed
    if char == '"' or (char in "bc" and following == '"'):
        state.in_string = True
        return position + (1 if char == '"' else 2)
    literal_end = _rust_char_or_lifetime_end(line, position)
    if literal_end is not None:
        return literal_end
    return position + 1


def _advance_raw_string(line: str, position: int, state) -> int:
    closing = '"' + "#" * state.raw_hashes
    found = line.find(closing, position)
    if found < 0:
        return len(line)
    state.raw_hashes = None
    return found + len(closing)


def _advance_block_comment(line: str, position: int, state) -> int:
    while position < len(line):
        pair = line[position:position + 2]
        if pair == "/*":
            state.comment_depth += 1
            position += 2
        elif pair == "*/":
            state.comment_depth -= 1
            position += 2
            if state.comment_depth == 0:
                return position
        else:
            position += 1
    return position


def _advance_standard_string(line: str, position: int, state) -> int:
    while position < len(line):
        char = line[position]
        if state.escape_pending:
            state.escape_pending = False
        elif char == "\\":
            state.escape_pending = True
        elif char == '"':
            state.in_string = False
            return position + 1
        position += 1
    # Rust standard strings cannot contain raw newlines; do not poison later lines.
    state.in_string = False
    state.escape_pending = False
    return position


def _rust_line_events(line: str, state):
    events = []
    position = 0
    while position < len(line):
        if state.raw_hashes is not None:
            position = _advance_raw_string(line, position, state)
        elif state.comment_depth:
            position = _advance_block_comment(line, position, state)
        elif state.in_string:
            position = _advance_standard_string(line, position, state)
        else:
            position = _advance_rust_code(line, position, state, events)
    return events


@dataclass
class _RustScanState:
    comment_depth: int = 0
    raw_hashes: Optional[int] = None
    in_string: bool = False
    escape_pending: bool = False


class _RustFunctionFinder:
    def __init__(self):
        self._state = _RustScanState()
        self._finished = []
        self._start = None
        self._name = ""
        self._opened = False
        self._balance = 0

    def find(self, lines):
        for index, line in enumerate(lines, start=1):
            for kind, position in _rust_line_events(line, self._state):
                self._handle(kind, position, line, index)
        self._finish_at_end_of_file(lines)
        return self._finished

    def _handle(self, kind: str, position: int, line: str, line_number: int):
        if kind == "fn":
            self._handle_fn(position, line, line_number)
        elif kind == "open":
            self._handle_open()
        else:
            self._handle_close(line_number)

    def _handle_fn(self, position: int, line: str, line_number: int):
        name = _rust_definition_name_at(line, position)
        if name is None:
            return
        if self._start is not None:
            if self._opened:
                return
            self._finish(self._start)
        self._start = line_number
        self._name = name
        self._opened = False
        self._balance = 0

    def _handle_open(self):
        if self._start is not None:
            self._opened = True
            self._balance += 1

    def _handle_close(self, line_number: int):
        if self._start is None:
            return
        if not self._opened:
            self._finish(self._start)
            return
        self._balance -= 1
        if self._balance == 0:
            self._finish(line_number)

    def _finish(self, end_line: int):
        self._finished.append(
            FunctionSpan(self._start, self._name, end_line - self._start + 1)
        )
        self._start = None
        self._name = ""
        self._opened = False
        self._balance = 0

    def _finish_at_end_of_file(self, lines):
        if self._state.comment_depth:
            raise ParseError("unterminated block comment", len(lines))
        if self._state.raw_hashes is not None:
            raise ParseError("unterminated raw string", len(lines))
        if self._start is not None:
            if self._opened:
                raise ParseError(f"unclosed fn `{self._name}`", self._start)
            self._finish(self._start)


@dataclass
class _PythonScanState:
    triple_delimiter: Optional[str] = None


@dataclass(frozen=True)
class _PythonLineFacts:
    first_def_name: Optional[str]
    code_indent: Optional[int]
    masked: bool
    blank: bool
    paren_delta: int


def _skip_single_quoted(line: str, position: int, quote: str) -> int:
    while position < len(line):
        char = line[position]
        if char == "\\":
            position += 2
            continue
        if char == quote:
            return position + 1
        position += 1
    return len(line)


def _advance_python_string(line: str, position: int, state) -> int:
    match = PYTHON_STRING_OPEN.match(line, position)
    if match is None:
        return position + 1
    quote = match.group("quote")
    after_prefix = match.end()
    if len(quote) != 3:
        return _skip_single_quoted(line, after_prefix, quote)
    closing = line.find(quote, after_prefix)
    if closing < 0:
        state.triple_delimiter = quote
        return len(line)
    return closing + len(quote)


def _python_def_name(line: str, position: int) -> Optional[str]:
    if not line.startswith("def", position):
        return None
    before = line[position - 1] if position > 0 else " "
    after = line[position + 3] if position + 3 < len(line) else " "
    if _is_word_character(before) or _is_word_character(after):
        return None
    match = PYTHON_FUNCTION_NAME.match(line, position + 3)
    return match.group(1) if match else "anonymous"


def _python_line_facts(line: str, state) -> _PythonLineFacts:
    position = 0
    blank = not line.strip()
    if state.triple_delimiter is not None:
        closing = line.find(state.triple_delimiter)
        if closing < 0:
            return _PythonLineFacts(None, None, True, blank, 0)
        state.triple_delimiter = None
        # The line that closes a triple-quoted string is part of the string
        # token, not a code-state line: its indentation must not terminate an
        # enclosing def measurement even when the closer sits at column 0.
        return _PythonLineFacts(None, None, True, blank, 0)
    def_name = None
    paren_delta = 0
    while position < len(line):
        char = line[position]
        if char == "#":
            break
        if def_name is None:
            def_name = _python_def_name(line, position)
        if char == "(":
            paren_delta += 1
        elif char == ")":
            paren_delta -= 1
        position = _advance_python_string(line, position, state)
    indent = len(line) - len(line.lstrip())
    return _PythonLineFacts(def_name, None if blank else indent, False, blank,
                            paren_delta)


class _PythonFunctionFinder:
    def __init__(self):
        self._state = _PythonScanState()
        self._finished = []
        self._start = None
        self._name = ""
        self._indent = 0
        self._last_content = 0
        self._paren_depth = 0

    def find(self, lines):
        for index, line in enumerate(lines, start=1):
            self._consume(_python_line_facts(line, self._state), index)
        if self._state.triple_delimiter is not None:
            raise ParseError("unterminated triple-quoted string", len(lines))
        if self._start is not None:
            self._finish()
        return self._finished

    def _consume(self, facts: _PythonLineFacts, line_number: int):
        if self._start is None:
            if facts.first_def_name is not None and facts.code_indent is not None:
                self._begin(facts.first_def_name, facts.code_indent,
                            line_number, facts.paren_delta)
            return
        if facts.masked:
            if not facts.blank:
                self._last_content = line_number
            return
        if facts.blank:
            return
        if self._paren_depth:
            self._paren_depth += facts.paren_delta
            self._last_content = line_number
            return
        if facts.code_indent > self._indent:
            self._last_content = line_number
            return
        self._finish()
        if facts.first_def_name is not None and facts.code_indent is not None:
            self._begin(facts.first_def_name, facts.code_indent, line_number,
                        facts.paren_delta)

    def _begin(self, name: str, indent: int, line_number: int,
               paren_delta: int = 0):
        self._start = line_number
        self._name = name
        self._indent = indent
        self._last_content = line_number
        self._paren_depth = paren_delta if paren_delta > 0 else 0

    def _finish(self):
        self._finished.append(
            FunctionSpan(self._start, self._name, self._last_content - self._start + 1)
        )
        self._start = None
        self._name = ""
        self._paren_depth = 0


def _pruned_directory(name: str) -> bool:
    return name.startswith(".") or name in PRUNED_DIR_NAMES


def _iter_source_files(root: Path):
    for directory, children, file_names in os.walk(root):
        children[:] = sorted(child for child in children if not _pruned_directory(child))
        for file_name in sorted(file_names):
            path = Path(directory, file_name)
            if path.suffix.lower() in R2_SCANNED_SUFFIXES:
                yield path.relative_to(root).as_posix(), path


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def _r2_entry(relative: str, text: str):
    count = physical_line_count(text)
    if count <= R2_LIMIT_LINES or relative in ALLOWED_LONG_FILES:
        return None
    return (relative, 0, 0, f"{relative}: R2 {count} lines > {R2_LIMIT_LINES}")


def _r1_entries(relative: str, suffix: str, text: str, errors):
    language = {
        ".rs": ("fn", find_rust_functions),
        ".py": ("def", find_python_functions),
    }.get(suffix.lower())
    if language is None:
        return []
    kind, finder = language
    try:
        spans = finder(text)
    except ParseError as failure:
        errors.append((relative, 2, failure.line,
                       f"{relative}:{failure.line}: ERROR {failure.message}"))
        return []
    return [
        (relative, 1, span.line,
         f"{relative}:{span.line}: R1 {kind} `{span.name}` is {span.length} "
         f"lines (> {R1_LIMIT_LINES})")
        for span in spans
        if span.length > R1_LIMIT_LINES
    ]


def scan_tree(root: Path):
    violations = []
    errors = []
    for relative, path in _iter_source_files(root):
        text = _read_text(path)
        oversized = _r2_entry(relative, text)
        if oversized is not None:
            violations.append(oversized)
        violations.extend(_r1_entries(relative, path.suffix, text, errors))
    return violations, errors


def main(argv=None) -> int:
    arguments = list(sys.argv[1:] if argv is None else argv)
    root = Path(arguments[0]) if arguments else Path(__file__).resolve().parents[1]
    if not root.is_dir():
        print(f"ERROR {root} is not a directory")
        return 2
    violations, errors = scan_tree(root)
    for entry in sorted(violations):
        print(entry[3])
    for entry in sorted(errors):
        print(entry[3])
    if errors:
        return 2
    return 1 if violations else 0


if __name__ == "__main__":
    sys.exit(main())
