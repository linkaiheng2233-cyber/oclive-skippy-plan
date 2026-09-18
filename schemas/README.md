# 感知契约版本状态

## 当前裁决

首个实现与兼容基线是已经落地的`v0.2`，由三份强类型契约组成：

- `SensorObservation v0.2`：节点/适配器的低频观察。
- `PerceptionState v0.2`：带revision、freshness和显式unknown的当前完整状态。
- `DeviceEvent v0.2`：只在有意义状态转换时产生的离散事件。

现有`sensor-observation.v0.1.schema.json`和`device-event.v0.1.schema.json`从未交付，只是早期探索草案。它们保留用于解释设计演进，不是实现SSOT，也不建立v0.1→v0.2兼容adapter。

## v0.2 P0闭集

SensorObservation variants：

- `GripContact { contact, strength_milli }`
- `MotionClass { motion }`
- `OrientationEstimate { gravity_mg_x, gravity_mg_y, gravity_mg_z, quality_milli }`
- `AuxControlContact { control_id: primary_trigger, contact }`
- `ShockObserved { severity_milli }`（diagnostic only）

PerceptionState facts：

- `carrier_state: Fact<Standby | Held | Ready>`
- `front_grip / rear_grip: Fact<Contact>`
- `motion: Fact<Idle | Moving>`
- `pose: Fact<Raised | Lowered>`
- `primary_control: Fact<Contact>`
- bounded `node_health`

DeviceEvent kinds：

- `carrier.held_entered / carrier.held_exited`
- `carrier.ready_entered / carrier.ready_exited`
- `control.primary.engaged / control.primary.released`

NodeStatus、interaction_mode、Host睡眠/生命周期、角色状态、低电和shock不属于DeviceEvent。v0.2不存在`Active` carrier state、`device.triggered`、`weapon.fired`或任何actuator variant。

`output-cue.v0.1.schema.json`不属于本次69A感知三契约重构；它会在RendererPort/ViewModel精确schema冻结时单独评审。

## v0.2落地门

每份schema必须同时具备Rust DTO、有效样例、逐类无效样例和自动校验。禁止任意payload透传；每个variant都要固定电压/物理单位或语义单位、范围、unknown/freshness、identity/version和时间字段。`mode.changed`由Host拥有，节点连接、低电与系统故障进入PerceptionState/SystemCue，不伪装成角色DeviceEvent。

当前实现位于`crates/ailive-gun-spirit-contracts`：Rust类型生成并提交三份v0.2 Schema，`schemas/fixtures/v0.2`提供3个有效和6个无效首批样例，测试同时检查反序列化、语义不变量及生成结果零漂移。OrientationEstimate固定点最终范围、完整NodeStatus/NodeHello和更多跨契约回放仍按ROADMAP继续实现，不能把当前首批DTO描述成全部协议已完成。

生成或检查：

```powershell
cargo run -p ailive-gun-spirit-contracts --example generate_schemas
cargo run -p ailive-gun-spirit-contracts --example generate_schemas -- --check
cargo test -p ailive-gun-spirit-contracts
```

## 真源与生成

Rust类型是语义真源。实现时由`ailive-gun-spirit-contracts`确定性生成并提交JSON Schema；CI重新生成到临时目录并要求与仓库文件零差异。BLE二进制和Web JSON是codec，不是独立契约；BLE实现必须通过共享golden vectors对拍字段、边界值、未知枚举、截断帧和版本拒绝行为。

## 时间与未知态

- 节点字段：`node_uid`、`boot_id`、`sequence`、`node_monotonic_ms`。
- Host接收字段：`host_boot_id`、`received_monotonic_ms`。
- 节点时间只在同一node/boot内排序；不同节点时间禁止直接比较。跨节点TTL和融合使用Host单调时间，wall-clock只用于诊断。
- PerceptionState事实使用`Known<T>`或`Unknown`判别联合。P0 unknown reason闭集：`never_observed`、`node_offline`、`stale`、`uncalibrated`、`sensor_fault`。
- Unknown不得携带可被融合或bridge读取的last-known value。需要排错时，把最后值放进隔离诊断摘要并明确标为historical。

## 证据强度与来源

- Known Fact使用`confidence_milli`整数`0..=1000`表示规则/校准下的证据强度或判定余量，不表示统计概率；Unknown禁止该字段。
- UI不得把`confidence_milli: 870`渲染成“87%概率”，只能根据版本化阈值投影为稳定/不稳定/需校准等用户语义。
- 每个Fact最多4个`source_ref`，每项只允许`observation_id`、`node_uid`、`capability`和`relation: supports | contradicts`。
- 原始值和完整Observation保存在有界诊断/回放缓冲区，通过observation id追踪；不得复制进每个PerceptionState快照。

## 协商与发布

- `NodeHello`报告明确的`supported_contract_versions`和对应capabilities；Host选择双方最高共同版本。不得仅凭SemVer、未知字段或“看起来兼容”推断支持。
- 无共同版本时节点进入`incompatible_protocol`且不参与融合。协商后未知/非法variant隔离当前帧并记录；有界窗口重复违规达到HostProfile阈值后隔离节点。
- PerceptionState只在语义Fact/节点健康等权威状态变化时递增revision。新订阅、重连、renderer恢复和健康心跳可以重发相同revision完整快照。
- 相同revision重发不是新状态，禁止产生DeviceEvent或角色回合。心跳周期不是schema字段，由HostProfile和实测决定。
