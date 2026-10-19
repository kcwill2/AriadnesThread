"""Unit tests for tools/theme_token_check.py (hex-literal drift guard, T13).

Run directly: python tools\\theme_token_check_tests.py
Every scenario runs in a throwaway git repo under a temp dir. The tool is a
drift guard for the not-yet-landed three-theme system (E16): it flags NEW
hardcoded hex literals in crates/ui/assets/** added after --base, and freezes
the per-file hex counts recorded in the baseline snapshot.
"""

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import theme_token_check

TOOL_PATH = str(Path(__file__).resolve().parent / "theme_token_check.py")

ASSETS = "crates/ui/assets"
STYLES = f"{ASSETS}/styles.css"
MAIN_JS = f"{ASSETS}/main.js"
BASELINE = "tools/theme_token_baseline.json"


def git(repo, *args):
    finished = subprocess.run(
        ["git", *args],
        cwd=str(repo),
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    if finished.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} failed: {finished.stderr}")
    return finished.stdout


def make_repo_with_base():
    root = Path(tempfile.mkdtemp(prefix="hex-"))
    git(root, "init", "-q")
    git(root, "config", "user.email", "hex@example.invalid")
    git(root, "config", "user.name", "Hex Tests")
    write_file(root, MAIN_JS, "const style = '#aabbcc';\n")
    write_file(root, STYLES, ".bg { color: #aabbcc; }\n")
    git(root, "add", "-A")
    git(root, "commit", "-q", "-m", "base")
    return root


def write_file(root, relative, text):
    target = Path(root) / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text, encoding="utf-8")


def add_child_commit(root, relative, text):
    write_file(root, relative, text)
    git(root, "add", "-A")
    git(root, "commit", "-q", "-m", f"touch {relative}")


def write_baseline(root, counts, location=None):
    payload = {
        "generated_from_head": "test",
        "counts": counts,
        "locations": sorted(
            f"{path}:{number}"
            for path, per_file in counts.items()
            for number in range(1, per_file + 1)
        ),
    }
    target = Path(root) / (location or BASELINE)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(payload, indent=2), encoding="utf-8")


def run_tool(root, *args):
    finished = subprocess.run(
        [sys.executable, TOOL_PATH, *args],
        cwd=str(root),
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    return finished.returncode, finished.stdout, finished.stderr


class AddedLiteralTests(unittest.TestCase):
    def test_new_hex_literal_in_child_commit_exits_one(self):
        root = make_repo_with_base()
        add_child_commit(root, MAIN_JS,
                         "const style = '#aabbcc';\nconst danger = '#ff0000';\n")
        write_baseline(root, {MAIN_JS: 2, STYLES: 1})
        code, out, err = run_tool(root, "--check-added", "--base", "HEAD~1")
        self.assertEqual(1, code)
        self.assertIn(MAIN_JS, out)
        self.assertIn("#ff0000", out)

    def test_existing_literal_moved_is_not_flagged(self):
        root = make_repo_with_base()
        add_child_commit(root, MAIN_JS,
                         "// comment\nconst style = '#aabbcc';\n")
        write_baseline(root, {MAIN_JS: 1, STYLES: 1})
        code, out, err = run_tool(root, "--check-added", "--base", "HEAD~1")
        self.assertEqual(0, code, out + err)

    def test_new_file_literals_are_flagged(self):
        root = make_repo_with_base()
        add_child_commit(root, f"{ASSETS}/palette.css", ".x { color: #101010; }\n")
        write_baseline(root, {MAIN_JS: 1, STYLES: 1})
        code, out, err = run_tool(root, "--check-added", "--base", "HEAD~1")
        self.assertEqual(1, code)
        self.assertIn("#101010", out)

    def test_changes_outside_assets_are_ignored(self):
        root = make_repo_with_base()
        add_child_commit(root, "crates/ui/src/lib.rs",
                         "// color #ff0000 is just a comment\n")
        write_baseline(root, {MAIN_JS: 1, STYLES: 1})
        code, out, err = run_tool(root, "--check-added", "--base", "HEAD~1")
        self.assertEqual(0, code, out + err)


class CountFreezeTests(unittest.TestCase):
    def test_styles_css_count_growth_is_violation(self):
        root = make_repo_with_base()
        write_baseline(root, {MAIN_JS: 1, STYLES: 0})
        code, out, err = run_tool(root, "--check-added", "--base", "HEAD")
        self.assertEqual(1, code)
        self.assertIn(STYLES, out)
        self.assertIn("frozen", out)

    def test_baseline_missing_baseline_file_is_internal_error(self):
        root = make_repo_with_base()
        code, out, err = run_tool(root, "--check-added", "--base", "HEAD")
        self.assertEqual(2, code)
        self.assertIn("baseline", (out + err).lower())


class CorruptBaselineTests(unittest.TestCase):
    """UTB-6 (round-2 audit): baseline JSON that parses but is NOT an object
    (e.g. `[]`) reached scan_frozen_counts and crashed `.get("counts")` with
    an AttributeError outside main()'s except tuple -- a traceback where the
    contract promises exit 2 + ERROR."""

    def test_non_dict_baseline_json_exits_two_not_traceback(self):
        root = make_repo_with_base()
        baseline = Path(root) / BASELINE
        baseline.parent.mkdir(parents=True, exist_ok=True)
        baseline.write_text("[]", encoding="utf-8")
        code, out, err = run_tool(root, "--check-added", "--base", "HEAD")
        combined = out + err
        self.assertEqual(2, code, combined)
        self.assertIn("ERROR", combined)
        self.assertNotIn("Traceback", combined)

    def test_json_scalar_baseline_exits_two_not_traceback(self):
        root = make_repo_with_base()
        baseline = Path(root) / BASELINE
        baseline.parent.mkdir(parents=True, exist_ok=True)
        baseline.write_text("null", encoding="utf-8")
        code, out, err = run_tool(root, "--check-added", "--base", "HEAD")
        combined = out + err
        self.assertEqual(2, code, combined)
        self.assertIn("ERROR", combined)
        self.assertNotIn("Traceback", combined)


class UsageTests(unittest.TestCase):
    def test_unknown_revision_exits_two(self):
        root = make_repo_with_base()
        write_baseline(root, {MAIN_JS: 1, STYLES: 1})
        code, out, err = run_tool(root, "--check-added", "--base", "nosuchrev")
        self.assertEqual(2, code)

    def test_missing_flags_are_usage_errors(self):
        root = make_repo_with_base()
        code, _, _ = run_tool(root, "--check-added")
        self.assertEqual(2, code)
        code, _, _ = run_tool(root)
        self.assertEqual(2, code)

    def test_update_baseline_writes_sorted_counts(self):
        root = make_repo_with_base()
        code, out, err = run_tool(root, "--update-baseline")
        self.assertEqual(0, code, err)
        payload = json.loads(
            (Path(root) / BASELINE).read_text(encoding="utf-8")
        )
        self.assertEqual(
            {MAIN_JS: 1, STYLES: 1}, payload["counts"]
        )
        self.assertTrue(all(":" in entry for entry in payload["locations"]))
        self.assertEqual(payload["locations"], sorted(payload["locations"]))


class RegexTests(unittest.TestCase):
    def test_literal_lengths_and_rejects(self):
        find = theme_token_check.find_hex_literals
        self.assertEqual(["#abc", "#abcd", "#abcdef", "#12345678"],
                         find("a #abc b #abcd c #abcdef d #12345678 e"))
        self.assertEqual([], find("#12345 #1234567890 #xyz use#${id}"))


if __name__ == "__main__":
    unittest.main()
