# Contract-version parity guard.
#
# The wire contract version is written in two languages:
#   - Rust  : crates/ailive-gun-spirit-host/src/bench.rs (EXPECTED_CONTRACT_*)
#   - C++   : firmware/jc3248w535/Jc3248w535/gs_wire.cpp (kContractMajor/Minor)
# plus the kernel shim's WireState/WireEvent builders.
#
# They have drifted twice already (host 0.3 vs board 0.4), each time costing a
# debugging cycle ending in a silent NACK. This check fails the build when the
# numbers disagree.
#
# Usage: ./scripts/check-contract-parity.ps1
# Exit code: 0 = consistent, 1 = drift (or a pattern is missing), 2 = file missing.
#
# ASCII only: Windows PowerShell 5.1 decodes this file as ANSI.

[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path

function Read-Version {
    param(
        [string]$Path,
        [string]$MajorPattern,
        [string]$MinorPattern
    )
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        Write-Output "check-contract-parity: missing $Path"
        exit 2
    }
    $text = Get-Content -LiteralPath $Path -Raw -Encoding UTF8
    $major = [regex]::Match($text, $MajorPattern)
    $minor = [regex]::Match($text, $MinorPattern)
    if (-not $major.Success -or -not $minor.Success) {
        Write-Output ("check-contract-parity: cannot find version constants in {0}" -f (Split-Path -Leaf $Path))
        exit 1
    }
    return [pscustomobject]@{
        Major = [int]$major.Groups[1].Value
        Minor = [int]$minor.Groups[1].Value
    }
}

$hostSide = Read-Version -Path (Join-Path $repoRoot 'crates\ailive-gun-spirit-host\src\bench.rs') `
    -MajorPattern 'EXPECTED_CONTRACT_MAJOR:\s*u16\s*=\s*(\d+)' `
    -MinorPattern 'EXPECTED_CONTRACT_MINOR:\s*u16\s*=\s*(\d+)'

$boardSide = Read-Version -Path (Join-Path $repoRoot 'firmware\jc3248w535\Jc3248w535\gs_wire.cpp') `
    -MajorPattern 'kContractMajor\s*=\s*(\d+)' `
    -MinorPattern 'kContractMinor\s*=\s*(\d+)'

$shimSide = Read-Version -Path (Join-Path $repoRoot 'firmware\kernel-shim\src\lib.rs') `
    -MajorPattern 'contract_major:\s*(\d+),' `
    -MinorPattern 'contract_minor:\s*(\d+),'

Write-Output ("check-contract-parity: host {0}.{1} / board {2}.{3} / shim {4}.{5}" -f `
    $hostSide.Major, $hostSide.Minor, $boardSide.Major, $boardSide.Minor, $shimSide.Major, $shimSide.Minor)

if ($hostSide.Major -ne $boardSide.Major -or $hostSide.Minor -ne $boardSide.Minor) {
    Write-Output 'check-contract-parity: FAIL (host and board disagree)'
    exit 1
}
if ($hostSide.Major -ne $shimSide.Major -or $hostSide.Minor -ne $shimSide.Minor) {
    Write-Output 'check-contract-parity: FAIL (host and kernel shim disagree)'
    exit 1
}

Write-Output 'check-contract-parity: PASS'
exit 0

