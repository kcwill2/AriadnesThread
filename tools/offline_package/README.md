# Ariadne's Thread — Offline Installer

This package contains everything needed to install Ariadne's Thread (AT) and its
VS Code extension on a machine without internet access.

## Contents

```
ariadne-offline-0.1.0-<platform>/
  bin/
    tool          The AT CLI + web server binary
    clang-worker  Clang-backed extraction sidecar (optional accelerator)
    ariadne       Alternative CLI entry point (same as 'tool')
  vsix/
    ariadne-extension-0.1.0.vsix   VS Code extension package
  install.ps1   Windows installer
  install.sh    Linux/macOS installer
```

## Quick Start

### Windows

1. Extract the zip archive.
2. Open PowerShell and run from the extracted directory:

```powershell
.\install.ps1
```

### Linux / macOS

1. Extract the archive:

```bash
tar xzf ariadne-offline-0.1.0-linux-x64.tar.gz
cd ariadne-offline-0.1.0-linux-x64
```

2. Run the installer:

```bash
./install.sh
```

## What the Installer Does

- Copies binaries to `~/.local/bin` (Unix) or `%LOCALAPPDATA%\Programs\AriadnesThread\bin` (Windows).
- Optionally installs the VS Code extension from the bundled `.vsix`.
- Optionally adds the bin directory to your PATH.

## Usage After Installation

```bash
# Start the web UI for a project
tool ui /path/to/project

# Or run the full analysis pipeline
tool optimize /path/to/project
```

The UI is served at `http://localhost:7742` by default.

## VS Code Integration

After installing both the binaries and the extension:

1. Set `ariadne.toolPath` in VS Code settings to the installed `tool` binary path
   (e.g., `~/.local/bin/tool` on Linux or `C:\Users\me\AppData\Local\Programs\AriadnesThread\bin\tool.exe` on Windows).
2. Open a C++ project in VS Code.
3. Run `Ariadne: Start Ariadne Server` from the Command Palette.

## Offline Build

This package was created with `tools/package_offline.py`. To rebuild:

```bash
# With internet (to fetch crates, npm packages, etc.)
python tools/package_offline.py --rebuild-binaries --rebuild-vsix

# Without internet (reuse cached dependencies)
python tools/package_offline.py
```

## License

MIT. See the [LICENSE](https://github.com/kcwill2/AriadnesThread) file in the
source repository.
