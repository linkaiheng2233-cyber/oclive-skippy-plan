# BLE 无线功能传感器网络

**SSOT 范围**：本文负责 BLE 节点拓扑、会话、信任、传输与故障语义；具体电池/机械参数以`HARDWARE_IMPLEMENTATION_PLAN.md`为准，DTO 以 contracts Rust 类型和生成 Schema 为准。
**最后更新**：2026-08-27
**状态**：Current

## 1. 拓扑

```text
front rail node ─┐
                  ├─ BLE GATT → Host registry → observation validator → perception core
rear grip node ──┘
```

前后节点是两个独立信任主体，各有稳定`node_uid`、bonding key、boot id、sequence、capability 集合、校准 revision 和独立受保护 1S LiPo。增加节点只新增该设备的注册/密钥/白名单记录，不重新生成其它节点密钥，也不建立“理想枪械蓝图”。

## 2. 注册与信任

1. 用户通过高阻尼 MAINT 实体键开启有时限的配对窗口。
2. Host 一次只展示一个候选设备的物理确认信息。
3. 建立 BlueZ bond 后写入 Host 独立 node registry：稳定 identity、能力、允许位置/assembly、协议范围和校准引用。
4. 正常运行关闭未授权配对，只接受 registry + bond 都匹配的节点。
5. 撤销/换节点只影响该 identity；BLE 地址不得作为永久身份。

P0 的 BLE 信任用于避免附近同类配件误接，不宣传为安全关键认证。长期 BLE 密钥不进入 OCLive 角色/聊天数据库。

## 3. 会话与版本

节点连接后先发`NodeHello`，至少协商：协议版本集合/范围、node uid、boot id、capabilities、firmware、hardware/assembly/calibration revision。Host 只在存在版本交集且注册能力允许时接受 Observation；不兼容节点进入明确健康状态，不猜字段。

`boot_id`每次节点冷启动变化；`sequence`在一个 boot 内单调。Host 对重复、乱序、旧 boot 和截断帧分级处理：单帧丢弃不污染状态，连续协议违规限速/隔离，相关 Fact 按 TTL 进入 unknown。

## 4. 数据路线

P0 使用已连接 GATT notification，正常链路同时包含：

- 状态变化后的即时 edge/Observation。
- 低频完整 CurrentState snapshot，修复丢失 edge。
- 更低频电池/健康状态。
- 仅在有界维护模式启用的低速原始调试流。

采样率、通知频率、快照周期和重连退避都是`ESTIMATED` HostProfile 参数，必须通过功耗、p95 延迟、误判和 Wi-Fi/BLE 共存实测冻结；不写进跨组件 Schema。

断连期间不缓存重放握持、运动或辅助触点动作。重连后重新 Hello 并发送新鲜 CurrentState：

- 同 boot 表示链路恢复，仍按 sequence/TTL 检查。
- 新 boot 表示节点重启，旧滤波和 sequence 上下文作废。
- 断连期间变化次数最多作为诊断计数，不注入角色状态机。

## 5. 故障与降级

| 故障 | Host 行为 | 角色/前端行为 |
|------|-----------|---------------|
| 单帧 CRC/长度/variant 错误 | 丢弃帧、计数、保留最后仍新鲜事实 | 不产生事件；必要时状态条提示 |
| TTL 过期/断连 | 受影响 Fact → unknown，节点 health 降级 | 不把 unknown 当 released；基础角色仍可用 |
| 版本无交集 | 不接受 Observation，标 incompatible | 维护页给稳定原因 |
| 连续协议违规 | 限速、临时隔离；阈值实测后固化 | 不让事件风暴进入 OCLive |
| 低电/关机中 | 减少非必要流，保留状态/心跳到截止 | SystemCue 优先；不等待动态 AI |

## 6. 编码与测试要求

- BLE 二进制 codec 必须与 JSON DTO 语义一一映射，不能让任意 JSON 穿透。
- 建立主机 Rust 与节点固件共享 golden vectors：最小/最大值、unknown variant、截断、错误长度、版本拒绝、boot/sequence 与重复帧。
- fuzz/fault injection 覆盖长度、枚举、计数器和断线重连；任何坏节点都只能使自身能力降级。
- 场地测试至少同时运行两个节点与 2.4 GHz Wi-Fi，记录 RSSI、断连、恢复、延迟和功耗。

实测填写`test-worksheets/03-无线传感节点测试表.md`；协议实现顺序见`ROADMAP.md`。
