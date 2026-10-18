#!/usr/bin/env python3
"""
Set up a local development environment for contributing to Ariadne's Thread.

Checks the toolchain, then pulls down everything a first-time contributor
needs: the crate dependencies for the Rust workspace, the npm packages for
the VS Code extension, and the rustfmt/clippy components that the repo's
hygiene gates expect. The python tooling in tools/ runs on the standard
library only, so there is nothing to pip-install.

Usage:
    python tools/setup_dev_env.py [--no-locked] [--skip-npm] [--build]

By default the crate fetch uses --locked so it matches the committed
Cargo.lock exactly. Pass --no-locked if you intentionally want cargo to
refresh the lock file. Pass --build to also warm a release build, since
debug indexing is about 20x slower than release at TU scale.
"""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
EXTENSION_DIR = REPO_ROOT / "extensions" / "ariadne-extension"
MIN_PYTHON = (3, 9)


def run(cmd: list[str], cwd: Path | None = None) -> None:
    """Run a command, streaming output to the terminal."""
    print(f"  $ {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=str(cwd) if cwd else None, check=False)
    if result.returncode != 0:
        raise SystemExit(f"Command failed (exit {result.returncode}): {' '.join(cmd)}")


def python_version_supported(version_info: tuple[int, ...]) -> bool:
    """True when the interpreter is at least MIN_PYTHON (major, minor)."""
    return tuple(version_info[:2]) >= MIN_PYTHON


def find_npm() -> str | None:
    """Locate npm, including the npm.cmd shim used on Windows."""
    return shutil.which("npm") or shutil.which("npm.cmd")


def check_toolchain(require_npm: bool) -> list[str]:
    """Return human-readable problems with the toolchain, empty if healthy."""
    problems = []
    if not python_version_supported(sys.version_info):
        wanted = ".".join(str(part) for part in MIN_PYTHON)
        problems.append(f"Python {wanted}+ required, found {sys.version.split()[0]}")
    if not (REPO_ROOT / "Cargo.toml").exists():
        problems.append(f"No Cargo.toml at {REPO_ROOT} - run this from a repo clone")
    if not shutil.which("cargo"):
        problems.append("cargo not found on PATH - install Rust from https://rustup.rs")
    if not shutil.which("git"):
        problems.append("git not found on PATH - required for contributing")
    if require_npm and not find_npm():
        problems.append("npm not found on PATH - install Node.js to build the VS Code extension")
    return problems


def ensure_rust_components() -> None:
    """Install rustfmt and clippy if rustup manages the toolchain."""
    rustup = shutil.which("rustup")
    if not rustup:
        print("  rustup not found - install rustfmt and clippy some other way")
        return
    for component in ("rustfmt", "clippy"):
        run([rustup, "component", "add", component])


def cargo_fetch_command(locked: bool) -> list[str]:
    """Build the cargo fetch invocation for the workspace."""
    cmd = ["cargo", "fetch"]
    if locked:
        cmd.append("--locked")
    return cmd


def npm_install_command(lockfile_exists: bool) -> list[str]:
    """Prefer 'npm ci' for reproducible installs when a lockfile exists."""
    return ["ci"] if lockfile_exists else ["install"]


def fetch_crate_deps(locked: bool) -> None:
    """Download every crate dependency the workspace resolves to."""
    run(cargo_fetch_command(locked), cwd=REPO_ROOT)


def install_extension_deps() -> None:
    """Install the VS Code extension's npm dependencies."""
    npm = find_npm()
    if not npm:
        raise SystemExit("npm disappeared mid-run - install Node.js and retry")
    lockfile = (EXTENSION_DIR / "package-lock.json").exists()
    run([npm] + npm_install_command(lockfile), cwd=EXTENSION_DIR)


def build_release() -> None:
    """Warm a release build so first runs are not stuck in debug."""
    run(["cargo", "build", "--release"], cwd=REPO_ROOT)


def print_next_steps() -> None:
    """Show the contributor what to run once setup finishes."""
    print("")
    print("Dev environment ready. Handy commands from the repo root:")
    print("  cargo build --release                          # the binary you actually want")
    print("  cargo test                                     # Rust suites")
    print("  python tools/testrun.py                        # one-line canonical test summary")
    print("  python -m unittest discover -s tools -p '*_tests.py'  # tool script suites (what CI runs)")
    print("  python tools/ui_smoke.py                       # build + boot the UI and check routes")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[2])
    parser.add_argument(
        "--no-locked",
        action="store_true",
        help="let cargo refresh Cargo.lock instead of using --locked",
    )
    parser.add_argument(
        "--skip-npm",
        action="store_true",
        help="skip the VS Code extension npm install",
    )
    parser.add_argument(
        "--build",
        action="store_true",
        help="also run a release cargo build after fetching dependencies",
    )
    args = parser.parse_args(argv)

    problems = check_toolchain(require_npm=not args.skip_npm)
    if problems:
        print("Toolchain problems:")
        for problem in problems:
            print(f"  - {problem}")
        return 1

    print("Rust components (rustfmt, clippy):")
    ensure_rust_components()

    print("Crate dependencies:")
    fetch_crate_deps(locked=not args.no_locked)

    if not args.skip_npm:
        print("VS Code extension dependencies:")
        install_extension_deps()

    if args.build:
        print("Release build:")
        build_release()

    print_next_steps()
    return 0


if __name__ == "__main__":
    sys.exit(main())
