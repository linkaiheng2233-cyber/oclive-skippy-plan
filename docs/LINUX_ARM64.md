# Linux / ARM64 适配路线

## 结论

实体硬件需要 Linux 适配，但 OCLive 内核本身不需要重写。OCLive 的 `server`、`host`、`runtime` 已有 `aarch64-unknown-linux-gnu` 交叉编译门禁；本仓需要补的是设备宿主层和真实 ARM 板验证。

当前标准入口已直接依赖 `oclive_kernel_host`，不再经过 `oclivenewnew-tauri`。最终板载进程不需要 Tauri、WebView、X11、Wayland 或桌面环境。

## 适配面

| 层 | Windows 开发期 | Linux/ARM64 实机 |
|----|----------------|------------------|
| 传感器输入 | 键盘、事件文件、mock source | `/dev/i2c-*`、`/dev/gpiochip*`、必要时 `/dev/spidev*` |
| 屏幕输出 | 普通模拟器窗口 | SPI 屏直驱，或按屏幕接口选择 DRM/KMS/framebuffer |
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
- 先接一个确定性按钮/霍尔开关，再接 IMU。
- 先点亮屏幕并显示静态图，再接 VisualCue。
- 验证 udev 权限、systemd、自启动、重启和断网降级。

### L5：实机稳定性

- 冷启动无需 SSH 手工干预。
- 连续运行至少 2 小时。
- 记录 RSS、CPU、温度、屏幕刷新延迟、事件误触和丢失。
- 通过后再冻结开发板、屏幕型号、电池与外壳。

## 首轮硬件顺序

1. Linux ARM64 开机并运行 headless host。
2. 屏幕显示固定 PNG。
3. mock DeviceEvent 改变屏幕状态。
4. 物理按钮/霍尔开关产生 DeviceEvent。
5. IMU 产生 `picked_up` / `raised`。
6. 接入 OCLive `visual_state_id`。
7. 加 systemd/udev/watchdog 并做两小时 soak。

语音、触觉、BLE、相机和电池优化不与这条路线并行开发。
