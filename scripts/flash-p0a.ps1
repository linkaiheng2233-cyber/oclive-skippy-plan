# One-shot build + flash for the P0-A standard configuration.
#
# The standard P0-A firmware is:
#   -BlobAssets -WithKernel -WithWifi
# i.e. portraits live in the ffat asset region, the Rust perception kernel is
# linked in, and the Wi-Fi link is enabled. Build size is ~1.22 MB (58% of the
# 2 MiB app1 partition).
#
# Usage:
#   ./scripts/flash-p0a.ps1                 # build + flash app (assets already in ffat)
#   ./scripts/flash-p0a.ps1 -WithAssets     # also (re)write the asset blob
#   ./scripts/flash-p0a.ps1 -SkipBuild      # flash the existing .ino.bin
#   ./scripts/flash-p0a.ps1 -Factory        # switch back to the factory app0
#
# The board must be in serial download mode (hold BOOT, tap RESET) for any
# write operation; that is a physical action only the owner can perform.
#
# ASCII only: Windows PowerShell 5.1 decodes this file as ANSI.

[CmdletBinding()]
param(
    [string]$Port = 'COM3',
    [string]$ToolRoot = (Join-Path $env:LOCALAPPDATA 'OCLive\jc3248w535'),
    [string]$Backup = '',
    [switch]$WithAssets,
    [switch]$SkipBuild,
    [switch]$Factory
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$python = Join-Path $ToolRoot 'assets-env\Scripts\python.exe'
if (-not (Test-Path -LiteralPath $python -PathType Leaf)) {
    throw "Missing python at $python"
}
if (-not $Backup) {
    $artifactRoot = Join-Path (Split-Path -Parent $repoRoot) 'oclive-四季宝-artifacts'
    $Backup = Join-Path $artifactRoot 'hardware\jc3248w535\2026-11-13\factory-backup\factory-flash-16MiB.bin'
}
if (-not (Test-Path -LiteralPath $Backup -PathType Leaf)) {
    throw "Missing factory backup at $Backup"
}

$output = Join-Path $ToolRoot 'output'
$image = Join-Path $output 'Jc3248w535.ino.bin'
$assets = Join-Path $output 'assets.bin'

if (-not $SkipBuild) {
    $kernelLibrary = Join-Path $ToolRoot 'kernel\libgs_kernel_shim.a'
    if (-not (Test-Path -LiteralPath $kernelLibrary -PathType Leaf)) {
        Write-Output 'flash-p0a: kernel static library missing, building it...'
        & (Join-Path $PSScriptRoot 'build-kernel-lib.ps1')
        if ($LASTEXITCODE -ne 0) { throw "kernel library build failed with $LASTEXITCODE" }
    }
    Write-Output 'flash-p0a: building standard firmware (BlobAssets + Kernel + Wifi)...'
    & (Join-Path $repoRoot 'firmware\jc3248w535\build.ps1') -BlobAssets -WithKernel -WithWifi
    if ($LASTEXITCODE -ne 0) { throw "firmware build failed with $LASTEXITCODE" }
}

if (-not (Test-Path -LiteralPath $image -PathType Leaf)) {
    throw "Missing $image"
}
Write-Output ("flash-p0a: image {0:N0} B" -f (Get-Item -LiteralPath $image).Length)

if ($Factory) {
    & $python (Join-Path $repoRoot 'firmware\jc3248w535\flash-prototype.py') --port $Port --backup $Backup --factory
    if ($LASTEXITCODE -ne 0) { throw "factory switch failed with $LASTEXITCODE" }
    return
}

# 只读校验：确认这块板与备份一致，且仍在下载模式。
& $python (Join-Path $repoRoot 'firmware\jc3248w535\flash-prototype.py') --port $Port --backup $Backup --check
if ($LASTEXITCODE -ne 0) { throw "board check failed with $LASTEXITCODE" }

if ($WithAssets) {
    if (-not (Test-Path -LiteralPath $assets -PathType Leaf)) {
        throw "Missing $assets; build without -SkipBuild first."
    }
    Write-Output 'flash-p0a: writing asset blob to ffat...'
    & $python (Join-Path $repoRoot 'firmware\jc3248w535\flash-prototype.py') --port $Port --backup $Backup --assets-force $assets
    if ($LASTEXITCODE -ne 0) { throw "asset write failed with $LASTEXITCODE" }
}

Write-Output 'flash-p0a: writing application to app1...'
& $python (Join-Path $repoRoot 'firmware\jc3248w535\flash-prototype.py') --port $Port --backup $Backup --image $image
if ($LASTEXITCODE -ne 0) { throw "image write failed with $LASTEXITCODE" }
Write-Output 'flash-p0a: done'
