# Background serial logger used to capture touch-event evidence while the owner
# taps the screen. Writes timestamped lines to a log file and exits after the
# deadline. ASCII only: Windows PowerShell 5.1 decodes this file as ANSI.

param(
    [string]$Port = 'COM3',
    [int]$Seconds = 420,
    [string]$Log = (Join-Path $env:TEMP 'serial-touch-log.txt')
)

$serial = New-Object System.IO.Ports.SerialPort $Port, 115200, 'None', 8, 'One'
$serial.ReadTimeout = 400
$serial.DtrEnable = $false
$serial.RtsEnable = $false
for ($attempt = 0; $attempt -lt 20; $attempt++) {
    try { $serial.Open(); break } catch { Start-Sleep -Milliseconds 500 }
}
if (-not $serial.IsOpen) {
    "logger: cannot open $Port" | Add-Content -LiteralPath $Log
    exit 1
}
"logger: open $Port for $Seconds s" | Add-Content -LiteralPath $Log
$deadline = (Get-Date).AddSeconds($Seconds)
$buffer = ''
while ((Get-Date) -lt $deadline) {
    $chunk = ''
    while ($serial.BytesToRead -gt 0) { $chunk += $serial.ReadExisting() }
    if ($chunk) {
        $buffer += $chunk
        while ($buffer -match "`r?`n") {
            $index = $buffer.IndexOfAny([char[]]@("`r", "`n"))
            $line = $buffer.Substring(0, $index)
            $buffer = $buffer.Substring($index).TrimStart("`r", "`n")
            if ($line) {
                ('[{0:HH:mm:ss}] {1}' -f (Get-Date), $line.Trim()) | Add-Content -LiteralPath $Log
            }
        }
    }
    Start-Sleep -Milliseconds 120
}
$serial.Close()
$serial.Dispose()
'logger: done' | Add-Content -LiteralPath $Log
