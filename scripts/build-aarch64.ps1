[CmdletBinding()]
param(
    [string]$ToolchainBin = $env:OCLIVE_AARCH64_TOOLCHAIN_BIN,
    [string]$TargetDir = (Join-Path ([System.IO.Path]::GetTempPath()) "oclive-aarch64-target"),
    [string]$ReportPath = ""
)

$ErrorActionPreference = "Stop"
$targetTriple = "aarch64-unknown-linux-gnu"
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

    throw "Missing $name. Set OCLIVE_AARCH64_TOOLCHAIN_BIN or pass -ToolchainBin."
}

if ($TargetDir -match "[^\x00-\x7F]") {
    throw "CARGO_TARGET_DIR must be an ASCII-only path on Windows because GNU ld cannot resolve the repository's Chinese path."
}

$gcc = Resolve-Tool "aarch64-none-linux-gnu-gcc"
$ar = Resolve-Tool "aarch64-none-linux-gnu-ar"
$readelf = Resolve-Tool "aarch64-none-linux-gnu-readelf"
$sizeTool = Resolve-Tool "aarch64-none-linux-gnu-size"

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

$env:CC_aarch64_unknown_linux_gnu = $gcc
$env:AR_aarch64_unknown_linux_gnu = $ar
$env:CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = $gcc
$env:CARGO_TARGET_DIR = $TargetDir

$startedAt = Get-Date
Push-Location $repoRoot
try {
    cargo build -p ailive-gun-spirit-host --release --locked --target $targetTriple
    if ($LASTEXITCODE -ne 0) {
        throw "AArch64 release build failed."
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
    elf64 = $header.Contains("Class:                             ELF64")
    machine_aarch64 = $header.Contains("Machine:                           AArch64")
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

if (-not ($result.elf64 -and $result.machine_aarch64)) {
    throw "Artifact exists but its ELF architecture/ABI does not match AArch64 Linux."
}