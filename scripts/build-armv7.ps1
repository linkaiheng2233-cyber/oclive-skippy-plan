[CmdletBinding()]
param(
    [string]$ToolchainBin = $env:OCLIVE_ARMV7_TOOLCHAIN_BIN,
    [string]$TargetDir = (Join-Path ([System.IO.Path]::GetTempPath()) "oclive-armv7-target"),
    [string]$ReportPath = ""
)

$ErrorActionPreference = "Stop"
$targetTriple = "armv7-unknown-linux-gnueabihf"
$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path

function Resolve-Tool([string]$name) {
    if ($ToolchainBin) {
        $candidate = Join-Path $ToolchainBin ($name + ".exe")
        if (Test-Path -LiteralPath $candidate) {
            return (Resolve-Path -LiteralPath $candidate).Path
        }
    }

    $command = Get-Command $name -ErrorAction SilentlyContinue
    if ($command) {
        return $command.Source
    }

    throw "Missing $name. Set OCLIVE_ARMV7_TOOLCHAIN_BIN or pass -ToolchainBin."
}

if ($TargetDir -match "[^\x00-\x7F]") {
    throw "CARGO_TARGET_DIR must be an ASCII-only path on Windows because GNU ld cannot resolve the repository's Chinese path."
}

$gcc = Resolve-Tool "arm-none-linux-gnueabihf-gcc"
$ar = Resolve-Tool "arm-none-linux-gnueabihf-ar"
$readelf = Resolve-Tool "arm-none-linux-gnueabihf-readelf"
$sizeTool = Resolve-Tool "arm-none-linux-gnueabihf-size"

$installedTargets = rustup target list --installed
if ($LASTEXITCODE -ne 0) {
    throw "rustup target list failed."
}
if ($installedTargets -notcontains $targetTriple) {
    rustup target add $targetTriple
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to install Rust target $targetTriple."
    }
}

$env:CC_armv7_unknown_linux_gnueabihf = $gcc
$env:AR_armv7_unknown_linux_gnueabihf = $ar
$env:CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER = $gcc
$env:CARGO_TARGET_DIR = $TargetDir

$startedAt = Get-Date
Push-Location $repoRoot
try {
    cargo build -p ailive-gun-spirit-host --release --locked --target $targetTriple
    if ($LASTEXITCODE -ne 0) {
        throw "ARMv7 release build failed."
    }
}
finally {
    Pop-Location
}
$elapsed = (Get-Date) - $startedAt

$artifact = Join-Path $TargetDir "$targetTriple\release\ailive-gun-spirit-host"
if (-not (Test-Path -LiteralPath $artifact)) {
    throw "Build succeeded but artifact was not found at $artifact."
}

$header = (& $readelf -h $artifact) -join "`n"
$dynamic = (& $readelf -d $artifact | Select-String "NEEDED" | ForEach-Object { $_.Line.Trim() })
$sectionSizes = (& $sizeTool $artifact) -join "`n"
$artifactFile = Get-Item -LiteralPath $artifact

$result = [ordered]@{
    schema_version = 1
    measured_at = (Get-Date).ToUniversalTime().ToString("o")
    target = $targetTriple
    toolchain = (& $gcc --version | Select-Object -First 1)
    elapsed_seconds = [math]::Round($elapsed.TotalSeconds, 1)
    artifact = $artifactFile.FullName
    size_bytes = $artifactFile.Length
    size_mib = [math]::Round($artifactFile.Length / 1MB, 2)
    sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $artifact).Hash.ToLowerInvariant()
    elf32 = $header.Contains("Class:                             ELF32")
    machine_arm = $header.Contains("Machine:                           ARM")
    hard_float_eabi = $header.Contains("hard-float ABI")
    needed = @($dynamic)
    size_output = $sectionSizes
}

$json = $result | ConvertTo-Json -Depth 4
$json

if ($ReportPath) {
    $resolvedReportPath = if ([System.IO.Path]::IsPathRooted($ReportPath)) {
        $ReportPath
    } else {
        Join-Path $repoRoot $ReportPath
    }
    $reportParent = Split-Path -Parent $resolvedReportPath
    if ($reportParent) {
        New-Item -ItemType Directory -Force -Path $reportParent | Out-Null
    }
    [System.IO.File]::WriteAllText($resolvedReportPath, $json + "`n", [System.Text.UTF8Encoding]::new($false))
}

if (-not ($result.elf32 -and $result.machine_arm -and $result.hard_float_eabi)) {
    throw "Artifact exists but its ELF architecture/ABI does not match ARMv7 hard-float Linux."
}
