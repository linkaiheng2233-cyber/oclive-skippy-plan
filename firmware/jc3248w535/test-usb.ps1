param(
    [string]$Port = 'COM3',
    [ValidateSet('neutral', 'happy', 'excited', 'shy', 'confused', 'sad', 'angry', 'idle', 'curious')]
    [string]$State,
    [switch]$CheckGallery,
    [ValidateRange(0, 300)]
    [int]$WatchSeconds = 0,
    [ValidateRange(0, 100)]
    [int]$MinimumTaps = 0
)

$ErrorActionPreference = 'Stop'
if ($MinimumTaps -gt 0 -and $WatchSeconds -eq 0) {
    throw 'Pass -WatchSeconds when requiring touch events.'
}

function Read-BoardLine {
    param([System.IO.Ports.SerialPort]$Serial)

    try {
        return $Serial.ReadLine().TrimEnd("`r")
    } catch [System.TimeoutException] {
        return $null
    }
}

function Wait-BoardReply {
    param(
        [System.IO.Ports.SerialPort]$Serial,
        [string]$Expected,
        [string]$Command
    )

    $Serial.WriteLine($Command)
    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    while ($timer.Elapsed.TotalSeconds -lt 3) {
        $reply = Read-BoardLine -Serial $Serial
        if ($reply -ceq $Expected) {
            Write-Host $reply
            return
        }
        if ($reply -cmatch '^ERR ') {
            throw "Board rejected $Command with $reply"
        }
        if ($reply -cmatch '^(TOUCH [0-9]+|MOOD [a-z]+)$') {
            Write-Host $reply
        }
    }
    throw "No '$Expected' reply on $($Serial.PortName) after '$Command'."
}

$serial = [System.IO.Ports.SerialPort]::new($Port, 115200)
$serial.NewLine = "`n"
$serial.ReadTimeout = 200
$serial.WriteTimeout = 1000
$serial.DtrEnable = $true

try {
    $serial.Open()
    $serial.DiscardInBuffer()
    Wait-BoardReply -Serial $serial -Expected 'READY JC3248W535 1' -Command 'PING'

    $states = if ($State) { @($State.ToLowerInvariant()) } else { @('neutral', 'happy', 'confused') }
    foreach ($mood in $states) {
        Wait-BoardReply -Serial $serial -Expected "OK STATE $mood" -Command "STATE $mood"
        if (-not $State) {
            Start-Sleep -Milliseconds 850
        }
    }

    if ($CheckGallery) {
        Wait-BoardReply -Serial $serial -Expected 'OK STATE neutral' -Command 'STATE neutral'
        Wait-BoardReply -Serial $serial -Expected 'OK STEP angry' -Command 'STEP prev'
        foreach ($mood in 'neutral', 'happy', 'excited', 'shy', 'confused', 'sad', 'angry', 'neutral') {
            Wait-BoardReply -Serial $serial -Expected "OK STEP $mood" -Command 'STEP next'
        }
        Wait-BoardReply -Serial $serial -Expected 'OK CURRENT neutral' -Command 'GET STATE'
    }

    if ($WatchSeconds -gt 0) {
        Write-Host "Listening on $Port for $WatchSeconds seconds. Tap Prev or Next."
        $timer = [System.Diagnostics.Stopwatch]::StartNew()
        $taps = 0
        $lastTouch = $null
        while ($timer.Elapsed.TotalSeconds -lt $WatchSeconds) {
            $reply = Read-BoardLine -Serial $serial
            if ($reply -cmatch '^TOUCH ([0-9]+)$') {
                $lastTouch = $Matches[1]
                Write-Host $reply
            } elseif ($reply -cmatch '^MOOD [a-z]+$' -and $null -ne $lastTouch) {
                Write-Host $reply
                $taps++
                $lastTouch = $null
                if ($MinimumTaps -gt 0 -and $taps -ge $MinimumTaps) {
                    break
                }
            }
        }
        Write-Host "Touch events with resulting mood received: $taps"
        if ($taps -lt $MinimumTaps) {
            throw "Expected at least $MinimumTaps touch events."
        }
    }
} finally {
    $serial.Dispose()
}
