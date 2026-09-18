[CmdletBinding()]
param(
    [switch]$SkipAudit
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Label,
        [Parameter(Mandatory = $true)]
        [string]$Command,
        [Parameter(Mandatory = $true)]
        [string[]]$Arguments
    )

    Write-Output "verify: $Label"
    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }
}

Push-Location $repoRoot
try {
    $manifests = @(
        'crates/ailive-gun-spirit-contracts/Cargo.toml',
        'crates/ailive-gun-spirit-perception-core/Cargo.toml',
        'crates/ailive-gun-spirit-host/Cargo.toml'
    )
    foreach ($manifest in $manifests) {
        Invoke-Checked "format $manifest" cargo @('fmt', '--manifest-path', $manifest, '--', '--check')
    }

    & (Join-Path $PSScriptRoot 'check-workspace-boundaries.ps1')
    if ($LASTEXITCODE -ne 0) {
        throw "workspace boundary check failed with exit code $LASTEXITCODE"
    }

    Invoke-Checked 'clippy' cargo @('clippy', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings')
    Invoke-Checked 'all targets' cargo @('test', '--workspace', '--all-targets', '--locked')
    Invoke-Checked 'doctests' cargo @('test', '--workspace', '--doc', '--locked')
    Invoke-Checked 'schema drift' cargo @(
        'run',
        '-p',
        'ailive-gun-spirit-contracts',
        '--example',
        'generate_schemas',
        '--locked',
        '--',
        '--check'
    )

    if (-not $SkipAudit) {
        Invoke-Checked 'cargo audit' cargo @('audit')
    }

    Write-Output 'verify: PASS'
}
finally {
    Pop-Location
}
