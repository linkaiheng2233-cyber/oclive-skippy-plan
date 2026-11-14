# Validate a gun-spirit distro directory against DISTRO_BASELINE.md and
# SCENARIO_FORMAT.md. Read-only: never writes or fixes anything.
#
# Usage:
#   ./scripts/check-distro.ps1
#   ./scripts/check-distro.ps1 -DistroDir distros/example-kit
#
# Exit code: 0 = PASS, 1 = validation failure, 2 = usage error.
# ASCII only: Windows PowerShell 5.1 decodes this file as ANSI.

[CmdletBinding()]
param(
    [string]$DistroDir = 'distros/example-kit'
)

$ErrorActionPreference = 'Stop'
$failures = New-Object System.Collections.Generic.List[string]
$checks = 0

function Fail([string]$message) {
    $script:failures.Add($message)
}

function Check([string]$name, [bool]$ok, [string]$detail) {
    $script:checks++
    if (-not $ok) {
        Fail "$name ($detail)"
    }
}

function Read-JsonFile([string]$path) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        Fail "missing file: $path"
        return $null
    }
    try {
        return (Get-Content -LiteralPath $path -Raw -Encoding UTF8 | ConvertFrom-Json)
    } catch {
        Fail "invalid JSON: $path ($($_.Exception.Message))"
        return $null
    }
}

# Reject absolute paths, parent traversal, URLs and drive letters.
function Test-SafeRelativePath([string]$path) {
    if ([string]::IsNullOrWhiteSpace($path)) { return $false }
    if ($path -match '^[A-Za-z]:') { return $false }
    if ($path -match '^[\\/]') { return $false }
    if ($path -match '\.\.') { return $false }
    if ($path -match '^[a-zA-Z]+://') { return $false }
    return $true
}

$root = (Resolve-Path -LiteralPath $DistroDir -ErrorAction SilentlyContinue)
if (-not $root) {
    Write-Output "check-distro: directory not found: $DistroDir"
    exit 2
}
$root = $root.Path

Write-Output "check-distro: $root"

# --- 1. manifest ---------------------------------------------------------
$manifestPath = Join-Path $root 'manifest.json'
$manifest = Read-JsonFile $manifestPath
if ($manifest -eq $null) {
    Write-Output 'check-distro: FAIL (no manifest)'
    exit 1
}

Check 'manifest.schema_version' ($manifest.schema_version -eq 1) "got $($manifest.schema_version)"
foreach ($field in 'id', 'name', 'distro_version', 'requires_firmware', 'requires_contract') {
    $value = $manifest.$field
    Check "manifest.$field" (-not [string]::IsNullOrWhiteSpace($value)) 'empty or missing'
}
Check 'manifest.role_ref.id' (-not [string]::IsNullOrWhiteSpace($manifest.role_ref.id)) 'missing role id'
Check 'manifest.role_ref.version' (-not [string]::IsNullOrWhiteSpace($manifest.role_ref.version)) 'missing role version'
$maxTotal = 0
if ($null -ne $manifest.limits) { $maxTotal = [int64]$manifest.limits.max_total_bytes }
Check 'manifest.limits.max_total_bytes' ($maxTotal -gt 0 -and $maxTotal -le 8388608) "got $maxTotal"

# --- 2. files list: path safety, existence, size, sha256 ------------------
$listed = @{}
$totalBytes = 0
$fileEntries = @($manifest.files)
Check 'manifest.files' ($fileEntries.Count -ge 4) "got $($fileEntries.Count) entries"
foreach ($entry in $fileEntries) {
    $relative = [string]$entry.path
    Check 'files.path.safe' (Test-SafeRelativePath $relative) "unsafe: $relative"
    $full = Join-Path $root $relative
    $exists = Test-Path -LiteralPath $full -PathType Leaf
    Check 'files.path.exists' $exists "missing: $relative"
    if (-not $exists) { continue }
    $length = (Get-Item -LiteralPath $full).Length
    $totalBytes += $length
    Check 'files.bytes' ([int64]$entry.bytes -eq $length) "$relative declared $($entry.bytes) actual $length"
    $hash = (Get-FileHash -LiteralPath $full -Algorithm SHA256).Hash.ToLowerInvariant()
    Check 'files.sha256' ($entry.sha256 -eq $hash) "$relative declared $($entry.sha256) actual $hash"
    $listed[$relative] = $true
}
Check 'manifest.limits.total' ($totalBytes -le $maxTotal -and $totalBytes -gt 0) "total $totalBytes max $maxTotal"

# --- 3. every content JSON must be listed in manifest.files --------------
$contentFiles = Get-ChildItem -LiteralPath $root -Recurse -File |
    Where-Object { $_.Extension -in '.json' -and $_.Name -ne 'manifest.json' }
foreach ($file in $contentFiles) {
    $relative = $file.FullName.Substring($root.Length + 1).Replace('\', '/')
    Check 'content.listed' ($listed.ContainsKey($relative)) "not in manifest.files: $relative"
}

# --- 4. expressions ------------------------------------------------------
$expressions = Read-JsonFile (Join-Path $root 'expressions.json')
$expressionIds = @()
if ($null -ne $expressions) {
    Check 'expressions.schema_version' ($expressions.schema_version -eq 1) "got $($expressions.schema_version)"
    $vocabulary = @($expressions.vocabulary)
    Check 'expressions.vocabulary' ($vocabulary.Count -ge 7) "got $($vocabulary.Count)"
    $expressionIds = @($vocabulary | ForEach-Object { [string]$_.id })
    Check 'expressions.unique' (($expressionIds | Select-Object -Unique).Count -eq $expressionIds.Count) 'duplicate id'
    Check 'expressions.default' ($expressionIds -contains [string]$expressions.default) "default $($expressions.default) not in vocabulary"
    foreach ($item in $vocabulary) {
        Check 'expressions.asset.safe' (Test-SafeRelativePath ([string]$item.asset)) "unsafe asset: $($item.asset)"
    }
    $required = 'neutral', 'happy', 'excited', 'shy', 'confused', 'sad', 'angry'
    foreach ($id in $required) {
        Check 'expressions.firmware_vocabulary' ($expressionIds -contains $id) "missing firmware id: $id"
    }
}

# --- 5. reactions --------------------------------------------------------
$reactions = Read-JsonFile (Join-Path $root 'reactions.json')
if ($null -ne $reactions) {
    Check 'reactions.schema_version' ($reactions.schema_version -eq 1) "got $($reactions.schema_version)"
    $knownEvents = 'carrier.held_entered', 'carrier.held_exited', 'carrier.ready_entered',
        'carrier.ready_exited', 'control.primary.engaged', 'control.primary.released'
    $items = @($reactions.reactions)
    Check 'reactions.items' ($items.Count -ge 1) 'no reactions'
    foreach ($item in $items) {
        Check 'reactions.event' ($knownEvents -contains [string]$item.event) "unknown event: $($item.event)"
        Check 'reactions.expression' ($expressionIds -contains [string]$item.expression) "unknown expression: $($item.expression)"
        $cooldown = [double]$item.cooldown_s
        Check 'reactions.cooldown' ($cooldown -ge 0 -and $cooldown -le 60) "cooldown $cooldown"
        $priority = [int]$item.priority
        Check 'reactions.priority' ($priority -ge 1 -and $priority -le 5) "priority $priority"
    }
}

# --- 6. presets ----------------------------------------------------------
$presets = Read-JsonFile (Join-Path $root 'presets.json')
if ($null -ne $presets) {
    Check 'presets.schema_version' ($presets.schema_version -eq 1) "got $($presets.schema_version)"
    $dwell = [double]$presets.presets.expression_min_dwell_s
    Check 'presets.dwell' ($dwell -ge 0.1 -and $dwell -le 5) "dwell $dwell"
    $pose = [double]$presets.presets.pose_raised_threshold_deg
    Check 'presets.pose_threshold' ($pose -ge 1 -and $pose -le 45) "threshold $pose"
    $grip = [int]$presets.presets.grip_strength_threshold_milli
    Check 'presets.grip_threshold' ($grip -ge 0 -and $grip -le 1000) "threshold $grip"
}

# --- 7. scenarios --------------------------------------------------------
$scenarioFiles = @()
$scenarioDir = Join-Path $root 'scenarios'
if (Test-Path -LiteralPath $scenarioDir -PathType Container) {
    $scenarioFiles = @(Get-ChildItem -LiteralPath $scenarioDir -File -Filter '*.scn.json')
}
Check 'scenarios.present' ($scenarioFiles.Count -ge 1) 'no scenario files'
foreach ($file in $scenarioFiles) {
    $scenario = Read-JsonFile $file.FullName
    if ($null -eq $scenario) { continue }
    $label = $file.Name
    Check "scenario[$label].schema_version" ($scenario.schema_version -eq 1) "got $($scenario.schema_version)"
    Check "scenario[$label].arena" (([double]$scenario.arena.width -gt 0) -and ([double]$scenario.arena.height -gt 0)) 'bad arena size'
    $kind = [string]$scenario.mode.kind
    Check "scenario[$label].mode" ('time_attack', 'wave', 'precision', 'reaction' -contains $kind) "kind $kind"
    $duration = [double]$scenario.mode.duration_s
    Check "scenario[$label].duration" ($duration -gt 0 -and $duration -le 600) "duration $duration"
    $coverShapes = 'box', 'circle', 'wall'
    foreach ($cover in @($scenario.cover)) {
        if ($null -eq $cover) { continue }
        Check "scenario[$label].cover" ($coverShapes -contains [string]$cover.shape) "shape $($cover.shape)"
    }
    $templates = 'drone', 'dummy', 'plate', 'box', 'wall'
    $pathKinds = 'sine', 'static', 'linear', 'circle'
    $targetTotal = 0
    foreach ($target in @($scenario.targets)) {
        if ($null -eq $target) { continue }
        Check "scenario[$label].template" ($templates -contains [string]$target.template) "template $($target.template)"
        $count = [int]$target.count
        $targetTotal += $count
        Check "scenario[$label].target_count" ($count -ge 1 -and $count -le 64) "count $count"
        Check "scenario[$label].path" ($pathKinds -contains [string]$target.path.kind) "path $($target.path.kind)"
    }
    Check "scenario[$label].target_total" ($targetTotal -le 64) "total $targetTotal"
    foreach ($audio in @($scenario.audio)) {
        if ($null -eq $audio) { continue }
        Check "scenario[$label].audio.safe" (Test-SafeRelativePath ([string]$audio.asset)) "unsafe: $($audio.asset)"
    }
}

# --- summary -------------------------------------------------------------
if ($failures.Count -gt 0) {
    Write-Output ("check-distro: FAIL ({0} checks, {1} failures)" -f $checks, $failures.Count)
    foreach ($failure in $failures) { Write-Output "  - $failure" }
    exit 1
}

Write-Output ("check-distro: PASS ({0} checks, {1} content files, {2} bytes)" -f $checks, $contentFiles.Count, $totalBytes)
exit 0
