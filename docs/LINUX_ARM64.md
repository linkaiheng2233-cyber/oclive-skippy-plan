# Linux ARMv7/ARM64 适配与板端部署路线

**SSOT 范围**：本文负责 Linux 架构、依赖裁剪、系统服务与板端验证方法；板屏选择以`HARDWARE_IMPLEMENTATION_PLAN.md`/`DISPLAY_SELECTION.md`为准。
**最后更新**：2026-08-31
**状态**：Current

## 1. 当前目标

P0 目标板改为 Orange Pi Zero 3W 6GB（Allwinner A733，ARM64）配 3.5inch HDMI touch。板端视频为USB-C DisplayPort Alt Mode，现有HDMI屏需要主动DP转HDMI适配器；其EDID、800×480时序、冷启动和包络必须实测。Luckfox Lyra Zero W（ARMv7）+ 2.8inch DSI是紧凑回退；已有ARMv7交叉编译证据只证明ABI/链接可行，不证明512MB、显示、无线、温度或续航。

板端必须本地运行Host、感知状态、OCLive角色/记忆、SQLite、Input Scheduler、Output Arbiter和最低Renderer。网络/外部LLM是可选表达能力；离线启动和设备闭环不能依赖服务器。6GB版本新增CPU量化3B模型候选，但先按独立、可失败的表达服务验证，不把模型加载成功设为最低UI或安全生命周期的启动条件。

## 2. 架构原则

- `contracts`与`perception-core`保持纯 Rust/平台无关；Linux/BlueZ/GPIO/DRM/Web 适配只在 Host。
- P0 是一个 Host 进程内的逻辑端口，不先拆微服务；systemd 负责启动、重启、资源限制和日志。
- 无桌面环境也能运行 headless OCLive；Renderer 技术通过`RendererPort`可替换。
- OCLive 兄弟仓从源码保留完整能力，但目标镜像只携带当前 Host 需要的二进制、角色、Web 资产、迁移和运行库。

## 3. OCLive Linux 裁剪审查

制作镜像前生成四类证据：

1. `cargo tree`/features：区分 contracts/core/Host 与 OCLive 传递依赖。
2. 目标 ELF：`file`、`readelf -h/-d`、`ldd`或 sysroot 对照，记录 ABI 与动态库。
3. 运行资源：冷启动 RSS/峰值 RSS、线程、文件、socket、SQLite、角色/资产与日志。
4. 服务/镜像清单：`保留 | Host替换 | 按需启用 | 镜像排除`，写明理由和回退。

“从镜像排除”不等于删除 OCLive 主仓源码。不得为了小板删掉角色连续性、安全生命周期或本地感知闭环。

## 4. 构建

### ARM64 P0

在确定镜像/sysroot 后增加固定 target 与可复现构建；没有板卡/镜像身份前，不把通用 GNU 交叉链接当最终可运行证明。最终产物名为`ailive-gun-spirit-host`。

### ARMv7 回退

```powershell
./scripts/build-armv7.ps1
```

或 Linux：

```bash
./scripts/build-armv7.sh
```

迁移 workspace、OCLive 依赖或 release profile 后必须重跑。结果记录在`ARMV7_VALIDATION.md`，并明确“交叉编译通过 ≠ Lyra 实机通过”。

## 5. Orange Pi bring-up

1. 固定板卡 revision、镜像、内核、设备树、bootloader、供电与 SD 卡。
2. 用稳定 5 V 桌面电源完成 headless Host help/mock 启动。
3. 通过USB-C DP Alt Mode主动转HDMI点亮测试图，接USB touch，验证EDID、800×480/旋转、背光、热插拔与20次冷启动。
4. 跑当前 Host/OCLive + mock renderer，记录 RSS、`MemAvailable`、线程、温度与首帧。
5. 接两个 BLE 节点/模拟器，验证 Wi-Fi/BLE 共存、断连、TTL unknown 和 1,000 条事件。
6. 做两小时稳定性、日志上限、数据库 checkpoint、正常/异常关机恢复。
7. 用`LOCAL_3B_INFERENCE.md`的固定语料测试CPU量化3B，分别记录模型加载、短上下文首字、解码速度、质量、RSS、温度和关键链路延迟；NPU/GPU不作为通过前提。
8. 最后接完整成品5V电源测四小时；P1再加UVC/hub。

全部数据填`ORANGE_PI_BRINGUP_WORKSHEET.md`，不能只写“能跑”。

## 6. systemd 目标

最少拆为：Host service、可选 renderer service（P0 可同进程）、受控关机/恢复钩子。要求：

- 非 root 运行，最小文件/设备权限。
- 明确工作目录、数据目录、角色目录、环境文件权限和重启策略。
- BLE/网络未就绪时 Host 进入 Degraded，不启动风暴。
- 日志由 journald/轮转控制；原始传感默认不持久化。
- shutdown coordinator 能先停新工作、checkpoint，再让 systemd/Linux 关机。

具体 unit 在 Orange Pi 镜像身份冻结后生成；当前不提交猜测的设备路径和用户组。

## 7. 板端验收

- 20 次冷启动全部进入最低可用 UI/维护态。
- 断网仍加载角色、状态和本地闭环。
- 两小时屏亮 + Wi-Fi + 双 BLE 无驱动崩溃或明显热降频。
- 1,000 条 DeviceEvent 无丢失，事件到本地画面延迟达到`ROADMAP.md`门槛。
- 数据库/日志上限、正常与异常断电恢复有证据。
- 资源数据决定是否启用 OCLive Resource Coordinator 的硬件特化策略；P0 只观察，不据未测估算抢占。

## 8. 安全

- loopback Web 只绑定本机，严格 Origin/Host/CSP/会话凭证和 UiIntent 白名单。
- 环境文件、BLE bond、Host registry、角色/状态库权限分离；诊断导出脱敏。
- 外部模型失败或密钥缺失只降级动态表达，不影响启动、感知、SystemCue 和关机。
