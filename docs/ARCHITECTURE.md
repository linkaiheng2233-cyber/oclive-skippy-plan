# 架构边界

## 1. 目标

器灵层把连续、嘈杂的物理信号转换成低频语义事件，再把即时确定性屏幕反馈和较慢的角色生成安全地组合起来。它不是第二套 OCLive 编排器，也不是硬件驱动集合的别名。

```text
Raw sensors
  │  GPIO / ADC / I2C / SPI / mock input
  ▼
Node / driver adapters
  │  filter / debounce / edge / aggregate
  ▼
SensorObservation v0.1
  │  BLE GATT or local adapter
  ▼
Rail-core sensor fusion
  ▼
DeviceEvent v0.1
  ▼
Spirit state machine ────────→ immediate VisualCue
  │
  └─ low-frequency trigger ─→ OCLive turn ─→ dynamic role expression
                                      │
                                      └────→ output arbiter
```

## 2. 状态机

v0.1 状态闭集：

```text
Sleeping → Held → Ready → Active
    ▲        │       │       │
    └────────┴───────┴───────┘  device.put_down / timeout
```

- `Sleeping`：设备放置或长期静止，屏幕关闭/低亮。
- `Held`：设备被拿起，允许一次唤醒反馈。
- `Ready`：设备被举起或进入准备姿态。
- `Active`：收到触发事件；射击模式保持静默。

状态机必须处理重复、乱序、过期和断线重连，不得依赖 LLM 才能完成转换。

## 3. 输入边界

`schemas/sensor-observation.v0.1.schema.json` 描述单个物理节点已经滤波的低频事实，例如 `grip.engaged`、`motion.raised` 或 `dock.entered`。它可以来自 BLE 节点，也可以来自主机内置传感器，但不直接驱动角色。

`schemas/device-event.v0.1.schema.json` 只描述导轨主机融合后的语义 DeviceEvent。原始 IMU 采样、压力曲线、按钮抖动和心率波形不得进入这两个协议。节点断连时相关事实必须变为 `unknown`，不得无限沿用最后一次握持状态。

进入 OCLive 的传感器回合必须具备类型化来源：

```text
TurnOrigin = user | sensor | system
```

sensor 的默认副作用策略：

- 不持久化为用户聊天。
- 不抽取长期记忆。
- 不修改好感、关系和人格。
- 不执行用户情绪分析。
- 允许单独写设备审计日志。

当前 OCLive `SendMessageRequest` 尚未具备完整来源/副作用字段；对应通用契约应在兄弟仓实现并保持普通客户端兼容。

## 4. 输出边界

`schemas/output-cue.v0.1.schema.json` 只定义 VisualCue。语音与触觉不进入 v0.1，避免在传感器—屏幕闭环完成前扩张输出栈。屏幕状态仲裁按以下顺序决定执行：

1. 丢弃已经超过 `ttl_ms` 的迟到输出。
2. 高优先级视觉状态可以打断低优先级。
3. 即时本地反馈先执行；动态回复到达后只能接管仍然有效的屏幕槽位。
4. 网络失败不回滚已经完成的设备状态转换。

## 5. 仓库分工

本仓：

- SensorObservation、DeviceEvent 与 Visual OutputCue schema。
- BLE 功能节点配对、状态同步和主机侧多传感器融合。
- 器灵状态机和屏幕状态仲裁。
- 桌面模拟器与事件回放。
- ARM Linux 服务、GPIO/IMU 与屏幕适配。
- systemd、日志导出、实机基准和结构文件。

OCLive 主仓：

- `TurnOrigin` / 回合副作用通用契约。
- 通用回合编排、角色包和视觉状态。
- sensor 回合不污染记忆/关系的内核测试。

角色资产：

- AN94、MP5 角色包留在 OCLive 角色包 SSOT 或市场。
- 本仓只保存硬件映射、缓存清单和明确授权的预渲染产物。
