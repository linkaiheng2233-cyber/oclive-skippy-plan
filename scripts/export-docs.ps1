# export-docs.ps1 - export field docs from this repo to the desktop copies.
#
# This script is intentionally ASCII-only. Windows PowerShell 5.1 decodes a
# BOM-less .ps1 as the ANSI code page, which would corrupt any CJK literal in
# the source (see GS-QUALITY-002). All CJK lives in export-map.tsv, which is
# read with an explicit UTF-8 decoder.
#
# Replayable export: mapping table + per-file SHA-256 comparison + refresh of
# the generated manifest block in the field package's export note.

[CmdletBinding()]
param(
    # Target roots. Defaults come from the %USERPROFILE%-relative paths declared
    # in export-map.tsv; overrides are accepted so no personal absolute path has
    # to be committed as a runtime contract.
    [string]$PersonalDocs,
    [string]$FieldPackage,
    # Verify only: report drift without writing anything.
    [switch]$Check
)

$ErrorActionPreference = 'Stop'

# GS-QUALITY-002: pin the console code page so native output is not mis-decoded.
try {
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    $OutputEncoding = [System.Text.Encoding]::UTF8
} catch { }

$utf8NoBom = [System.Text.UTF8Encoding]::new($false)

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$mapPath = Join-Path $PSScriptRoot 'export-map.tsv'

if (-not (Test-Path -LiteralPath $mapPath)) {
    throw "mapping table not found: $mapPath"
}

$beginMarker = '<!-- EXPORT-MANIFEST:BEGIN -->'
$endMarker = '<!-- EXPORT-MANIFEST:END -->'

# --- parse the mapping table (explicit UTF-8, no BOM assumption) ---
$roots = @{}
$map = @()
$lines = [System.IO.File]::ReadAllLines($mapPath, [System.Text.Encoding]::UTF8)
foreach ($line in $lines) {
    if ([string]::IsNullOrWhiteSpace($line)) { continue }
    if ($line.StartsWith('# root')) {
        $parts = $line.Substring(1).Trim() -split "`t"
        if ($parts.Count -ge 3) { $roots[$parts[1].Trim()] = $parts[2].Trim() }
        continue
    }
    if ($line.StartsWith('#')) { continue }
    $parts = $line -split "`t"
    if ($parts.Count -lt 3) {
        throw "malformed mapping row: $line"
    }
    $map += @{ Kind = $parts[0].Trim(); Target = $parts[1].Trim(); Source = $parts[2].Trim() }
}

function Resolve-Root {
    param([string]$Kind)
    $rel = $roots[$Kind]
    if (-not $rel) { throw "no root declared for kind '$Kind' in export-map.tsv" }
    $normalized = $rel -replace '/', '\'
    return (Join-Path $env:USERPROFILE $normalized)
}

if (-not $PersonalDocs) { $PersonalDocs = Resolve-Root 'Personal' }
if (-not $FieldPackage) { $FieldPackage = Resolve-Root 'Package' }

function Get-TargetRoot {
    param([string]$Kind)
    if ($Kind -eq 'Package') { return $FieldPackage }
    return $PersonalDocs
}

# Get-FileHash is not resolvable in every Windows PowerShell 5.1 environment on
# this machine (Microsoft.PowerShell.Utility fails to expose it), so hashing is
# done through .NET directly. Behaviour is identical and host-independent.
function Get-Sha256Hex {
    param([Parameter(Mandatory = $true)][string]$Path)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    $stream = [System.IO.File]::OpenRead($Path)
    try {
        $bytes = $sha.ComputeHash($stream)
    }
    finally {
        $stream.Dispose()
        $sha.Dispose()
    }
    return ([System.BitConverter]::ToString($bytes) -replace '-', '')
}

$rows = @()
$exported = 0
$stale = 0
$missingSource = 0
$missingTargetRoot = 0

foreach ($item in $map) {
    $sourcePath = Join-Path $repoRoot $item.Source
    $targetRoot = Get-TargetRoot -Kind $item.Kind
    $targetPath = Join-Path $targetRoot $item.Target

    if (-not (Test-Path -LiteralPath $sourcePath)) {
        Write-Output ("MISSING-SOURCE  " + $item.Source)
        $missingSource++
        continue
    }

    $sourceHash = Get-Sha256Hex -Path $sourcePath
    $sourceSize = (Get-Item -LiteralPath $sourcePath).Length

    $targetHash = ''
    if (Test-Path -LiteralPath $targetPath) {
        $targetHash = Get-Sha256Hex -Path $targetPath
    }
    else {
        $missingTargetRoot++
    }

    if ($targetHash -ne $sourceHash) {
        if ($Check) {
            Write-Output ("STALE           " + $item.Target)
            $stale++
        }
        else {
            $parent = Split-Path -Parent $targetPath
            if (-not (Test-Path -LiteralPath $parent)) {
                New-Item -ItemType Directory -Path $parent -Force | Out-Null
            }
            Copy-Item -LiteralPath $sourcePath -Destination $targetPath -Force
            $newHash = Get-Sha256Hex -Path $targetPath
            if ($newHash -ne $sourceHash) {
                throw "post-copy verification failed: $($item.Target)"
            }
            Write-Output ("EXPORTED        " + $item.Target)
            $exported++
        }
    }

    $rows += @{
        Kind   = $item.Kind
        Target = $item.Target
        Source = $item.Source
        Size   = $sourceSize
        Sha    = $sourceHash.Substring(0, 12)
    }
}

$mode = if ($Check) { 'check' } else { 'export' }
$stamp = (Get-Date).ToString('yyyy-MM-dd HH:mm')
$stampUtcOffset = [System.TimeZoneInfo]::Local.Id

# --- generated manifest block (ASCII labels, CJK values from the map) ---
$block = @()
$block += $beginMarker
$block += '| field | file | repo source | bytes | sha256 |'
$block += '|---|---|---|---:|---|'
foreach ($r in $rows) {
    $block += ('| ' + $r.Kind + ' | `' + $r.Target + '` | `' + $r.Source + '` | ' + $r.Size + ' | `' + $r.Sha + '` |')
}
$block += ''
$block += ('generated: ' + $stamp + ' (' + $stampUtcOffset + ') | mode: ' + $mode + ' | files: ' + $rows.Count + ' | sha256 prefix only')
$block += 'repo root: ' + $repoRoot
$block += 'source of truth: the repository. This block is rewritten by scripts/export-docs.ps1 on every run; do not edit inside the markers.'
$block += $endMarker
$blockText = ($block -join "`n")

if (-not $Check) {
    # The note file name is CJK, so it is declared in the mapping table rather
    # than as a literal here.
    $notePath = $null
    $noteFile = $null
    $noteLine = [System.IO.File]::ReadAllLines($mapPath, [System.Text.Encoding]::UTF8) |
        Where-Object { $_ -like '# note*' } |
        Select-Object -First 1
    if ($noteLine) {
        $noteFile = ($noteLine.Substring(1).Trim() -split "`t")[1].Trim()
        $notePath = Join-Path $FieldPackage ($noteFile -replace '/', '\')
    }

    if (Test-Path -LiteralPath $notePath) {
        $text = [System.IO.File]::ReadAllText($notePath, [System.Text.Encoding]::UTF8)
        $pattern = '(?s)' + [regex]::Escape($beginMarker) + '.*?' + [regex]::Escape($endMarker)
        if ([regex]::IsMatch($text, $pattern)) {
            $evaluator = [System.Text.RegularExpressions.MatchEvaluator] { param($m) $blockText }
            $text = [regex]::Replace($text, $pattern, $evaluator)
        }
        else {
            $text = $text.TrimEnd() + "`n`n" + $blockText + "`n"
        }
        [System.IO.File]::WriteAllText($notePath, $text, $utf8NoBom)
        Write-Output ("MANIFEST        " + $noteFile)
    }
    else {
        Write-Output ("MANIFEST-SKIP   note file not found under: " + $FieldPackage)
    }
}

Write-Output ''
Write-Output ("export-docs: mapped=" + $map.Count + " exported=" + $exported + " stale=" + $stale + " missing-source=" + $missingSource + " missing-target=" + $missingTargetRoot + " mode=" + $mode)

if ($missingSource -gt 0) { exit 2 }
if ($Check -and $stale -gt 0) { exit 1 }
exit 0
