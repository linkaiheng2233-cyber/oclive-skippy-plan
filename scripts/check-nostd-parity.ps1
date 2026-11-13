[CmdletBinding()]
param()

# Gate: the same simulator scenario must emit byte-identical JSON Lines when
# the shared crates are built with std and with no_std + alloc.
#
# The simulator always runs as a std host binary. `--no-default-features` only
# disables the host `std` feature, which in turn switches contracts and
# perception-core to their no_std + alloc configuration. The host crate itself
# is never compiled for no_std.
#
# This file must stay pure ASCII: Windows PowerShell 5.1 decodes .ps1 files as
# ANSI, so non-ASCII comments would be corrupted there.

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$scenarios = @('default', 'repeat', 'stale')
$configurations = @('std', 'no_std')
$cargo = (Get-Command cargo -ErrorAction Stop).Source

$workDir = Join-Path ([System.IO.Path]::GetTempPath()) (
    'ailive-gun-spirit-parity-' + [guid]::NewGuid().ToString('N')
)
New-Item -ItemType Directory -Path $workDir | Out-Null

function Invoke-Simulator {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Scenario,
        [Parameter(Mandatory = $true)]
        [string]$Configuration,
        [Parameter(Mandatory = $true)]
        [string]$OutputPath
    )

    $arguments = @(
        'run',
        '-q',
        '-p',
        'ailive-gun-spirit-host',
        '--bin',
        'ailive-gun-spirit-sim',
        '--locked'
    )
    if ($Configuration -eq 'no_std') {
        $arguments += '--no-default-features'
    }
    $arguments += @('--', '--scenario', $Scenario)

    $stderrPath = "$OutputPath.stderr"
    $process = Start-Process `
        -FilePath $cargo `
        -ArgumentList $arguments `
        -WorkingDirectory $repoRoot `
        -RedirectStandardOutput $OutputPath `
        -RedirectStandardError $stderrPath `
        -NoNewWindow `
        -Wait `
        -PassThru
    if ($process.ExitCode -ne 0) {
        $stderrText = ''
        if (Test-Path -LiteralPath $stderrPath) {
            $stderrText = Get-Content -LiteralPath $stderrPath -Raw
        }
        throw "simulator failed for scenario $Scenario ($Configuration) with exit code $($process.ExitCode): $stderrText"
    }
}

$exitCode = 0
try {
    foreach ($scenario in $scenarios) {
        $stdPath = Join-Path $workDir "$scenario.std.jsonl"
        $noStdPath = Join-Path $workDir "$scenario.no_std.jsonl"

        Write-Output "check-nostd-parity: scenario $scenario (std)"
        Invoke-Simulator -Scenario $scenario -Configuration 'std' -OutputPath $stdPath
        Write-Output "check-nostd-parity: scenario $scenario (no_std)"
        Invoke-Simulator -Scenario $scenario -Configuration 'no_std' -OutputPath $noStdPath

        $stdLength = (Get-Item -LiteralPath $stdPath).Length
        $noStdLength = (Get-Item -LiteralPath $noStdPath).Length
        if ($stdLength -eq 0 -or $noStdLength -eq 0) {
            Write-Output "check-nostd-parity: EMPTY output for scenario $scenario"
            Write-Output "  std:    $stdLength bytes $stdPath"
            Write-Output "  no_std: $noStdLength bytes $noStdPath"
            $exitCode = 1
            break
        }

        $stdHash = (Get-FileHash -LiteralPath $stdPath -Algorithm SHA256).Hash
        $noStdHash = (Get-FileHash -LiteralPath $noStdPath -Algorithm SHA256).Hash
        if ($stdLength -ne $noStdLength -or $stdHash -ne $noStdHash) {
            Write-Output "check-nostd-parity: MISMATCH for scenario $scenario"
            Write-Output "  std:    $stdHash ($stdLength bytes) $stdPath"
            Write-Output "  no_std: $noStdHash ($noStdLength bytes) $noStdPath"
            $differences = Compare-Object `
                -ReferenceObject (Get-Content -LiteralPath $stdPath) `
                -DifferenceObject (Get-Content -LiteralPath $noStdPath)
            $differences | Select-Object -First 5 | ForEach-Object {
                $preview = $_.InputObject
                if ($preview.Length -gt 160) {
                    $preview = $preview.Substring(0, 160) + '...'
                }
                Write-Output ("  " + $_.SideIndicator + " " + $preview)
            }
            $exitCode = 1
            break
        }

        $frameCount = @(Get-Content -LiteralPath $stdPath).Count
        Write-Output "check-nostd-parity: scenario $scenario identical ($frameCount frames, sha256 $stdHash)"
    }

    if ($exitCode -eq 0) {
        Write-Output "check-nostd-parity: PASS ($($scenarios.Count) scenarios x $($configurations.Count) feature configurations)"
    }
}
catch {
    Write-Output "check-nostd-parity: ERROR $($_.Exception.Message)"
    $exitCode = 1
}
finally {
    if ($exitCode -eq 0) {
        Remove-Item -LiteralPath $workDir -Recurse -Force -ErrorAction SilentlyContinue
    }
    else {
        Write-Output "check-nostd-parity: artifacts kept in $workDir"
    }
}
exit $exitCode
