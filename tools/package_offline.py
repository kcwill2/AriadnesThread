#!/usr/bin/env python3
"""
Package Ariadne's Thread for offline installation.

Builds the release Rust binaries and VS Code extension, then bundles
everything into a distributable archive that can be installed on a machine
with no internet access.

Usage:
    python tools/package_offline.py [--rebuild-binaries] [--rebuild-vsix] [--no-archive]

By default, reuses existing release binaries in target/release/ and the
existing .vsix in extensions/ariadne-extension/.  Use --rebuild-* to force
a fresh build of either component.
"""

from __future__ import annotations

import argparse
import os
import platform
import shutil
import subprocess
import sys
import tarfile
import zipfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
VERSION = "0.1.0"
PACKAGE_NAME = "ariadne-offline"
DIST_DIR = REPO_ROOT / "dist"


def detect_platform() -> tuple[str, str]:
    """Return (os_name, arch) normalized for file naming."""
    system = platform.system().lower()
    if system == "windows":
        os_name = "win"
    elif system == "darwin":
        os_name = "macos"
    else:
        os_name = system

    machine = platform.machine().lower()
    if machine in ("x86_64", "amd64"):
        arch = "x64"
    elif machine in ("aarch64", "arm64"):
        arch = "arm64"
    else:
        arch = machine

    return os_name, arch


def exe_suffix() -> str:
    return ".exe" if platform.system().lower() == "windows" else ""


def run(cmd: list[str], cwd: Path | None = None, env: dict | None = None) -> None:
    """Run a command, streaming output to the terminal."""
    print(f"  $ {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=str(cwd) if cwd else None, env=env, check=False)
    if result.returncode != 0:
        raise SystemExit(f"Command failed (exit {result.returncode}): {' '.join(cmd)}")


def build_rust_binaries(rebuild: bool) -> dict[str, Path]:
    """Build or reuse the release Rust binaries."""
    binaries = {"tool", "ariadne", "clang-worker"}
    result: dict[str, Path] = {}
    release_dir = REPO_ROOT / "target" / "release"

    if not rebuild:
        missing = []
        for name in binaries:
            candidate = release_dir / f"{name}{exe_suffix()}"
            if candidate.exists():
                result[name] = candidate
            else:
                missing.append(name)
        if not missing:
            print(f"  Reusing existing binaries in {release_dir}")
            return result
        print(f"  Missing: {', '.join(missing)} — will build.")

    cmd = ["cargo", "build", "--release"]
    for name in binaries:
        cmd.extend(["--bin", name])
    run(cmd, cwd=REPO_ROOT)

    for name in binaries:
        path = release_dir / f"{name}{exe_suffix()}"
        if not path.exists():
            raise SystemExit(f"Build did not produce expected binary: {path}")
        result[name] = path

    return result


def build_vsix(rebuild: bool) -> Path:
    """Build or reuse the VS Code extension .vsix package."""
    ext_dir = REPO_ROOT / "extensions" / "ariadne-extension"
    vsix_name = f"ariadne-extension-{VERSION}.vsix"
    vsix_path = ext_dir / vsix_name

    if not rebuild and vsix_path.exists():
        print(f"  Reusing existing {vsix_path}")
        return vsix_path

    npm = shutil.which("npm") or shutil.which("npm.cmd")
    if not npm:
        raise SystemExit(
            "npm not found on PATH. Install Node.js/npm to build the VS Code extension."
        )

    node_modules = ext_dir / "node_modules"
    if not node_modules.exists():
        print("  Running npm install...")
        run([npm, "install"], cwd=ext_dir)

    print("  Compiling TypeScript...")
    # npm scripts are invoked via npm so the project's PATH picks up local bins
    run([npm, "run", "compile"], cwd=ext_dir)

    vsce = shutil.which("vsce") or shutil.which("vsce.cmd")
    if not vsce:
        npx = shutil.which("npx") or shutil.which("npx.cmd")
        if not npx:
            raise SystemExit(
                "npx not found. Cannot package the VS Code extension without vsce."
            )
        print("  Packaging with @vscode/vsce via npx...")
        run([npx, "--yes", "@vscode/vsce", "package", "--allow-missing-repository"], cwd=ext_dir)
    else:
        print("  Packaging with vsce...")
        run([vsce, "package", "--allow-missing-repository"], cwd=ext_dir)

    if not vsix_path.exists():
        raise SystemExit(f"vsce did not produce expected file: {vsix_path}")

    return vsix_path


def create_package(binaries: dict[str, Path], vsix_path: Path) -> Path:
    """Stage all artifacts into the distributable package directory."""
    os_name, arch = detect_platform()
    pkg_root_name = f"{PACKAGE_NAME}-{VERSION}-{os_name}-{arch}"
    pkg_root = DIST_DIR / pkg_root_name
    if pkg_root.exists():
        shutil.rmtree(pkg_root)
    pkg_root.mkdir(parents=True)

    bin_dir = pkg_root / "bin"
    bin_dir.mkdir()

    print("  Copying binaries...")
    for name, src in binaries.items():
        dest = bin_dir / f"{name}{exe_suffix()}"
        shutil.copy2(src, dest)
        print(f"    {dest.name}")

    vsix_dir = pkg_root / "vsix"
    vsix_dir.mkdir()
    shutil.copy2(vsix_path, vsix_dir / vsix_path.name)
    print(f"    {vsix_path.name}")

    # Copy installer scripts and README from the offline_package directory
    installer_src = REPO_ROOT / "tools" / "offline_package"
    ext = ".ps1" if os_name == "win" else ".sh"
    installer_name = f"install{ext}"
    installer_src_file = installer_src / installer_name
    if installer_src_file.exists():
        shutil.copy2(installer_src_file, pkg_root / installer_name)
    readme_src = installer_src / "README.md"
    if readme_src.exists():
        shutil.copy2(readme_src, pkg_root / "README.md")

    return pkg_root


def create_archive(pkg_root: Path) -> Path:
    """Create a distributable archive from the package directory."""
    os_name, arch = detect_platform()
    pkg_root_name = pkg_root.name
    archive_base = DIST_DIR / pkg_root_name

    if os_name == "win":
        archive_path = pkg_root.with_suffix(".zip")
        if archive_path.exists():
            archive_path.unlink()
        with zipfile.ZipFile(archive_path, "w", zipfile.ZIP_DEFLATED) as zf:
            for file_path in pkg_root.rglob("*"):
                if file_path.is_file():
                    rel = file_path.relative_to(pkg_root.parent)
                    zf.write(file_path, rel)
        print(f"  Created {archive_path}")
    else:
        archive_path = pkg_root.with_suffix(".tar.gz")
        if archive_path.exists():
            archive_path.unlink()
        with tarfile.open(archive_path, "w:gz") as tf:
            tf.add(pkg_root, arcname=pkg_root_name)
        print(f"  Created {archive_path}")

    return archive_path


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Package Ariadne's Thread for offline installation."
    )
    parser.add_argument(
        "--rebuild-binaries",
        action="store_true",
        help="Force rebuilding the Rust release binaries even if they exist.",
    )
    parser.add_argument(
        "--rebuild-vsix",
        action="store_true",
        help="Force rebuilding the VS Code extension .vsix even if it exists.",
    )
    parser.add_argument(
        "--no-archive",
        action="store_true",
        help="Skip creating the distributable archive; just stage the package directory.",
    )
    args = parser.parse_args()

    os_name, arch = detect_platform()
    print(f"Platform: {os_name}-{arch}")
    print(f"Version:  {VERSION}")
    print()

    print("--- Building Rust release binaries ---")
    binaries = build_rust_binaries(args.rebuild_binaries)

    print("\n--- Packaging VS Code extension ---")
    vsix_path = build_vsix(args.rebuild_vsix)

    print("\n--- Staging package ---")
    pkg_root = create_package(binaries, vsix_path)

    if not args.no_archive:
        print("\n--- Creating distributable archive ---")
        archive_path = create_archive(pkg_root)
        print(f"\nDone! Offline package at:\n  {archive_path}")
        print(f"\nExtract the archive on the target machine and run:")
        ext = ".ps1" if os_name == "win" else ".sh"
        print(f"  ./install{ext}")
    else:
        print(f"\nDone! Package staged at:\n  {pkg_root}")

    print(f"\nBinaries included: {list(binaries.keys())}")
    print(f"Extension:         {vsix_path.name}")


if __name__ == "__main__":
    main()
