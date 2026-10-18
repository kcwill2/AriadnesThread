<#
.SYNOPSIS
    Installs Ariadne's Thread (AT) offline — no internet access required.

.DESCRIPTION
    1. Copies the Rust binaries (tool, clang-worker, ariadne) to the
       install directory (default: %LOCALAPPDATA%\Programs\AriadnesThread\bin).
    2. Optionally installs the VS Code extension from the bundled .vsix.
    3. Optionally adds the bin directory to the user PATH.

    Run this script from the package directory after extracting the archive.

.PARAMETER InstallDir
    Override the installation directory.

.PARAMETER SkipVsCode
    Skip VS Code extension installation.

.PARAMETER SkipPath
    Skip adding the install directory to PATH.

.PARAMETER Force
    Overwrite existing installation without prompting.

.EXAMPLE
    .\install.ps1
    .\install.ps1 -InstallDir "C:\Tools\ariadne"
    .\install.ps1 -SkipVsCode -SkipPath
#>
[CmdletBinding()]
param(
    [string]$InstallDir = "",
    [switch]$SkipVsCode,
    [switch]$SkipPath,
    [switch]$Force
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ExeSuffix = ".exe"
$BinDir = if ($InstallDir) { $InstallDir } else {
    Join-Path $env:LOCALAPPDATA "Programs\AriadnesThread"
}
$BinDir = Join-Path $BinDir "bin"

function Write-Step([string]$msg) {
    Write-Host "==> $msg" -ForegroundColor Cyan
}

function Confirm-Step([string]$msg) {
    if ($Force) { return $true }
    $answer = Read-Host "$msg [Y/n]"
    return ($answer -eq "" -or $answer -eq "y" -or $answer -eq "Y")
}

# --- Determine install directory ---
if (-not $InstallDir) {
    Write-Host "Ariadne's Thread Offline Installer"
    Write-Host "=================================="
    Write-Host ""
    Write-Host "This will install the AT binaries to:"
    Write-Host "  $BinDir"
    Write-Host ""
}

# --- Copy binaries ---
Write-Step "Copying binaries to $BinDir"
New-Item -ItemType Directory -Path $BinDir -Force | Out-Null

$pkgBinDir = Join-Path $ScriptDir "bin"
foreach ($binary in @("tool", "clang-worker", "ariadne")) {
    $src = Join-Path $pkgBinDir ($binary + $ExeSuffix)
    $dst = Join-Path $BinDir ($binary + $ExeSuffix)
    if (-Not (Test-Path -LiteralPath $src)) {
        Write-Warning "Binary not found: $src"
        continue
    }
    Copy-Item -LiteralPath $src -Destination $dst -Force
    Write-Host "  Installed $binary$ExeSuffix"
}

# --- Install VS Code extension ---
if (-not $SkipVsCode) {
    $vsixDir = Join-Path $ScriptDir "vsix"
    $vsixFile = Get-ChildItem -LiteralPath $vsixDir -Filter "*.vsix" -ErrorAction SilentlyContinue |
        Select-Object -First 1

    if ($vsixFile) {
        $codeCmd = Get-Command "code" -ErrorAction SilentlyContinue
        if ($codeCmd) {
            if (Confirm-Step "Install VS Code extension from $($vsixFile.Name)?") {
                Write-Step "Installing VS Code extension"
                & code --install-extension $vsixFile.FullName --force
                if ($LASTEXITCODE -eq 0) {
                    Write-Host "  Extension installed successfully."
                } else {
                    Write-Warning "Extension install failed (code CLI exit $LASTEXITCODE)."
                }
            }
        } else {
            Write-Warning "'code' CLI not found. Install the extension manually in VS Code:"
            Write-Host "  1. Open VS Code"
            Write-Host "  2. Ctrl+Shift+P → 'Extensions: Install from VSIX...'"
            Write-Host "  3. Select: $($vsixFile.FullName)"
        }
    } else {
        Write-Warning "No .vsix file found in $vsixDir"
    }
}

# --- Add to PATH ---
if (-not $skipPath) {
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (-not $currentPath.Contains($BinDir)) {
        if (Confirm-Step "Add $BinDir to user PATH?") {
            [Environment]::SetEnvironmentVariable(
                "Path",
                "$currentPath;$BinDir",
                "User"
            )
            Write-Host "  Added to user PATH. Restart terminal/shell to apply."
        }
    } else {
        Write-Host "  Already in PATH."
    }
}

Write-Host ""
Write-Host "=================================="
Write-Host "Installation complete!"
Write-Host ""
Write-Host "Binaries installed to: $BinDir"
Write-Host "Run 'tool --help' to verify."
Write-Host ""
Write-Host "For VS Code integration, configure ariadne.toolPath to:"
Write-Host "  $BinDir\tool.exe"
Write-Host "Or use the Command Palette: 'Ariadne: Start Ariadne Server'"
