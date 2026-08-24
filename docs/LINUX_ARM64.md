# Linux / ARM64 适配路线

## 结论

实体硬件需要 Linux 适配，但 OCLive 内核本身不需要重写。OCLive 的 `server`、`host`、`runtime` 已有 `aarch64-unknown-linux-gnu` 交叉编译门禁；本仓需要补的是设备宿主层和真实 ARM 板验证。

当前标准入口已直接依赖 `oclive_kernel_host`，不再经过 `oclivenewnew-tauri`。最终板载进程不需要 Tauri、WebView、X11、Wayland 或桌面环境。

本项目采用主云端算力。这里的「云端」指 LLM 推理端点，不代表板子是哑终端：DeviceEvent、器灵状态机、即时屏幕反馈、超时取消、断网降级、自启动和 watchdog 都必须在板端。

| 板端保留 | 云端承担 |
|----------|----------|
| 传感器采集与滤波 | LLM 重推理 |
| DeviceEvent 与状态机 | 动态角色短回复 |
| 轻量 OCLive Host / 云端网关 | 后续可选的重型能力，不进入 v0.1 |
| PNG/HUD 渲染与屏幕驱动 | — |
| 本地确定性反馈与断网降级 | — |
| 日志、systemd、watchdog | — |

## 适配面

| 层 | Windows 开发期 | Linux/ARM64 实机 |
|----|----------------|------------------|
| 传感器输入 | 键盘、事件文件、mock source | `/dev/i2c-*`、`/dev/gpiochip*`、必要时 `/dev/spidev*` |
| 屏幕输出 | 320 × 240 普通模拟器窗口/文件 sink | 4-wire SPI 直驱为首选；可选通用 MIPI-DBI DRM/KMS |
| OCLive | 本机 headless host | 同一 `oclive_kernel_host` ARM64 构建 |
| 配置/日志 | 本地目录 | `/etc/oclive-spirit`、持久数据目录和 journald |
| 生命周期 | 手动启动 | systemd、自启动、重启、watchdog |
| 权限 | 当前用户 | udev 规则或专用用户加入 gpio/i2c/spi 组 |

现代 Linux GPIO 不以旧 sysfs 方案为主，优先使用 gpio character device（`/dev/gpiochip*`）及 libgpiod 语义。I2C/SPI 驱动必须隐藏在平台适配器后，状态机不得直接访问设备文件。

## 代码分层目标

```text
platform-neutral
  DeviceEvent / state machine / visual policy / replay

adapters/mock
  keyboard / fixture / desktop display

adapters/linux
  gpio / i2c / spi / display / watchdog

app
  config / lifecycle / OCLive client / observability
```

平台无关逻辑在 Windows 和 Linux 跑同一组测试。Linux 代码使用 Cargo feature 与 `cfg(target_os = "linux")` 隔离，不能让 Windows 模拟器依赖板卡库。

## 分阶段验证

### L0：纯 headless 依赖

- 标准二进制只依赖 `oclive_kernel_host` / `oclive_kernel_runtime`。
- 默认构建不拉入 Tauri/WebView。
- Windows `cargo check` / `cargo test` 通过。

### L1：平台接口与 Windows mock

- 抽象 `SensorSource`、`DisplaySink` 和单调时钟。
- 键盘与事件回放产生真实 DeviceEvent。
- 桌面窗口消费与未来硬件相同的 VisualCue。

### L2：Linux x86_64 无硬件烟测

- 在无 GPIO/屏幕的 Linux 环境运行 mock adapter。
- 验证路径、信号退出、日志和长期运行。
- 不在这一阶段假装完成硬件适配。

### L3：ARM64 交叉编译

- 安装 `aarch64-unknown-linux-gnu` target 与交叉链接器。
- 对默认 headless binary 执行 locked `cargo check`/`cargo build`。
- CI 只证明架构可编译，不证明设备能用。

### L4：真实开发板 bring-up

- 使用 64 位精简 Linux 发行版。
- 开发板、屏幕和传感器先以松散桌面套件连接，不安装到 RADIAN。
- 首选样屏与冻结门见 `DISPLAY_SELECTION.md`；先按 320 × 240 RGB565 和 4-wire SPI 点屏，不安装桌面环境。
- 先接一个确定性按钮，再接 IMU 和 BLE Central。
- 先点亮屏幕并显示静态图，再接 VisualCue。
- 接通 Wi-Fi/手机热点与云端 LLM，验证超时、重连和断网本地降级。
- 验证 udev 权限、systemd、自启动、重启和断网降级。

### L5：实机稳定性

- 冷启动无需 SSH 手工干预。
- 连续运行至少 2 小时。
- 记录 RSS、CPU、温度、屏幕刷新延迟、事件误触和丢失。
- 通过后再冻结开发板、屏幕型号、电池与外壳。

## 首轮硬件顺序

1. Windows/mock 按 320 × 240 验证全部核心表情与 HUD。
2. Linux ARM64 开机并运行 headless host。
3. 2.8 英寸候选屏通过 SPI 显示固定 PNG，并记录首帧、全帧和局部刷新延迟。
4. mock DeviceEvent 改变屏幕状态。
5. 物理按钮与 IMU 产生 SensorObservation，再由融合层产生 DeviceEvent。
6. Grip Node 经 BLE 接入，验证配对、断连未知态和重连状态快照。
7. 无磁性底座时，握持释放 + 持续静止可以进入待机。
8. 接入 OCLive `visual_state_id`。
9. 加 systemd/udev/watchdog 并做两小时 soak。
10. 样屏、显示背板和功耗通过后才冻结屏幕舱，再开始 RADIAN MODEL 1 导轨适配。

语音、触觉和相机不与这条路线并行开发。Grip Node 的 BLE/CR2032 与屏幕后置主机电池都属于 S4，但主机电芯只能在屏亮/屏灭、Wi-Fi、BLE 和云端回合功耗实测后选择。
