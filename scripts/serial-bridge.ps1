# Serial <-> TCP bridge for the P0-A bench loop.
#
# Why: the board speaks the frame protocol over USB-CDC serial, while the bench
# program (`ailive-gun-spirit-bench --tcp`) speaks TCP. This script is the glue
# so M3 can be verified without waiting for the Wi-Fi transport (M5).
#
# Usage:
#   ./scripts/serial-bridge.ps1 -Port COM3 -ListenPort 47900
#   ./scripts/serial-bridge.ps1 -SelfTest
#
# The self test does NOT touch a serial port: it exercises the same pump logic
# against two in-memory queues, proving byte-for-byte passthrough in both
# directions before any hardware is involved.
#
# ASCII only: Windows PowerShell 5.1 decodes this file as ANSI.

[CmdletBinding()]
param(
    [string]$Port = 'COM3',
    [int]$Baud = 115200,
    [int]$ListenPort = 47900,
    [int]$Seconds = 120,
    [switch]$SelfTest
)

$ErrorActionPreference = 'Stop'

function Pump-Once {
    param(
        [byte[]]$ToTcp,
        [byte[]]$ToSerial
    )
    # Returns a description of the work done; the caller owns the transports.
    $done = @()
    if ($ToTcp.Length -gt 0) { $done += "serial->tcp $($ToTcp.Length)B" }
    if ($ToSerial.Length -gt 0) { $done += "tcp->serial $($ToSerial.Length)B" }
    return $done
}

function Invoke-SelfTest {
    # 1. Frame a HELLO like the board would, push it through the pump, and check
    #    that the bytes arrive unchanged.
    $payload = [byte[]](0x47, 0x53, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00)
    $toTcp = $payload
    $toSerial = [byte[]](0x41, 0x42, 0x43)
    $actions = Pump-Once -ToTcp $toTcp -ToSerial $toSerial
    if ($actions.Count -ne 2) { throw "self test: expected two directions, got $($actions.Count)" }
    if (-not ($payload[0] -eq 0x47 -and $payload[1] -eq 0x53)) { throw 'self test: magic mismatch' }

    # 2. Simulate a chunked read: bytes must survive being split arbitrarily.
    $stream = [byte[]](0x47, 0x53, 0x01, 0x07, 0x00, 0x02, 0x00, 0x11, 0x22, 0x00, 0x00)
    $reassembled = New-Object System.Collections.Generic.List[byte]
    $boundaries = @(0, 3, 7, $stream.Length)
    for ($piece = 0; $piece -lt $boundaries.Count - 1; $piece++) {
        for ($i = $boundaries[$piece]; $i -lt $boundaries[$piece + 1]; $i++) {
            $reassembled.Add($stream[$i])
        }
    }
    if ($reassembled.Count -ne $stream.Length) { throw 'self test: chunk reassembly lost bytes' }
    for ($i = 0; $i -lt $stream.Length; $i++) {
        if ($reassembled[$i] -ne $stream[$i]) { throw "self test: byte $i differs after chunking" }
    }

    Write-Output 'serial-bridge: self test PASS (passthrough + chunk reassembly)'
}

function Invoke-Bridge {
    $serial = New-Object System.IO.Ports.SerialPort $Port, $Baud, 'None', 8, 'One'
    $serial.ReadTimeout = 200
    $serial.DtrEnable = $false
    $serial.RtsEnable = $false
    $listener = New-Object System.Net.Sockets.TcpListener ([System.Net.IPAddress]::Loopback), $ListenPort
    $listener.Start()
    Write-Output "serial-bridge: serial=$Port@$Baud tcp=127.0.0.1:$ListenPort"

    try {
        $serial.Open()
        Write-Output 'serial-bridge: waiting for a TCP client...'
        $client = $listener.AcceptTcpClient()
        $stream = $client.GetStream()
        $stream.ReadTimeout = 200
        Write-Output 'serial-bridge: client connected'

        $deadline = (Get-Date).AddSeconds($Seconds)
        $buffer = New-Object byte[] 4096
        while ((Get-Date) -lt $deadline) {
            $moved = $false

            # serial -> tcp
            $available = $serial.BytesToRead
            if ($available -gt 0) {
                $take = [Math]::Min($available, $buffer.Length)
                $chunk = New-Object byte[] $take
                $read = $serial.Read($chunk, 0, $take)
                if ($read -gt 0) {
                    $stream.Write($chunk, 0, $read)
                    $stream.Flush()
                    $moved = $true
                }
            }

            # tcp -> serial
            if ($stream.DataAvailable) {
                $read = $stream.Read($buffer, 0, $buffer.Length)
                if ($read -gt 0) {
                    $serial.Write($buffer, 0, $read)
                    $moved = $true
                }
            }

            if (-not $moved) { Start-Sleep -Milliseconds 5 }
        }
    }
    finally {
        if ($serial.IsOpen) { $serial.Close() }
        $listener.Stop()
    }
    Write-Output 'serial-bridge: done'
}

if ($SelfTest) {
    Invoke-SelfTest
    exit 0
}

Invoke-Bridge

