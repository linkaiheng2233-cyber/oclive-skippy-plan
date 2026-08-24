# OCLive 四季宝器灵

**Skippy Spirit** 是 OCLive 的第一个实体角色宿主：把角色包、记忆、情绪和场景编排接入物理事件，让设备能感知用户拿起、举起、触发和放回，并通过小屏表情与状态作出符合人格的回应。

当前状态：**仓库与无头内核骨架已建立；硬件尚未启动。下一里程碑是器灵模拟器与类型化输入来源契约。**

首台原型载体固定为 **森柏龙 RADIAN MODEL 1**。实施顺序是桌面套件 → 套件外壳/导轨适配 → 上枪验证，不在软件和电子闭环完成前修改或连接载体内部机构。

套件机械目标是「一套电子核心 + 可更换导轨安装件」：首发覆盖带标准皮卡汀尼导轨的 wargame 载体，其它安装标准通过独立转接件扩展。传感器默认不走机内线，导轨主机使用内置 IMU，握把等远端感知使用可拆卸 BLE 功能节点；状态判断不依赖磁性放置底座。

## 项目分层

| 层 | 职责 | 所在位置 |
|----|------|----------|
| OCLive Core | 人格、记忆、情绪、场景、角色包与回合编排 | 兄弟仓 `../oclivenewnew` |
| Spirit Runtime | DeviceEvent、状态机、触发策略、屏幕状态仲裁与离线兜底 | 本仓 |
| Skippy Host | wargame 实体配件、ARM Linux、传感器与小屏适配 | 本仓后续阶段 |

本项目不做智能火控、弹道计算、自动瞄准、武器控制或军警用途。v0.1 只验证「器灵是否真的活了」。

## v0.1 闭环

```text
模拟/实体传感器
  → 节点/驱动层滤波、去抖、边沿检测
  → 低频 SensorObservation
  → 导轨主机多传感器融合
  → 语义 DeviceEvent
  → 器灵状态机与即时反馈
  → 必要时触发 OCLive Fast 回合
  → 屏幕表情与 HUD 状态
```

v0.1 只包含：

- 一个角色：AN94。
- 两种模式：把玩、射击。
- 四个核心动作：拿起、举起、触发、放回。
- 一套状态证据：Grip Node 判断握持，主机 IMU 判断移动、静止与举起；握持释放 + 持续静止进入待机。
- PNG 表情与极简 HUD。
- 断网时仍完整可用的本地屏幕反馈。

ASR、TTS、扬声器、预渲染音频、触觉输出、相机、AI 视觉、心率、精确弹药计数、Live2D、第二个及更多 BLE 功能节点和手机本地模型均在 MVP 之后评审。

## 当前骨架

本仓由 OCLive `robot-soul` 工厂模板生成，并已将生成器默认的桌面 Tauri 依赖替换为 `oclive_kernel_host` 纯无头依赖：

- `src/main.rs`：标准无头 OCLive HTTP 宿主。
- `src/main_monolith.rs`：Monolith 焊接入口。
- `roles/default`：RobotSoulPack 烟测夹具，不是正式 AN94 资产。
- `schemas/`：器灵输入/输出协议。
- `docs/`：架构、路线和决策记录。

当前 `Cargo.toml` 通过相对路径依赖同级 `../oclivenewnew`，适合本地协同开发。发布前将根据 OCLive crate 的发行方式改成锁定的 Git revision 或正式版本依赖。

算力采用主云端：板端运行传感器采集、DeviceEvent、状态机、轻量 OCLive Host、屏幕和断网降级；云端只负责 LLM 重推理。网络不可用时动态角色文本可以降级，但拿起、举起、触发、放回对应的屏幕反馈必须继续工作。

## 启动无头宿主

```powershell
$env:OCLIVE_HTTP_API_MOCK_LLM = "1"
$env:OCLIVE_ROLES_DIR = "$PWD\roles"
cargo run -- --port 8420
```

默认 API 端口为 `8420`。生成器的完整配置参考见 `CONFIG_REFERENCE.md`。

## 路线入口

- [架构边界](docs/ARCHITECTURE.md)
- [实施路线](docs/ROADMAP.md)
- [关键决策](docs/DECISIONS.md)
- [Linux/ARM64 适配路线](docs/LINUX_ARM64.md)
- [RADIAN MODEL 1 套件路线](docs/PROTOTYPE_KIT.md)
- [通用导轨机械接口](docs/MECHANICAL_INTERFACE.md)
- [传感器候选与实测计划](docs/SENSOR_OPTIONS.md)
- [BLE 无线功能传感器网络](docs/WIRELESS_SENSOR_NETWORK.md)
- [无线节点电池与功耗预算](docs/POWER_BUDGET.md)
- [SensorObservation v0.1 Schema](schemas/sensor-observation.v0.1.schema.json)
- [DeviceEvent v0.1 Schema](schemas/device-event.v0.1.schema.json)
- [OutputCue v0.1 Schema](schemas/output-cue.v0.1.schema.json)

## 许可证

本仓新增代码使用 MIT。OCLive 兄弟仓仍遵循其 Apache-2.0 许可证；正式角色图像、声音与文本内容分别声明授权，不自动继承本仓代码许可证。
