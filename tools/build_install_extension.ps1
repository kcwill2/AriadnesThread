#Requires -Version 5.1
<#
.SYNOPSIS
    Builds and installs the Ariadne VS Code extension.

.DESCRIPTION
    1. Installs npm dependencies (if missing)
    2. Compiles TypeScript (dist/extension.js)
    3. Packages a .vsix with @vscode/vsce
    4. Installs the .vsix via the VS Code CLI

.PARAMETER SkipNpmInstall
    Skip "npm install" (use the existing node_modules).

.PARAMETER SkipInstall
    Only build and package; do not install into VS Code.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File tools\build_install_extension.ps1
#>
[CmdletBinding()]
param(
    [switch]$SkipNpmInstall,
    [switch]$SkipInstall
)

$ErrorActionPreference = "Stop"

function Resolve-Tool {
    param([string]$Name, [string]$Fallback)
    $cmd = Get-Command $Name -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    if ($Fallback -and (Test-Path -LiteralPath $Fallback)) { return $Fallback }
    throw "'$Name' was not found on PATH. Install it or add it to PATH."
}

$extDir = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..\extensions\ariadne-extension")).Path
Write-Host "Extension dir : $extDir"

foreach ($tool in @("npm", "npx")) {
    $null = Resolve-Tool -Name $tool
}

Push-Location -LiteralPath $extDir
try {
    $pkg = Get-Content -LiteralPath "package.json" -Raw | ConvertFrom-Json
    $vsixName = "{0}-{1}.vsix" -f $pkg.name, $pkg.version

    if (-not $SkipNpmInstall -or -not (Test-Path -LiteralPath "node_modules")) {
        Write-Host "==> npm install"
        npm install
        if ($LASTEXITCODE -ne 0) { throw "npm install failed (exit $LASTEXITCODE)" }
    }

    Write-Host "==> npm run compile"
    npm run compile
    if ($LASTEXITCODE -ne 0) { throw "compile failed (exit $LASTEXITCODE)" }

    Write-Host "==> vsce package"
    if (Test-Path -LiteralPath $vsixName) {
        Remove-Item -LiteralPath $vsixName -Force
    }
    npx --yes @vscode/vsce package --allow-missing-repository
    if ($LASTEXITCODE -ne 0) { throw "vsce package failed (exit $LASTEXITCODE)" }
    if (-not (Test-Path -LiteralPath $vsixName)) {
        throw "expected vsix not found: $vsixName"
    }
    $vsixPath = (Resolve-Path -LiteralPath $vsixName).Path
    Write-Host "Built : $vsixPath"

    if ($SkipInstall) {
        Write-Host "==> Skipping install (-SkipInstall)"
        return
    }

    $code = $null
    $cmd = Get-Command "code" -ErrorAction SilentlyContinue
    if ($cmd) {
        $code = $cmd.Source
    } else {
        $fallback = Join-Path $env:LOCALAPPDATA "Programs\Microsoft VS Code\bin\code.cmd"
        if (Test-Path -LiteralPath $fallback) { $code = $fallback }
    }
    if (-not $code) {
        throw "VS Code CLI ('code') not found. Open the Command Palette and run 'Shell Command: Install 'code' command in PATH', then re-run."
    }

    Write-Host "==> code --install-extension $vsixName"
    & $code --install-extension $vsixPath --force
    if ($LASTEXITCODE -ne 0) { throw "install failed (exit $LASTEXITCODE)" }

    Write-Host "==> Installed '$($pkg.displayName)' v$($pkg.version). Reload VS Code windows (Ctrl+Shift+P -> 'Developer: Reload Window') to activate."
}
finally {
    Pop-Location
}
