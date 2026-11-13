# JC3248W535 桌面 USB 原型

晶彩 3.5 英寸 ESP32-S3 一体屏的沐沐情绪预览。屏幕底部 `Prev` 和 `Next` 可循环查看平静、开心、兴奋、害羞、疑惑、难过、生气。七张图片随固件存入 Flash，设备仅供电时也能切换。连接电脑时，串口可指定表情，并接收触屏切换后的表情和点击序号。界面使用原厂驱动的 480×320 横屏方向和 LVGL 8。

## 构建

准备 Arduino CLI 1.2.2、Arduino-ESP32 3.0.2、LVGL 8.3.11 和带 Pillow 12.3.0 的 Python 3.10 或更高版本。Windows 上 `build.ps1` 默认使用 `%LOCALAPPDATA%\OCLive\jc3248w535` 作为工具与构建目录。将 `arduino-cli.exe` 放入其 `bin` 子目录，然后在 PowerShell 中安装依赖。

```powershell
$tools = Join-Path $env:LOCALAPPDATA 'OCLive\jc3248w535'
$cli = Join-Path $tools 'bin\arduino-cli.exe'
$env:ARDUINO_DIRECTORIES_DATA = Join-Path $tools 'data'
$env:ARDUINO_DIRECTORIES_DOWNLOADS = Join-Path $tools 'downloads'
$env:ARDUINO_DIRECTORIES_USER = Join-Path $tools 'user'
$env:ARDUINO_BOARD_MANAGER_ADDITIONAL_URLS = 'https://espressif.github.io/arduino-esp32/package_esp32_index.json'
& $cli core update-index
& $cli core install esp32:esp32@3.0.2
& $cli lib install lvgl@8.3.11
python -m venv (Join-Path $tools 'assets-env')
& (Join-Path $tools 'assets-env\Scripts\python.exe') -m pip install Pillow==12.3.0
./firmware/jc3248w535/build.ps1
```

构建脚本默认从仓库旁的 `oclive-四季宝-artifacts/hardware/jc3248w535/2026-11-09/vendor/JC3248W535EN/1-Demo/Demo_Arduino/DEMO_LVGL` 取原厂屏幕、触摸和 LVGL 驱动。可用 `-VendorSketch` 指定从[晶彩原厂资料包](https://pan.jczn1688.com/directlink/1/HMI%20display/JC3248W535EN.zip)提取的 `DEMO_LVGL` 目录。脚本校验源文件 SHA-256，再在工具目录中暂存并编译。

立绘默认从旁边的 `oclivenewnew/distros/chat-pro/roles/mumu/portrait_catalog.json` 解析。可用 `-MumuRole` 指定角色包目录，或用 `-AssetPython` 指定已装 Pillow 的 Python。构建将原图等比缩放至 260×260 的 RGB565 画布，并在 `output/mumu-assets.json` 记录选用的资产 ID、源文件及输出哈希。每种情绪所选的强度见 `pack-portraits.py`。

## 实机验收

当前样机用 `flash-prototype.py` 安装，要求 Python 环境中有 `esptool==5.6.0` 及同一块板的完整 16 MiB 出厂备份。脚本检查出厂分区表和原应用与备份吻合，然后写入空闲的 `app1`。回读校验通过后，脚本才切换启动选项。备份位置见[硬件到货清单](../../docs/test-worksheets/01-硬件到货清单.md)。

保持 USB 连接，按住 `BOOT`，点按 `RESET`，再松开 `BOOT`，即可进入下载模式。可以先用 `--check` 验证下载链路，也可以在同一命令中传 `--image` 安装。需要切回出厂程序时，再进入下载模式并传 `--factory`。实机上的 `app1` 在 `0x210000`，因此使用独立应用 `.ino.bin`。Arduino 同时生成的 `.merged.bin` 用于整板烧录。[乐鑫下载模式说明](https://docs.espressif.com/projects/esptool/en/latest/esp32s3/advanced-topics/boot-mode-selection.html#manual-bootloader)

```powershell
$tools = Join-Path $env:LOCALAPPDATA 'OCLive\jc3248w535'
$image = Join-Path $tools 'output\Jc3248w535.ino.bin'
python ./firmware/jc3248w535/flash-prototype.py --port COM3 --backup $backup --image $image
```

`$backup` 是同一块板的出厂 `.bin` 文件路径。若 Windows 在重启时重新枚举串口，等待设备管理器重新显示端口，再用下方脚本确认程序正常启动。

这块样机也实测可以用 Arduino-ESP32 3.0.2 自带的 `esptool.exe` v4.6 执行 `--chip esp32s3 --port COM3 --before usb_reset --after no_reset --no-stub flash_id`，让板子进入并停留在下载模式。之后即可运行上面的 Python 命令。

烧录并正常启动后，从仓库根目录执行：

```powershell
./firmware/jc3248w535/test-usb.ps1 -Port COM3 -CheckGallery -WatchSeconds 45 -MinimumTaps 3
```

脚本检查七张表情的顺序，以及首尾双向循环。看到 `Tap Prev or Next` 后，依次点 `Next`、`Next`、`Prev`，屏幕应显示 `Happy 2 / 7`、`Excited 3 / 7`、`Happy 2 / 7`，电脑收到相应的三对 `TOUCH` 和 `MOOD`。用 `-State shy` 等参数可指定一张表情。

串口调试消息为 ASCII 行，以 `LF` 或 `CRLF` 结尾。`PING` 对应 `READY JC3248W535 1`，`STATE <状态>` 对应 `OK STATE <状态>`，`STEP next` 和 `STEP prev` 走与触屏相同的切换逻辑，`GET STATE` 查询当前表情。一次触屏点击产生 `TOUCH <序号>` 和 `MOOD <切换后的情绪>`。现有调试命令 `STATE idle` 和 `STATE curious` 分别映射到平静与疑惑。消息用于本台架和测试脚本。
