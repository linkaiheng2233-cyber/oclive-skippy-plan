# Provision Wi-Fi credentials to the board over serial WITHOUT echoing the password.
#
# Why a script: the password must never appear in a chat transcript or in the repo.
# `Read-Host -AsSecureString` keeps it off the screen while still letting us send it
# to the board, which stores it in its own NVS partition.
#
# Usage:
#   ./scripts/wifi-provision.ps1 -Ssid SUPERMUMU0090
#   ./scripts/wifi-provision.ps1                       # will ask for both
#
# ASCII only: Windows PowerShell 5.1 decodes this file as ANSI.

[CmdletBinding()]
param(
    [string]$Port = 'COM3',
    [int]$Baud = 115200,
    [string]$Ssid = ''
)

$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($Ssid)) {
    $Ssid = Read-Host 'Wi-Fi SSID (no spaces)'
}
if ($Ssid -match '\s') {
    throw 'SSID must not contain spaces.'
}

$secure = Read-Host -AsSecureString 'Wi-Fi password (not echoed)'
$plain = [System.Net.NetworkCredential]::new('', $secure).Password
if ([string]::IsNullOrWhiteSpace($plain)) {
    throw 'Empty password.'
}
if ($plain -match '\s') {
    throw 'Password must not contain spaces (the board parses by space).'
}

$serial = New-Object System.IO.Ports.SerialPort $Port, $Baud, 'None', 8, 'One'
$serial.ReadTimeout = 500
$serial.DtrEnable = $false
$serial.RtsEnable = $false
$serial.Open()
try {
    Start-Sleep -Milliseconds 300
    $serial.DiscardInBuffer()
    $serial.Write("WIFI $Ssid $plain`r`n")
    Start-Sleep -Seconds 2
    $reply = ''
    while ($serial.BytesToRead -gt 0) {
        $reply += $serial.ReadExisting()
    }
    # Never print the command we sent; only the board's reply.
    if ($reply) {
        ($reply.Trim() -split "`r?`n" | Where-Object { $_ -and $_ -notmatch [regex]::Escape($plain) }) |
            ForEach-Object { Write-Output $_ }
    } else {
        Write-Output 'wifi-provision: no reply from board'
    }

    Write-Output 'wifi-provision: waiting for association...'
    $deadline = (Get-Date).AddSeconds(25)
    while ((Get-Date) -lt $deadline) {
        Start-Sleep -Milliseconds 600
        $chunk = ''
        while ($serial.BytesToRead -gt 0) {
            $chunk += $serial.ReadExisting()
        }
        if ($chunk) {
            $lines = $chunk.Trim() -split "`r?`n"
            foreach ($line in $lines) {
                if ($line -and $line -notmatch [regex]::Escape($plain)) {
                    Write-Output $line
                }
            }
            if ($chunk -match 'WIFI connected ip=') { break }
            if ($chunk -match 'WIFI lost') { break }
        }
    }
}
finally {
    $serial.Close()
    $serial.Dispose()
}
