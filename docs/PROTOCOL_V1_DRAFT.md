# P0-A USB / Wi-Fi 最小帧协议草案（S2）

**状态**：`CANDIDATE`（2026-11-13；S2 台架草案，未冻结）
**最后核对**：2026-11-13
**适用阶段**：P0-A，CP2（最小联机语义）与 CP5（USB → Wi-Fi 闭环）
**唯一职责**：定义 P0-A 在 USB-CDC 与 TCP 上**共用**的最小帧封装、消息类型、会话基线、去重与按类型分派的校验规则。
**不负责**：v0.3 载荷字段语义（真源为 `crates/ailive-gun-spirit-contracts`）· 正式 `NodeHello` / `AssetRef`（`GS-CONTRACT-004`）· 角色表达内容与 OCLive 回合（兄弟仓）· 硬件电气与 Wi-Fi 配置 · 二进制 codec 的最终库选型。
**上游**：[ADR-067](DECISIONS.md)（契约 v0.3、角速率独立流）· [ADR-069](DECISIONS.md)（板上感知、USB/Wi-Fi 同语义）· [P0A_SOFTWARE_DESIGN.md](P0A_SOFTWARE_DESIGN.md) §3–§5 · [ARCHITECTURE.md](ARCHITECTURE.md) 的协商与隔离规则。
**下游**：CP2 台架测试程序、板端 `comms` 任务、后续 S3 正式协商。

> 本文是**临时约定**，不是契约版本，也不修改任何 Rust 类型、`schemas/` 或既有文档。**帧协议版本 1** 与**载荷契约版本 0.3** 是两件独立的事：前者管封装、消息与会话，后者管 `PerceptionState`、`DeviceEvent`、`AngularRateSample` 的字段语义。S2 实现并实测后才讨论冻结；正式 `NodeHello` 与 `AssetRef` 落地时由 S3 替换。

---

## 目录

1. 范围、术语与不做什么
2. 传输无关性
3. 帧格式
4. 载荷编码
5. 消息类型
6. 会话、基线与去重
7. 按类型分派的校验
8. 标识符
9. 错误处理与降级
10. 台架最小实现清单
11. 未决问题
12. 参考与变更记录
附录 A：HELLO 二进制布局（候选）
附录 B：EXPR / ACK / NACK / PING 固定布局（候选）
附录 C：二进制枚举标签（候选）

---

## 1. 范围、术语与不做什么

### 1.1 范围

本文覆盖 P0-A 的**枪侧板（ESP32-S3，感知生产者）与 PC（OCLive 宿主）之间**的最小联机协议：

- 同一套语义跑在 **USB-CDC** 与 **TCP** 两种字节流上；
- 上行：`PerceptionState` 快照、`DeviceEvent` 事件、可选 `AngularRateSample` 高频流；
- 下行：表情 ID 指令、心跳与确认；
- 会话建立、重连、基线、去重、版本与故障处理。

不覆盖：P0-B 的 BLE 多节点拓扑、相机/瞄准视觉、OCLive 内部回合、角色包与 `AssetRef`、TLS/账号体系、固件升级。

### 1.2 术语

| 术语 | 含义 |
|---|---|
| 帧（frame） | 本文定义的 `magic/ver/type/flags/len/payload/crc` 封装单位。 |
| 载荷（payload） | 帧内承载的一条消息正文，二进制或 JSON 文本，由 `len` 定界。 |
| 会话（session） | 一次链路建立后，从 `HELLO` 协商成功到链路断开/复位之间的状态。 |
| 基线（baseline） | 首连或重连后收到的**第一份当前快照**；它定义 PC 的起点，不回放历史。 |
| 可靠消息 | 需要 ACK、失败按同一字节重试的消息：`HELLO`、`EVENT`、`EXPR`。 |
| 最新值消息 | 不 ACK、不重传、旧帧无价值：`STATE`、`AIM`。 |
| 控制消息 | 固定小载荷：`ACK/NACK`、`PING/PONG`。 |
| S1 / S2 / S3 | S1 = 已实测的 ASCII 行协议；S2 = 本文帧协议草案；S3 = 正式 `NodeHello` + `AssetRef` 协商。 |

### 1.3 与 S1、S3 的关系

- **S1 保留**：当前板上固件仍是 ASCII 行协议（`PING/STATE/STEP/GET STATE/TOUCH/MOOD`），用于点屏、触摸和表情联调。S2 固件是**另一套构建/运行模式**，不要求 S1 与 S2 在同一个串口上同时可用。
- **不得混用**：PC 测试程序在同一连接上要么按 S1 行解析，要么按 S2 帧解析，**不得**先试一种再自动切另一种。若 S2 测试程序收到 `READY JC3248W535 1` 之类的 ASCII 行，应报告「板上是 S1 固件」并退出，而不是当作坏帧反复重连。
- **S3 替换而非兼容**：`GS-CONTRACT-004` 的正式 `NodeHello`（版本 + 能力协商）与 `AssetRef`（表达下行）实现后，S2 的 `HELLO` 与 `EXPR` 由新帧版本替换。S2 没有已部署的生产者（除台架），因此**不承诺向后兼容**；只保留 golden vectors 作为回归证据。

### 1.4 规范性用语

- **必须 / 禁止**：实现不符合即视为协议错误。
- **应当 / 不得**：默认遵守；偏离必须有记录与测试理由。
- **可以**：可选实现。
- 文中「设备」指枪侧 ESP32-S3；「PC」指 Windows 台架测试程序或 OCLive 宿主桥接层。

---

## 2. 传输无关性

### 2.1 一条语义，两种承载

USB-CDC 与 TCP 都只是**有序字节流**。协议栈分为两层：

```text
应用语义层：HELLO / STATE / EVENT / EXPR / ACK / NACK / PING / PONG / AIM
              ↑ 同一套消息、同一套会话规则、同一套校验
帧层：       magic / ver / type / flags / len / payload / crc16
              ↑ 同一套字节布局
传输适配层： USB-CDC 读线程        TCP socket 读线程
              ↑ 只负责开闭、读写、超时、重连；不解释消息
```

传输适配层**只搬运字节**。它不得自行拆消息、补消息、重排、去重、重放或改变校验结果。

### 2.2 传输层吸收的差异

| 差异 | USB-CDC | TCP | 由谁吸收 |
|---|---|---|---|
| 连接建立/断开 | PC 打开/关闭 COM 口；设备侧 DTR 可能变化 | PC 监听，设备作为客户端 connect；socket 关闭即断链 | 传输适配层 |
| 重连与退避 | 重新枚举/打开串口 | 重新 connect | 传输适配层；默认 0.5/1/2/5 s 封顶 |
| 半包/粘包 | USB 包 64 B，一次 `read` 可能只有半个帧 | TCP 任意分段/合并，一次 `read` 可能有多个帧 | **帧解析器**，不是传输层 |
| 读超时 | 串口读超时 | socket 读超时/`select` | 传输适配层；超时后交给帧解析器决定丢弃/断开 |
| 传输重传 | 无 | TCP 自身会重传丢失段 | TCP 栈；应用层不得依赖它做消息语义，见 §2.6/§5.8 |
| 最大传输单元 | USB 全速 64 B 包 | 受 MSS/MTU 影响，可能被分片 | 帧解析器按字节流重组 |
| 发现/寻址 | COM 口号或 VID/PID | IP/端口；S2 用静态配置 | 传输适配层/部署配置；S2 不做 mDNS 发现 |
| 链路错误计数 | 串口错误、断开 | socket 错误、RST、超时 | 传输层记录；**不计入应用协议违规** |
| 背压 | 设备 CDC 缓冲有限 | TCP 发送窗口 | 传输层 + 发送队列策略（§2.6） |

### 2.3 必须逐项一致的部分

以下内容**不得**因 USB/TCP 而不同：

1. 帧字节布局、字节序、CRC、最大帧长；
2. 消息类型码、方向、载荷语义；
3. 载荷编码模式（二进制/JSON）及其选择规则；
4. `HELLO` 协商字段与 ACK 规则；
5. 首连/重连基线、`boot_id` 判定、事件不回放；
6. `event_id` 去重、`revision` 单调、`STATE`/`EVENT`/`AIM` 的**分类型校验**；
7. 标识符约束与长度上限；
8. 故障处置（丢弃/NACK/断开/隔离）及其计数；
9. 心跳语义与「断连即本地预设」的降级行为。

**可以**因传输不同而调整的只有：连接建立方式、重连退避的具体秒数、读超时具体值、心跳周期的 profile 值（默认值仍应一致，且必须记录）。这些属于部署参数，不属于消息语义。

### 2.4 连接角色与生命周期

**USB-CDC**

- PC 是 USB 主机，打开串口（COMx）；设备是 CDC ACM 设备。
- 串口参数按 115200 8N1 配置即可，USB CDC 实际不依赖波特率；不要依赖波特率做流控。
- 设备**可以**观察 DTR 判断主机是否打开端口，但**不得**把它当作唯一链路判据；心跳与 `HELLO` 才是权威。
- USB 断开可能只是拔线，也可能是整板掉电。整板掉电会改变 `boot_id`；仅拔线不会。

**TCP**

- 建议 PC 作为 TCP 服务器监听，设备作为客户端连接 PC。理由：PC 的 IP/端口更稳定，设备只需配置一个目标地址；PC 侧防火墙只需放行一个入站端口。
- 候选端口 `47800`，仅作草案值；端口与「谁监听」列入 §11 未决问题。
- 设备侧静态配置：Wi-Fi SSID/密码、PC 地址、端口。S2 **不做** mDNS/广播发现。
- 设置 `TCP_NODELAY`，减少小帧被 Nagle 合并造成的延迟。
- 单客户端：S2 只允许一个 PC 连接。第二个连接直接关闭；多客户端列入未决问题。
- S2 **无 TLS、无认证**，只允许在受信任的本地台架/局域网使用，禁止把端口暴露到公网或不可信网络。

**链路建立**

1. 链路可用后，设备**立即**发送 `HELLO`（带 `ACK_REQ`）。
2. PC 在收到合法 `HELLO` 前，**禁止**接受 `STATE/EVENT/AIM`；收到则丢弃并计数（必要时 NACK）。
3. PC 校验 `HELLO`，发送 `ACK` 或 `NACK`。
4. 设备收到 `ACK` 后，**立即**发送当前 `STATE` 基线；收到 `NACK` 则关闭/退避，不发送任何状态。
5. PC 收到基线 `STATE` 后进入 `ESTABLISHED`。PC **不得**在基线之前下发 `EXPR`；设备对基线前的 `EXPR` 回 NACK（`not_ready`）。

**链路断开**

- 心跳超时（默认 3 s，§5.7）或传输层报错即视为断开。
- 设备：停止向 PC 发送，清空待发事件队列（**不缓存重放**），继续本地感知与表情预设；重新等待链路。
- PC：标记设备离线/事实过期，不伪造 `unknown` 事件、不触发角色回合；等待新 `HELLO`。
- 重连退避默认 0.5/1/2/5 s 封顶（测试起点，见 §11）。

### 2.5 字节流、半包与重同步

帧解析器必须按**字节流状态机**工作，不能假设一次读到的就是完整帧：

1. 在缓冲区中查找 `47 53`（`GS`）。找不到则丢弃多余字节并计数。
2. 找到后等待 7 字节帧头齐全。
3. 校验 `ver`、`type`、`flags`、`len`；`len` 超出上限立即按硬违规处理。
4. 等待 `len + 2` 字节（载荷 + CRC）齐全。
5. 校验 CRC；失败丢弃该候选帧并继续。
6. 成功后按 `type` 分派到类型解码器。

**半包**：任何时刻都不得把半个帧交给载荷解码器。读超时发生在帧中间时，丢弃该半帧并关闭/重连，不要「猜」剩余字节。

**粘包**：一次读到的缓冲区可以包含多个完整帧；解析器循环处理，直到不足一个完整帧为止。

**重同步**：

- 坏 magic：逐字节滑动查找下一个 `47 53`。
- 坏 CRC：丢弃**按当前 `len` 声明出的整帧**（帧头 + 载荷 + CRC），从下一字节继续查找 magic。这样避免在载荷内部误命中 magic；如果 `len` 本身被破坏，后续会重新同步或超时断开。
- 连续垃圾/无有效帧超过 1 s 或超过 4096 字节：关闭连接并计数，避免无限扫描。

### 2.6 发送优先级与「不阻塞」

板端 `comms_task` 与感知/UI 任务解耦，发送队列必须**有界**：

| 优先级 | 内容 | 队列满时 |
|---|---|---|
| 最高 | `HELLO`、`ACK/NACK`、`PONG` | 不得丢；必要时挤掉低优先级 |
| 高 | `EVENT` | 有界队列（建议 8 条）；满则丢最旧并计数，**不阻塞 reducer** |
| 中 | `STATE` | 只保留最新快照；可合并，不排队历史快照 |
| 低 | `AIM` | 直接丢弃/跳帧；绝不排队积压 |
| 最低 | 诊断日志 | 可丢 |

- 发送队列满**不得**阻塞 `sensor_task`、`reducer_task` 或 `ui_task`；即时屏幕反馈不等待 PC。
- 单写者：所有帧由同一个 `comms_task` 串行写出，禁止多任务并发写导致帧交错。
- `STATE` 可以合并：队列里已有旧快照时，用新快照替换它，而不是追加。
- `EVENT` 不能合并；边沿语义丢失后只能靠后续 `STATE` 收敛当前事实，不能伪造事件。
- `AIM` 与可靠消息共享链路时，设备应在两帧 `AIM` 之间优先插入待发的 `STATE/EVENT`，避免高频流把事件压在后面（TCP 上尤其重要）。

---

## 3. 帧格式

### 3.1 总布局

一帧 = **7 字节帧头** + `len` 字节载荷 + **2 字节 CRC**。总开销 9 字节。

| 偏移 | 长度 | 字段 | 类型 | 字节序 | 说明 |
|---:|---:|---|---|---|---|
| 0 | 2 | `magic` | u16 | 小端 | 线上字节必须是 `47 53`（ASCII `GS`）；按小端读为 `0x5347` |
| 2 | 1 | `ver` | u8 | — | 帧协议版本，S2 固定 `1` |
| 3 | 1 | `type` | u8 | — | 消息类型，见 §5.1 |
| 4 | 1 | `flags` | u8 | — | bit0 `ACK_REQ`，bit1 `IS_ACK`，bit2–7 保留为 0 |
| 5 | 2 | `len` | u16 | 小端 | 载荷字节数；不含帧头与 CRC；S2 范围 `1..=4096` |
| 7 | `len` | `payload` | bytes | — | 载荷；编码由 `HELLO.encoding` 决定 |
| 7+`len` | 2 | `crc16` | u16 | 小端 | 覆盖帧头 + 载荷；不覆盖自身 |

**最大帧长**：`7 + 4096 + 2 = 4105` 字节。`len` 字段虽然是 u16，但 S2 不使用 4096 以上的值。

> 实现提醒：`magic` 最容易写错。比较时直接比较两个字节 `0x47, 0x53`，或按小端读成 `0x5347`；不要按大端写成 `0x4753`。

### 3.2 字段规则

**`magic`**

- 固定线上字节 `47 53`。
- 不是可配置项；不做「宽容匹配」。

**`ver`**

- 帧协议版本，S2 为 `1`。
- 与载荷里的 `contract_version`（v0.3）无关。
- 任何不等于 `1` 的帧按硬违规处理（§9.2）：不尝试解析，直接关闭连接。原因：帧头布局本身可能已变，继续解析不安全。

**`type`**

- 取值见 §5.1。
- `0x00` 无效；`0x0A–0x7F` 保留；`0x80–0xFF` 实验/厂商扩展。
- 未知标准类型：丢弃该帧；若 `ACK_REQ` 置位则回 `NACK(unknown_type)`；不进入载荷解码。
- 未知实验类型：静默丢弃并计数，不回 NACK（避免高频放大），除非已通过协商显式启用。

**`flags`**

| 位 | 名称 | 含义 |
|---:|---|---|
| 0 | `ACK_REQ` | 发送方要求对方回 `ACK` 或 `NACK` |
| 1 | `IS_ACK` | 本帧是 `ACK`/`NACK` 响应 |
| 2–7 | 保留 | 发送方必须置 0；接收方忽略 |

- `HELLO`、`EVENT`、`EXPR` **必须**置 `ACK_REQ`。
- `STATE`、`PING`、`PONG`、`AIM` **禁止**置 `ACK_REQ`。
- `ACK/NACK` **必须**置 `IS_ACK`，**禁止**置 `ACK_REQ`；其他类型**禁止**置 `IS_ACK`。
- 违反上述规则：丢弃该帧并计数，不进入载荷解码；若该类型本身允许 ACK/NACK，再回 `NACK(bad_flags)`。重复达到阈值按 §9.3 隔离。

**`len`**

- 载荷字节数，小端 u16。
- S2 合法范围 `1..=4096`；`0` 对所有 v1 类型都非法。
- `len > 4096`：硬违规。**不得**先按 `len` 分配缓冲区；可以直接回 `NACK(too_large)` 后关闭，或直接关闭。两种情况都必须记录前 7 字节。
- `len` 与实际收到的字节数不一致：按截断处理（§9.2）。

**`payload`**

- 编码由 `HELLO` 协商（§4）；`HELLO` 自身例外，固定二进制（§4.4）。
- 不得包含 NUL 作为填充；不得用隐式对齐/填充字节。

**`crc16`**

- CRC-16/CCITT-FALSE：多项式 `0x1021`，初值 `0xFFFF`，不反射，xorout `0x0000`。
- 覆盖范围：帧头 7 字节 + 载荷 `len` 字节。
- 校验值 `123456789`（ASCII）应为 `0x29B1`，用于实现自检。
- 线上小端：先低字节、后高字节。
- 不要误用 CRC-16/MODBUS（反射、结果不同）。

### 3.3 帧示例

**PING，nonce = `0x11223344`**

```text
47 53 01 07 00 04 00 44 33 22 11 B2 9E
```

- `47 53` magic，`01` ver，`07` PING，`00` flags，`04 00` len=4；
- 载荷 `44 33 22 11` 是 `0x11223344` 的小端；
- CRC `B2 9E` 是 `0x9EB2` 的小端。

**PONG，回显同一 nonce**

```text
47 53 01 08 00 04 00 44 33 22 11 70 47
```

**ACK：确认一个 EVENT**

```text
47 53 01 05 02 03 00 03 00 00 0C AC
```

- `05` ACK，`02` flags=`IS_ACK`，`03 00` len=3；
- 载荷 `03` 表示被确认的是 EVENT，`00` 表示成功，`00` 表示无 detail；
- CRC `0C AC` 是 `0xAC0C` 的小端。

**NACK：EXPR 载荷过大**

```text
47 53 01 06 02 03 00 04 06 00 B8 5B
```

- `06` NACK，`04` 表示被拒绝的是 EXPR，`06` 是 `too_large`，无 detail；
- CRC `B8 5B` 是 `0x5BB8` 的小端。

**坏 CRC**：把上面 PING 的 `B2` 改为 `B3`，接收方必须丢弃整帧、计数 `crc_errors`，且不改变任何状态。

---

## 4. 载荷编码

### 4.1 目标：二进制

S2 的最终目标是用**紧凑二进制**编码载荷，理由：

1. **带宽与延迟**：`STATE` 1 Hz 不大，但 `AIM` 若启用是 50–100 Hz；JSON 的字段名、数字文本和空白会放大 5–10 倍，并增加 Wi-Fi 抖动下的排队时间。
2. **确定性**：定长整数、显式枚举、长度前缀让帧长可预测，便于嵌入式缓冲与 golden vectors 对拍。
3. **解析成本**：板端不需要通用 JSON 解析器，减少 flash/堆与失败面。
4. **无浮点歧义**：`AngularRateSample.rate_mdps_*` 是 i32，`Permille` 是 u16；二进制不引入 JSON 浮点/科学计数法解析差异。
5. **与契约一致**：v0.3 类型本身就是纯值类型；二进制 codec 只需做字段投影，不另造业务语义。

### 4.2 台架阶段可以先用 JSON 文本

S2 **允许**每个连接选择 JSON 文本载荷，用于低频台架：

- 板上 S1 已经验证过串口联调；JSON 可读，现场用普通串口工具即可看字段。
- `STATE` 1 Hz、`EVENT` 边沿、`EXPR` 偶尔下发，低频阶段带宽不是瓶颈。
- 语义尚未冻结时，JSON 让 PC 测试程序先跑通会话/去重/重连，不必等二进制 codec 定稿。

JSON 模式的约束：

- 载荷是**恰好一个 UTF-8 JSON 对象**；帧边界仍由 `len` 决定，不依赖换行。
- 可以在对象末尾带一个 `\n` 便于人读，但 `len` 把它算进去；接收方按「对象 + 可选尾随空白」解析。
- 禁止一帧多个 JSON 对象（NDJSON）、注释、BOM、尾随逗号。
- 字段名、枚举字符串、范围与 `schemas/*.v0.3.schema.json` 一致；`contract_version` 为字符串 `"0.3"`。
- 载荷 `len` 仍受 4096 上限约束；超过即按超长处理。JSON 模式更应使用短 ID；若某份合法快照仍超过 4096，设备必须改用二进制或缩短标识符，不得截断载荷。
- `AIM` **不提供** JSON 模式：高频流必须用二进制；若协商为 JSON，设备不得声明 `AIM` 能力。

> S2 的 JSON 模式**不是** S1 的换行行协议。S1 没有帧头/CRC；S2 的 JSON 仍套在帧里。两者不得在同一连接上混用（§1.3）。

### 4.3 两种编码不得混用同一连接

- 编码模式在 `HELLO` 中声明，**每个连接只选一种**：`1 = binary`，`2 = JSON text`。
- 从 `HELLO ACK` 之后开始，该连接上所有非 `HELLO` 载荷都使用同一编码。
- 不允许逐帧切换、不允许自动探测、不允许「二进制失败后回退 JSON」。
- 切换编码必须断开并重新 `HELLO`；新连接可以选另一种，但双方必须显式同意。
- 接收方收到不符合协商编码的载荷：丢弃 + `NACK(bad_payload)`（若可回）；重复则按协议故障隔离。

### 4.4 `HELLO` 是唯一的固定二进制载荷

存在一个先有鸡还是先有蛋的问题：接收方要先知道编码，才能解析载荷；但编码本身在 `HELLO` 里。

因此规定：**`HELLO` 的载荷永远是固定二进制布局**（附录 A）。接收方先用固定的 HELLO 解析器读取版本、能力、`boot_id` 与 `encoding`，再用该编码解析后续载荷。JSON 模式也不例外。

### 4.5 二进制编码的候选规则

以下是 S2 实现前的**候选约束**，最终以 CP2 冻结的 golden vectors 为准；Rust 类型仍是语义真源：

- 所有多字节整数**小端**；有符号整数用二进制补码。
- 不使用可变长整数（varint）；第一版用固定宽度，便于 C/Python/嵌入式实现。
- 字符串：`u8` 字节长度 + UTF-8 字节。标识符长度上限见 §8；协议 profile 硬上限 64 字节。
- 枚举：`u8` 标签，按 Rust 声明顺序从 1 开始（附录 C）；`0` 保留/非法。
- `Fact<T>`：`u8` 标签（`0 = Unknown`，`1 = Known`）；Known 后接值、`observed_at`、`fresh_until`、`confidence`、`source_refs` 数量与条目；Unknown 后接 `reason`、`since_revision`。Unknown **不得**携带 last-known value。
- `PerceptionState` 字段顺序按契约声明：`contract_version`（major u8 + minor u8）、`host_boot_id`、`revision`、`produced_at_monotonic_ms`、六个 Fact、`node_health`。
- `node_health`：`u8` 数量（≤8）+ 每项 `node_uid` + `state`。
- `DeviceEvent`：`contract_version`、`event_id`、`host_boot_id`、`occurred_at_monotonic_ms`、`caused_by_perception_revision`、`kind`。
- `AngularRateSample`：`contract_version`、`node_uid`、`boot_id`、`sequence`、`node_monotonic_ms`、三个 i32、`quality_milli`。
- 解码后必须**恰好消费完** `len` 字节；有尾随字节即 `bad_payload`。
- 禁止隐式默认值、禁止把未知枚举值透传。

Codec 选型（手写定长布局 / `postcard` / CBOR）列入 §11；无论选哪种，都必须用**同一组 golden vectors**验证 Rust DTO 与 PC 测试程序的字节输出一致。

### 4.6 版本与 golden vectors

- **帧版本** `ver=1`：管帧头、类型码、flags、长度上限、CRC。
- **契约版本** `0.3`：管载荷字段语义。
- 两者独立协商；不得用 SemVer 或「看起来兼容」推断支持。
- 每个消息类型至少准备：1 个有效向量、边界长度向量、截断向量、未知枚举/坏 CRC 向量。
- golden vectors 同时服务 Rust 单测与 PC 测试程序；任何一侧改字节都必须先改向量并说明原因。

---

## 5. 消息类型

### 5.1 类型表

| `type` | 名称 | 方向 | 载荷 | 频率 | 可靠性 | `ACK_REQ` |
|---:|---|---|---|---|---|---|
| `0x01` | `HELLO` | 设备 → PC | 固定二进制 HELLO（附录 A） | 连接建立；USB 周期重发，TCP 最多 3 次 | 重试直到 ACK/NACK | 必须 |
| `0x02` | `STATE` | 设备 → PC | `PerceptionState` v0.3 | 1 Hz + revision 变化 + 重连基线 | 最新值优先，不 ACK | 禁止 |
| `0x03` | `EVENT` | 设备 → PC | `DeviceEvent` v0.3 | 边沿，低频 | 停等可靠，重试 3 次 | 必须 |
| `0x04` | `EXPR` | PC → 设备 | 表情 ID + hold + request_id（附录 B） | 低频 | 停等可靠，重试 3 次 | 必须 |
| `0x05` | `ACK` | 双向 | `ack_type` + `result=0` + detail | 响应 | 不重传 | 禁止 |
| `0x06` | `NACK` | 双向 | `ack_type` + `result≠0` + detail | 响应 | 不重传 | 禁止 |
| `0x07` | `PING` | 双向 | `nonce u32` | 心跳，默认 1 Hz | 不重传 | 禁止 |
| `0x08` | `PONG` | 双向 | 回显 `nonce u32` | 回应 | 不重传 | 禁止 |
| `0x09` | `AIM` | 设备 → PC | `AngularRateSample` v0.3 | 50–100 Hz，**可选** | 不重传，丢即丢 | 禁止 |
| `0x0A–0x7F` | 保留 | — | — | — | — | — |
| `0x80–0xFF` | 实验/厂商 | — | — | — | — | — |

`0x00` 无效。S2 只实现 `0x01–0x09`。

### 5.2 `HELLO`

**方向**：设备 → PC，连接建立后第一帧。
**目的**：在进入状态流之前，声明帧版本、契约版本、能力、设备身份与载荷编码。

字段（完整二进制布局见附录 A）：

| 字段 | 说明 |
|---|---|
| `frame_ver_min` / `frame_ver_max` | 支持的帧版本范围；S2 都是 `1` |
| `contract_versions` | 支持的契约版本列表；S2 为 `["0.3"]` |
| `encoding` | `1 = binary`，`2 = JSON text` |
| `contract_caps` | v0.3 `Capability` 位图：前握/后握/运动/姿态/主触点/冲击诊断 |
| `proto_caps` | 协议能力位图：`STATE`/`EVENT`/`EXPR`/`AIM`/`PING`/快照先行 |
| `node_uid` | 设备稳定身份（1..=64 字节 profile 上限；契约上限 128） |
| `boot_id` | 本次启动身份（1..=64 字节 profile 上限；契约上限 128） |
| `fw_version` | 可选固件版本，0 表示不提供 |

**规则**

- `HELLO` 必须置 `ACK_REQ`；载荷固定二进制，不受 `encoding` 影响。
- 设备发出后等待 ACK：超时 500 ms，重试使用**完全相同的字节**（同一 `boot_id`）。
- USB 没有可靠的「主机已打开端口」信号：设备应在上电/链路可用后每 1 s 重发 `HELLO`，直到收到 ACK；收到 NACK 或明确不兼容才停止。TCP 上重试 3 次仍无 ACK 则关闭 socket，按退避重连。
- PC 校验：帧版本有交集、契约版本有交集、编码支持、能力位合法、ID 长度合法。
- 成功：PC 回 `ACK(ack_type=HELLO, result=0)`；设备随后发基线 `STATE`。
- 失败：PC 回 `NACK`，`result` 说明原因，然后关闭连接；设备退避后重试。无共同契约版本时，PC 标记 `incompatible_protocol`，不得把任何观察送入融合。
- PC 对重复 `HELLO`（ACK 丢失导致）必须幂等：无论处于 `SYNCING` 还是 `ESTABLISHED`，已接受过同一 `boot_id` 的 `HELLO` 则重新 ACK，不重复建会话、不清基线。
- `HELLO` 之前收到的任何非 `HELLO` 帧：丢弃；若重复，NACK 后关闭。

**与正式 `NodeHello` 的区别**：本 `HELLO` 是 S2 临时握手，不是 `contracts` 里的 `NodeHello`，也不写入 `schemas/`。正式协商见 `GS-CONTRACT-004`。

### 5.3 `STATE`

**载荷**：恰好一个 `PerceptionState` v0.3。

**发送时机**

1. `HELLO ACK` 后**立即**发送当前快照作为基线。
2. 每当 `revision` 发生变化，在发送该 revision 的任何 `EVENT` **之前**发送对应快照。
3. 无变化时每 1 s 至少一份快照（心跳式兜底）。
4. 重发同 revision 快照用于证据包络刷新（`observed_at/fresh_until/confidence/source_refs`）时，不得制造新 revision。

**规则**

- 不置 `ACK_REQ`，不需要 ACK；丢失由下一份快照修复。
- `STATE` 是**最新值消息**：队列里只保留最新快照，允许合并。
- `STATE` 本身**不得**触发 OCLive 角色回合；它只作为上下文与基线（§6.6）。
- `host_boot_id`、`revision`、`node_health`、`Fact` 的校验见 §7.3。
- 同一 revision 的语义载荷必须一致；只允许证据包络变化。语义不同即 `revision_collision`。

### 5.4 `EVENT`

**载荷**：恰好一个 `DeviceEvent` v0.3。

**发送时机**

- 感知状态机产生边沿后发送；必须在**该 revision 的 `STATE` 已在该连接上发出之后**发送。
- 同一连接内 `EVENT` 采用**停等**：一次最多一个未确认 `EVENT`；收到 ACK 后再发下一条。
- 发送失败：200 ms 超时，最多重试 3 次；仍失败则丢弃并计数，不无限重试。
- 设备在链路断开时**清空**待发事件队列；重连后不重放历史事件。

**规则**

- 置 `ACK_REQ`；PC 在验证并接受后回 ACK。
- PC 以 `(boot_id, event_id)` 去重；重复事件回 ACK，但**不得**再次触发回合。
- `caused_by_perception_revision` 在同一 boot 内必须单调不减；倒退按 `event_order` 拒绝。
- 事件 revision 必须不大于 PC 当前已知的 `STATE.revision`；若大于，PC 可短窗口等待对应快照（§7.4），超时 NACK `state_gap`，不得用未来 revision 排回合。
- 重连后的基线之前的事件视为陈旧：不处理，NACK `stale_event` 或静默丢弃 + 计数。
- 允许 revision 跳号（例如队列丢弃/链路丢失），因为快照会收敛当前事实；**不要求 revision 连续**（§6.4）。

### 5.5 `EXPR`

**方向**：PC → 设备，表达下行。
**目的**：下发一个**表情 ID**，不是正式 `AssetRef`。

固定载荷（附录 B）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `expr_id` | u8 | 1–7 对应 `neutral/happy/excited/shy/confused/sad/angry`；0 与 8–255 非法 |
| `flags` | u8 | 保留，S2 必须为 0 |
| `hold_ms` | u16 | 0 = 设备默认；1–60000 = 期望保持时长 |
| `request_id` | u32 | PC 每次请求递增；用于设备去重与 ACK |

**规则**

- 置 `ACK_REQ`；设备验证后回 ACK。
- 设备对 `request_id` 做有界去重（建议最近 32 条）：重复请求只重新 ACK，不重复改表情。
- `expr_id` 必须是设备已声明的表情；未知 ID 回 `NACK(bad_payload)`，屏幕不变。
- ACK 的含义是「已接收且语法有效并交给表情仲裁器」，**不保证**立即可见：系统 Overlay（掉线/低电/故障）可以抢占；触摸手动覆盖也可以按设备现有规则优先。
- PC 一次只保留一个未确认 `EXPR`；超时 200 ms、最多重试 3 次，重试使用同一 `request_id`。
- 不下发路径、URL、像素或任意文本；不引入 `AssetRef` 字段。正式 `AssetRef` 落地后由 S3 替换本消息。
- 设备端仍遵守「全设备只有一个当前表情 ID」与 300 ms 最短驻留；`EXPR` 不绕过系统 Overlay。

### 5.6 `ACK` / `NACK`

**载荷**（固定布局见附录 B）：

| 字段 | 类型 | 说明 |
|---|---|---|
| `ack_type` | u8 | 被确认/拒绝的帧类型：正常为 HELLO=1、EVENT=3、EXPR=4；对未知类型回 `unknown_type` 时为原始 `type` 值 |
| `result` | u8 | 0 = ACK；1–255 = NACK 原因码 |
| `detail_len` | u8 | 可选 detail 的字节数，0–160 |
| `detail` | UTF-8 | 可选，人读原因；建议 ASCII；不得含完整 MAC/秘密 |

**规则**

- `ACK/NACK` 必须置 `IS_ACK`，禁止置 `ACK_REQ`；不得再被 ACK。
- 只对**置了 `ACK_REQ`** 的帧发送；正常是 `HELLO`、`EVENT`、`EXPR`，也可以是带 `ACK_REQ` 的未知类型帧（仅回 `unknown_type`）。不对 `STATE`、`AIM`、`PING/PONG` 发送。
- 由于每个方向每种可靠消息只有**一个未确认窗口**，`ack_type` 足以关联，不需要额外序号。
- ACK 丢失：发送方按超时重发同一帧；接收方按 `event_id`/`request_id`/`boot_id` 去重并重新 ACK。
- NACK 不自动触发重传；发送方必须修正或放弃，并记录原因。
- `detail` 只用于诊断，不参与控制；接收方不得因 detail 内容改变行为。

**NACK 原因码**

| `result` | 名称 | 含义 |
|---:|---|---|
| 0 | `ok` | ACK |
| 1 | `bad_payload` | 载荷解析/字段校验失败 |
| 2 | `unknown_type` | 未知标准类型 |
| 3 | `unsupported_frame_version` | 帧版本不支持 |
| 4 | `unsupported_contract_version` | 契约版本不支持 |
| 5 | `not_negotiated` | 未完成 HELLO 就收到该消息 |
| 6 | `too_large` | 载荷超限 |
| 7 | `identity_mismatch` | `boot_id`/`node_uid`/`host_boot_id` 不匹配 |
| 8 | `revision_regressed` | STATE revision 倒退 |
| 9 | `revision_collision` | 同 revision 但语义不同 |
| 10 | `event_order` | EVENT revision/时间倒退 |
| 11 | `state_gap` | EVENT 的因果 revision 缺少对应 STATE |
| 12 | `not_ready` | 基线未完成/会话未就绪 |
| 13 | `busy` | 接收队列满/暂时无法处理 |
| 14 | `unsupported_encoding` | 不支持的载荷编码 |
| 15 | `bad_flags` | flags 与类型不匹配 |
| 16 | `stale_event` | 重连后早于基线的事件 |
| 17 | `internal_error` | 接收方内部错误 |

> 注：`revision_regressed`（8）与 `revision_collision`（9）当前不会由 `STATE` 触发 NACK——`STATE` 不 ACK；保留给未来或诊断日志。`unsupported_frame_version`（3）在 S2 通常是硬断开（帧头布局可能已变），只在能安全回写时使用。

### 5.7 `PING` / `PONG`

- 载荷：`nonce u32` 小端（4 字节），`PONG` 原样回显。
- 双方都可以发起；默认由**设备**每 1 s 发 `PING`，PC 回 `PONG`。PC 在 1 s 内没有收到任何设备帧时也可以发 `PING`。
- 同一方向一次最多一个未响应 `PING`；`PONG` 必须匹配最近的 `nonce`，不匹配则丢弃并计数。
- 心跳超时：默认 3 s 未收到任何合法帧（或 PONG），判定链路断开。
- `PING/PONG` 不置 `ACK_REQ`，不触发状态变化、不触发回合、不写入感知。
- 设备侧心跳用于进入离线预设；PC 侧心跳用于标记设备离线与事实过期。周期与超时值列入 §11 实测项。

### 5.8 `AIM`（可选高频流）

**载荷**：恰好一个 `AngularRateSample` v0.3。
**方向**：设备 → PC。
**默认**：关闭；只有 `HELLO.proto_caps` 声明且 PC 接受后才启用。

**规则**

- 50–100 Hz（目标值待实测），每帧一条样本；不批量、不压缩。
- 带宽估算：短 ID 时整帧约 70 B，100 Hz 约 7 KB/s；64 B ID 时约 170 B/帧、约 17 KB/s；若两个 ID 都接近 128 B，约 300 B/帧、约 30 KB/s。§8 的短 ID 约束是 AIM 可用的前提。
- **不置 `ACK_REQ`，不 ACK，不重传，不做去重，不做连续性/顺序校验。**
- 接收方可以在队列忙、帧过期或积分器已消费更新样本时**直接丢弃**当前 AIM 帧；最新值优先。
- 只做身份与范围校验：`node_uid`/`boot_id` 必须匹配当前会话；`quality_milli ≤ 1000`；长度/字段合法。
- **不得**送入 `PerceptionReducer`，**不得**更新 `NodeSession.last_sequence`，**不得**产生 `DeviceEvent`。
- 未协商时收到 AIM：静默丢弃 + 计数；不回 NACK（高频流回 NACK 会放大）。
- `AIM` 必须使用二进制编码；JSON 模式不得声明 AIM。
- 设备发送优先级最低；队列满时跳帧，不积压。
- TCP 上启用 AIM 时，TCP 自身的重传与队头阻塞可能让「旧帧」晚到；接收方仍按最新值丢弃。若实测发现可靠消息被 AIM 阻塞，应改为独立连接或后续 UDP，列入 §11。

**为什么不能校验连续性**：见 §7.6。

### 5.9 未知与保留类型

- `0x00`：无效，直接丢弃/关闭。
- `0x0A–0x7F`：保留。v1 接收方按未知标准类型处理：丢弃 + 可选 `NACK(unknown_type)`。
- `0x80–0xFF`：实验/厂商扩展。只有双方在 `HELLO` 之外另有约定时才可启用；v1 默认静默丢弃 + 计数。
- 任何情况下都不得把未知类型载荷转发给感知内核、OCLive 或角色表达层。

---

## 6. 会话、基线与去重

### 6.1 会话状态机

**设备侧**

| 状态 | 进入 | 允许发送 | 退出 |
|---|---|---|---|
| `LINK_DOWN` | 上电/断链 | 无（本地表情继续） | 链路可用 → `WAIT_ACK` |
| `WAIT_ACK` | 已发 HELLO | HELLO（重试） | 收到 ACK → `SYNCING`；NACK/超时 → 退避 |
| `SYNCING` | HELLO ACK | 基线 STATE | 基线发出 → `ESTABLISHED` |
| `ESTABLISHED` | 基线已发 | STATE/EVENT/AIM/PING/ACK/NACK/PONG | 断链/硬违规 → `LINK_DOWN` |
| `INCOMPATIBLE` | HELLO 被 NACK（版本/编码） | 无 | 重新配置/重启后重试 |

**PC 侧**

| 状态 | 进入 | 允许接收 | 退出 |
|---|---|---|---|
| `LISTEN/OPEN` | 启动/重连 | 仅 HELLO | 合法 HELLO → `SYNCING` |
| `SYNCING` | 已 ACK HELLO | 基线 STATE、PING、重复 HELLO | 首个合法 STATE → `ESTABLISHED` |
| `ESTABLISHED` | 基线完成 | 全部 | 断链 → `OFFLINE`；硬违规 → `QUARANTINED` |
| `OFFLINE` | 断链/心跳超时 | 无 | 新 HELLO → `SYNCING` |
| `QUARANTINED` | 连续违规 | 无 | 超时/人工确认后重试 |

### 6.2 首连

1. 设备发 `HELLO`（`boot_id=B`，能力/版本/编码）。
2. PC 校验并 ACK。
3. 设备发当前 `STATE`（`revision=R0`）作为**基线**。
4. PC 设置 `last_state_revision=R0`、`baseline_revision=R0`，清空待处理事件。
5. 之后只接受 `caused_by_perception_revision > R0` 的新事件；任何早于/等于基线的边沿都视为历史，不触发回合。
6. 设备在发基线前**不得**发送事件；PC 在基线前收到事件按 `not_ready` 处理。

### 6.3 重连

**同一 `boot_id`（链路恢复）**

- PC 保留：`last_state_revision`、EVENT 去重缓存、会话日志。
- PC 重置：AIM 积分器/滤波器（断链造成样本缺口）、发送中的 EXPR 等待状态（可重发同一 `request_id`）。
- 设备重发当前 `STATE` 作为新基线；PC 接受 `revision >= last_state_revision` 的快照。
- 设备**不重放**断链期间的事件；PC 不接受基线之前的事件。
- 若同 boot 的 `revision` 反而小于重连前的值：按协议故障处理（§9.3）。

**新 `boot_id`（设备重启）**

- `HELLO.boot_id` 与上个会话不同即判定重启。
- 即使在同一条 TCP 连接上出现不同 `boot_id` 的 `HELLO`，也按重启处理；设备应在新 boot 时主动关闭旧连接。
- PC 清空：`last_state_revision`、EVENT 去重缓存、待处理事件、AIM 积分器、所有旧 TTL 事实。
- 新会话以第一份 `STATE` 建基线；`revision` 允许从 0 重新开始。
- 设备重启后事件 ID 可以重新从 1 开始；因为 `boot_id` 变了，`(boot_id, event_id)` 仍然唯一。

**仅 TCP 重连/USB 重新打开不等于重启**：必须看 `HELLO.boot_id`。设备可能自供电，USB 重插不改变 `boot_id`；PC 重启也不改变设备 `boot_id`。

### 6.4 `revision` 规则

这里的「严格序号」定义为：**同一 boot 内单调不减、不重复**；**不要求连续**。

**STATE**

| 情况 | 处置 |
|---|---|
| `R > last` | 接受；更新快照与 `last=R`；允许跳号（快照是全量） |
| `R == last` 且语义相同 | 接受为证据包络刷新；不产生事件/回合 |
| `R == last` 但语义不同 | `revision_collision`；丢弃 + 计数；重复则隔离 |
| `R < last` | `revision_regressed`；丢弃 + 计数；重复则隔离 |
| 新 boot 的首个快照 | 直接建基线，不要求 `R=0` |

**EVENT**

| 情况 | 处置 |
|---|---|
| `caused_by_revision > baseline` 且 `>= last_event_revision` | 接受（需已有对应 STATE，§7.4） |
| `caused_by_revision < last_event_revision` | `event_order`；NACK + 丢弃 |
| `caused_by_revision <= baseline`（重连后） | `stale_event`；NACK/丢弃 |
| 重复 `event_id` | 重新 ACK，不处理 |
| `occurred_at_monotonic_ms` 倒退 | `bad_payload`/`event_order`；丢弃 + 计数 |
| 跳号 | 允许；计数 `event_gap`，由快照收敛事实 |

**为什么允许跳号**：事件可能在设备发送队列溢出或链路中断时丢失；`STATE` 是全量快照，能恢复当前事实。要求事件 revision 连续会把正常降级误判为协议故障，并诱发重放历史事件——那正是 §6.2 禁止的。

### 6.5 `event_id` 去重

- 去重键：`(boot_id, event_id)`；`boot_id` 变化即清空缓存。
- 缓存大小建议 128 条（FIFO/有界）；命中则**重新 ACK**，不再次触发回合。
- 同 boot 内相同 `event_id` 但载荷不同：`identity_mismatch`/`bad_payload`；丢弃 + 计数；重复则隔离。事件 ID 复用是固件 bug，不得静默接受。
- 去重必须在**任何**状态修改/回合排程之前完成；不得先改状态再发现重复。
- 重连后同一 boot 保留去重缓存；新 boot 清空（事件 ID 可重用）。

### 6.6 重复快照不触发角色回合

- OCLive 回合只由**新的、去重后的 `EVENT`** 触发；`STATE` 只作为上下文/基线。
- 相同 `revision` 的快照重发（证据刷新、心跳、重连）**不得**产生 `DeviceEvent`、不得排回合、不得改写角色情绪/记忆。
- 状态变化但没有对应边沿事件时，PC 只更新上下文，不主动生成回合；这是「瞬时屏幕反馈在板端、角色表达低频」的边界。
- 即使 `STATE` 中 `Fact` 值发生变化，只要没有对应 `EVENT`，也不得把快照桥接成伪事件。
- 该规则同时适用于首连基线、重连基线和同 revision 重发。

### 6.7 `boot_id` 与 `host_boot_id` 的临时映射【临时】

`PerceptionState` 与 `DeviceEvent` 的字段名是 `host_boot_id`，但 P0-A 中这两个对象由**板上感知内核**产生并直接上行（ADR-069）。这产生一个契约命名与部署现实之间的缺口，ADR-069 也明确要求「明确设备启动会话到主机契约的映射」。

本草案暂定 **方案 A**：

- `HELLO.boot_id` = 设备本次启动身份。
- `STATE.host_boot_id` 与 `EVENT.host_boot_id` **必须等于**当前会话的 `HELLO.boot_id`。
- PC 把它理解为「**感知生产者会话 ID**」；PC 自己的 boot ID 只写入本地日志/会话元数据，**不得**在验证前重写载荷字段。
- 这样 `InputScheduler` 的 `event.host_boot_id == perception.host_boot_id` 检查可以原样成立，事件与快照的会话一致性能被验证。
- 板端实现：`STATE` 与 `EVENT` 的 `host_boot_id` 都由板端填为设备 `boot_id`；`perception-core` 只产出 `DeviceEventKind`，板端 `comms` 负责生成 `event_id`、填 `occurred_at_monotonic_ms`/`caused_by_perception_revision`，封装成完整 `DeviceEvent`。PC 只校验，不重写。
- PC 重启不会改变线上 `host_boot_id`；设备重启会改变它。PC 自身的重启由本地会话记录标识。

**未选方案**：

- **方案 B**：PC 收到后把 `STATE/EVENT.host_boot_id` 重写为 PC 自己的 boot ID。需要在入内核前深拷贝并保持一致；设备会话身份另存。优点是字段名更贴近「主机」；缺点是改写接收数据、增加出错面。
- **方案 C**：契约新增 `producer_boot_id`/`node_boot_id`（或明确 `host_boot_id` 语义）。这是长期正解，属于 `GS-CONTRACT-004` 或下一次契约版本的讨论范围。

方案 A 是 S2 的临时约定，**不是对 v0.3 契约的重新解释**；所有者裁决见 §11。若选 B/C，必须同步改本文与测试。

### 6.8 时钟与 TTL

- 设备只使用**单调时钟**；`node_monotonic_ms`、`produced_at_monotonic_ms`、`occurred_at_monotonic_ms` 都是同一 boot 内的设备单调时间。
- PC 只使用**本地接收单调时间**做 TTL/新鲜度判断；**不得**把设备单调时间与 PC 单调时间直接比较，也不得用 wall-clock 参与状态转换。
- 不同节点/不同 boot 的单调时间不可比较；P0-A 单节点也不例外（重启后归零）。
- PC 收到快照时记录 `received_monotonic_ms`；若超过 TTL 没有新快照/心跳，把相关事实视为过期/离线，不伪造 `unknown` 事件。
- 设备时间戳只用于同一 boot 内排序、去重和诊断；`Fact.observed_at/fresh_until` 不从 PC 时钟回填。

---

## 7. 按类型分派的校验

### 7.1 顺序：先分派，再校验

传输层只做**与类型无关**的封装校验：

1. magic、`ver`、`len` 上限、CRC；
2. `type` 是否已知；
3. 按 `type` 分派到对应解码器；
4. 类型专属的载荷、身份、会话、序号/去重校验；
5. 只有全部通过，才允许进入 Host 桥接/感知消费。

**禁止**在知道 `type` 之前做以下事情：

- 通用 JSON 解析/分配大缓冲；
- 通用「序号必须 +1」检查；
- 通用「所有消息都要 ACK」策略；
- 把未知类型当字符串/透传载荷交给下游。

原因很具体：`STATE`/`EVENT` 要严格单调，`AIM` 恰恰**不能**校验连续性；`STATE` 不要 ACK，`AIM` 更不能 ACK。统一校验会把正确行为判成故障（§7.6）。

### 7.2 通用校验（分派后、各类型共享）

- 载荷必须恰好消费 `len` 字节；无尾随字节。
- `contract_version` 必须是双方协商的 `0.3`；JSON 为 `"0.3"`，二进制为 major/minor `0/3`。
- 标识符按 §8：1..=64 字节（profile），UTF-8 合法；空/超长/控制字符拒绝。
- 数值范围：`Permille`/`confidence_milli`/`quality_milli` 0..=1000；枚举必须是已知值；`source_refs ≤ 4`；`node_health ≤ 8`；`observed_at ≤ fresh_until`。
- `Fact::Unknown` 不得携带 last-known value；`DeviceEvent` 不得由 revision 0 因果（契约校验）。
- 解码失败/校验失败：**不得**部分应用；不得写入 reducer、OCLive 或屏幕。
- 失败的帧计数并按 §9.3 限速/隔离；不得把错误内容写进证据日志的敏感字段（完整 MAC 等）。

### 7.3 `STATE` 校验

1. 解析 `PerceptionState`；`contract_version=0.3`。
2. `host_boot_id == HELLO.boot_id`（方案 A）；否则 `identity_mismatch`，关闭连接。
3. `revision` 与 `last_state_revision` 比较（§6.4）。
4. 同 revision：用 `has_same_revision_payload` 语义比较；只允许证据包络变化。
5. `node_health` 的 `node_uid` 不重复；`Fact` 的 `source_refs` 不超限；时间戳合法。
6. `produced_at_monotonic_ms` 在同一 boot 内不得倒退；否则 `bad_payload`/计数；重复则隔离。
7. 通过后更新快照与基线；**不**排回合。
8. 失败：丢弃 + 计数；重复违规按 §9.3。

### 7.4 `EVENT` 校验

1. 解析 `DeviceEvent`；`contract_version=0.3`。
2. `host_boot_id == HELLO.boot_id`；否则 `identity_mismatch`。
3. `event_id` 长度/字符符合 §8；`kind` 必须是 6 个已知边沿之一。
4. 去重：`(boot_id, event_id)` 已见过 → 重新 ACK，不处理。
5. 时序：`caused_by_perception_revision >= last_event_revision`；否则 `event_order`。
6. 因果：`caused_by_perception_revision <= last_state_revision` 才可立即处理。
   - 若大于：放入有界待定队列（建议 8 条），等待 `STATE` 追上；最多 1000 ms。
   - 超时仍无对应快照：NACK `state_gap`，丢弃，不排回合。
   - 设备收到 `state_gap` 后应立即重发当前 `STATE`，并可重试同一 `EVENT`（同一 `event_id`）一次。
7. 任何会话中：`caused_by_perception_revision <= baseline_revision` 视为 `stale_event`，NACK/丢弃。
8. 时间：`occurred_at_monotonic_ms` 不得倒退；否则 `bad_payload`/`event_order`。
9. 通过后交给 Host 排程，ACK；回合是否真正生成由 OCLive 决定，ACK 不代表回合完成。
10. 若 Host 队列满：NACK `busy`；设备可重试，仍失败则丢弃并计数。

### 7.5 `EXPR` 校验

1. 解析固定 8 字节（或 JSON 等价对象）。
2. `expr_id` 1–7；否则 `bad_payload`；屏幕不变。
3. `flags` 必须为 0；否则 `bad_flags`。
4. `hold_ms` 0–60000；越界 `bad_payload`。
5. `request_id` 去重；重复只重新 ACK，不重复应用。
6. 通过后交给设备表情仲裁；ACK 表示「已接受」，不保证可见（系统 Overlay 可抢占）。
7. 基线未完成：`not_ready`。

### 7.6 `AIM` 校验：不校验连续性

**只校验**：

- `contract_version=0.3`；
- `node_uid`/`boot_id` 与当前 HELLO 会话一致；
- 三个 `rate_mdps_*` 是 i32（无范围限制，符号/轴向未标定）；
- `quality_milli ≤ 1000`；
- `sequence`/`node_monotonic_ms` 只读取，不判断跳号/倒退；
- 载荷长度符合编码。

**不做**：

- 不要求 `sequence == last + 1`；
- 不要求 `node_monotonic_ms` 连续；
- 不去重、不重排、不缓存历史、不重传、不 ACK；
- 不把 `sequence` 写进 `NodeSession.last_sequence`；
- 不因丢帧/跳号标记节点故障。

**原因**（ADR-067 §二.5、§二.6）：

1. `AngularRateSample.sequence` 是**该节点全部出站消息共用**的单调账本。P0-A 单板内，`SensorObservation` 也从这个账本取号；AIM 帧的序号本来就与观测序号交错，**即使零丢包也不连续**。
2. 若对 AIM 做连续性校验，正常的交错就会被判成丢包/乱序；若再把 AIM 合流进 `PerceptionReducer`，一次丢包就会命中按节点共用的 `SequenceCollision`/`SequenceRegressed`，**打断同一帧的握持与姿态融合**。
3. AIM 是「最新即有效」的控制流：旧帧没有保留价值，积分/平滑/回中/渲染都在 PC 侧；重传只会增加延迟和队头阻塞。
4. 因此必须**先按 `type` 分派**：`STATE/EVENT` 走 revision/事件规则，`AIM` 走独立消费路径，两者不共享序号校验。

### 7.7 `ACK/NACK`、`PING/PONG` 校验

**ACK/NACK**

- `ack_type` 正常必须是 `HELLO/EVENT/EXPR` 之一；若本帧是对未知类型回 `NACK(unknown_type)`，允许为原始 `type` 值；其他情况 `bad_payload`。
- `result` 0 表示 ACK；1–17 为已知 NACK；其他值按未知错误处理（记录，不崩溃）。
- `detail_len ≤ 160`；超长 `bad_payload`。
- 未请求的 ACK/NACK：记录并忽略；不得改变状态。
- ACK 重复：幂等；不得重复应用被确认的消息。

**PING/PONG**

- 载荷恰好 4 字节；否则 `bad_payload`/丢弃。
- PONG 必须匹配最近 PING 的 nonce；否则丢弃 + 计数。
- 未发送 PING 却收到 PONG：忽略。
- PING 频率限制（建议 ≤10 次/s）；超限丢弃 + 计数。

### 7.8 失败处置矩阵

| 故障 | 检测点 | 处置 | 会话影响 | 计数 |
|---|---|---|---|---|
| 坏 magic | 帧解析 | 滑动找 magic | 保持；持续垃圾则断开 | `garbage_bytes` |
| 坏 CRC | 帧解析 | 丢弃整帧；不 NACK | 保持；重复按阈值隔离 | `crc_errors` |
| 截断/半包 | 帧解析 | 丢弃半帧 | 读超时/流关闭则断开重连 | `truncated` |
| 未知标准类型 | 分派 | 丢弃；可 NACK `unknown_type` | 保持；重复隔离 | `unknown_types` |
| 未知实验类型 | 分派 | 静默丢弃 | 保持 | `unknown_types` |
| 帧版本 ≠1 | 帧头 | 不解析；断开 | 断开 | `bad_versions` |
| 契约版本不支持 | HELLO/载荷 | NACK + 关闭/隔离 | `incompatible_protocol` | `bad_contracts` |
| `len > 4096` | 帧头 | 不分配；可选 NACK 后关闭 | 断开 | `oversize` |
| 载荷解析/字段非法 | 类型解码 | NACK（若有 ACK_REQ）/丢弃 | 保持；重复隔离 | `bad_payloads` |
| `boot`/`node_uid` 不匹配 | 类型校验 | NACK `identity_mismatch` | 重复则断开 | `identity_errors` |
| `revision` 倒退 | STATE | 丢弃；不更新基线 | 重复则断开 | `revision_regressions` |
| `revision` 碰撞 | STATE | 丢弃；不更新 | 重复则断开 | `revision_collisions` |
| EVENT 顺序倒退 | EVENT | NACK `event_order` | 保持 | `event_order_errors` |
| 陈旧 EVENT | EVENT | NACK `stale_event`/丢弃 | 保持 | `stale_events` |
| 重复 EVENT | EVENT | 重新 ACK；不处理 | 保持 | `duplicate_events` |
| EVENT 早于 STATE | EVENT | 短窗口等待；超时 NACK `state_gap` | 保持 | `state_gaps` |
| EXPR 非法 ID | EXPR | NACK `bad_payload` | 保持 | `bad_exprs` |
| 心跳丢失 | 心跳 | 断链/离线预设 | 断开重连 | `heartbeat_timeouts` |
| 发送队列满 | 发送 | 丢 AIM/合并 STATE/有界丢 EVENT | 保持；计数 | `tx_drops` |
| 连续违规 | 隔离器 | 断开 + 隔离 30 s → 持久 `protocol_fault` | 隔离 | `protocol_faults` |

### 7.9 为什么不能把校验统一成一套

- 对 `AIM` 套 `STATE/EVENT` 的连续性规则 → 正常交错被误判，感知流被瞄准流打断（§7.6）。
- 对 `STATE` 套事件停等/ACK → 每秒反向 ACK，且丢一份快照无所谓；增加无意义流量。
- 对 `EVENT` 套快照的「最新值优先」→ 边沿被合并/丢失，角色错过动作。
- 对未知 `type` 先解析载荷 → 任意字节进入 JSON/codec，可能大分配/崩溃/注入。
- 把 `AIM` 的 `sequence` 与 `STATE.revision` 比较 → 两个不同域，数值无意义。

因此：**帧层只保证「这一帧字节完整且类型已知」，语义校验必须按类型分派。**

---

## 8. 标识符

### 8.1 契约上限

v0.3 通过 `string_id!` 定义了四个标识符类型：

- `NodeUid`、`BootId`、`ObservationId`、`EventId`；
- 每个都是 `String` 的透明包装，`try_new` 拒绝空串与超过 **128 UTF-8 字节**的值；
- JSON Schema 标注 `minLength=1, maxLength=128`；反序列化同样执行校验。

注意：**128 是字节数，不是字符数**。中文/emoji 在 UTF-8 下占多字节；标识符应使用 ASCII，避免长度与日志问题。

### 8.2 `NodeUid`

- **跨重启稳定**；不得使用会随机化的 BLE/Wi-Fi MAC 作为唯一来源。
- P0-A 单板建议：`gs-jc3248-<6 位十六进制>` 或 `node-bench-01`，长度 ≤32 字节。
- 若用 MAC 后缀，使用芯片基础 MAC（eFuse），不是 Wi-Fi 随机地址。
- P0-A 只有一个节点，PC 可以静态配置能力集作为兜底；`HELLO.contract_caps` 是设备自声明，两者不一致时以协商结果为准并记录。

### 8.3 `BootId` 命名建议

**要求**：每次冷启动必须变化；同一次启动内稳定；不得包含空格/控制字符；≤32 字节（profile 建议）。

**推荐格式**：

```text
bt-<MAC 后 3 字节，6 位大写十六进制>-<启动计数，至少 4 位十六进制>
例：bt-A1B2C3-0007
```

- `MAC 后 3 字节`：从芯片基础 MAC 取最后 3 字节，例如 `A0:B7:65:12:34:56` → `123456`。台架内足以区分单板，且不把完整 MAC 写进日志/外部证据。
- `启动计数`：存 NVS/RTC 持久区，每次冷启动递增；建议 16 位（4 位十六进制），溢出前换用 32 位（8 位十六进制）。若持久区不可用，用 16 位随机数替代；同一次上电内不得重复。
- 不要把复位原因、传感器状态、时间戳等易变信息拼进 `boot_id`；它只需要「这一次启动的唯一名字」。

**为什么不能更长**：

1. **契约硬上限 128 字节**；超过直接被 `string_id!`/Schema 拒绝。
2. **帧预算**：`host_boot_id` 会出现在每一份 `STATE` 和 `EVENT` 里；`source_refs` 还可能重复 `node_uid`/`observation_id`。如果所有 ID 都用满 128 字节，6 个 Fact × 4 个 source_ref 的合法快照可轻易超过 4096 字节载荷上限（§3.2）。S2 因此把 profile 硬上限压到 **64 字节**，建议 ≤32 字节。
3. **日志与证据**：`boot_id` 会进入串口日志、测试记录和脱敏摘要；长字符串降低可读性，不增加唯一性。
4. **隐私**：完整 MAC 属于设备标识，不应进入外部评审材料；3 字节后缀 + 启动计数已够台架区分。
5. **它是身份，不是描述**：不要写「2026-11-13-第三次上电-电池供电」这类散文；语义信息放日志，不放 ID。

**碰撞说明**：24 位 MAC 后缀 + 启动计数在单板台架足够；多板同场时理论上可能碰撞。正式产品应使用 6 字节后缀或预配 UUID，并保持 ≤64 字节。

### 8.4 `ObservationId` / `EventId`

- `ObservationId` 在 P0-A 不直接上行（板上内核本地消费）；若诊断日志需要，建议 `obs-<boot 后缀>-<seq>`，≤64 字节。
- `EventId` 每次启动内唯一；建议 `ev-<boot 后缀>-<seq>` 或简单的十进制序号，≤64 字节。`boot_id` 已单独携带，`EventId` 不需要再嵌入完整 boot。
- **去重键是 `(boot_id, event_id)`**；不要只按 `event_id` 全局去重（不同 boot 可复用序号）。
- 接收方**不得**依赖解析 `event_id` 的文本来做排序；排序用 `caused_by_perception_revision` + 停等传输（§7.4）。ID 的格式只用于日志与去重。

### 8.5 字符集与日志

- 契约允许任意非空 UTF-8；S2 profile **应当**只使用 `[A-Za-z0-9._:-]`。
- 禁止空格、NUL、换行、控制字符；禁止前后空白。
- 大小写敏感；同一设备不要周期性改变大小写风格。
- 长度按 UTF-8 字节计算；日志同时记录字节长度。
- 完整 MAC、Wi-Fi 密码、IP 等留在本地记录；外部证据只给脱敏后缀。

---

## 9. 错误处理与降级

### 9.1 故障分类

| 类别 | 例子 | 处理原则 |
|---|---|---|
| 帧层软故障 | 坏 CRC、未知标准类型、坏载荷 | 丢单帧 + 计数；重复按阈值隔离 |
| 帧层硬故障 | 帧版本不支持、`len` 超限、magic 持续不匹配 | 立即断开本次连接 |
| 会话/身份故障 | `boot_id` 不匹配、无共同契约版本、revision 倒退/碰撞 | 断开/隔离；不得进入融合 |
| 语义/顺序故障 | EVENT 倒退、陈旧事件、事件早于快照 | NACK/丢弃；不排回合 |
| 传输故障 | TCP RST、USB 拔出、读超时 | 断开重连；不计应用违规 |
| 降级 | 心跳丢失、PC 不在线、传感器失效 | 本地预设/`unknown`；不伪造事件 |

### 9.2 具体处置

**坏 CRC**

- 丢弃整帧，计数 `crc_errors`；**不 NACK**（帧不可信，`type` 也不可信）。
- 单次不改变状态；滚动 10 s 内 3 次则断开并隔离（§9.3）。
- 不得把坏帧载荷的部分内容送入解码器。

**截断**

- 帧未收齐（含 CRC 不足）时读超时/流关闭：丢弃半帧，重置解析器。
- TCP 半关闭/RST：断开重连。
- USB 读超时：关闭串口并重开；设备侧回到 `LINK_DOWN`。
- 截断**不 NACK**；没有完整帧可确认。

**未知 `type`**

- 帧头/CRC 合法时：丢弃载荷；若 `ACK_REQ` 置位且类型在标准保留区，回 `NACK(unknown_type)`。
- 实验区 `0x80–0xFF`：静默丢弃 + 计数，不回 NACK。
- 不因单个未知类型立即断开；重复达到阈值再隔离。
- 绝不把未知载荷转发给感知/OCLive。

**版本不匹配**

- `ver != 1`：硬违规，立即关闭；不解析载荷；记录前 7 字节。此时通常不发 NACK：接收方无法假定 `type` 的语义仍然有效。
- `HELLO` 无共同契约版本/编码：PC 回 `NACK(unsupported_contract_version/unsupported_encoding)` 后关闭；标记 `incompatible_protocol`，不参与融合。
- 协商成功后载荷里出现其他 `contract_version`：`bad_payload`/`unsupported_contract_version`；丢弃 + 计数；重复隔离。
- 不尝试「猜兼容」或按旧版本解析。

**超长**

- `len > 4096`：不分配载荷缓冲；可回 `NACK(too_large)` 后关闭，或直接关闭。
- `len == 0`：非法；丢弃/NACK；重复则关闭。
- 连续垃圾超过 4096 字节或 1 s：关闭。

**身份不匹配**

- `STATE/EVENT.host_boot_id` 与 `HELLO.boot_id` 不一致：`identity_mismatch`；不更新状态；重复则关闭。
- `AIM.node_uid/boot_id` 不匹配：静默丢弃（高频流不放大 NACK）。
- 同一 `boot_id` 下 `NodeUid` 变化：按协议故障处理。

**revision 故障**

- 倒退：丢弃快照，保留旧基线；重复则断开/隔离。
- 碰撞（同 revision 不同语义）：丢弃，保留旧快照；重复则断开。
- 不因倒退而「回滚」已消费的事件或角色状态；事件去重与状态更新是单向的。

**事件故障**

- 顺序倒退：NACK `event_order`，丢弃；不排回合。
- 陈旧：NACK `stale_event` 或静默丢弃；不排回合。
- 早于 STATE：短窗口等待；超时 NACK `state_gap`，丢弃。
- 重复：重新 ACK；不排回合。

**队列/背压**

- 设备 TX 满：按 §2.6；不阻塞感知/UI。
- PC RX 满/处理慢：NACK `busy`；设备有限重试；最终丢弃并计数；`STATE` 继续刷新。
- 任何情况下都不得通过阻塞采样来「保证送达」。

### 9.3 分级隔离阈值

沿用 `ARCHITECTURE.md` 的初始值（全部标记 `ESTIMATED`，需故障注入与实测）：

1. 单条非法**应用**帧：丢弃、限速记账；不改变状态。
2. 滚动 **10 s 内 3 条**非法应用帧：断开本次连接，隔离 **30 s**。
3. 连续 **3 个连接周期**都触发隔离：进入持久 `protocol_fault`，需要维护页/人工确认才重新启用。
4. 硬违规（超长、协商后错误帧版本、身份不匹配重复出现）：可**直接终止本次连接**，不必等 3 条。

说明：

- 传输层错误（TCP 重传、USB 噪声、RSSI 差）**不计入**应用协议违规；但坏 CRC 属于应用帧完整性失败，计入。
- 隔离期间设备继续本地感知/表情预设；PC 标记 `NodeHealthState::ProtocolFault`/`IncompatibleProtocol`，不参与融合。
- `NodeHealthState` 是 v0.3 契约枚举；PC 侧映射：无共同版本 → `IncompatibleProtocol`；重复违规 → `ProtocolFault`；心跳丢失 → `Offline`；部分能力/质量下降 → `Degraded`。

### 9.4 链路丢失与离线预设

- 断链 >3–5 s：设备进入**离线预设**（本地表情轮换/静止），不等待 PC、不等待 LLM。
- PC：相关事实按 TTL 标记过期/离线；不产生 `unknown` 伪事件、不触发回合。
- 重连：先建快照基线，稳定 2–3 s 后再切回自由模式（P0A 设计 §5）。
- 断链期间设备仍可本地响应触摸/感知；恢复后 PC 只接受恢复后的新事件，不回放。
- 离线预设是**降级**，不是协议状态；它不需要 PC 参与。

### 9.5 与 `GS-CONTRACT-004` 的关系

`GS-CONTRACT-004` 仍未关闭：正式 `NodeHello`（版本 + 能力协商）与 `AssetRef`/表达下行类型尚未实现。本文的处理：

- **`HELLO` 不是 `NodeHello`**：它只是 S2 台架握手，字段是 `NodeHello` 的子集/临时形状，不写入 `contracts`，不生成 schema。
- **`EXPR` 不是 `AssetRef`**：只传 1–7 的表情 ID，不传资产 ID 空间、路径、URL、像素或动画引用。正式 `AssetRef` 落地后，由 S3 用新消息/新帧版本替换，不得把 `AssetRef` 硬塞进 `EXPR` 的数值字段。
- **不扩展 v0.3 闭集**：不为协议方便给 `SensorObservationKind`、`DeviceEventKind`、`Capability` 加成员；未知 variant 一律拒绝，不转发。
- **不修改 `PerceptionState`/`DeviceEvent` 字段**：`host_boot_id` 的临时映射见 §6.7，属于部署约定，不是契约变更。
- **`AIM` 仍受 `GS-CONTRACT-005` 约束**：`AngularRateSample` 的三轴符号、轴向、量程未经实机标定；S2 默认关闭，启用也不代表字段已验证。
- **替换策略**：正式协商落地时，帧版本抬到 2；`HELLO` 换成 `NodeHello` 语义；`EXPR` 换成 `AssetRef` 下行；S2 不作兼容承诺，但保留 golden vectors 与 CP2 故障用例作为回归。

### 9.6 版本演进规则

- 帧封装/类型码/flags/长度/CRC 变化 → **帧版本** `ver` +1。
- 载荷字段语义变化 → **契约版本** +1（由 contracts 真源与 schema 零漂移门管理）。
- `HELLO` 同时协商两者；无交集即 `incompatible_protocol`，不融合。
- 不得用 SemVer、字段存在性、未知字段或「看起来一样」推断兼容。
- 未知 variant/枚举值：拒绝该帧，不当作任意字符串转发。
- **HELLO 引导版本的注意事项**：如果未来 `ver=2` 的帧头布局与 v1 不同，v1 接收方无法解析 v2 的 HELLO 来完成协商。届时应使用稳定的引导帧（固定 v1 头 + 版本列表）或单独的发现通道；这属于 S3 设计，不在 S2 解决。S2 只支持 `ver=1`，遇到其他版本直接关闭。

---

## 10. 台架最小实现清单

### 10.1 PC 测试程序的最小动作

目标：在**不改 OCLive、不改板端感知内核**的前提下，证明 §3–§9 的语义。程序应把「传输适配」与「帧/会话逻辑」分开，同一套逻辑跑 USB 与 TCP。

建议 CLI（名称可变）：

```text
protocol-bench --transport usb --port COM5
protocol-bench --transport tcp --listen 0.0.0.0:47800
protocol-bench --encoding binary|json
protocol-bench --mood happy --hold-ms 5000
protocol-bench --fault crc|truncate|unknown-type|bad-ver|oversize|revision-regress|duplicate-event|event-before-state
```

最小动作：

1. **打开链路**：USB 打开 COM 口；TCP 监听/接受一个客户端。设置读超时与 `TCP_NODELAY`（TCP）。
2. **帧解析**：实现 §3 的字节流状态机；统计 `frames/crc_errors/truncated/unknown_types/oversize/resyncs`；打印每帧 hex 与解析结果。
3. **握手**：接收 `HELLO`，校验帧版本/契约版本/编码/能力/ID 长度；回 `ACK`；记录 `node_uid`、`boot_id`、`contract_caps`、`proto_caps`、`fw_version`；超时/NACK 路径也要覆盖。
4. **收基线**：接收第一份 `STATE`，解析并打印 `revision`、六个 Fact、`node_health`；设置 `last_state_revision`/`baseline_revision`；确认**没有**触发任何回合。
5. **收快照**：持续接收 1 Hz `STATE`；验证同 revision 重发只刷新证据、不排回合；验证 revision 前进、跳号、倒退/碰撞的处置。
6. **收事件**：接收 `EVENT`，验证 `host_boot_id`、`event_id`、`caused_by_revision`、`occurred_at`；回 `ACK`；验证重复事件只 ACK 不排回合；验证事件早于快照时的等待/超时。
7. **发表情**：把 `neutral/happy/excited/shy/confused/sad/angry` 映射为 1–7；发送 `EXPR`（递增 `request_id`），等 `ACK`；超时重发同一 `request_id`；验证重复不重复改屏；验证未知 ID 被 NACK 且屏幕不变。
8. **断线/重连**：关闭 socket/串口再重开；验证同 `boot_id` 基线、去重缓存保留、无历史事件重放；通过测试钩子/模拟器制造新 `boot_id`，验证 revision/去重/AIM 状态清空。
9. **故障注入**：对合法帧做位翻转、截断、改 type、改 ver、改 len、重复/倒退 revision、事件先于快照、错误 `boot_id`；断言 §9.2 的处置与计数，且**没有任何坏数据进入下游**。
10. **双传输对拍**：同一场景分别跑 USB 与 TCP；比较消息类型序列、revision/kind/事件顺序、ACK 次数、断连恢复时间；记录 p50/p95 延迟。语义应一致，只有传输参数不同。
11. **日志与证据**：保存每帧 hex、解析后的 JSONL、计数器、起止时间、固件提交/构建时间、PC 程序版本；外部评审前脱敏（完整 MAC、SSID、IP、密码）。

### 10.2 用例矩阵（CP2 / CP5）

| 用例 | 注入/操作 | 期望 |
|---|---|---|
| 首连 | 正常 HELLO + STATE | ACK；基线建立；无回合 |
| 重连（同 boot） | 断开后重开 | 新基线；去重保留；无历史事件 |
| 设备重启（新 boot） | 新 `boot_id` | 清 revision/去重/AIM；新基线 |
| 同 revision 重发 | 重发相同快照 | 更新证据；无事件/回合 |
| 同 revision 语义不同 | 改一个 Fact 值 | `revision_collision`；丢弃 |
| revision 倒退 | 发旧快照 | 拒绝；旧基线保留 |
| 重复 EVENT | 同 `event_id` 重发 | 重新 ACK；不排回合 |
| 陈旧 EVENT | 重连后发 ≤baseline 的事件 | `stale_event`；不排回合 |
| EVENT 先于 STATE | 发 R+1 事件再发 R+1 快照 | 等待；快照到后处理；超时 `state_gap` |
| 坏 CRC | 翻转一位 | 丢弃；无状态变化；计数 |
| 截断 | 少发 3 字节 | 丢弃半帧；超时/关闭重连 |
| 未知 type | `type=0x0A` | 丢弃；可选 NACK；不转发 |
| 错误帧版本 | `ver=2` | 断开；不解析 |
| 超长 | `len=5000` | 断开；不分配 |
| 未知 EXPR ID | `expr_id=99` | NACK；屏幕不变 |
| 心跳丢失 | 停 PONG | 3 s 后离线预设；PC 标离线 |
| USB/TCP 对拍 | 同一场景 | 语义一致；延迟有记录 |
| S1 固件误接 | 收到 ASCII `READY...` | 明确报错退出，不反复重连 |

### 10.3 日志与证据

每份台架记录至少包含：

- 传输（USB COMx / TCP host:port）、编码（binary/JSON）、PC 程序版本/提交；
- 设备固件提交/构建时间、`node_uid`、`boot_id`、能力位；
- 每帧方向、`type`、`len`、CRC 结果、解析摘要；错误帧保留 hex（脱敏）；
- 计数器：`frames/crc_errors/truncated/unknown_types/oversize/duplicate_events/stale_events/revision_regressions/heartbeat_timeouts/tx_drops`；
- 延迟：`HELLO→ACK`、`EVENT→ACK`、`STATE` 接收间隔、USB/TCP p50/p95；
- 判定：通过/失败、失败原因、下一步；
- 不写完整 MAC、Wi-Fi 密码、个人网络信息。

### 10.4 非目标

本轮不要求：

- 接入 OCLive 正式角色包/LLM；
- AIM 实机标定/积分效果；
- TLS/认证/公网；
- mDNS/自动发现/多客户端；
- OTA、日志上传、性能调优；
- P0-B 多节点/BLE；
- 生产级安装包。

---

## 11. 未决问题

### 11.1 需要所有者裁决

| 编号 | 问题 | 选项 | 影响 |
|---|---|---|---|
| Q1 | S2 是否启用 `AIM` | 默认关闭（推荐）/ 仅 USB / USB+TCP | 带宽、复杂度、IMU 未标定（`GS-CONTRACT-005`） |
| Q2 | 载荷编码最终选型 | 手写定长二进制（推荐先做）/ `postcard` / CBOR；JSON 仅台架 | codec、golden vectors、板端依赖 |
| Q3 | `host_boot_id` 映射 | 方案 A 设备填（本文临时）/ B PC 重写 / C 契约加 `producer_boot_id` | 契约语义、重连、`GS-CONTRACT-004` |
| Q4 | TCP 角色/端口/发现 | PC 监听 47800（推荐）/ 设备监听 / mDNS | 部署、防火墙、设备配置 |
| Q5 | Wi-Fi 安全 | 受信局域网无 TLS（S2 临时）/ 预共享 token / TLS | 安全与实现成本；S2 禁止公网 |

### 11.2 需要实测裁决

| 编号 | 问题 | 当前草案值 | 判据 |
|---|---|---|---|
| M1 | 心跳周期/超时 | 1 s / 3 s | 断连检测时间、误判率 |
| M2 | 最大载荷/ID 上限 | 4096 B / 64 B（建议 32） | 最坏合法 STATE 是否装得下 |
| M3 | `STATE` 是否每次 revision 都发 | 是（快照先行） | 事件到 PC 的延迟、流量 |
| M4 | `EVENT`/`EXPR` 停等超时 | 200 ms ×3 | USB/TCP 丢包与重试 |
| M5 | AIM over TCP 的队头阻塞 | 默认关闭 | p95 延迟、可靠消息被压情况 |
| M6 | 故障隔离阈值 | 10 s 内 3 条 / 30 s / 3 周期 | 故障注入 + 实机 |
| M7 | 表达式 `hold_ms`/仲裁 | 0=默认，1–60000 | 触摸/系统 Overlay/PC 优先级 |
| M8 | JSON 模式是否只在 USB 台架 | 草案允许 USB/TCP | Wi-Fi 带宽与可读性 |

### 11.3 推迟到 S3（正式协商）

- `NodeHello` 的完整版本/能力协商字段与状态机；
- `AssetRef` ID 空间、表达下行（表情/动画/短文本）类型；
- 多节点/多客户端；
- TLS/认证/配对；
- 固件版本/诊断/OTA 通道；
- 分片/压缩/批量；
- AIM 的正式流（若需要可靠性，另立独立序号/传输，不复用共享账本）。

---

## 12. 参考与变更记录

### 12.1 参考

- [AGENTS.md](../AGENTS.md)：仓库边界、P0-A 形态、架构红线。
- [ADR-067](DECISIONS.md)：节点模型泛化、v0.3 契约、`AngularRateSample` 独立流。
- [ADR-069](DECISIONS.md)：P0-A 由 ESP32-S3 运行感知内核，USB/Wi-Fi 同语义。
- [P0A_SOFTWARE_DESIGN.md](P0A_SOFTWARE_DESIGN.md)：分层、任务/时钟、S1→S3 分阶段、降级。
- [ARCHITECTURE.md](ARCHITECTURE.md)：协商、隔离、revision 与时间模型。
- [schemas/README.md](../schemas/README.md)：v0.3 类型、fixtures、零漂移门。
- [contracts/src/lib.rs](../crates/ailive-gun-spirit-contracts/src/lib.rs)：`SensorObservation`、`AngularRateSample`、`PerceptionState`、`DeviceEvent`、`ContractVersion`、`string_id!`。
- [hardware README](../firmware/jc3248w535/README.md)：现有 7 表情与 S1 ASCII 协议。

### 12.2 变更记录

| 日期 | 版本 | 变更 | 状态 |
|---|---|---|---|
| 2026-11-13 | v1 草案 | 初稿：帧格式、消息类型、会话/去重、按类型分派、标识符、降级、台架清单、未决项 | `CANDIDATE`；未实现、未实测 |

---

## 附录 A：HELLO 二进制布局（候选）

`HELLO` 是唯一不受 `encoding` 影响的载荷，固定二进制、无对齐/填充，字段顺序如下：

| 偏移 | 长度 | 字段 | 说明 |
|---:|---:|---|---|
| 0 | 1 | `frame_ver_min` | 最小帧版本；S2=`1` |
| 1 | 1 | `frame_ver_max` | 最大帧版本；S2=`1` |
| 2 | 1 | `contract_count` | 1–4 |
| 3 | 变长 | `contract_versions` | 每项：`u8 len` + UTF-8；例如 `03 "0.3"` |
| … | 1 | `encoding` | 1=binary，2=JSON |
| … | 2 | `contract_caps` | u16 小端；`Capability` 位图 |
| … | 2 | `proto_caps` | u16 小端；协议能力位图 |
| … | 1 | `node_uid_len` | 1–64（profile） |
| … | 变长 | `node_uid` | UTF-8 |
| … | 1 | `boot_id_len` | 1–64（profile） |
| … | 变长 | `boot_id` | UTF-8 |
| … | 1 | `fw_len` | 0–64；0 表示不提供 |
| … | 变长 | `fw_version` | UTF-8 |

**`contract_caps` 位**

| 位 | 能力 |
|---:|---|
| 0 | `FrontGrip` |
| 1 | `RearGrip` |
| 2 | `Motion` |
| 3 | `Orientation` |
| 4 | `PrimaryControl` |
| 5 | `ShockDiagnostic` |
| 6–15 | 保留，置 0 |

**`proto_caps` 位**

| 位 | 能力 |
|---:|---|
| 0 | `STATE` |
| 1 | `EVENT` |
| 2 | `EXPR` |
| 3 | `AIM` |
| 4 | `PING` |
| 5 | `SNAPSHOT_ON_CHANGE`（每次 revision 变化先发快照） |
| 6–15 | 保留，置 0 |

S2 设备**应当**置 `SNAPSHOT_ON_CHANGE`；未置不改变接收方的因果等待规则（§7.4），只表示事件最坏要等到下一份 1 Hz 快照才能处理。

**解析规则**

- 顺序读取，不允许对齐填充；读取越界即 `bad_payload`。
- 恰好消费完 `len` 字节；尾随字节 `bad_payload`。
- `node_uid`/`boot_id` 长度必须在 1..=64；空/超长拒绝。
- `encoding` 只接受 1/2；其他 `unsupported_encoding`。
- `proto_caps` 未声明 `STATE`/`EVENT`/`PING` 的设备不构成可用 P0-A 节点；PC 可 NACK `bad_payload`。
- `proto_caps` 声明 `AIM` 但 `encoding=2`：PC 忽略 AIM 并记录；设备不应这样声明。
- 保留能力位应为 0；接收方忽略未知能力位并记录，不因单个未知位拒绝 HELLO（但缺 `STATE`/`EVENT`/`PING` 仍不可用）。

## 附录 B：EXPR / ACK / NACK / PING 固定布局（候选）

**EXPR（8 字节，PC → 设备）**

| 偏移 | 长度 | 字段 |
|---:|---:|---|
| 0 | 1 | `expr_id`（1–7） |
| 1 | 1 | `flags`（0） |
| 2 | 2 | `hold_ms`（u16 小端，0–60000） |
| 4 | 4 | `request_id`（u32 小端） |

**ACK / NACK（3 + detail 字节，双向）**

| 偏移 | 长度 | 字段 |
|---:|---:|---|
| 0 | 1 | `ack_type`（1/3/4） |
| 1 | 1 | `result`（0=ACK；1–17=NACK） |
| 2 | 1 | `detail_len`（0–160） |
| 3 | `detail_len` | `detail` UTF-8 |

**PING / PONG（4 字节，双向）**

| 偏移 | 长度 | 字段 |
|---:|---:|---|
| 0 | 4 | `nonce`（u32 小端） |

JSON 模式下，上述载荷的等价对象为：

```json
{"expr_id":2,"flags":0,"hold_ms":5000,"request_id":42}
{"ack_type":3,"result":0,"detail":""}
{"nonce":287454020}
```

字段名与二进制一致；数值范围与校验规则相同。

## 附录 C：二进制枚举标签（候选）

以下标签仅服务二进制 codec；按 Rust 声明顺序从 1 开始，`0` 保留。**不是契约的一部分**，冻结前以 golden vectors 为准。

| 枚举 | 标签 |
|---|---|
| `Contact` | 1=`engaged`，2=`released` |
| `MotionClass` | 1=`idle`，2=`moving` |
| `Pose` | 1=`raised`，2=`lowered` |
| `CarrierState` | 1=`standby`，2=`held`，3=`ready` |
| `PrimaryControlId` | 1=`primary_trigger` |
| `GripBinding` | 1=`front`，2=`rear` |
| `UnknownReason` | 1=`never_observed`，2=`node_offline`，3=`stale`，4=`uncalibrated`，5=`sensor_fault` |
| `EvidenceRelation` | 1=`supports`，2=`contradicts` |
| `Capability` | 1=`front_grip`，2=`rear_grip`，3=`motion`，4=`orientation`，5=`primary_control`，6=`shock_diagnostic` |
| `NodeHealthState` | 1=`healthy`，2=`degraded`，3=`offline`，4=`incompatible_protocol`，5=`protocol_fault` |
| `DeviceEventKind` | 1=`carrier.held_entered`，2=`carrier.held_exited`，3=`carrier.ready_entered`，4=`carrier.ready_exited`，5=`control.primary.engaged`，6=`control.primary.released` |
| `ContractVersion` | major=0，minor=3（两字节） |
| `Fact` 标签 | 0=`Unknown`，1=`Known` |
