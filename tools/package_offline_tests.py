"""Unit tests for tools/package_offline.py (offline distribution packager).

Run directly: python tools\\package_offline_tests.py
Or discovered: python -m unittest discover -s tools -p "*_tests.py"

Everything here is OFFLINE (UTA2-R2; this script had zero tests before):
every subprocess launch is a mocked subprocess.run, REPO_ROOT/DIST_DIR are
redirected into throwaway temp dirs, and detect_platform/exe_suffix are
stubbed so expectations never depend on the host OS. No cargo, no npm, no
network. Fakes are stdlib-only (unittest.mock).
"""

import contextlib
import io
import subprocess
import sys
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

import package_offline

VERSION = package_offline.VERSION
BINARY_NAMES = ("tool", "ariadne", "clang-worker")
VSIX_NAME = "ariadne-extension-%s.vsix" % VERSION


def make_which(available):
    """shutil.which stand-in resolving only the named tools."""
    def fake_which(name):
        return "/fake/bin/" + name if name in available else None
    return fake_which


def make_recording_run(on_call=None):
    """subprocess.run stand-in recording (argv, cwd) and returning rc 0.

    on_call(argv, cwd) may create output files -- the fake 'builds' the
    binaries/vsix the packager expects to find afterwards."""
    calls = []

    def fake_run(cmd, cwd=None, env=None, check=False):
        calls.append(([str(token) for token in cmd], cwd))
        if on_call is not None:
            on_call([str(token) for token in cmd], cwd)
        return subprocess.CompletedProcess(cmd, returncode=0)

    return fake_run, calls


def cargo_bin_pairs(argv):
    """The ('--bin', name) pairs of a cargo build argv, order-normalized
    (package_offline iterates a set, so the pair ORDER is not contract)."""
    return sorted(
        (argv[index], argv[index + 1])
        for index in range(len(argv) - 1) if argv[index] == "--bin")


class ScratchRepoMixin:
    """A throwaway repo root with REPO_ROOT/DIST_DIR/exe_suffix patched."""

    def patch_repo(self, detect_platform_value=None):
        holder = tempfile.TemporaryDirectory()
        self.addCleanup(holder.cleanup)
        self.root = Path(holder.name) / "repo"
        self.release_dir = self.root / "target" / "release"
        self.ext_dir = self.root / "extensions" / "ariadne-extension"
        self.dist_dir = self.root / "dist"
        patches = [
            mock.patch.object(package_offline, "REPO_ROOT", self.root),
            mock.patch.object(package_offline, "DIST_DIR", self.dist_dir),
            mock.patch.object(package_offline, "exe_suffix",
                              return_value=""),
        ]
        if detect_platform_value is not None:
            patches.append(mock.patch.object(
                package_offline, "detect_platform",
                return_value=detect_platform_value))
        for patcher in patches:
            patcher.start()
            self.addCleanup(patcher.stop)

    def make_binaries(self, names):
        self.release_dir.mkdir(parents=True, exist_ok=True)
        made = {}
        for name in names:
            binary = self.release_dir / name
            binary.write_bytes(b"MZ fake binary " + name.encode("ascii"))
            made[name] = binary
        return made


class RunCommandTests(unittest.TestCase):
    def test_zero_returncode_returns_none_and_echoes_command(self):
        fake_run, calls = make_recording_run()
        with mock.patch.object(package_offline.subprocess, "run", fake_run):
            captured = io.StringIO()
            with contextlib.redirect_stdout(captured):
                outcome = package_offline.run(["cargo", "build"])
        self.assertIsNone(outcome)
        self.assertEqual(1, len(calls))
        self.assertIn("$ cargo build", captured.getvalue())

    def test_nonzero_returncode_raises_systemexit_naming_exit_and_command(self):
        def always_seven(cmd, cwd=None, env=None, check=False):
            return subprocess.CompletedProcess(cmd, returncode=7)

        with mock.patch.object(package_offline.subprocess, "run", always_seven):
            with self.assertRaises(SystemExit) as raised:
                with contextlib.redirect_stdout(io.StringIO()):
                    package_offline.run(["cargo", "build"])
        message = str(raised.exception)
        self.assertIn("exit 7", message)
        self.assertIn("cargo build", message)


class BuildRustBinariesTests(ScratchRepoMixin, unittest.TestCase):
    def setUp(self):
        self.patch_repo()

    def test_reuse_when_all_binaries_present_and_cargo_untouched(self):
        made = self.make_binaries(BINARY_NAMES)
        fake_run, calls = make_recording_run()
        with mock.patch.object(package_offline.subprocess, "run", fake_run):
            captured = io.StringIO()
            with contextlib.redirect_stdout(captured):
                result = package_offline.build_rust_binaries(rebuild=False)
        self.assertEqual(calls, [], "reuse must not invoke cargo")
        self.assertEqual(result, {name: made[name] for name in BINARY_NAMES})
        self.assertIn("Reusing existing binaries", captured.getvalue())

    def test_missing_binary_builds_then_collects_all_three(self):
        self.make_binaries(("tool", "ariadne"))

        def fake_build(argv, _cwd):
            self.make_binaries(BINARY_NAMES)

        fake_run, calls = make_recording_run(on_call=fake_build)
        with mock.patch.object(package_offline.subprocess, "run", fake_run):
            captured = io.StringIO()
            with contextlib.redirect_stdout(captured):
                result = package_offline.build_rust_binaries(rebuild=False)
        self.assertEqual(1, len(calls))
        argv, cwd = calls[0]
        self.assertEqual(["cargo", "build", "--release"], argv[:3])
        self.assertEqual(
            cargo_bin_pairs(argv),
            sorted(("--bin", name) for name in BINARY_NAMES))
        self.assertEqual(str(self.root), str(cwd))
        self.assertEqual(sorted(result), sorted(BINARY_NAMES))
        self.assertIn("Missing: clang-worker", captured.getvalue())

    def test_rebuild_flag_forces_cargo_even_when_binaries_exist(self):
        self.make_binaries(BINARY_NAMES)
        fake_run, calls = make_recording_run(
            on_call=lambda argv, cwd: self.make_binaries(BINARY_NAMES))
        with mock.patch.object(package_offline.subprocess, "run", fake_run):
            with contextlib.redirect_stdout(io.StringIO()):
                package_offline.build_rust_binaries(rebuild=True)
        self.assertEqual(1, len(calls))

    def test_build_not_producing_expected_binary_raises_systemexit(self):
        fake_run, _calls = make_recording_run()
        with mock.patch.object(package_offline.subprocess, "run", fake_run):
            with self.assertRaises(SystemExit) as raised:
                with contextlib.redirect_stdout(io.StringIO()):
                    package_offline.build_rust_binaries(rebuild=True)
        self.assertIn("Build did not produce expected binary",
                      str(raised.exception))


class BuildVsixTests(ScratchRepoMixin, unittest.TestCase):
    def setUp(self):
        self.patch_repo()
        self.ext_dir.mkdir(parents=True)
        self.vsix_path = self.ext_dir / VSIX_NAME

    def test_reuse_existing_vsix_without_touching_npm(self):
        self.vsix_path.write_bytes(b"fake vsix")
        with mock.patch.object(package_offline.shutil, "which",
                               side_effect=AssertionError(
                                   "which must not run on the reuse path")):
            captured = io.StringIO()
            with contextlib.redirect_stdout(captured):
                result = package_offline.build_vsix(rebuild=False)
        self.assertEqual(result, self.vsix_path)
        self.assertIn("Reusing existing", captured.getvalue())

    def test_missing_npm_raises_systemexit(self):
        with mock.patch.object(package_offline.shutil, "which",
                               side_effect=make_which(set())):
            with self.assertRaises(SystemExit) as raised:
                package_offline.build_vsix(rebuild=False)
        self.assertIn("npm not found on PATH", str(raised.exception))

    def _package_vsix_on_call(self, argv, _cwd):
        if argv[-2:] == ["package", "--allow-missing-repository"]:
            self.vsix_path.write_bytes(b"fake vsix")

    def test_npm_install_compile_then_vsce_package_sequence(self):
        fake_run, calls = make_recording_run(on_call=self._package_vsix_on_call)
        with mock.patch.object(package_offline.shutil, "which",
                               side_effect=make_which({"npm", "vsce"})):
            with mock.patch.object(package_offline.subprocess, "run", fake_run):
                with contextlib.redirect_stdout(io.StringIO()):
                    result = package_offline.build_vsix(rebuild=False)
        npm = "/fake/bin/npm"
        self.assertEqual(
            [argv for argv, _cwd in calls],
            [[npm, "install"],
             [npm, "run", "compile"],
             ["/fake/bin/vsce", "package", "--allow-missing-repository"]])
        self.assertTrue(all(str(cwd) == str(self.ext_dir)
                            for _argv, cwd in calls))
        self.assertEqual(result, self.vsix_path)

    def test_existing_node_modules_skips_install(self):
        (self.ext_dir / "node_modules").mkdir()
        fake_run, calls = make_recording_run(on_call=self._package_vsix_on_call)
        with mock.patch.object(package_offline.shutil, "which",
                               side_effect=make_which({"npm", "vsce"})):
            with mock.patch.object(package_offline.subprocess, "run", fake_run):
                with contextlib.redirect_stdout(io.StringIO()):
                    package_offline.build_vsix(rebuild=False)
        self.assertEqual([argv[1] for argv, _cwd in calls],
                         ["run", "package"])

    def test_npx_fallback_when_vsce_missing(self):
        fake_run, calls = make_recording_run(on_call=self._package_vsix_on_call)
        with mock.patch.object(package_offline.shutil, "which",
                               side_effect=make_which({"npm", "npx"})):
            with mock.patch.object(package_offline.subprocess, "run", fake_run):
                captured = io.StringIO()
                with contextlib.redirect_stdout(captured):
                    package_offline.build_vsix(rebuild=False)
        self.assertEqual(
            calls[-1][0],
            ["/fake/bin/npx", "--yes", "@vscode/vsce", "package",
             "--allow-missing-repository"])
        self.assertIn("npx", captured.getvalue())

    def test_missing_vsce_and_npx_raises_systemexit(self):
        fake_run, _calls = make_recording_run(
            on_call=self._package_vsix_on_call)
        with mock.patch.object(package_offline.shutil, "which",
                               side_effect=make_which({"npm"})):
            with mock.patch.object(package_offline.subprocess, "run", fake_run):
                with self.assertRaises(SystemExit) as raised:
                    with contextlib.redirect_stdout(io.StringIO()):
                        package_offline.build_vsix(rebuild=False)
        self.assertIn("npx not found", str(raised.exception))

    def test_packaging_without_vsix_output_raises_systemexit(self):
        fake_run, _calls = make_recording_run()
        with mock.patch.object(package_offline.shutil, "which",
                               side_effect=make_which({"npm", "vsce"})):
            with mock.patch.object(package_offline.subprocess, "run", fake_run):
                with self.assertRaises(SystemExit) as raised:
                    with contextlib.redirect_stdout(io.StringIO()):
                        package_offline.build_vsix(rebuild=False)
        self.assertIn("vsce did not produce expected file",
                      str(raised.exception))


class CreatePackageTests(ScratchRepoMixin, unittest.TestCase):
    def setUp(self):
        self.patch_repo(detect_platform_value=("win", "x64"))
        self.binaries = self.make_binaries(BINARY_NAMES)
        self.vsix_path = self.ext_dir / VSIX_NAME
        self.vsix_path.parent.mkdir(parents=True)
        self.vsix_path.write_bytes(b"fake vsix")

    def _run_create(self):
        captured = io.StringIO()
        with contextlib.redirect_stdout(captured):
            pkg_root = package_offline.create_package(
                self.binaries, self.vsix_path)
        return pkg_root, captured.getvalue()

    def test_stages_binaries_vsix_and_win_installer_with_readme(self):
        installer_dir = self.root / "tools" / "offline_package"
        installer_dir.mkdir(parents=True)
        (installer_dir / "install.ps1").write_text("Write-Host 'install'\n",
                                                   encoding="utf-8")
        (installer_dir / "README.md").write_text("# offline readme\n",
                                                 encoding="utf-8")
        pkg_root, output = self._run_create()
        self.assertEqual(pkg_root.name,
                         "ariadne-offline-%s-win-x64" % VERSION)
        for name in BINARY_NAMES:
            staged = pkg_root / "bin" / name
            self.assertTrue(staged.is_file(), output)
            self.assertEqual(staged.read_bytes(), self.binaries[name].read_bytes())
        self.assertEqual((pkg_root / "vsix" / VSIX_NAME).read_bytes(),
                         b"fake vsix")
        # (the installer copy is a silent copy2 -- pinned by content, not by
        # an output line: the script prints only binary/vsix names)
        self.assertIn("Write-Host", (pkg_root / "install.ps1").read_text(
            encoding="utf-8"))
        self.assertIn("offline readme", (pkg_root / "README.md").read_text(
            encoding="utf-8"))

    def test_missing_installer_inputs_are_skipped_not_fatal(self):
        pkg_root, _output = self._run_create()
        self.assertFalse((pkg_root / "install.ps1").exists())
        self.assertFalse((pkg_root / "README.md").exists())
        self.assertTrue((pkg_root / "bin" / "tool").is_file())

    def test_stale_package_root_is_replaced_not_merged(self):
        pkg_root_path = (self.dist_dir
                         / ("ariadne-offline-%s-win-x64" % VERSION))
        pkg_root_path.mkdir(parents=True)
        stale = pkg_root_path / "stale-from-last-run.txt"
        stale.write_text("stale\n", encoding="utf-8")
        pkg_root, _output = self._run_create()
        self.assertEqual(pkg_root, pkg_root_path)
        self.assertFalse(stale.exists())


class CreateArchiveTests(ScratchRepoMixin, unittest.TestCase):
    def setUp(self):
        self.patch_repo()

    def _make_pkg_root(self):
        pkg_root = self.dist_dir / ("ariadne-offline-%s-platform" % VERSION)
        (pkg_root / "bin").mkdir(parents=True)
        (pkg_root / "bin" / "tool").write_bytes(b"payload")
        return pkg_root

    def test_windows_platform_selects_zip_and_replaces_stale_archive(self):
        with mock.patch.object(package_offline, "detect_platform",
                               return_value=("win", "x64")):
            pkg_root = self._make_pkg_root()
            stale_archive = pkg_root.with_suffix(".zip")
            stale_archive.write_bytes(b"stale archive bytes")
            with contextlib.redirect_stdout(io.StringIO()):
                archive_path = package_offline.create_archive(pkg_root)
        self.assertEqual(archive_path, stale_archive)
        self.assertNotIn(b"stale archive bytes", archive_path.read_bytes())
        with zipfile.ZipFile(archive_path) as produced:
            names = [entry.replace("\\", "/")
                     for entry in produced.namelist()]
        self.assertIn("%s/bin/tool" % pkg_root.name, names)

    def test_other_platform_selects_tar_gz(self):
        with mock.patch.object(package_offline, "detect_platform",
                               return_value=("linux", "x64")):
            pkg_root = self._make_pkg_root()
            with contextlib.redirect_stdout(io.StringIO()):
                archive_path = package_offline.create_archive(pkg_root)
        self.assertTrue(str(archive_path).endswith(".tar.gz"))
        with tarfile.open(archive_path, "r:gz") as produced:
            names = produced.getnames()
        self.assertIn("%s/bin/tool" % pkg_root.name, names)


class MainWiringTests(unittest.TestCase):
    """main() is argparse-only glue (it takes no argv parameter and reads
    sys.argv), so the fakes patch sys.argv and the four stage functions."""

    def setUp(self):
        holder = tempfile.TemporaryDirectory()
        self.addCleanup(holder.cleanup)
        self.staged_dir = Path(holder.name) / "pkg"
        self.staged_dir.mkdir()
        self.vsix = Path(holder.name) / VSIX_NAME
        self.vsix.write_bytes(b"fake vsix")
        self.archive = Path(holder.name) / "pkg.zip"
        self.archive.write_bytes(b"PK fake")
        self.recorded = {"build_rust": [], "build_vsix": [], "stage": [],
                         "archive": []}

    def _patch_stages(self, extra_patches=()):
        def record_build_rust(rebuild):
            self.recorded["build_rust"].append(rebuild)
            return {"tool": Path("x/tool")}

        def record_build_vsix(rebuild):
            self.recorded["build_vsix"].append(rebuild)
            return self.vsix

        def record_stage(binaries, vsix_path):
            self.recorded["stage"].append((sorted(binaries), vsix_path))
            return self.staged_dir

        def record_archive(pkg_root):
            self.recorded["archive"].append(pkg_root)
            return self.archive

        patches = [
            mock.patch.object(package_offline, "build_rust_binaries",
                              record_build_rust),
            mock.patch.object(package_offline, "build_vsix",
                              record_build_vsix),
            mock.patch.object(package_offline, "create_package",
                              record_stage),
            mock.patch.object(package_offline, "create_archive",
                              record_archive),
            mock.patch.object(package_offline, "detect_platform",
                              return_value=("win", "x64")),
        ]
        for patcher in patches + list(extra_patches):
            patcher.start()
            self.addCleanup(patcher.stop)

    def _run_main(self, argv):
        patches = [mock.patch.object(sys, "argv",
                                     ["package_offline.py", *argv])]
        for patcher in patches:
            patcher.start()
            self.addCleanup(patcher.stop)
        captured = io.StringIO()
        with contextlib.redirect_stdout(captured):
            package_offline.main()
        return captured.getvalue()

    def test_default_flags_forward_false_and_archive_runs(self):
        self._patch_stages()
        output = self._run_main([])
        self.assertEqual([False], self.recorded["build_rust"])
        self.assertEqual([False], self.recorded["build_vsix"])
        self.assertEqual(1, len(self.recorded["archive"]))
        self.assertIn("Offline package at", output)
        self.assertIn("Platform: win-x64", output)
        self.assertIn("Version:  %s" % VERSION, output)
        self.assertIn("./install.ps1", output)

    def test_rebuild_flags_forward_true_to_both_builders(self):
        self._patch_stages()
        self._run_main(["--rebuild-binaries", "--rebuild-vsix"])
        self.assertEqual([True], self.recorded["build_rust"])
        self.assertEqual([True], self.recorded["build_vsix"])

    def test_no_archive_skips_create_archive_and_reports_staged_dir(self):
        self._patch_stages()
        output = self._run_main(["--no-archive"])
        self.assertEqual([], self.recorded["archive"])
        self.assertIn("Package staged at", output)
        self.assertIn(str(self.staged_dir), output)

    def test_stage_receives_binaries_and_vsix_from_the_builders(self):
        self._patch_stages()
        self._run_main([])
        self.assertEqual([(["tool"], self.vsix)], self.recorded["stage"])

    def test_unknown_flag_exits_two(self):
        self._patch_stages()
        with mock.patch.object(sys, "argv", ["package_offline.py", "--bogus"]):
            with self.assertRaises(SystemExit) as raised:
                with contextlib.redirect_stderr(io.StringIO()):
                    package_offline.main()
        self.assertEqual(2, raised.exception.code)


if __name__ == "__main__":
    unittest.main()
