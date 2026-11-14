# Regenerate the CJK subset font used by the on-device UI.
#
# Why this exists: LVGL's built-in fonts are Latin-only, and its bundled
# `lv_font_simsun_16_cjk` covers ~1000 common characters but was missing 33 of
# the characters this UI actually uses (they render as tofu boxes). A full CJK
# font does not fit the 2 MiB app partition, so the UI ships a subset generated
# from a system font:
#
#   - source font : C:\Windows\Fonts\simhei.ttf  (SimHei, TTF, GB2312+ coverage)
#   - character set: every CJK character that appears in the UI sources
#   - output      : firmware/jc3248w535/Jc3248w535/lv_font_ui_16.c
#
# Re-run this whenever you add or change Chinese text in the UI. The generated
# .c file is committed so builds do not need Node.js.
#
# Requires: node + lv_font_conv (npm install -g lv_font_conv)
#
# Usage:
#   ./scripts/build-cjk-font.ps1            # scan sources, regenerate
#   ./scripts/build-cjk-font.ps1 -Check     # fail if the font is stale
#
# ASCII only: Windows PowerShell 5.1 decodes this file as ANSI.

[CmdletBinding()]
param(
    [string]$Font = 'C:\Windows\Fonts\simhei.ttf',
    [string]$Size = '16',
    [switch]$Check
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$sketch = Join-Path $repoRoot 'firmware\jc3248w535\Jc3248w535'
$output = Join-Path $sketch 'lv_font_ui_16.c'

$sources = @(
    (Join-Path $sketch 'display_app.cpp'),
    (Join-Path $sketch 'settings_ui.cpp')
) | Where-Object { Test-Path -LiteralPath $_ }

if ($sources.Count -eq 0) {
    throw 'No UI sources found; check the sketch path.'
}

# Collect every non-ASCII character that the UI can render.
$characters = New-Object System.Collections.Generic.SortedSet[string]
foreach ($source in $sources) {
    $text = Get-Content -LiteralPath $source -Raw -Encoding UTF8
    # Strip comments first: only characters that can actually be rendered at
    # runtime belong in the subset (comments would bloat it for nothing).
    $text = [regex]::Replace($text, '/\*.*?\*/', '', 'Singleline')
    $text = [regex]::Replace($text, '//[^\r\n]*', '')
    foreach ($character in $text.ToCharArray()) {
        if ([int]$character -ge 0x2E80) {
            [void]$characters.Add([string]$character)
        }
    }
}
$symbols = -join $characters
Write-Output ("build-cjk-font: {0} CJK characters from {1} sources" -f $symbols.Length, $sources.Count)

if (-not (Test-Path -LiteralPath $Font -PathType Leaf)) {
    throw "Missing source font: $Font"
}

# Generate straight to the final path: lv_font_conv derives the include guard from
# the file name, so a temporary name containing a dot would emit `#if XXX.C`.
$outputDirect = $output
$arguments = @(
    '--font', $Font,
    '--size', $Size,
    '--bpp', '4',
    '--format', 'lvgl',
    '-r', '0x20-0x7F',
    '--symbols', $symbols,
    '-o', $outputDirect,
    '--force-fast-kern-format',
    '--no-compress',
    '--lv-include', 'lvgl.h'
)

if ($Check) {
    if (-not (Test-Path -LiteralPath $output -PathType Leaf)) {
        Write-Output 'build-cjk-font: FAIL (font file missing)'
        exit 1
    }
    $existing = Get-Content -LiteralPath $output -Raw -Encoding UTF8
    $missing = @()
    foreach ($character in $characters) {
        # The Opts header records the generated symbol list verbatim.
        if ($existing.IndexOf($character) -lt 0) { $missing += $character }
    }
    if ($missing.Count -gt 0) {
        Write-Output ("build-cjk-font: FAIL (font is stale, missing: {0})" -f (-join $missing))
        exit 1
    }
    Write-Output 'build-cjk-font: PASS (font covers all UI characters)'
    exit 0
}

& lv_font_conv @arguments
if ($LASTEXITCODE -ne 0) {
    throw "lv_font_conv failed with $LASTEXITCODE"
}
$bytes = (Get-Item -LiteralPath $output).Length
Write-Output ("build-cjk-font: wrote {0} ({1:N0} B)" -f $output, $bytes)


