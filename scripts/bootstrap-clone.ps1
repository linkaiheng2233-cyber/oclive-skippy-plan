# One-command bootstrap: clone BOTH repositories into the expected sibling layout.
#
# Why both: this repository's Rust workspace depends on the sibling OCLive repo
#   ../oclivenewnew/kernel/crates/oclive_kernel_{host,runtime,types}
# Cloning only this repo leaves those path dependencies unresolved, so
# `cargo build` / `./scripts/verify.ps1` fail immediately.
#
# Layout produced (default root E:\OCLive):
#   <Root>\oclive-四季宝器灵    branch docs/personal-document-archive-20261030
#   <Root>\oclivenewnew         branch fix/chat-pro-rc9-feedback-20261106
#
# Usage:
#   ./scripts/bootstrap-clone.ps1
#   ./scripts/bootstrap-clone.ps1 -Root D:\OCLive -SkipVerify
#
# Requires: git, network access to GitHub, PowerShell 7.
# ASCII only: Windows PowerShell 5.1 decodes this file as ANSI.

[CmdletBinding()]
param(
    [string]$Root = 'E:\OCLive',
    [string]$SpiritRemote = 'https://github.com/linkaiheng2233-cyber/oclive-skippy-plan.git',
    [string]$SpiritBranch = 'docs/personal-document-archive-20261030',
    [string]$OcliveRemote = 'https://github.com/linkaiheng2233-cyber/oclivenewnew.git',
    [string]$OcliveBranch = 'fix/chat-pro-rc9-feedback-20261106',
    [switch]$SkipVerify
)

$ErrorActionPreference = 'Stop'
$env:GIT_TERMINAL_PROMPT = '0'

$spiritPath = Join-Path $Root 'oclive-四季宝器灵'
$oclivePath = Join-Path $Root 'oclivenewnew'

New-Item -ItemType Directory -Force -Path $Root | Out-Null

function Invoke-Clone {
    param([string]$Url, [string]$Branch, [string]$Target)
    if (Test-Path -LiteralPath (Join-Path $Target '.git')) {
        Write-Output "already cloned: $Target"
        return
    }
    Write-Output "cloning $Url ($Branch) -> $Target"
    git clone --branch $Branch $Url $Target
    if ($LASTEXITCODE -ne 0) {
        throw "clone failed for $Url (exit $LASTEXITCODE)"
    }
}

Invoke-Clone -Url $SpiritRemote -Branch $SpiritBranch -Target $spiritPath
Invoke-Clone -Url $OcliveRemote -Branch $OcliveBranch -Target $oclivePath

# Verify the sibling dependency crates are actually present: without them the
# workspace cannot resolve its path dependencies.
$required = @(
    'kernel\crates\oclive_kernel_host',
    'kernel\crates\oclive_kernel_runtime',
    'kernel\crates\oclive_kernel_types'
)
foreach ($relative in $required) {
    $full = Join-Path $oclivePath $relative
    if (-not (Test-Path -LiteralPath $full -PathType Container)) {
        throw "sibling dependency missing: $full (wrong branch?)"
    }
}

Write-Output 'bootstrap-clone: both repositories present'
Push-Location $spiritPath
try {
    Write-Output ("spirit HEAD: " + (git rev-parse --short HEAD) + " (" + (git rev-parse --abbrev-ref HEAD) + ")")
    Push-Location $oclivePath
    try {
        Write-Output ("oclive HEAD: " + (git rev-parse --short HEAD) + " (" + (git rev-parse --abbrev-ref HEAD) + ")")
    }
    finally {
        Pop-Location
    }
    if (-not $SkipVerify) {
        Write-Output 'bootstrap-clone: running cargo check (first build may take a few minutes)...'
        cargo check -p ailive-gun-spirit-host --locked
        if ($LASTEXITCODE -ne 0) {
            throw "cargo check failed with $LASTEXITCODE"
        }
        Write-Output 'bootstrap-clone: running ./scripts/verify.ps1 -SkipAudit'
        & (Join-Path $spiritPath 'scripts\verify.ps1') -SkipAudit
        if ($LASTEXITCODE -ne 0) {
            throw "verify.ps1 failed with $LASTEXITCODE"
        }
    }
    Write-Output 'bootstrap-clone: OK'
}
finally {
    Pop-Location
}
