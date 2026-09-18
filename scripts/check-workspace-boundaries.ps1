[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path

Push-Location $repoRoot
try {
    $metadataJson = & cargo metadata --no-deps --format-version 1 --locked
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata failed with exit code $LASTEXITCODE"
    }

    $metadata = $metadataJson | ConvertFrom-Json
    $packagesById = @{}
    foreach ($package in $metadata.packages) {
        $packagesById[$package.id] = $package
    }

    $actualMembers = @(
        $metadata.workspace_members |
            ForEach-Object { $packagesById[$_].name } |
            Sort-Object
    )
    $expectedMembers = @(
        'ailive-gun-spirit-contracts',
        'ailive-gun-spirit-host',
        'ailive-gun-spirit-perception-core'
    ) | Sort-Object

    if (Compare-Object $expectedMembers $actualMembers) {
        throw "workspace members drifted. expected=[$($expectedMembers -join ', ')] actual=[$($actualMembers -join ', ')]"
    }

    $allowedDependencies = @{
        'ailive-gun-spirit-contracts' = @('schemars', 'serde', 'serde_json')
        'ailive-gun-spirit-perception-core' = @('ailive-gun-spirit-contracts')
        'ailive-gun-spirit-host' = @(
            'ailive-gun-spirit-contracts',
            'ailive-gun-spirit-perception-core',
            'oclive_kernel_host',
            'oclive_kernel_runtime',
            'oclive_kernel_types',
            'oclive_monolith_builtin',
            'serde',
            'serde_json',
            'tracing',
            'tracing-subscriber'
        )
    }

    foreach ($memberName in $expectedMembers) {
        $package = $metadata.packages | Where-Object name -eq $memberName
        $actualDependencies = @($package.dependencies.name | Sort-Object -Unique)
        $expectedDependencies = @($allowedDependencies[$memberName] | Sort-Object)
        if (Compare-Object $expectedDependencies $actualDependencies) {
            throw "dependency boundary drifted for $memberName. expected=[$($expectedDependencies -join ', ')] actual=[$($actualDependencies -join ', ')]"
        }

        if ($memberName -ne 'ailive-gun-spirit-host') {
            $forbidden = @($actualDependencies | Where-Object { $_ -like 'oclive_*' })
            if ($forbidden.Count -gt 0) {
                throw "$memberName must not depend on OCLive: $($forbidden -join ', ')"
            }
        }
    }

    Write-Output 'workspace-boundaries: PASS (3 members; dependency allowlists match)'
}
finally {
    Pop-Location
}
