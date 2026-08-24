# BLE 无线功能传感器网络

## 1. 设计目标

外部传感器按作用做成独立 BLE 功能节点，贴装在握把、后托等位置；主导轨核心作为 BLE Central。节点不直接调用 OCLive，也不发送高频原始流，而是输出已经滤波的 `SensorObservation`，由主核心融合成 `DeviceEvent`。

```text
Grip Node ───────┐
Shoulder Node ───┼─ BLE SensorObservation ─→ rail core fusion ─→ DeviceEvent
Core IMU ─────────┘                                      │
                                                        └→ screen / OCLive
```

节点按功能拆分，不按单颗传感器拆分：一个握持节点可以同时拥有压力片、电容触摸、温度补偿和电池测量，不需要四个 BLE 地址。

## 2. v0.1 节点拓扑

| 节点 | 传感器 | 输出观察 | 状态 |
|------|--------|----------|------|
| Rail Core | 6 轴 IMU | `motion.moving/raised/lowered/idle`、试验性 `impact.estimated` | 必做 |
| Rail Core | 物理测试键 | `trigger.test` | 必做 |
| Grip Node | 薄膜压力传感器 + BLE MCU | `grip.engaged/released` | 首个无线节点 |
| Shoulder Node | 压力/接触传感器 + BLE MCU | `shoulder.engaged/released` | 握持节点通过后再做 |

v0.1 不设置磁性放置底座、充电仓或外部磁体；套件在任何场地只依赖自身传感器完成状态判断。

## 3. 为什么握持节点优先用压力而不是只用电容

- 电容触摸体积小，但手套、湿度、握把材料和接地条件会改变阈值。
- 薄膜压力传感器可以隔着外层感知压力，更适合判断「真的握住」；代价是需要模拟前端、受力结构和逐节点校准。
- 若单一压力点存在握姿盲区，可先调整力集中结构，再考虑双点，不在第一版直接增加节点数量。

Tekscan A201 类薄膜传感器厚度约 0.203 mm、感应区约 9.5 mm，但官方也明确建议使用运放调理与力集中结构来改善线性和重复性。因此节点大小的关键不是压力膜厚度，而是受力结构、模拟前端、电池和固定方式。

## 4. BLE 传输策略

v0.1 使用已连接的 GATT notification，而不是依赖无连接广播承载动作：

- 节点配对后分配稳定 `node_id` 与能力清单。
- 状态变化立即通知；普通时刻不连续上传高频原始采样。
- 每条观察带本地单调时间、递增 `sequence` 和唯一 `observation_id`。
- 主核心去重并确认节点在线；断连时相关事实进入 `unknown`，不能凭最后一次握持永久推断。
- 节点周期性上报 `node.status`：电量、固件版本、能力、校准版本和故障码。
- 重新连接后只补状态快照，不重放已经过期的历史动作。

BLE 节点需要 bonding/白名单，避免附近另一套设备的节点被错误接入。该措施服务于可靠性和隔离，不代表安全关键认证。

## 5. 状态融合

精准状态来自多证据融合，不把任何一个传感器当作绝对真相：

| 设备状态 | 主要证据 | 回退/否决条件 |
|----------|----------|---------------|
| Standby | grip released + motion idle 超时 | grip 未知时延长超时，只允许低置信度待机 |
| Held | `grip.engaged`，或明显运动的临时低置信度推断 | grip released 且持续静止后退出 |
| Ready | grip engaged + `motion.raised` 在稳定窗口内成立 | 姿态降低、握持释放或置信度不足 |
| Active | Ready + `trigger.test`；未来可融合 impact estimate | v0.1 不把震动估计宣称为真实击发 |

握持节点负责「手是否在握」，主模块 IMU 负责「载体是否在移动、静止或被举起」。两类证据由状态机组合：用户即使静止瞄准，只要仍在握持就不会进入待机；挂在身上移动但没有握持时，也不会被误判为 Ready。

## 6. 尺寸预算

以下是设计目标，不是已完成结构尺寸：

| 对象 | 原型尺寸目标 | 定制版目标 | 最大影响因素 |
|------|--------------|------------|--------------|
| Grip Node 电子盒（不含压力膜） | ≤40 × 28 × 12 mm | CR2032：≤36 × 26 × 9 mm；CR1632 紧凑版：≤30 × 22 × 8 mm | 电池、天线净空、固定方式 |
| Shoulder Node | ≤40 × 25 × 10 mm | ≤30 × 20 × 8 mm | 受力面积、电池、缓冲层 |
| 压力感应区 | 约 10 mm 级起步 | 由握姿实测决定 | 力集中结构与覆盖盲区 |

器件本体远小于完整节点：

- Nordic nRF52832 芯片封装可到 6 × 6 mm QFN 或约 3.0 × 3.2 mm WLCSP；官方模块列表也有约 15 × 6 mm 的 nRF52832 模块。
- nRF52840 官方模块列表中存在约 7 × 9 mm 的模块，证明射频模块不是唯一尺寸瓶颈。
- Bosch BMI270 为 2.5 × 3.0 × 0.8 mm。

最终节点通常由电池、天线 keep-out、受力结构和外壳主导，而不是传感器芯片。原型优先使用带天线的 BLE 模块；只有尺寸实测确实不够，才进入裸 SoC 自定义射频 PCB。

## 7. MCU 与功耗方向

握持节点优先评估 nRF52832 级 BLE MCU：其 ADC、低功耗比较器、GPIO、I2C/SPI 和内存足以完成压力采样、滤波、校准、压力唤醒和通知。nRF52840 可作为开发便利或需要更多固件空间时的备选，不因参数更高默认进入量产节点。

Nordic 官方规格给出的 nRF52832 System OFF 电流可低至亚微安级、LPCOMP 可从深睡中唤醒，而 BLE 收发峰值约 5 mA 级，因此节点必须由压力事件唤醒并尽量睡眠，不持续流式上传压力曲线。

下场默认电源是可更换 CR2032，CR1632 仅为紧凑备选；不依赖磁吸充电，不在握把受力区放软包锂电。纽扣电池的标准放电电流远低于射频瞬时峰值，必须预留储能与测量位置，并在低温、电量末期和重连脉冲下验证 brownout。详细决定、续航工况和机械要求见 `POWER_BUDGET.md`。

## 8. 原型顺序

1. 用软件模拟 `grip.engaged/released`，完成状态融合测试。
2. 使用现成 BLE 模块 + 压力片 + 桌面电源做 Grip Node V0。
3. 完成压力零点、按压力度和释放迟滞校准。
4. 加 CR2032 电池夹，实测连接、延迟、射频脉冲、低温和续航；再与 CR1632 做 A/B。
5. 做握把表面固定件，检查不同手型与手套。
6. Grip Node 通过后再复制架构做 Shoulder Node。
7. 只有原型体积成为真实问题时，才设计定制 PCB。

## 9. 官方器件资料

- [Nordic nRF52832 产品规格](https://docs.nordicsemi.com/r/bundle/ps_nrf52832/page/nrf52832_ps.html)
- [Nordic nRF52832 机械尺寸](https://docs.nordicsemi.com/r/bundle/ps_nrf52832/page/mec_spec.html)
- [Nordic nRF52832 第三方模块尺寸列表](https://www.nordicsemi.com/Products/nRF52832/Modules)
- [Nordic nRF52840 模块尺寸列表](https://www.nordicsemi.com/Products/nRF52840/Modules)
- [Bosch BMI270](https://www.bosch-sensortec.com/en/products/motion-sensors/imus/bmi270)
- [Tekscan FlexiForce A201](https://www.tekscan.com/flexiforce-a201-sensor)
- [Murata CR2032 数据表](https://www.murata.com/-/media/webrenewal/products/batteries/micro/cr/standard/ds-cr2032-003-je_202307.ashx?cvid=20231214062707000000&la=en-us)
- [Murata CR1632 数据表](https://www.murata.com/en-global/products/productdata/8808561049630/CR1632-DATASHEET.pdf)
