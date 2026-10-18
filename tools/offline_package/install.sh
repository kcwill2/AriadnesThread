#!/usr/bin/env bash
#
# Offline installer for Ariadne's Thread (AT).
#
# Copies the Rust binaries (tool, clang-worker, ariadne) to the install
# directory, optionally installs the VS Code extension from the bundled .vsix,
# and optionally adds the bin directory to PATH.
#
# Run this script from the package directory after extracting the archive.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

INSTALL_DIR="${INSTALL_DIR:-${HOME}/.local}"
BIN_DIR="${INSTALL_DIR}/bin"
SKIP_VSCODE=false
SKIP_PATH=false
FORCE=false

usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Offline installer for Ariadne's Thread.

Options:
  --prefix DIR   Install to DIR/bin (default: ~/.local)
  --skip-vscode  Skip VS Code extension installation
  --skip-path    Skip adding bin to PATH
  -f, --force    Overwrite existing files without prompting
  -h, --help     Show this help message
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --prefix)  INSTALL_DIR="$2"; BIN_DIR="${INSTALL_DIR}/bin"; shift 2 ;;
        --skip-vscode) SKIP_VSCODE=true; shift ;;
        --skip-path) SKIP_PATH=true; shift ;;
        -f|--force) FORCE=true; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage; exit 1 ;;
    esac
done

confirm() {
    if [ "$FORCE" = "true" ]; then return 0; fi
    local prompt="$1"
    read -r -p "$prompt [Y/n] " answer
    [[ "$answer" = "" || "$answer" = "y" || "$answer" = "Y" ]]
}

echo "=================================="
echo " Ariadne's Thread Offline Installer"
echo "=================================="
echo ""
echo "Binaries will be installed to: $BIN_DIR"
echo ""

# --- Copy binaries ---
mkdir -p "$BIN_DIR"
PKG_BIN_DIR="${SCRIPT_DIR}/bin"
for binary in tool clang-worker ariadne; do
    src="${PKG_BIN_DIR}/${binary}"
    if [ ! -f "$src" ]; then
        echo "  Warning: binary not found: $src" >&2
        continue
    fi
    dst="${BIN_DIR}/${binary}"
    if [ -f "$dst" ] && [ "$FORCE" != "true" ]; then
        echo "  Warning: $dst already exists, overwriting"
    fi
    cp "$src" "$dst"
    chmod +x "$dst"
    echo "  Installed $binary"
done

# --- Install VS Code extension ---
if [ "$SKIP_VSCODE" = "false" ]; then
    VSIX_DIR="${SCRIPT_DIR}/vsix"
    VSIX_FILE=""
    if [ -d "$VSIX_DIR" ]; then
        VSIX_FILE="$(find "$VSIX_DIR" -name '*.vsix' -type f | head -1)"
    fi
    if [ -n "$VSIX_FILE" ]; then
        if command -v code >/dev/null 2>&1; then
            if confirm "Install VS Code extension from $(basename "$VSIX_FILE")?"; then
                echo "==> Installing VS Code extension"
                code --install-extension "$VSIX_FILE" --force
            fi
        else
            echo "  Warning: 'code' CLI not found." >&2
            echo "  Install the extension manually in VS Code:" >&2
            echo "    1. Open VS Code" >&2
            echo "    2. Ctrl+Shift+P -> 'Extensions: Install from VSIX...'" >&2
            echo "    3. Select: $VSIX_FILE" >&2
        fi
    else
        echo "  Warning: No .vsix found in $VSIX_DIR" >&2
    fi
fi

# --- Add to PATH ---
if [ "$SKIP_PATH" = "false" ]; then
    case ":$PATH:" in
        *":$BIN_DIR:"*)
            echo "  Already in PATH."
            ;;
        *)
            if [ "$SHELL" = *"zsh" ] || [ "$SHELL" = *"bash" ]; then
                PROFILE_RC="$HOME/.profile"
                case "$SHELL" in
                    *zsh) PROFILE_RC="$HOME/.zshrc" ;;
                    *bash) PROFILE_RC="$HOME/.bashrc" ;;
                esac
                if confirm "Add $BIN_DIR to PATH in $PROFILE_RC?"; then
                    echo "" >> "$PROFILE_RC"
                    echo "# Ariadne's Thread" >> "$PROFILE_RC"
                    echo "export PATH=\"\$PATH:$BIN_DIR\"" >> "$PROFILE_RC"
                    echo "  Added to $PROFILE_RC (restart your shell to apply)."
                fi
            else
                echo "  Shell not zsh/bash; add $BIN_DIR to your PATH manually."
            fi
            ;;
    esac
fi

echo ""
echo "=================================="
echo " Installation complete!"
echo ""
echo "Binaries installed to: $BIN_DIR"
echo "Run 'tool --help' to verify."
echo ""
echo "For VS Code integration, set ariadne.toolPath to:"
echo "  $BIN_DIR/tool"
echo "Or use the Command Palette: 'Ariadne: Start Ariadne Server'"
