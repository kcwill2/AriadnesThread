"""Unit tests for tools/checkpoint_guard.py (fresh-session recovery guard).

Run directly: python tools\\checkpoint_guard_tests.py
Every fixture is a throwaway git repo under a temp directory; the real repo
and the user's git configuration are never touched.
"""

import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import checkpoint_guard

GUARD_PATH = str(Path(__file__).resolve().parent / "checkpoint_guard.py")


def run_guard(cwd, *arguments):
    finished = subprocess.run(
        [sys.executable, GUARD_PATH, *[str(argument) for argument in arguments]],
        cwd=str(cwd),
        capture_output=True,
        text=True,
        errors="replace",
    )
    return finished.returncode, finished.stdout + finished.stderr


def run_git(cwd, *arguments):
    finished = subprocess.run(
        ["git", *[str(argument) for argument in arguments]],
        cwd=str(cwd),
        capture_output=True,
        text=True,
        errors="replace",
    )
    assert finished.returncode == 0, finished.stderr
    return finished.stdout


def write_file(root, relative, text):
    target = Path(root) / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text, encoding="utf-8")


def append_file(root, relative, text):
    target = Path(root) / relative
    with target.open("a", encoding="utf-8") as handle:
        handle.write(text)


def init_seed_repo(root):
    run_git(root, "-c", "init.defaultBranch=main", "init", "-q")
    run_git(root, "config", "user.name", "Guard Fixture")
    run_git(root, "config", "user.email", "guard-fixture@example.invalid")
    write_file(root, "crates/alpha/first.rs", "fn alpha() {}\n")
    write_file(root, "tools/tool_main.py", "print('seed')\n")
    write_file(root, "docs/notes.md", "# notes\n")
    run_git(root, "add", "-A")
    run_git(root, "commit", "-q", "-m", "seed commit")


def make_dirty_repo(root):
    init_seed_repo(root)
    append_file(root, "crates/alpha/first.rs", "fn touched() {}\n")
    append_file(root, "tools/tool_main.py", "print('tool edit')\n")
    write_file(root, "crates/alpha/untracked_note.md", "untracked\n")


def python_command(code_body):
    return f'"{sys.executable}" -c "{code_body}"'


def passing_command():
    return python_command("print('ok')")


def failing_command():
    return python_command("import sys; sys.exit(1)")


def section_of(report_text, heading, next_heading):
    start = report_text.index(heading) + len(heading)
    end = report_text.index(next_heading, start)
    return report_text[start:end]


class StatusReportTests(unittest.TestCase):
    def setUp(self):
        holder = tempfile.TemporaryDirectory()
        self.addCleanup(holder.cleanup)
        self.root = Path(holder.name) / "repo"
        self.root.mkdir()
        self.scratch = Path(holder.name)

    def test_status_exits_zero_and_reports_head_branch_numstat_untracked(self):
        make_dirty_repo(self.root)
        expected_sha = run_git(self.root, "rev-parse", "--short", "HEAD").strip()
        exit_code, stdout = run_guard(self.root, "status")
        self.assertEqual(0, exit_code)
        self.assertIn("NEVER resumed", stdout)
        self.assertIn(expected_sha, stdout)
        self.assertIn("main", stdout)
        self.assertIn("crates/alpha/first.rs", stdout)
        self.assertIn("tools/tool_main.py", stdout)
        self.assertIn("UNTRACKED", stdout)
        self.assertIn("crates/alpha/untracked_note.md", stdout)

    def test_status_census_lists_registered_worktree_with_branch_and_dirty_count(self):
        make_dirty_repo(self.root)
        worktree = self.scratch / "side-wt"
        self.addCleanup(shutil.rmtree, worktree, True)
        run_git(self.root, "worktree", "add", "-q", worktree, "-b", "side-branch")
        write_file(worktree, "stray_in_side.txt", "dirty\n")
        exit_code, stdout = run_guard(self.root, "status")
        self.assertEqual(0, exit_code)
        self.assertIn("WORKTREE CENSUS", stdout)
        self.assertIn("side-wt", stdout)
        self.assertIn("branch=side-branch", stdout)
        self.assertIn("dirty=1", stdout)

    def test_status_census_flags_unregistered_kilo_worktree_dir_as_orphan(self):
        make_dirty_repo(self.root)
        write_file(self.root, ".kilo/worktrees/orphan-demo/stray.rs", "orphan\n")
        exit_code, stdout = run_guard(self.root, "status")
        self.assertEqual(0, exit_code)
        self.assertIn("WORKTREE CENSUS", stdout)
        self.assertIn("orphan-demo", stdout)
        self.assertIn("ORPHAN", stdout)

    def test_status_census_flags_detached_worktree_as_detached(self):
        make_dirty_repo(self.root)
        worktree = self.scratch / "detached-wt"
        self.addCleanup(shutil.rmtree, worktree, True)
        run_git(self.root, "worktree", "add", "-q", "--detach", worktree, "HEAD")
        exit_code, stdout = run_guard(self.root, "status")
        self.assertEqual(0, exit_code)
        self.assertIn("detached-wt", stdout)
        self.assertIn("branch=DETACHED", stdout)


class ResumeRunTests(unittest.TestCase):
    def setUp(self):
        holder = tempfile.TemporaryDirectory()
        self.addCleanup(holder.cleanup)
        self.root = Path(holder.name) / "repo"
        self.root.mkdir()
        self.scratch = Path(holder.name)

    def test_resume_with_explicit_scope_and_cmd_classifies_and_writes_skeleton(self):
        make_dirty_repo(self.root)
        spec = self.scratch / "handoff.md"
        spec.write_text("## Notes\nplain prose only\n", encoding="utf-8")
        exit_code, stdout = run_guard(
            self.root, "resume",
            "--spec", spec,
            "--scope", "crates/alpha/**",
            "--cmd", passing_command(),
            "--cmd", failing_command(),
        )
        self.assertEqual(1, exit_code)
        self.assertIn("green", stdout)
        self.assertIn("red", stdout)
        report = self.root / "tasks" / "handoff-resume.md"
        self.assertTrue(report.is_file())
        text = report.read_text(encoding="utf-8")
        for heading in (
            "# Resume bootstrap for",
            "## Rule",
            "## Disk state",
            "## Command verdicts",
            "## In-scope edits",
            "## Stray edits (out of scope)",
            "## Remaining handoff steps",
        ):
            self.assertIn(heading, text)
        in_scope = section_of(text, "## In-scope edits", "## Stray edits")
        stray = section_of(text, "## Stray edits (out of scope)",
                           "## Remaining handoff steps")
        self.assertIn("crates/alpha/first.rs", in_scope)
        self.assertIn("crates/alpha/untracked_note.md", in_scope)
        self.assertIn("tools/tool_main.py", stray)

    def test_resume_without_touch_set_or_commands_exits_one_asking_for_args(self):
        make_dirty_repo(self.root)
        spec = self.scratch / "vague-handoff.md"
        spec.write_text("# Old handoff\n\nNothing machine readable here.\n",
                        encoding="utf-8")
        exit_code, stdout = run_guard(self.root, "resume", "--spec", spec)
        self.assertEqual(1, exit_code)
        self.assertIn("--scope", stdout)
        self.assertIn("--cmd", stdout)
        self.assertIn("explicit", stdout.lower())

    def test_resume_with_missing_spec_exits_one_asking_for_args(self):
        make_dirty_repo(self.root)
        exit_code, stdout = run_guard(
            self.root, "resume", "--spec", self.scratch / "gone.md")
        self.assertEqual(1, exit_code)
        self.assertIn("--scope", stdout)

    def test_resume_parses_m19_shaped_touch_set_and_fenced_command(self):
        init_seed_repo(self.root)
        write_file(self.root, "crates/demo/demo.rs", "fn demo() {}\n")
        run_git(self.root, "add", "-A")
        run_git(self.root, "commit", "-q", "-m", "add demo crate")
        append_file(self.root, "crates/demo/demo.rs", "fn touched() {}\n")
        spec = self.scratch / "m19-handoff.md"
        spec.write_text(
            "\n".join([
                "# M19 handoff",
                "",
                "## Touch-set (hard boundary)",
                "",
                "- `crates/demo/**` (demo)",
                "",
                "## Gates",
                "",
                "```text",
                "python -c \"print('ok')\"",
                "```",
                "",
            ]),
            encoding="utf-8",
        )
        exit_code, stdout = run_guard(self.root, "resume", "--spec", spec)
        self.assertEqual(0, exit_code)
        self.assertIn("crates/demo/**", stdout)
        report = self.root / "tasks" / "m19-handoff-resume.md"
        self.assertTrue(report.is_file())
        text = report.read_text(encoding="utf-8")
        in_scope = section_of(text, "## In-scope edits", "## Stray edits")
        self.assertIn("crates/demo/demo.rs", in_scope)
        self.assertIn("green", text)


class CliArgumentTests(unittest.TestCase):
    def setUp(self):
        holder = tempfile.TemporaryDirectory()
        self.addCleanup(holder.cleanup)
        self.root = Path(holder.name) / "repo"
        self.root.mkdir()

    def test_unknown_subcommand_exits_two_with_usage(self):
        exit_code, stdout = run_guard(self.root, "bogus")
        self.assertEqual(2, exit_code)
        self.assertIn("usage", stdout.lower())

    def test_missing_subcommand_exits_two_with_usage(self):
        exit_code, stdout = run_guard(self.root)
        self.assertEqual(2, exit_code)
        self.assertIn("usage", stdout.lower())

    def test_resume_option_without_value_exits_two(self):
        exit_code, stdout = run_guard(self.root, "resume", "--spec")
        self.assertEqual(2, exit_code)
        self.assertIn("usage", stdout.lower())


class ModuleContractTests(unittest.TestCase):
    def test_glob_matching_covers_exact_paths_tree_dirs_and_misses(self):
        self.assertTrue(checkpoint_guard.path_in_scope(
            "crates/alpha/first.rs", ["crates/alpha/**"]))
        self.assertTrue(checkpoint_guard.path_in_scope(
            "docs/notes.md", ["docs/notes.md"]))
        self.assertTrue(checkpoint_guard.path_in_scope(
            "crates/beta/src/lib.rs", ["crates/beta"]))
        self.assertFalse(checkpoint_guard.path_in_scope(
            "tools/tool_main.py", ["crates/alpha/**", "docs/notes.md"]))

    def test_parse_handoff_extracts_touch_set_globs_and_commands(self):
        with tempfile.TemporaryDirectory() as holder:
            spec = Path(holder) / "spec.md"
            spec.write_text(
                "\n".join([
                    "# Handoff",
                    "",
                    "## Touch-set (hard)",
                    "",
                    "- `crates/lang-scala/**` (new crate)",
                    "- `tools/helper.py` (tool)",
                    "  also mentions `walkthroughs/m19-walkthrough.html` inline",
                    "",
                    "## Gates",
                    "",
                    "- `cargo test -p lang-scala`",
                    "",
                    "```bash",
                    "cargo test -p lang-scala",
                    "python tools/checkpoint_guard.py status",
                    "```",
                    "",
                ]),
                encoding="utf-8",
            )
            parsed = checkpoint_guard.parse_handoff(spec)
        self.assertEqual(
            ["crates/lang-scala/**", "tools/helper.py",
             "walkthroughs/m19-walkthrough.html"],
            parsed.globs,
        )
        self.assertEqual(
            ["cargo test -p lang-scala",
             "python tools/checkpoint_guard.py status"],
            parsed.commands,
        )


if __name__ == "__main__":
    unittest.main()
