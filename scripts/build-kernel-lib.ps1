# Build the Rust perception kernel as a static library for ESP32-S3, then stage it
# where build.ps1 can link it.
#
# Why a separate script: the kernel crate lives outside the Arduino world and needs
# the Xtensa Rust fork (`espup`) plus `-Zbuild-std`. Keeping this out of build.ps1
# means ordinary firmware builds stay fast and do not depend on the Rust toolchain.
#
# Usage:
#   ./scripts/build-kernel-lib.ps1
#   ./scripts/build-kernel-lib.ps1 -Force
#
# Output: %LOCALAPPDATA%\OCLive\jc3248w535\kernel\libgs_kernel_shim.a
# ASCII only: Windows PowerShell 5.1 decodes this file as ANSI.

[CmdletBinding()]
param(
    [string]$ToolRoot = (Join-Path $env:LOCALAPPDATA 'OCLive\jc3248w535'),
    [string]$ExportScript = (Join-Path $env:USERPROFILE 'export-esp.ps1'),
    [switch]$Force
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$shimDir = Join-Path $repoRoot 'firmware\kernel-shim'
$stageDir = Join-Path $ToolRoot 'kernel'
$stagedLibrary = Join-Path $stageDir 'libgs_kernel_shim.a'

if (-not (Test-Path -LiteralPath $ExportScript -PathType Leaf)) {
    throw "Missing $ExportScript. Run 'espup install --targets esp32s3' first."
}
if (-not (Test-Path -LiteralPath $shimDir -PathType Container)) {
    throw "Missing kernel shim at $shimDir"
}

. $ExportScript

$env:CARGO_TARGET_DIR = Join-Path $ToolRoot 'kernel-build'
Push-Location $shimDir
try {
    $arguments = @(
        '+esp', 'build',
        '-Zbuild-std=core,alloc',
        '--release',
        '--target', 'xtensa-esp32s3-none-elf'
    )
    if ($Force) { $arguments += '--offline' }
    $output = & cargo @arguments 2>&1
    $code = $LASTEXITCODE
    $output | Where-Object { $_ -match 'error|warning: unused|Compiling gs-kernel-shim|Finished' } |
        ForEach-Object { $_.ToString().Trim() }
    if ($code -ne 0) {
        $output | Select-Object -Last 20
        throw "kernel static library build failed with exit code $code"
    }
}
finally {
    Pop-Location
}

$built = Join-Path $env:CARGO_TARGET_DIR 'xtensa-esp32s3-none-elf\release\libgs_kernel_shim.a'
if (-not (Test-Path -LiteralPath $built -PathType Leaf)) {
    throw "Expected static library at $built"
}
New-Item -ItemType Directory -Force -Path $stageDir | Out-Null
Copy-Item -LiteralPath $built -Destination $stagedLibrary -Force
$size = (Get-Item -LiteralPath $stagedLibrary).Length
Write-Output ("kernel-lib: staged {0} ({1:N0} B)" -f $stagedLibrary, $size)
