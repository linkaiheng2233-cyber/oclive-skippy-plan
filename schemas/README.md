# 感知契约版本状态

## 当前裁决

当前基线与实现真源是 **`v0.3`**，由四份强类型契约组成：

- `SensorObservation v0.3`：节点/适配器的低频观察。
- `AngularRateSample v0.3`：**指向流的单帧角速率采样**（独立顶层消息，非观察）。
- `PerceptionState v0.3`：带revision、freshness和显式unknown的当前完整状态。
- `DeviceEvent v0.3`：只在有意义状态转换时产生的离散事件。

### 版本状态表

| 版本 | 文件 | 状态 |
|---|---|---|
| `v0.1` | `sensor-observation` · `device-event` | **从未交付**，早期探索草案。保留用于解释设计演进，**不是实现SSOT**，不建立 v0.1→v0.2 兼容adapter |
| `v0.2` | `sensor-observation` · `perception-state` · `device-event` | **已冻结为历史件**（2026-10-12）。三份**不再重新生成**：重生成会产生一份既自称 v0.2、又描述 v0.3 类型的文件 |
| **`v0.3`** | 上述**四份** | **当前基线**，由 `generated_schema_documents()` 产出 |

`output-cue.v0.1.schema.json`不属于感知契约重构；它会在RendererPort/ViewModel精确schema冻结时单独评审。

### 为什么 v0.2 → v0.3 没有兼容层

`ContractVersion` 的 `V0_2` 变体是被**替换**而不是保留的（见 `contracts/src/lib.rs`）。保留它会让一条标着 `"0.2"` 的消息仍然**解析通过**，而它的载荷只能按 v0.3 的类型解释——那是对外宣称支持一个本仓并未实现的语义。

v0.2 从未有过**已部署的生产者**：ESP32 固件尚不存在，唯一生产者是同仓模拟器，与本仓同改。因此不存在需要承接的兼容负担，做法与本仓对 v0.1 的处理先例一致。裁决全文见 `docs/DECISIONS.md` 的 **ADR-067**。

## v0.3 闭集

SensorObservation variants：

- `GripContact { grip: front|rear, contact, strength_milli }` ← **v0.3 新增判别字段**
- `MotionClass { motion }`
- `OrientationEstimate { gravity_mg_x, gravity_mg_y, gravity_mg_z, quality_milli }`
- `AuxControlContact { control_id: primary_trigger, contact }`
- `ShockObserved { severity_milli }`（diagnostic only）

**`GripContact.grip` 为什么是必填**：双板形态下「前握还是后握」可由**哪个 `node_uid` 上报**推断；P0-A 单板形态下两个 FSR 挂在同一节点上，该推断不成立——判别必须来自**载荷自身**，否则内核收到两条 `GripContact` 时无法区分。这与 `AuxControlContact.control_id` 是同一范式。

AngularRateSample（**独立顶层消息，不是 `SensorObservation` 的变体**）：

- `{ contract_version, node_uid, boot_id, sequence, node_monotonic_ms, rate_mdps_x/y/z, quality_milli }`
- 三轴角速率单位为**毫度每秒**（milli-degrees per second）；符号与轴向约定**尚未经实机标定验证**。
- 它**不进 `perception-core`**、**不融合进 `PerceptionState`**、**没有对应的 `Capability` 成员**。理由见 ADR-067 §二.5：100 Hz「最新即有效」的控制流与低频可追溯的语义事实是两种东西；合流会让瞄准流的丢包/乱序命中按节点共用的 `SequenceCollision`/`SequenceRegressed` 账本，从而打断同一帧的握持与姿态融合。

PerceptionState facts：

- `carrier_state: Fact<Standby | Held | Ready>`
- `front_grip / rear_grip: Fact<Contact>` ← 这两个是**语义槽位**（前手握持 / 后手握持），不是"前节点/后节点"；单板形态下由同一节点填充
- `motion: Fact<Idle | Moving>`
- `pose: Fact<Raised | Lowered>`
- `primary_control: Fact<Contact>`
- bounded `node_health`

DeviceEvent kinds：

- `carrier.held_entered / carrier.held_exited`
- `carrier.ready_entered / carrier.ready_exited`
- `control.primary.engaged / control.primary.released`

NodeStatus、interaction_mode、Host睡眠/生命周期、角色状态、低电和shock不属于DeviceEvent。不存在`Active` carrier state、`device.triggered`、`weapon.fired`或任何actuator variant。

## v0.3 落地门

每份schema必须同时具备Rust DTO、有效样例、逐类无效样例和自动校验。禁止任意payload透传；每个variant都要固定电压/物理单位或语义单位、范围、unknown/freshness、identity/version和时间字段。`mode.changed`由Host拥有，节点连接、低电与系统故障进入PerceptionState/SystemCue，不伪装成角色DeviceEvent。

当前实现位于`crates/ailive-gun-spirit-contracts`：Rust类型生成并提交四份v0.3 Schema，`schemas/fixtures/v0.3`提供 **5 个有效和 9 个无效**首批样例，测试同时检查反序列化、语义不变量及生成结果零漂移。OrientationEstimate固定点最终范围、完整NodeStatus/NodeHello和更多跨契约回放仍按ROADMAP继续实现，不能把当前首批DTO描述成全部协议已完成。

**节点拓扑的约束在 `PerceptionProfile`，不在契约里**：契约层不做拓扑校验（它只看到单条消息）。「同一 `Capability` 至多一个 owner」由 `perception-core` 的 profile 构造期强制，见 ADR-065 / ADR-067。

生成或检查：

```powershell
cargo run -p ailive-gun-spirit-contracts --example generate_schemas
cargo run -p ailive-gun-spirit-contracts --example generate_schemas -- --check
cargo test -p ailive-gun-spirit-contracts
```

## 真源与生成

Rust类型是语义真源。实现时由`ailive-gun-spirit-contracts`确定性生成并提交JSON Schema；CI重新生成到临时目录并要求与仓库文件零差异（**只比当前版本**——历史版本已冻结）。BLE二进制和Web JSON是codec，不是独立契约；BLE实现必须通过共享golden vectors对拍字段、边界值、未知枚举、截断帧和版本拒绝行为。

## 时间与未知态

- 节点字段：`node_uid`、`boot_id`、`sequence`、`node_monotonic_ms`。
- Host接收字段：`host_boot_id`、`received_monotonic_ms`。
- 节点时间只在同一node/boot内排序；不同节点时间禁止直接比较。跨节点TTL和融合使用Host单调时间，wall-clock只用于诊断。
- **`sequence` 是每个节点一条共用账本，不按传感器分别计数**：同一节点内 `sequence` 相同而载荷不同即 `SequenceCollision`。单板形态下 ESP32 必须对全部观察种类使用同一个单调序号，否则第一条就撞。
- PerceptionState事实使用`Known<T>`或`Unknown`判别联合。P0 unknown reason闭集：`never_observed`、`node_offline`、`stale`、`uncalibrated`、`sensor_fault`。
- Unknown不得携带可被融合或bridge读取的last-known value。需要排错时，把最后值放进隔离诊断摘要并明确标为historical。

## 证据强度与来源

- Known Fact使用`confidence_milli`整数`0..=1000`表示规则/校准下的证据强度或判定余量，不表示统计概率；Unknown禁止该字段。`AngularRateSample.quality_milli`同此口径。
- UI不得把`confidence_milli: 870`渲染成“87%概率”，只能根据版本化阈值投影为稳定/不稳定/需校准等用户语义。
- 每个Fact最多4个`source_ref`，每项只允许`observation_id`、`node_uid`、`capability`和`relation: supports | contradicts`。
- 原始值和完整Observation保存在有界诊断/回放缓冲区，通过observation id追踪；不得复制进每个PerceptionState快照。

## 协商与发布

- `NodeHello`报告明确的`supported_contract_versions`和对应capabilities；Host选择双方最高共同版本。不得仅凭SemVer、未知字段或“看起来兼容”推断支持。
- 无共同版本时节点进入`incompatible_protocol`且不参与融合。协商后未知/非法variant隔离当前帧并记录；有界窗口重复违规达到HostProfile阈值后隔离节点。
- **`NodeHello` 仍未实现**（`GS-CONTRACT-004`）。P0-A 单板形态可由 Host 侧静态配置能力集，不阻塞；真实板要"自己声明能力"时才需要。
- PerceptionState只在语义Fact/节点健康等权威状态变化时递增revision。新订阅、重连、renderer恢复和健康心跳可以重发相同revision完整快照。
- 相同revision重发不是新状态，禁止产生DeviceEvent或角色回合。心跳周期不是schema字段，由HostProfile和实测决定。
