"""Unit tests for tools/setup_dev_env.py (contributor environment bootstrap).

Run directly: python tools\\setup_dev_env_tests.py
These tests cover the pure decision logic only; no network fetch, npm run,
or cargo invocation happens here.
"""

import contextlib
import io
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

import setup_dev_env


class PythonVersionSupportedTests(unittest.TestCase):
    def test_accepts_minimum_version(self):
        self.assertTrue(setup_dev_env.python_version_supported(setup_dev_env.MIN_PYTHON + (0,)))

    def test_accepts_newer_version(self):
        newer = (setup_dev_env.MIN_PYTHON[0] + 1, 0, 3)
        self.assertTrue(setup_dev_env.python_version_supported(newer))

    def test_rejects_older_minor(self):
        older = (setup_dev_env.MIN_PYTHON[0], setup_dev_env.MIN_PYTHON[1] - 1, 9)
        self.assertFalse(setup_dev_env.python_version_supported(older))

    def test_rejects_older_major(self):
        self.assertFalse(setup_dev_env.python_version_supported((2, 7, 18)))


class CargoFetchCommandTests(unittest.TestCase):
    def test_locked_by_default_shape(self):
        self.assertEqual(setup_dev_env.cargo_fetch_command(True), ["cargo", "fetch", "--locked"])

    def test_unlocked_omits_flag(self):
        self.assertEqual(setup_dev_env.cargo_fetch_command(False), ["cargo", "fetch"])


class NpmInstallCommandTests(unittest.TestCase):
    def test_prefers_ci_with_lockfile(self):
        self.assertEqual(setup_dev_env.npm_install_command(True), ["ci"])

    def test_falls_back_to_install_without_lockfile(self):
        self.assertEqual(setup_dev_env.npm_install_command(False), ["install"])


class CheckToolchainTests(unittest.TestCase):
    def test_reports_missing_npm_when_required(self):
        original = setup_dev_env.find_npm
        setup_dev_env.find_npm = lambda: None
        try:
            problems = setup_dev_env.check_toolchain(require_npm=True)
        finally:
            setup_dev_env.find_npm = original
        npm_problem = [problem for problem in problems if "npm" in problem]
        self.assertEqual(len(npm_problem), 1)

    def test_no_npm_problem_when_skipped(self):
        original = setup_dev_env.find_npm
        setup_dev_env.find_npm = lambda: None
        try:
            problems = setup_dev_env.check_toolchain(require_npm=False)
        finally:
            setup_dev_env.find_npm = original
        self.assertEqual([problem for problem in problems if "npm" in problem], [])

    def test_repo_checkout_passes_layout_check(self):
        problems = setup_dev_env.check_toolchain(require_npm=False)
        self.assertEqual([problem for problem in problems if "Cargo.toml" in problem], [])


class LayoutConstantsTests(unittest.TestCase):
    def test_repo_root_contains_workspace_manifest(self):
        self.assertTrue((setup_dev_env.REPO_ROOT / "Cargo.toml").exists())

    def test_extension_dir_contains_package_manifest(self):
        self.assertTrue((setup_dev_env.EXTENSION_DIR / "package.json").exists())


class MainWiringTests(unittest.TestCase):
    """Audit row 10: main()'s flag->argv wiring table plus the problem-list
    exit 1. run/which/find_npm/check_toolchain are monkeypatched; nothing
    external (rustup, cargo, npm, network) ever runs."""

    def setUp(self):
        holder = tempfile.TemporaryDirectory()
        self.addCleanup(holder.cleanup)
        self.ext_dir = Path(holder.name) / "extension"
        self.ext_dir.mkdir()
        self.recorded_runs = []
        self.recorded_require_npm = []

        def fake_run(cmd, cwd=None):
            self.recorded_runs.append([str(token) for token in cmd])

        def fake_which(name):
            return "/bin/rustup" if name == "rustup" else None

        def healthy_toolchain(require_npm):
            self.recorded_require_npm.append(require_npm)
            return []

        patches = [
            mock.patch.object(setup_dev_env, "EXTENSION_DIR", self.ext_dir),
            mock.patch.object(setup_dev_env, "run", fake_run),
            mock.patch.object(setup_dev_env, "find_npm", lambda: "/bin/npm"),
            mock.patch.object(setup_dev_env.shutil, "which",
                              side_effect=fake_which),
            mock.patch.object(setup_dev_env, "check_toolchain",
                              healthy_toolchain),
        ]
        for patcher in patches:
            patcher.start()
            self.addCleanup(patcher.stop)

    def _main(self, argv):
        captured = io.StringIO()
        with contextlib.redirect_stdout(captured):
            return_code = setup_dev_env.main(argv)
        return return_code, captured.getvalue()

    def test_default_flags_run_the_locked_wired_sequence(self):
        return_code, _output = self._main([])
        self.assertEqual(0, return_code)
        self.assertEqual(self.recorded_runs, [
            # ensure_rust_components launches the shutil.which RESULT as
            # argv[0], hence the resolved /bin/rustup path in the table.
            ["/bin/rustup", "component", "add", "rustfmt"],
            ["/bin/rustup", "component", "add", "clippy"],
            ["cargo", "fetch", "--locked"],
            ["/bin/npm", "install"],
        ])
        self.assertEqual([True], self.recorded_require_npm)

    def test_lockfile_present_prefers_npm_ci(self):
        (self.ext_dir / "package-lock.json").write_text("{}\n",
                                                        encoding="utf-8")
        return_code, _output = self._main([])
        self.assertEqual(0, return_code)
        self.assertEqual(["/bin/npm", "ci"], self.recorded_runs[-1])

    def test_no_locked_flag_drops_locked_from_fetch(self):
        return_code, _output = self._main(["--no-locked"])
        self.assertEqual(0, return_code)
        self.assertIn(["cargo", "fetch"], self.recorded_runs)
        self.assertNotIn(["cargo", "fetch", "--locked"], self.recorded_runs)

    def test_skip_npm_skips_the_require_and_the_npm_command(self):
        return_code, _output = self._main(["--skip-npm"])
        self.assertEqual(0, return_code)
        self.assertEqual([False], self.recorded_require_npm)
        npm_runs = [cmd for cmd in self.recorded_runs if "npm" in cmd[0]]
        self.assertEqual([], npm_runs)

    def test_build_flag_appends_the_release_build(self):
        return_code, _output = self._main(["--build"])
        self.assertEqual(0, return_code)
        self.assertEqual(["cargo", "build", "--release"],
                         self.recorded_runs[-1])

    def test_problems_print_and_exit_one_without_running_anything(self):
        problems = ["npm not found on PATH - install Node.js",
                    "git not found on PATH - required for contributing"]
        with mock.patch.object(setup_dev_env, "check_toolchain",
                               lambda require_npm: problems):
            return_code, output = self._main([])
        self.assertEqual(1, return_code)
        self.assertIn("Toolchain problems:", output)
        for problem in problems:
            self.assertIn(problem, output)
        self.assertEqual([], self.recorded_runs)


if __name__ == "__main__":
    unittest.main()
