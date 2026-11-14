param(
    [string]$ToolRoot = (Join-Path $env:LOCALAPPDATA 'OCLive\jc3248w535'),
    [string]$VendorSketch,
    [string]$MumuRole,
    [string]$AssetPython,
    # 附加编译宏，例如 "-DDIAG_RADIO"。用于把 Wi-Fi/BLE 协议栈链进来测体积与内存。
    [string]$ExtraDefines,
    # 外部资产模式：像素不进固件，改为生成 assets.bin 供写入 ffat 资产区。
    [switch]$BlobAssets,
    # 链接 Rust 感知内核静态库（先跑 scripts/build-kernel-lib.ps1 生成）
    [switch]$WithKernel,
    # 启用 Wi-Fi STA 链路（需要外部资产，否则二进制超过 2 MiB 分区）
    [switch]$WithWifi
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..\..')).Path
$workspace = Split-Path -Parent $repoRoot
if (-not $VendorSketch) {
    $artifactRoot = Join-Path $workspace 'oclive-四季宝-artifacts'
    $VendorSketch = Join-Path $artifactRoot 'hardware\jc3248w535\2026-11-09\vendor\JC3248W535EN\1-Demo\Demo_Arduino\DEMO_LVGL'
}
if (-not $MumuRole) {
    $MumuRole = Join-Path $workspace 'oclivenewnew\distros\chat-pro\roles\mumu'
}
if (-not $AssetPython) {
    $AssetPython = Join-Path $ToolRoot 'assets-env\Scripts\python.exe'
}
if (-not (Get-Command -Name $AssetPython -ErrorAction SilentlyContinue)) {
    throw 'Set up the asset Python environment in the firmware README, or pass -AssetPython.'
}

$cli = Join-Path $ToolRoot 'bin\arduino-cli.exe'
if (-not (Test-Path -LiteralPath $cli -PathType Leaf)) {
    throw "Arduino CLI is missing at $cli. See the firmware README for setup."
}

# Stage the sketch and the matching Guition drivers outside the source tree.
$sketch = Join-Path $ToolRoot 'sketch\Jc3248w535'
$output = Join-Path $ToolRoot 'output'
$null = New-Item -ItemType Directory -Path $sketch, $output -Force

$drivers = @{
    'bsp_err_check.h'     = '486b084b7073b44b1b9c0f758b29c46fce83d70a4456c6fcf5d5238756b1aadb'
    'display.h'           = '29f93fd86c24c48fedc4b6fe58aa445acaa8267ece31dfdc660116e426cfaa42'
    'esp_bsp.c'           = '0e67b0fb1fa5ce01fd1afbcda147e5b024b7b00fa8e90e1c36f81d08c0502b2f'
    'esp_bsp.h'           = '2da0e2e529bbc73b88d999c2429f092b4fc1f7b95a0e8dabb043b3126e83b61a'
    'esp_lcd_axs15231b.c' = '51dd1dd9239b95852874f681ed1c7932baa545e2e7d93855e2fcfecf07d4be14'
    'esp_lcd_axs15231b.h' = '4e22f6494ddaf3053f3b058ecb043f8dc86d6fbb7b78fdc2a83cee8b74dff037'
    'esp_lcd_touch.c'     = '804eeea836e75a8a9c815fb53932a7d3c8811f2b167c3707f55bd400d376328b'
    'esp_lcd_touch.h'     = '25460bf4fcf72ab0faf47535d7d23e6901907b01ef46eb44ac0b68464049365a'
    'lv_port.c'           = 'eff32844dd424b85f83ad189b9f46ade08876d50c4649282907de9908ca38438'
    'lv_port.h'           = '48a77fff478799712bff844e001ee94bf6f3634fed9ffee99b6bbc5c42eae571'
}

foreach ($name in $drivers.Keys) {
    $source = Join-Path $VendorSketch $name
    if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
        throw "Missing Guition JC3248W535 driver $source. Pass -VendorSketch to locate DEMO_LVGL."
    }
    if ((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash -ne $drivers[$name]) {
        throw "Unexpected Guition driver revision in $source."
    }
    Copy-Item -LiteralPath $source -Destination $sketch
}

foreach ($name in 'Jc3248w535.ino', 'display_app.cpp', 'imu.cpp', 'imu.h', 'gs_wire.cpp', 'gs_wire.h', 'kernel_bridge.h', 'wifi_link.cpp', 'wifi_link.h', 'settings_ui.cpp', 'settings_ui.h', 'version.h', 'lv_font_ui_16.c', 'lv_conf.h', 'portrait_assets.h') {
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot "Jc3248w535\$name") -Destination $sketch
}
$assetArguments = @($MumuRole, $sketch)
if ($BlobAssets) {
    $assetArguments += @('--blob', (Join-Path $output 'assets.bin'))
}
& $AssetPython -B (Join-Path $PSScriptRoot 'pack-portraits.py') @assetArguments
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

$env:ARDUINO_DIRECTORIES_DATA = Join-Path $ToolRoot 'data'
$env:ARDUINO_DIRECTORIES_DOWNLOADS = Join-Path $ToolRoot 'downloads'
$env:ARDUINO_DIRECTORIES_USER = Join-Path $ToolRoot 'user'
$env:ARDUINO_BOARD_MANAGER_ADDITIONAL_URLS = 'https://espressif.github.io/arduino-esp32/package_esp32_index.json'

$core = Join-Path $env:ARDUINO_DIRECTORIES_DATA 'packages\esp32\hardware\esp32\3.0.2'
$library = Join-Path $env:ARDUINO_DIRECTORIES_USER 'libraries\lvgl\library.properties'
if (-not (Test-Path -LiteralPath $core -PathType Container) -or
    -not (Test-Path -LiteralPath $library -PathType Leaf) -or
    -not (Select-String -LiteralPath $library -Pattern '^version=8\.3\.11$' -Quiet)) {
    throw 'Install Arduino-ESP32 3.0.2 and LVGL 8.3.11 into this tool directory before building.'
}

$board = 'esp32:esp32:esp32s3:FlashSize=16M,FlashMode=qio,PSRAM=opi,USBMode=hwcdc,CDCOnBoot=cdc,PartitionScheme=fatflash'
# Rust 内核静态库：位于 compiler.libraries.ldflags（Arduino 把它放在 --start-group 内），
# 因此可以直接给完整路径，不必依赖 -L/-l 的搜索路径。
$kernelFlags = ''
if ($WithKernel) {
    $kernelLibrary = Join-Path (Join-Path $ToolRoot 'kernel') 'libgs_kernel_shim.a'
    if (-not (Test-Path -LiteralPath $kernelLibrary -PathType Leaf)) {
        throw "Missing $kernelLibrary. Run scripts/build-kernel-lib.ps1 first."
    }
    $kernelFlags = '"' + $kernelLibrary + '"'
}
$lvglFlags = '-MMD -c -DLV_CONF_INCLUDE_SIMPLE -I"' + $sketch + '"'
if ($BlobAssets) {
    $lvglFlags = $lvglFlags + ' -DUSE_BLOB_ASSETS'
}
if ($WithKernel) {
    $lvglFlags = $lvglFlags + ' -DGS_KERNEL_ENABLED'
}
if ($WithWifi) {
    $lvglFlags = $lvglFlags + ' -DGS_WIFI_ENABLED'
}
if ($ExtraDefines) {
    $lvglFlags = $lvglFlags + ' ' + $ExtraDefines
}
& $cli compile --fqbn $board --build-path (Join-Path $ToolRoot 'build') --output-dir $output `
    --build-property "compiler.c.extra_flags=$lvglFlags" `
    --build-property "compiler.cpp.extra_flags=$lvglFlags" `
    --build-property "compiler.libraries.ldflags=$kernelFlags" `
    $sketch
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
Copy-Item -LiteralPath (Join-Path $ToolRoot 'sketch\mumu-assets.json') -Destination $output








