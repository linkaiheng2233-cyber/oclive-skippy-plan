# ARMv7 / Luckfox Lyra 可行性验证

## 结论

截至2026-08-27，三crate workspace 的正式`ailive-gun-spirit-host` Release二进制已经重新通过ARMv7 GNU/Linux hard-float交叉编译与最终链接。`contracts`、`perception-core`、`ring`、bundled SQLite、Rustls、SQLx和OCLive headless host均未阻断当前 workspace 构建。2026-08-25 的迁移前结果保留为历史对照，不再承担当前产物证明。

这项结果只证明「可以生成适用于 Luckfox Lyra Cortex-A7 的 Linux 可执行文件」，不等于已经证明 Lyra Zero W 的 512 MB 内存、DSI、Wi-Fi/BLE 共存、温度和续航达标。板卡冻结仍需真实硬件运行数据。

## 验证环境

| 项目 | 实测值 |
|---|---|
| 主机 | Windows x86_64 |
| Rust | `rustc 1.97.1 (8bab26f4f 2026-07-14)` |
| Cargo | `cargo 1.97.1` |
| Rust target | `armv7-unknown-linux-gnueabihf` |
| C/C++ 工具链 | Arm GNU Toolchain 14.2.Rel1 / GCC 14.2.1 |
| 工具链 SHA-256 | `2cab956da2d1ce52ab7e7dbbef9d3689bea1c16647230cbf67414d555f043cec` |
| 构建命令 | `cargo build --release --locked --target armv7-unknown-linux-gnueabihf` |

工具链来自 Arm 官方 AArch32 GNU/Linux hard-float Windows 包：

- `arm-gnu-toolchain-14.2.rel1-mingw-w64-x86_64-arm-none-linux-gnueabihf.zip`
- <https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads>

## 当前 workspace ARMv7 产物数据（2026-08-27）

| 指标 | MEASURED |
|---|---:|
| 冷目标目录 Release 构建 | 116.1 s |
| 文件大小 | 23,270,116 bytes / 22.19 MiB |
| `.text` | 15,711,823 bytes |
| `.data` | 220,252 bytes |
| `.bss` | 3,612 bytes |
| ELF | ELF32、little-endian、PIE |
| 架构/ABI | ARM、EABI5、hard-float |
| SHA-256 | `3c08dc2b202d56a5a7f67d2813fe7cc5eb7411c0f9bd45291d948b43156c0015` |

动态依赖：

```text
libgcc_s.so.1
libm.so.6
libc.so.6
ld-linux-armhf.so.3
```

复现使用`./scripts/build-armv7.ps1 -ToolchainBin <Arm GNU 14.2.Rel1 bin>`；目标目录保持 ASCII-only。原始机器绝对路径只保存在本地忽略的`tmp/`报告中，不作为仓库契约。

## 迁移前历史产物数据（2026-08-25）

| 指标 | 实测值 |
|---|---:|
| 冷目标目录 Release 构建 | 约 1 分 40 秒 |
| 文件大小 | 23,371,692 bytes / 22.29 MiB |
| `.text` | 15,722,591 bytes |
| `.data` | 218,184 bytes |
| `.bss` | 3,612 bytes |
| ELF | ELF32、little-endian、PIE |
| 架构/ABI | ARM、EABI5、hard-float |
| SHA-256 | `c818fce510c27cc6760cff6cf0f3bdcdb97e0556775409824a4d560faa2a443b` |

动态依赖只有目标系统的基础运行库：

```text
libgcc_s.so.1
libm.so.6
libc.so.6
ld-linux-armhf.so.3
```

当前交叉产物仍只适用于提供上述 glibc/hard-float动态加载器的用户空间。Buildroot或厂商镜像只有在C library/动态加载器匹配时才能直接运行，否则必须使用目标镜像/SDK sysroot重新链接。

## 当前宿主内存基线

同一提交的 Windows x86_64 Release 宿主在 mock LLM、默认烟测角色、无请求空载条件下测得：

| 指标 | 实测值 |
|---|---:|
| Release 文件 | 22,308,864 bytes / 21.28 MiB |
| Working Set | 11.0 MiB |
| Private Memory | 3.5 MiB |
| 线程数 | 15 |

这不是 512 MB 板卡的验收数据：尚未包含真实角色包加载、持续 SQLite 写入、BLE、DSI framebuffer/UI、网络回合和 Linux 页缓存。它只能说明 OCLive headless host 的空载常驻量级没有立即否决 512 MB。

## 厂商数据与本项目数据分开看

| 项目 | 厂商公开数据 | 本项目已验证 | 尚未验证 |
|---|---|---|---|
| CPU/ABI | RK3506B，3 × Cortex-A7 1.2 GHz + Cortex-M0 | ELF32 ARM EABI5 hard-float 已最终链接 | 真机 CPU 占用、调度与温度 |
| 内存 | 512 MB DDR3L | Windows mock 空载工作集 11.0 MiB | ARM RSS、页缓存、角色包/SQLite/渲染峰值 |
| 显示 | 2-lane DSI；厂商适配表包含 Waveshare 2.8inch DSI | UI 目标与 ABI 已明确 | 本项目真机点屏、20 次冷启和首帧延迟 |
| 无线 | 2.4 GHz Wi-Fi 6、BT 5.2/BLE | BLE 事件协议已规划 | Wi-Fi + BLE Central 两小时共存 |
| 存储 | 256 MB SPI NAND + TF | Release ELF 22.29 MiB | TF 上的系统、角色包、数据库、日志、更新完整容量预算；不先承诺 NAND-only |

Lyra 的入选理由是 DSI、无线与小板拓扑，而不是 CPU 或 RAM 领先。Orange Pi Zero 2W 的官方配置为四核 Cortex-A53 1.5 GHz、1–4 GB LPDDR4、30 × 65 mm、Wi-Fi 5/BT 5.0 和 mini-HDMI；若 512 MB 门失败，它提供明确的内存回退，但会增加视频接头的结构代价。

## Windows 路径问题

第一次构建在仓库默认 `target/` 下完成全部 Rust/C 依赖编译，但 Arm GNU `ld.exe` 无法解析含中文的目标文件路径，在最终链接时报告文件不存在。把 `CARGO_TARGET_DIR` 改到纯 ASCII 临时路径后，同一源码成功链接。

Windows 必须使用：

```powershell
./scripts/build-armv7.ps1 `
  -ToolchainBin C:\toolchains\arm-gnu\bin `
  -TargetDir C:\temp\oclive-armv7-target
```

脚本会拒绝带非 ASCII 字符的 `TargetDir`，并在构建后核验 ELF32、ARM 和 hard-float ABI。Linux/CI 可使用：

```bash
sudo apt-get install gcc-arm-linux-gnueabihf binutils
./scripts/build-armv7.sh
```

## Lyra Zero W 实机冻结门

购买板卡后必须补齐下列数据，未通过前不把 Lyra 写成量产定板：

1. Ubuntu 22.04 armhf 或官方兼容用户空间中，二进制可启动并完成角色包、SQLite 和 HTTP host 烟测。
   首轮使用 TF 卡承载 rootfs 与数据，不把 256 MB SPI NAND 当作完整 OCLive 数据盘。
2. 启动角色包后 OCLive RSS 稳态不高于 128 MiB；事件回放和网络回合峰值不高于 256 MiB。
3. 正常运行期间系统 `MemAvailable` 不低于 128 MiB，无 OOM、无持续 swap 抖动。
4. Waveshare 2.8inch DSI LCD 冷启动 20 次全部点亮；触控可选，但背光控制必须工作。
5. Wi-Fi 联网与 BLE Central 连接 Grip Node 同时运行 2 小时，无持续断连或驱动崩溃。
6. 1000 条 DeviceEvent 回放无丢失；事件到本地画面变化的 p95 不高于 100 ms。
7. 屏亮、Wi-Fi、BLE 和 OCLive 同时工作时记录整板功耗、最高温度与降频状态。

若 ARMv7 实机门失败：优先保留同一 DSI 屏，转向内存更充足的 64 位 Linux 板；不能通过删掉本地 OCLive、角色记忆或安全生命周期来迁就板卡。

## 官方资料

- [Luckfox Lyra Zero W 产品规格](https://www.luckfox.com/Luckfox-Lyra-Zero-W)
- [Luckfox Lyra DSI 适配文档](https://wiki.luckfox.com/Luckfox-Lyra-Pi/DSI/)
- [Orange Pi Zero 2W 官方规格](https://www.orangepi.org/html/hardWare/computerAndMicrocontrollers/details/Orange-Pi-Zero-2W.html)
- [Arm GNU Toolchain 下载](https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads)
