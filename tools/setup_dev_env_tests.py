"""Unit tests for tools/setup_dev_env.py (contributor environment bootstrap).

Run directly: python tools\\setup_dev_env_tests.py
These tests cover the pure decision logic only; no network fetch, npm run,
or cargo invocation happens here.
"""

import sys
import unittest
from pathlib import Path

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


if __name__ == "__main__":
    unittest.main()
