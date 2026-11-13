# Kernel heap probe: build a throwaway crate OUTSIDE the workspace and measure
# the perception kernel's peak heap use. The probe needs a counting global
# allocator, which requires `unsafe impl GlobalAlloc`; the repo forbids unsafe,
# so the probe source lives here and is only compiled in a temporary directory.
#
# Usage: ./scripts/heap-probe.ps1
# ASCII only: Windows PowerShell 5.1 decodes this file as ANSI.

[CmdletBinding()]
param(
    [string]$ProbeRoot = (Join-Path ([System.IO.Path]::GetTempPath()) 'gs-heap-probe')
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$repoForToml = $repoRoot -replace '\\', '/'

if (-not (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'heap-probe\main.rs') -PathType Leaf)) {
    throw 'Missing scripts/heap-probe/main.rs'
}

New-Item -ItemType Directory -Force -Path (Join-Path $ProbeRoot 'src') | Out-Null
$manifest = @"
[package]
name = "gs-heap-probe"
version = "0.0.0"
edition = "2021"

[dependencies]
ailive-gun-spirit-contracts = { path = "$repoForToml/crates/ailive-gun-spirit-contracts" }
ailive-gun-spirit-perception-core = { path = "$repoForToml/crates/ailive-gun-spirit-perception-core" }

[workspace]
"@
Set-Content -LiteralPath (Join-Path $ProbeRoot 'Cargo.toml') -Value $manifest -Encoding utf8NoBOM
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'heap-probe\main.rs') -Destination (Join-Path $ProbeRoot 'src\main.rs') -Force

Push-Location $ProbeRoot
try {
    $output = & cargo run --release 2>&1
    $code = $LASTEXITCODE
    $output | Where-Object { $_ -match 'KERNEL_HEAP|^error' } | ForEach-Object { $_.ToString().Trim() }
    if ($code -ne 0) {
        $output | Select-Object -Last 20
    }
    exit $code
}
finally {
    Pop-Location
}
