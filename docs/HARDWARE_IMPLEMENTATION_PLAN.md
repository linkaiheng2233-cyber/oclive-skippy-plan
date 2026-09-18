# 双区域节点硬件实施计划

状态：Current SSOT（2026-08-31）  
证据等级：除明确标注 `DATASHEET` 外，本文尺寸、重量、功耗、阈值和时序均为 `ESTIMATED`，硬件尚未 bring-up。到货实测用 `MEASURED` 标注并通过 ADR 覆盖，不静默改数字。

本文件覆盖旧路线中“Lyra Zero W + 单 Grip Node”及“Orange Pi Zero 2W 2GB”的当前执行优先级。旧文档继续保留为设计历史和回退资料；当前模块化 bring-up 使用 Orange Pi Zero 3W 6GB（Allwinner A733）、3.5 英寸 480×800 HDMI 屏、前下导轨节点和后握把节点。主板已选定为采购/实测基线，但板端Linux、USB-C DP转HDMI、热、功耗和本地3B能力仍是`OPEN`实机门。

## 1. 当前拓扑

```text
Front camera (P1) ── concealed USB ───────────┐
                                              │
Front rail node ───── BLE SensorObservation ──┼→ Orange Pi / fusion / OCLive / display
                                              │
Rear grip node ────── BLE SensorObservation ──┘
```

- 主机舱：Orange Pi Zero 3W 6GB / A733 + 板载散热器与小风扇 + USB-C DP Alt Mode主动转HDMI + 3.5inch 480×800 HDMI IPS touch + rechargeable main battery + power management；yaw/pitch/roll 三轴移动。
- 前下导轨节点：XIAO nRF52840 Sense + onboard IMU + FSR 408 + protected 1S LiPo；电子盒位于下导轨后段，FSR 可在可移动前握把/护片内。
- 后握把节点：薄握把套 FSR + 握把底部 XIAO/电池盒 + 独立只读 trigger 辅助传感器。
- P1 摄像头：UVC MJPEG；护木隐藏 USB 主线 + 三轴附近可更换短跳线；接主机，不接显示面板。
- 不新增顶部传感节点。区域内部允许隐藏短线，区域之间使用 BLE。

安全红线：所有感知只读；任意节点、传感器、主机或电池故障只能让器灵失去观察，不能改变原发射器电机、MOSFET、供弹、火控、扳机行程和原机构行为。

## 2. 信息链路

```text
force/contact/motion
  → resistance/edge/acceleration
  → ADC/GPIO/IMU sample
  → node filter/debounce/hysteresis/calibration
  → SensorObservation
  → BLE GATT notification
  → host TTL/sequence validation
  → multi-node fusion
  → DeviceEvent
  → Spirit state machine / OCLive sensor turn
  → immediate VisualCue
  → HDMI display
```

节点负责硬件阈值，OCLive 只消费语义。前后握持共用 `grip.engaged/released`，由 `node_id` 区域化：

```text
front_rail_01 + grip.engaged
rear_grip_01  + grip.engaged
```

`unknown` 是主机在断连、TTL 过期或状态快照无效时维护的事实状态，不伪装为 `released`。

状态语义已于 2026-08-26 冻结；角度、阈值、时间窗和 confidence 数值仍为 `ESTIMATED`，必须实测：

| 状态 | 进入/保持规则 | 降级 |
|------|---------------|------|
| Standby | 前后都明确 released + motion.idle 持续超时 | 任一握持 unknown 时不得把 unknown 当 released；省电显示策略可独立降级，但不生成确定 Standby |
| Held | 进入：rear engaged；或 front engaged + motion moving。保持：进入 Held 后，任一 grip engaged 即使 motion idle 也继续 Held | 所有 grip unknown 时只允许低置信度 motion 推断，不产生确定握持事实 |
| Ready | 保留单一 Ready 状态；rear engaged + motion.raised 为较低/中置信度，front + rear engaged + motion.raised 为高置信度 | rear released/unknown 不产生 Ready；front released/unknown 时允许单手 Ready，但不得标为高置信度 |

只读`primary_control`触点独立于carrier state：engaged/released产生中性控制边沿，carrier仍为Standby/Held/Ready之一；它不产生Active状态、不确认机构动作，过期不重放。

FSR V0 起始电路：`3V3 → FSR → ADC tap → RM 47 kΩ → GND`。47 kΩ、25–50 Hz、engaged 80 ms、released 250–300 ms 只是台架起点，必须由结构、手套和至少 100 次动作实测冻结。

## 3. 能源链路

三个独立电源域：

```text
main battery → protection/NTC/true power-path → regulated 5 V
             → Orange Pi Zero 3W + DP→HDMI + HDMI screen + USB touch/hub + P1 camera

front protected 1S LiPo → switch → XIAO BAT path → MCU/IMU/FSR
rear protected 1S LiPo  → switch → XIAO BAT path → MCU/FSR/trigger aux
```

前后 BLE 节点不与主机共地。USB 摄像头与主机共用 VBUS/GND。

主机设计占位。下列旧H618预算已由A733路线替代，新数字只用于准备电源和假体，全部标记为`ESTIMATED`：

| 工况 | 平均功耗 | 峰值检查 |
|------|----------|----------|
| P0 主机 + 屏 + BLE + touch，不加载本地3B | 4.5–7 W | 10–15 W |
| P0 + CPU量化3B间歇角色回复 | 5.5–9 W（取决于回复占空比） | 12–18 W |
| P1 再加 hub + UVC/只读视觉 | 6–11 W | 12–20 W |

本轮不按上述宽区间直接购买最终电池。按 85% 转换效率、80% 可用能量，4 h标称能量公式为`average_W × 4 / 0.85 / 0.8`；本地3B必须分别记录模型常驻但空闲、短句低占空比和连续压力生成，不能用连续满载冒充真实场地平均，也不能反过来忽略峰值。1S/2S只改变电压/mAh表达，不改变Wh、质量和体积。没有USB功耗仪/电源分析仪实测前，不冻结主电池；原5V/3A级电源也不能直接假定能同时覆盖A733、屏幕与推理峰值。

主机电源第一阶段已冻结为轻量、可升级路线：

- `19A`：P0主机仍以连续4 h实际场地负载为硬门；旧H618路线的20.6–29.4 Wh不再作为当前采购区间。先测Zero 3W基础闭环、本地3B真实占空比和屏幕功耗，再用`average_W × 4 / efficiency`计算；优先做满足门槛的最小/轻量方案，不先为8 h把三轴移动舱做重。
- `20A`：P0使用不拆壳、不改电芯的完整成品电源/充电宝采集真实数据，输出固定5V。台架电源建议具有≥5A量程并从受控限流起测；最终电源额定能力由启动、屏幕、风扇、主动视频转接、Wi-Fi/BLE、3B峰值和持续负载共同决定，不能把旧“至少3A”当作新板结论。禁止使用PD诱骗板请求9/12V。5V经受保护分配点分别进入Orange Pi与屏幕，屏幕不反向代供主板。
- P0 不承诺边充边用，充电时正常关闭 Linux 和负载。成品电源的低电关断、自动休眠、按键语义、静态耗电和 5 V 压降必须作为选型测试项；若其行为不可控则更换，不拆开改造。
- `21A`：P0 先实现“关机请求键 → OCLive checkpoint → Linux 正常 shutdown → 明确 safe-to-cut 指示 → 用户切断主 5 V”的手动闭环。任意硬开关不得在日常流程中直接切正在运行的 Linux。
- 普通成品充电宝通常不向 Linux 暴露可信的剩余电量/低电信号，因此 P0 不虚构自动欠压关机。若实物提供可验证遥测/告警，可接入同一 power provider；否则自动低电保存、延迟断电与真正 5 V 切断留给 V1 监督 MCU/power-path 方案。
- V1 是否采用定制 1S 升压、2S 降压、专用主机电池匣或监督 MCU，必须由 P0 的平均/峰值功耗、4 h 余量、厚度、温升和三轴手感决定。
- `22A`：P0 成品电源与屏幕、Orange Pi 一起位于三轴移动主机舱内，优先保持一体外观和内部短线。移动质量必须包含屏、板、电源、外壳、关节从动件、接头和线缆，用 225/285/345 g 假体先测；完整移动舱超过 350 g 时先减容量/外壳/接口或切紧凑件，仍不通过才把电池移到固定导轨底座并设计单根受控 5 V 柔性跨轴线，不能靠无限增加关节摩擦掩盖超重。
- `23A`：P0 保持 Linux、BLE 和触摸控制器运行，只管理背光状态：active → dim → backlight_off。30 s 无交互降亮、120 s Standby 关背光和 ≤300 ms 唤醒均为 `ESTIMATED` 起点；握持、运动、触摸、按键或关键角色事件可唤醒。屏幕选型必须验证软件可控背光/PWM 或可靠控制引脚；显示全黑不算关背光，P0 不通过反复断开整个 HDMI 板电源省电。
- `23A` 后续优化门：先实测各亮度功耗、唤醒延迟、误唤醒和户外可见度。只有背光策略仍不能满足 4 h，才依次评审环境光传感器、CPU governor/外设电源域和更深系统低功耗；不在 P0 使用可能破坏 BLE/HDMI 恢复的 Linux suspend。
- `24A`：P0 使用高耐久/工业级 MicroSD 启动和保存 OCLive，容量/料号待镜像与日志实测。ext4 保留日志；角色状态通过 SQLite 原子事务、周期 checkpoint、状态变化 checkpoint 和安全关机 checkpoint 保存；原始 FSR/IMU 高频采样默认不持久化。系统与应用日志必须轮转并设总量上限，支持配置/角色数据导出和恢复。
- `24A` 后续优化门：在副本镜像和专用测试卡上做异常断电、日志写满、数据库恢复和卡更换测试。若出现不可接受的启动失败、checkpoint 回退或写入寿命问题，再评审 USB SSD；只有 USB 体积/功耗仍不可接受时才重开带 eMMC 主板选型。不能只因“SSD/eMMC更高级”提前增加线缆或换板。

XIAO 原型节点仅使用受保护可充 1S LiPo。CR2032/CR1632 是不可充电电池；若后续采用，必须换专用低功耗节点与不可充电电源路径，不能接 XIAO 充电路径。

节点电池机械方向已冻结为可拆式、同电气规范、两种尺寸候选：

- 前后均为受保护可充 1S LiPo，标称 3.7 V、充满 4.2 V；电池接口的极性、防呆、保护要求和电量语义相同，但前后电池匣不强制机械互换。
- 前节点继续保留 200–260 mAh 大尺寸候选，后节点保留 120–160 mAh 小尺寸候选；容量和电芯型号均未冻结。若实测证明小电池满足 8 h + 20% 余量，可缩小；若不满足则在各自机械上限内增大。
- 电芯、PCM、导线、接点和壳体组成独立硬壳电池匣；电芯不裸露、不靠导线承载、不被卡扣或握持力挤压。电池匣使用非磁性机械滑轨/卡扣方向，P0 可先用带检修盖的可拆托盘验证。
- `ESTIMATED` 电池匣包络：后节点约 35–38 × 16–20 × 8–10 mm、6–9 g；前节点约 36–42 × 22–24 × 8–11 mm、8–12 g。相对固定软包每节点预计增加 3–6 cm³ 与 2–5 g，必须用尺寸/质量假体修正节点总包络和完整套件质量。
- 前后节点续航门相同：连续 8 h 真实会话回放后保留约 20% 容量。平均电流、低温、重连峰值、充电温升和老化数据决定最终大/小电池，而不是纸面 mAh。
- 节点使用凹入式物理硬开关。P0 可在电池正极串联开关；关断时真正断开电池。若 USB 仍可给板供电或关断时不能给电池充电，按明确维护行为记录，不伪装为完全无电。
- 充电路线冻结为 `14B`：P0 使用 XIAO 自带 USB-C 作为维护、烧录和装机直充入口；外壳必须让接口可达并配置防尘/防撞结构。P0 充电时节点停用，硬开关置于允许电池连接的位置；实际充电电流、充满时间、温升以及 USB 接入时是否同时启动节点均需实测并写入维护说明。
- 两种电池匣从 P0 起统一预留非磁性、机械防反的 `BAT+ / BAT−` 充电触点基准与充电座空间，但 P0 不要求完成独立充电座。V1 再开发 USB-C 输入、专用 1S CC/CV 充电管理的非磁性充电座，使拆下的大小电池匣可独立充电；两种匣可用同一触点定义和可调/分档定位结构。
- 不得把常见 7.4/11.1 V wargame 电池或其无电压防呆的插头直接复用为节点 1S 电池接口。
- P0 电量检测采用 XIAO nRF52840 Sense 板载电池分压与 ADC：按官方时序用 P0.14 控制读取路径、由 P0.31 采样。固件上报真实 `battery_mv` 和粗粒度 `normal / low / critical`，不把单次电压换算成虚假精确百分比。电量提供者做成可替换接口；协议为未来专用 fuel gauge 的可选 SOC/健康度字段预留版本化扩展，载板保留可访问 I2C 与待选小板/焊盘空间但 P0 不装。
- 节点低电采用“告警 → 临界降级 → 有余量受控关机”：先停止原始调试流、高频日志等非必要负载，保留握持/运动结论与 BLE 心跳；到实测 `shutdown_enter_mv + confirm_ms` 后发送 `shutdown_pending`，等待主机限时确认，记录最小本地关机原因并进入低功耗关机。无确认也必须在截止时间内关机，不能等到 PCM 切断或 brownout。
- 关机余量不先写死为百分比。到货后在低温、BLE 重连/发射峰值和老化电芯下测出 brownout/PCM 边界，反向选择关机点；验收要求最差工况连续 10 次都能完成最终通知、主机 checkpoint 和节点关机，并仍留有可测电压余量。进入低功耗关机后由换电/充电并切换物理开关或复位重新启动，不依赖电压回升自动唤醒。

## 4. 接口契约

每个接口必须同时记录：机械、电气、传输协议、语义和时间。

| 逻辑接口 | V0 路径 | 约束 |
|----------|---------|------|
| FRONT_FSR | two-wire keyed connector → divider → logical ADC alias | 简单分压；RM 与可选远端 EOL 实测冻结 |
| REAR_FSR | two-wire keyed connector → divider → logical ADC alias | 简单分压；后握独立 calibration profile |
| TRIGGER_AUX | dry contact → pulled-up GPIO | input-only；物理 pin 待接线图冻结 |
| FRONT_IMU | XIAO Sense onboard IMU | 固定载体参考，不随屏幕旋转 |
| IMU_FRAME | sensor axes → assembly mount_transform → canonical carrier frame | 右手系：+X向前、+Y向左、+Z向上；transform绑定assembly_id |
| IMU_CALIBRATION | explicit UI wizard → temporary raw capture → validated profile | 不后台静默学习；用户选择放低/举起点并预览后保存 |
| NODE_BAT | removable protected 1S LiPo cartridge → hard switch → verified BAT+/BAT− | 前后同电气规范、两种机械尺寸；P0 XIAO USB-C 装机直充，预留 V1 非磁性充电座触点/包络 |
| NODE_BAT_MON | onboard divider: P0.14 control → P0.31 ADC | P0 输出 battery_mv + coarse state；未来 fuel gauge 不破坏上层契约 |
| HOST_5V_P0 | complete protected 5 V source → protected distribution → separate host/display branches | fixed 5V；台架≥5A量程、限流起测；最终额定按峰值实测；不得请求9/12V；禁止屏幕反供主板 |
| HOST_SHUTDOWN_P0 | physical request button → Linux service → OCLive checkpoint/shutdown → safe-to-cut indication | P0 手动切电；无可信电源遥测时不宣称自动欠压关机 |
| DISPLAY_BACKLIGHT | host policy → verified PWM/control pin → backlight driver | active/dim/off；不把黑画面当关背光；触摸/主机保持运行 |
| PHYSICAL_KEYS | recessed high-force POWER + MAINT → GPIO/input service | POWER负责背光/安全关机；MAINT负责返回/维护授权；锁定时保留最小操作 |
| HOST_STORAGE_P0 | high-endurance MicroSD → ext4 journal → SQLite atomic state | 高频传感样本不落盘；checkpoint、日志上限、导出/恢复必测 |
| BLE_ENROLLMENT | recessed maintenance button → timed pairing window → one candidate confirmation | P0一次只登记一个节点；约60 s仅为ESTIMATED；平时关闭未授权配对 |
| BLE_TRUST | BlueZ per-device bond store + trusted identity allowlist | 每节点独立密钥；新增/撤销不重配其它节点；OCLive数据库不保存BLE长期密钥 |
| NODE_REGISTRY | stable node_uid + capabilities + optional bindings + assembly/calibration refs | 记录实际存在能力，不保存理想配件蓝图；BLE地址不得充当永久身份 |
| DISPLAY | USB-C DP Alt Mode → active DP-to-HDMI → internal short HDMI → screen | 主动转接纳入供电/EDID/包络；不跨三轴 |
| TOUCH | USB HID | P1 与相机并发时经 USB 2.0 hub |
| CAMERA P1 | UVC USB 2.0 MJPEG | hidden main cable + replaceable short jumper |

FSR V0 电气与维修接口已冻结：

- 采用简单分压，不在 P0 增加运放或带电位器比较器模块。起点为 `3V3 → (FSR || optional R_EOL) → ADC tap → RM 47 kΩ → GND`；47 kΩ、R_EOL、ADC gain/reference/acquisition time 均待台架实测。
- 前后传感件各使用藏在节点盒内的微型两针防呆锁定连接器，连接器前后均做应变释放。具体系列、线规和额定插拔次数待尺寸假体后冻结。
- 不默认直接加热焊接 FSR 聚合物尾片；按实际料号采用厂商允许的 solder tab、压接或夹持转接。连接器和焊点不承受握持或拉线载荷。
- 在 FSR/传感夹层远端预留 EOL 高阻电阻位置，仍保持两线接口。EOL 必须位于被诊断短线和连接器的传感器一侧；放在节点 PCB 上不能发现中间开路。
- EOL 只增强正常 released 与线路 open 的区分；短路、机械持续预压和 ADC stuck-high 仍由极值/不变性与上下文诊断。台架比较不装、候选高阻值和开路注入后再决定是否装及阻值。

协议字段单位：`battery_mv` 用 mV；`battery_state` 为 `normal / low / critical`；`power_state` 至少区分 `running / shutdown_pending`；未来 `soc_permille`/健康度只能作为带来源和有效位的可选字段，不能由粗略电压伪造。`pressure_norm` 用 0–1000，不宣称牛顿；`sequence` 每节点单调递增；`boot_id` 每次冷启动变化；短时 TTL 使用节点/主机单调时钟。若上传 accel/gyro，协议必须冻结 mg 或 mm/s²、mdps 和轴向坐标系。

BLE P0 数据与重连路线已冻结：

- `28A`：正常链路同时包含即时edge event与周期CurrentState snapshot。状态改变立即通知，完整当前状态约1 Hz兜底；事件丢失可由下一快照修正。FSR/IMU原始值只在有界维护模式输出，不作为下场常态。
- 建议快照字段至少覆盖 `node_uid / boot_id / sequence / monotonic_ms / current state bits / battery / fault_flags / confidence`；具体二进制布局和Characteristic划分在schema实现时版本化。
- `29A`：节点内部较快采样、BLE低频传结论。FSR 25–50 Hz、IMU 26–52 Hz、CurrentState 1 Hz、电池30–60 s、调试原始流最高约10 Hz均为 `ESTIMATED` 起点；握持/释放、motion变化和关键fault立即通知。通过功耗、p95延迟和误判实测冻结。
- `30A`：断连期间不缓存重放握持、运动或trigger等过期动作。相关事实按TTL进入unknown；重连后发送HELLO和新鲜CurrentState。boot_id未变表示链路恢复，变化表示节点重启并必须丢弃旧滤波/sequence上下文。断线期间变化次数可作为诊断计数上传，但不能注入角色状态机。
- 自动重连退避从约0.5/1/2/5 s封顶起测；具体扫描、连接参数和TTL按双节点、Wi-Fi与场地干扰实测。恢复后先校验信任、schema/capability、assembly/calibration，再接受新鲜状态。

关机语义作为生命周期事实单独处理：

- `shutdown_pending`/关机原因不以自由文本“关键词”落库，而使用结构化 `LifecycleEvent`，至少含 `event_id / scope(host|node_uid) / reason / requested_at / clean / pre_shutdown_consumed / resume_consumed`。角色层只接收规范化语义，不接收底层错误码、路径或堆栈。
- `31A`：关机前与恢复后分别允许一次角色反馈。两阶段使用同一event_id但独立消费位；关机前没来得及反馈不阻止关机，恢复后仍可说明。Orange Pi保存`last_shutdown`和有界生命周期历史，节点只存自身最小原因。
- `32A-R1`：关机现场不调用AI/网络，只从预生成表达缓存选择与reason/phase匹配且近期未重复的短句；不可用时立即使用固定兜底。AI只在电量正常、系统空闲且缓存不足/角色revision变化时异步补充候选。缓存不是角色长期记忆。
- 表达缓存项至少含 `phrase_id / reason_class / phase(pre_shutdown|recovered) / text / context_revision / generated_at / last_used_at / used_count`。候选先过长度、格式、角色边界和禁止控制指令校验；每类约3条只是`ESTIMATED`起点。
- `33A`：reason使用小型稳定枚举：`manual / low_battery / maintenance / update / thermal / storage_fault / unclean_restart`，可版本化扩展。角色只看到reason_class、友好scope和phase；`diagnostic_code`及原始错误只进系统诊断层。
- Orange Pi在checkpoint保存`last_shutdown`，另保留有界生命周期历史（最近32条或90天为ESTIMATED起点）。瞬时传感事件不补发；生命周期两阶段分别原子标记消费，不能重复污染角色记忆。缺少clean marker时在下次启动派生`unclean_restart`。

Schema 实现前任务：

1. `node.status.power_source` 增加 `lipo_1s`。
2. 前/后 grip 继续由 node_id/capability 区分。
3. 移除`trigger.test/device.triggered`；P0使用只读`AuxControlContact(control_id=primary_trigger)`及`control.primary.engaged/released`，不得推导机构完成动作。
4. 补有效/无效样例、sequence wrap、unknown 派生和重连快照测试。
5. 补 `battery_mv / battery_state / power_state / boot_id`、低电迟滞、关机确认超时和未来 SOC 字段缺失时的兼容测试。
6. 增加节点注册表与配对状态机：`pending / trusted / revoked`；节点上报稳定 `node_uid`、每次冷启动变化的 `boot_id`、`capabilities`、`schema/firmware_version` 和 `assembly_id`。安装 `bindings` 可为空且由主机维护，不由角色包或节点固件硬编码。
7. 增加edge + snapshot对拍、1 Hz快照修复丢event、重连HELLO、boot_id变化、过期事件不重放和LifecycleEvent双阶段各消费一次的测试。
8. 增加表达缓存补充/校验/去重/固定兜底、AI不可用、关机截止时间、unclean restart和角色事件中无diagnostic_code的测试。
9. 增加坐标变换基准向量、装反、assembly变化、用户双点+动作校准、预览拒绝/回滚、开机受动不自动归零和shock不生成trigger测试。

BLE登记与生态边界已冻结：

- `25A`：长按凹入式维护键开启限时配对界面；一次只确认一个候选节点，成功或超时即关闭窗口。触摸UI负责显示、命名和确认，但实体动作是进入授权窗口的根条件。
- `26A-R1`：每个节点与主机建立独立BLE bond；新增节点只在BlueZ bond store和可信白名单中增加该节点，旧节点密钥不轮换。丢失/损坏时只撤销对应身份和bond。仅主机信任库丢失、整机重置或确认泄露时才全量重配。P0不增加组密钥或自定义应用层加密；V1可在不改变注册表语义的情况下增加二维码/OOB出厂身份。
- `27A-R1`：节点注册表借鉴OCLive“登记实际能力”的思想，但不是角色蓝图、枪型蓝图或理想配置清单。最小记录为 `node_uid / display_name / trust_state / capabilities / optional bindings / firmware_version / schema_version / assembly_id / calibration_ref / last_seen`。
- `node_uid` 是跨重启稳定的永久设备身份，不能只取可能随机变化的BLE地址；具体采用硬件ID派生还是首次预配随机UUID待固件安全评审。`boot_id` 每次冷启动变化；`assembly_id` 标识节点板、传感器与机械夹层的装配，变化时旧校准失效。
- `bindings` 只描述当前现实安装，例如 `grip.front`、`grip.rear`、`carrier.motion`，不是“必须安装哪些件”的蓝图。缺失节点使相关事实为unknown或降低融合证据，不让角色包加载失败。角色包只消费稳定DeviceEvent，不感知FSR型号、节点厂商或枪型拓扑。

IMU方向、语义与用户校准已冻结：

- `34A`：全系统使用右手载体坐标 `+X向前 / +Y向左 / +Z向上`。节点外壳用不对称定位降低装反；实际IMU轴通过`mount_transform`映射到载体坐标。transform绑定`assembly_id`，换安装方向后旧姿态校准失效。重力只能判断上下，不能自动猜出枪口方向，因此不做全自动安装识别。
- `35A-R1`：`pose.raised/lowered/unknown`由用户明确设置的姿态样本得到，不使用全产品统一角度。校准向导让用户自行选择其自然放低点和举起点，记录重力方向、pitch/roll分布、陀螺偏置和过渡范围；Ready仍需后握等融合证据，raised不是瞄准或目标语义。
- 校准功能分三层而不混写：①设备静止时的gyro bias/self-test；②安装方向mount_transform；③用户行为profile（放低、举起、运动过渡）。建议快速流程为静止3 s、放低2 s、举起2 s、自然举放5次并预览时间线，数值均为`ESTIMATED`。
- 用户可为同一装配保存和切换多个命名profile；profile至少绑定`node_uid + assembly_id + binding`并记录calibration_version、创建时间、bias、transform、两姿态中心/离散、迟滞和最短稳定时间。assembly或binding变化要求重新确认/校准，不能静默套用。
- 原始IMU仅在用户主动进入校准/维护模式时临时录制，生成profile后默认丢弃或由用户显式导出。正常运行不后台自学习、不持续落盘，避免阈值漂移、写卡和“今天与昨天行为不同”。提供预览、接受、取消、恢复上一版和重置入口。
- `36A`：P0只输出`motion.idle/moving`、`pose.raised/lowered/unknown`；较大冲击为诊断`shock.observed`，不生成device.triggered。冲击可帮助发现跌落/松动，但不宣称识别击发，且不进入弹道、目标或火控路线。

屏幕交互与实体按键P0路线已冻结：

- `37A-R1`：使用两个独立、凹入、高阻尼/高操作力实体键。POWER短按唤醒/熄背光、长按请求安全关机；MAINT短按返回/确认最小操作、长按请求维护入口。触摸负责正常UI，实体键保证触摸/显示异常或UI锁定时仍能安全退出与授权维护。
- 屏幕舱贴合枪身/平面时，外圈海绵和刚性止挡先承力；LCD和按钮均退入保护平面。泡棉不能持续压按钮，也不能成为唯一硬限位。按钮位置需通过戴手套、贴墙、装包和碰撞假体测试。
- UI锁定后只保留POWER背光/关机、MAINT返回/查看提示等安全动作。进入配对/校准等维护操作，建议要求载体非Ready且静止、长按MAINT约3 s，并由另一实体键或屏幕二次确认；时间仍为ESTIMATED。高阻尼不能替代凹入、位置防护和软件长按门。
- `38A-R1`：P0主界面角色优先，边缘小状态栏显示主机电量、前后节点、网络与模式；详细电压、RSSI、压力和版本仅进维护页。Ready时进一步减少文字；不显示准星、弹道、目标识别或射击参数。
- UI契约预留`interaction_mode`，以后可增加把玩/下场模式并改变信息密度、触摸权限和角色主动性，但P0只实现一个角色优先默认模式，不提前复制两套页面/状态机。模式不得改变底层传感事实或安全关机语义。
- `39A-R1`：运动/Ready时锁定配对、解绑、校准和设置写操作；关键修改使用大触摸目标和长按/二次确认。锁定提示必须可由实体键读取/退出；只有静止且非Ready时才能解锁维护。首轮触摸目标约12 mm为ESTIMATED，需戴手套实测。

枪械特化感知内核路线已冻结：

- `40A-R1`：建立独立于OCLive的“器灵枪械感知内核”。它拥有节点注册、校准引用、数据新鲜度、证据融合、载体状态机和故障降级，只负责回答“物理上发生了什么”；OCLive继续拥有角色、记忆、情绪、回合和回复，只回答“角色如何看待并表达”。
- `41A-R1 / 43B-R1`：感知内核位于本仓独立Rust crate，不依赖OCLive crate，也不依赖BlueZ、GPIO、SQLite、DRM或具体屏幕。P0由同一个`ailive-gun-spirit-host`进程组合感知crate与OCLive，但只有bridge/composition root可同时引用双方；两个内核源码互相零依赖、互相不知道。暂不增加第二进程、第三仓库、IPC服务、插件系统或复杂任务图。
- `42A-R1`：感知内核输出持续的`PerceptionState`快照与离散的`DeviceEvent`边沿。快照供重连、自愈、即时UI和诊断读取；只有经过策略筛选的事件进入OCLive触发角色回合，避免每秒快照导致重复说话。
- 物理适配器只把BLE/mock/Linux输入转换成`SensorObservation`；核心不读取设备文件。OCLive不读取ADC、IMU原始流、BLE地址或传感器阈值，也不得回写物理事实。
- 该内核是wargame发射器/枪械交互语义的特化，不是OCLive角色内核的特化。它可借鉴OCLive的薄核、稳定契约、注册表、回放和确定性状态转换，但不复制角色蓝图、记忆、场景编排或模型调用。
- P0合作方向为单向事实链：`adapters → perception kernel → bridge → OCLive`。bridge显式转换双方类型，不把任何一侧内部对象直接透传给另一侧。角色回复失败不得回滚物理状态；当前观察链不定义`actuator.*`、波箱、电机、供弹或火控控制接口。

共同前端与输出仲裁路线已冻结：

- `45A-R1`：只有`ailive-gun-spirit-host`的Output Arbiter拥有最终屏幕控制权。枪械感知内核不渲染像素/HTML，也不认识屏幕；OCLive只返回`RoleCue`，不直接操作DRM、WebView或显示状态。
- 感知侧由host把`PerceptionState / DeviceEvent / LifecycleEvent`投影为确定性`SystemViewState/SystemCue`；OCLive侧贡献角色表情、短文本与情绪表现。两者共同表现前端，但不共享渲染状态。
- 首轮画面按三层组织：底层`RoleStage`显示角色；常驻`StatusBar`显示电量、节点、网络和模式；最高`SystemOverlay`显示关机、维护确认、锁定和必须处理的故障。系统层不能被角色回复遮挡，普通状态也不应无故覆盖角色主体。
- 所有候选输出携带来源、优先级、TTL及可选`related_event_id/context_revision`。迟到或上下文已失效的RoleCue丢弃；OCLive不可用时保留系统UI和本地角色兜底，不回滚感知状态。
- 具体前端技术、像素布局、动画系统与主题均在Output Arbiter/renderer外侧按需替换，不进入两个内核。

宿主策略、桥接扩展与角色回合节流已冻结：

- `44A-R1`：`interaction_mode`由`ailive-gun-spirit-host`交互策略层拥有，只能由明确用户操作/维护流程切换。模式可改变信息密度、触摸权限、角色主动程度和DeviceEvent投递策略，但不得改变SensorObservation事实、感知融合、安全关机或故障语义；OCLive可读取模式上下文，不能自行切换。
- `46A-R1`：感知内核的新事件/能力先进入版本化本地契约，只有bridge存在显式、测试覆盖的映射时才投递OCLive。未映射事实继续用于PerceptionState、系统UI和诊断，默认不作为自由字符串进入Prompt，也不要求OCLive同步升级。
- `47A-R1`：即时系统反馈不等待角色回合；只有有意义且通过门控的状态转换触发OCLive。bridge按event_id去重，支持短窗口合并、每类冷却和进行中回合抑制；周期快照、阈值抖动和重复Held不得反复触发角色说话。具体时间窗保持ESTIMATED并由交互回放冻结。

未来语音/文字/感知输入调度边界已冻结，但语音实现仍在P0之后：

- `48A-R1`：语音、文字和sensor turn的主对话调度属于OCLive侧通用Input Scheduler。感知内核不认识ASR、对话队列、角色或当前是否有人说话；integration bridge只投递中性、版本化的设备上下文。
- `49A-R1`：OCLive回合采用“一个PrimaryInput + 有界AuxContext”。没有显式语音/文字时，通过门控的DeviceEvent可成为sensor-origin PrimaryInput；语音/文字存在时，它们是PrimaryInput，近期DeviceContext只作为辅助上下文。角色包在OCLive侧把中性事实映射为角色情绪偏置、回复概率/冷却和表现，感知内核不输出OCLive情绪词表。
- `50A-R1`：对话优先级为显式用户语音/文字高于环境sensor chatter；VAD/语音采集开始后，尚未提交的sensor turn取消并合并进语音上下文，已生成但失去context_revision/TTL有效性的RoleCue丢弃。ASR partial只更新临时输入，不触发角色回复，final transcript才可提交。
- 安全/运行通道独立于对话优先级：critical low battery、shutdown、维护确认和故障SystemCue立即由host处理，不被语音压住，也不等待Input Scheduler或OCLive。
- DeviceContext只包含稳定事实、置信度、时效和少量近期变化，不携带原始ADC/IMU、音频或无限事件历史。合并窗口、缓存大小与取消时序保持ESTIMATED，待语音阶段用回放冻结。

输入合并与枪械特化角色扩展格式已冻结：

- `51A-R1`：通过门控的sensor事件先进入约500–800 ms的对话合并窗口；窗口内VAD/显式文字开始则并入该用户回合，否则才允许形成sensor PrimaryInput。该时间仅为ESTIMATED，不延迟SystemCue、屏幕唤醒或安全处理。
- `52A-R1`：每个回合的DeviceContext使用有界快照：当前Held/Ready/Standby、interaction_mode、节点健康/电量等级、最近约2 s内最多4个语义变化、event time/confidence/context_revision。时间和数量均为ESTIMATED；不包含原始采样、音频、诊断堆栈和无限历史。
- `53A-R2`：`device_reactions`不加入OCLive标准角色包schema。它属于项目所有者二次开发的枪械特化角色扩展包/伴随profile，使用独立schema/version并引用基础OCLive角色身份；专用loader/adapter使OCLive宿主能够识别其反应策略。
- 枪械扩展包只描述设备事件、适用模式、情绪偏置、主动回复概率、冷却与表现偏好，不复制或重新定义基础角色的人格、记忆与通用角色包格式。未安装/不兼容扩展时，基础角色仍可加载，相关事件默认不主动回复但可保留为通用上下文。

枪械角色扩展的包装、导入与文档边界已冻结：

- `54A-R1`：使用独立伴随包。它以稳定`base_role_id`、版本要求和可选内容摘要引用已安装的标准OCLive角色包，不复制基础人格、记忆、Prompt和通用资产。
- `55A-R1`：专用loader属于本仓`ailive-gun-spirit-host`的OCLive integration adapter，不修改OCLive标准角色包loader/schema。所谓“OCLive识别”是本二次开发宿主验证并投影为OCLive通用输入，而不是OCLive主仓原生承诺枪械格式。
- `56A-R1`：伴随包缺失、损坏、签名/校验失败或版本不兼容时，基础角色照常启动，枪械专属主动反应关闭，维护页显示稳定原因；不得猜字段、跨角色套用或退回不兼容策略。
- 文档隔离门：四季宝感知内核、枪械扩展包、硬件HostProfile和机械/电气路线只在`oclive-四季宝器灵`仓维护；OCLive主仓只记录真正跨宿主通用的内核契约。本项目通过链接引用OCLive SSOT，不向主仓复制四季宝设计文档。

感知契约、代码地基和伴随包形态已进一步冻结：

- `69A`：首个实现基线采用强类型`SensorObservation v0.2 + PerceptionState v0.2 + DeviceEvent v0.2`三契约。每个kind使用封闭payload与明确单位/unknown语义；现有v0.1仅保留为未交付历史草案，不制作兼容adapter。`mode.changed`留在Host，连接/低电/系统故障进入状态或SystemCue，不混入角色DeviceEvent。
- `70B-R1`：迁移为`ailive-gun-spirit-contracts`、`ailive-gun-spirit-perception-core`、`ailive-gun-spirit-host`三个crate的最小workspace；仍只部署一个host进程。contracts只放DTO/version/schema，core只放确定性融合与状态机，host拥有OCLive bridge、Input Scheduler/Output Arbiter接线、BLE/Linux/Web和导入器。
- 命名冻结：仓库slug为`oclive-四季宝器灵`，正式产品名为`A.I.Live-ai枪娘器灵`，新技术标识使用ASCII `ailive-gun-spirit`/`ailive.gun-spirit`。旧Skippy命名只作历史/迁移识别。
- `71A-R2`：正式使用独立“枪械交互伴随包”（Gun Interaction Companion Pack），格式`ailive.gun-spirit.interaction/1`；经77A细化后的受管目录为`content/gun-interaction-packs/<pack_id>/<version>/gun-interaction.json`。基础OCLive角色包与伴随包分别导入、分别校验、分别存储，通过`base_role.id + version_req`关联。
- 导入体验沿用OCLive传统：用户从电脑选择目录或压缩包；专用importer在临时区完成schema、identity/version、摘要、大小和路径安全校验后原子安装。不得直接执行外部目录、修改基础角色包或把伴随包交给OCLive标准角色loader。多包命中同一角色时由本机绑定注册表显式选择一个活动版本，不按目录顺序猜测。
- P0伴随包只允许声明事件/模式映射、情绪偏置、主动回复概率、冷却及稳定视觉语义ID；不携带代码、shell、远程URL、任意路径、模型资产、密钥或用户记忆。失败时fail-open并显示维护原因。

契约真源、时间与未知态细节已冻结：

- `72A`：`ailive-gun-spirit-contracts`内的Rust强类型是语义真源，JSON Schema由相同类型生成、提交并在CI检查零差异。有效/无效fixtures覆盖跨字段不变量；BLE二进制、Web JSON和诊断格式都是codec，不得另立语义。BLE使用golden vectors做跨语言对拍。
- `73B`：观察携带`node_uid + boot_id + sequence + node_monotonic_ms`，只在同一节点/boot内排序。Host补`host_boot_id + received_monotonic_ms`，跨节点新鲜度、TTL与融合窗口只使用Host单调时间；不同节点时钟不直接比较，wall-clock不参与状态机。节点重启更换boot_id并允许sequence归零。
- `74A`：PerceptionState每个可失效事实使用`Known{value, observed_at, fresh_until, confidence, source_refs}`或`Unknown{reason, since_revision}`。P0 unknown reason闭集为`never_observed/node_offline/stale/uncalibrated/sensor_fault`；Unknown不能继续携带旧值影响融合或角色，last-known只可留在隔离诊断摘要。
- `75A-R1`：Known Fact使用`confidence_milli: 0..=1000`表达规则/校准下的证据强度，不是统计概率；Unknown无confidence，UI只投影为正常/不稳定/需校准等语义，不显示百分比。
- `76A-R1`：每个Fact最多4个`source_ref`，只含`observation_id/node_uid/capability/relation(supports|contradicts)`；完整Observation留在有界诊断回放缓冲，不嵌入PerceptionState。
- `77A-R1`：同一伴随包多个SemVer版本并存，本机binding显式选择活动`pack_id + version`。新版本验证/原子安装后才切换；相同id/version/digest重复导入幂等，相同id/version但digest不同拒绝为`version_collision`；旧版显式删除且当前活动版不可删。
- `78A-R1`：BLE `NodeHello`列出明确支持的contract versions/capabilities，Host选择最高共同版本；无交集则节点为`incompatible_protocol`且不参与融合。协商版本内未知/非法variant先隔离单帧并记故障，有界窗口重复违规再隔离节点；不把未知数据透传核心/OCLive。
- `79A-R1`：PerceptionState只在语义/健康事实改变时增加revision。新订阅、重连、renderer恢复和心跳可重发相同revision完整快照，但不得产生DeviceEvent或角色回合；心跳周期留给HostProfile和实测。
- `80A-R1`：伴随包每个event只有一条default规则及按mode的字段覆盖，合并为`default → mode override → Host安全/用户上限`。P0不提供表达式、条件链、优先级规则、脚本或Prompt；event/mode/visual id必须来自受控注册表。
- `81A-R1`：主动回复概率由Host Input Scheduler执行，顺序为事件有效性/去重/TTL→规则解析→active_reply→Host上限→cooldown→概率抽取→sensor turn，再服从语音优先/合并。测试注入确定RNG，生产记录decision id/有效概率/结果。感知`confidence_milli`与调度`probability_milli`严格分离。
- `82A-R1`：模式用`active_reply: disabled | probabilistic{probability_milli}`显式控制。disabled只禁止独立sensor turn，不删除事实、事件、System UI、诊断或用户主回合的中性DeviceContext。
- `83A-R1`：binding精确钉住`pack_id + pack_version`，基础角色每次加载/升级后重验`base_role.id + version_req`。失配标记`inactive_incompatible_role`并fail-open；Host可推荐兼容候选，但切换必须用户确认，不自动换包、不忽略范围。
- 四职责架构门禁：感知内核产事实；伴随包声明策略；Host/Input Scheduler执行调度；OCLive理解和表达。新增功能若跨界，必须先说明理由/替代/代价/回退并新增ADR，不能顺手耦合。Output Arbiter唯一前端所有权继续有效。
- `84A-R1`：协议违规分级隔离，初值`ESTIMATED`为单帧丢弃/限速记账、10秒3条非法应用帧→隔离30秒、连续3个连接周期触发→持久`protocol_fault`并需维护确认。错误协商版本/超长等硬违规可直接断本次连接；RSSI/普通断连不计。全部阈值归HostProfile并待fault injection/实机覆盖。
- `85A-R1`：P0本机伴随包不强制作者签名，但必须规范内容清单+SHA-256、明确用户确认及严格schema/路径/文件类型/大小验证；UI不得把摘要称为作者认证。未来公开分发的Ed25519身份、信任库、换钥/吊销另立ADR。
- `86A-R1`：包索引、精确binding、禁用原因和有界导入/切换审计进入Host独立SQLite`ailive-gun-spirit-state.db`，与OCLive聊天/记忆DB分离；包目录不可变，DB只存元数据/相对路径/摘要。
- `87A-R1`：SensorObservation P0仅含GripContact、MotionClass、低频滤波OrientationEstimate、只读AuxControlContact和诊断ShockObserved。Host由方向估计+用户校准派生pose；NodeStatus独立，unknown由Host派生，禁止正常原始ADC/IMU流。
- `88A-R1`：PerceptionState含carrier_state Fact<Standby|Held|Ready>、前后grip、motion、pose、primary_control和有界node_health；移除Active。睡眠归Host，辅助触点不改变carrier state。
- `89A-R1`：DeviceEvent闭集为carrier held/ready entered/exited及control.primary engaged/released。没有device.triggered/weapon.fired；shock、节点/电量、模式和生命周期不进入DeviceEvent。
- `90A-R1`：外部pack先复制到受管根同文件系统`.staging/<operation_id>`，验证后由`install_operations`记录阶段，再原子rename发布并补记installed；安装与binding切换分事务。启动按journal、目录和digest恢复；缺文件标记missing/disabled，孤立目录隔离，不依赖跨盘rename。
- `91A-R1`：field SQLite使用WAL+FULL、foreign_keys和明确busy_timeout；受控checkpoint并限制WAL，非正常启动做恢复检查，运行中备份使用SQLite一致性机制。development可显式NORMAL，但发布/断电测试必须使用field FULL；参数等实测提交延迟、关机预算和写放大后修正。
- `92A-R1`：诊断分类限额，初值均ESTIMATED：安装/绑定/安全审计90天或1000条，协议/节点故障30天或5000条，资源/renderer按时间+容量，field原始传感默认关闭、维护采集10分钟或20 MiB后自动过期。Host不复制OCLive对话、原始音频或角色记忆，导出须显式触发并带脱敏清单。
- `93A-R1`：采用持久化白名单。Host保存节点注册/信任、pack/binding、已确认校准、设备设置和Lifecycle；OCLive独立保存角色连续状态与预生成表达。Held/Ready/grip/motion/pose/control、滤波/去抖、BLE会话、boot/sequence、概率和单调cooldown全部瞬态，重启先Unknown。新字段未声明owner/恢复语义时默认不落盘。
- `94A-R1`：field产品采用Low卸载可选负载、Critical Reserve停止新写操作/回合并限时checkpoint/poweroff、监督MCU/PMIC/锁存负载开关完成主rail切断的两级链路；BMS只兜底。阈值/迟滞/deadline/预留Wh均待实测。首轮台架允许按现有手动safe-to-cut切电跑通同一软件顺序，但不得作为成品自动保护。
- `95A-R1`：启动经StorageRecovery、OCLive连续状态恢复、NodeRearm和SensorBaseline进入Ready/Degraded。首次观察只建立baseline，不从Unknown产生held/ready/control事件；之后新边沿才发。缺节点保持Unknown并降级；clean/unclean只作为一次LifecycleContext进入OCLive，不补播旧传感动作。
- 待冻结：OrientationEstimate固定点范围/发送频率、启动必要capability集合、baseline/degraded等待时间及低电阈值在DTO实现与实机中确认。

OCLive Resource Coordinator审查与57–59裁决（2026-08-26）：

- 旧“蓝图steps/dual-core调度”不是当前Stable执行路径；Stable明确不执行`steps[]`。当前真实机制是Host级Resource Coordinator，蓝图只声明能力意图，不能分配CPU/GPU或直接启动/卸载资源。
- 已实现能力是控制面准入：通过`sysinfo`取得RAM与逻辑/物理CPU数量，通过resource adapter上报估算RAM/CPU线程，按安全余量、pending/active lease、优先级、公平老化、超时与可逆抢占做允许/拒绝。它不处理token/PCM/帧业务流。
- 当前CPU部分不是OS调度器：不读取持续CPU利用率、频率或温度，不设置affinity、nice、cgroup或线程上限；`cpu_thread_reservation`只是并发容量账本。Linux仍负责真正的线程调度。
- 对Orange Pi有价值的对象是未来可选重负载：本地ASR、TTS、摄像头解码/视觉、复杂renderer和后台生成。FSR/IMU感知、节点健康、安全关机、实体键和基础SystemUI不得经过可拒绝/可抢占准入。
- 可行接法是由本仓integration layer通过OCLive进程内`ResourceAdapterRegistrar`登记owner-namespaced HostExtension；枪械感知内核仍然不知道协调器。硬件策略写在本仓专用`distro.oclive.toml`/HostProfile，不写进OCLive蓝图或枪械角色扩展包。
- Orange Pi无NVIDIA GPU时GPU快照会unavailable，但同一snapshot仍可提供RAM/CPU拓扑；CPU/RAM-only准入可工作。Zero 3W 6GB的RAM安全余量、A76/A55核心保留和各adapter估算必须按基础闭环、3B、renderer与未来相机组合实测，不能照搬桌面默认或旧2GB路线。
- `57A`：P0只使用资源快照和诊断观察，不执行任务拒绝、排队或抢占；关键链路始终绕过协调器。
- `58A`：本仓现在建立Orange Pi测量/HostProfile字段骨架，所有限额留待实机MEASURED数据，不将桌面默认值写成板卡事实。
- `59C-R1`：P1仅为可选重任务接入协调；轻任务可进程内，原生重任务可拆systemd服务，按RSS、线程、释放回收和崩溃隔离实测逐项决定。
- 实机填空与裁剪矩阵见`ORANGE_PI_BRINGUP_WORKSHEET.md`。这不是硬件专用调度器分叉：四季宝复用OCLive通用协调器，只特化HostProfile、adapter和降级策略。

诊断与Profile决策：

- `60C`：维护页显示有界健康摘要，同时允许显式导出经过轮转、容量限制和隐私过滤的完整诊断包；renderer异常时后台诊断仍由host保存，诊断生成不得阻塞关机或关键链路。
- `61B`：使用development与field两套HostProfile，共享板卡资源基线、安全边界和schema。development启用详细诊断/维护能力，field关闭原始流和无关服务并收紧现场策略；两者分别冒烟，差异清单受版本控制。
- HostProfile是部署/资源策略，不等于`interaction_mode`。Profile切换经维护流程选择并受控重启，不能由角色或普通触摸在运行中任意切换。
- `62C`：P0 renderer与`ailive-gun-spirit-host`同进程，但只能通过可序列化、版本化`RendererPort`接收完整ViewModel。使用有界最新值通道，慢渲染跳过旧revision；反向触摸使用窄`UiIntent`，安全实体键不经过renderer。
- renderer记录呈现耗时、last presented revision、跳帧/错误/重初始化、心跳和缓存诊断。Orange Pi实测若出现host崩溃/长阻塞/OOM、资源无法归因/释放、事件p95门失败、独立更新需求或大型原生图形运行时，再评审独立`skippy-renderer`服务；否则保持单进程。

Web前端、资产与UI状态决策：

- `63C-R1`：采用Web前端承载Vue触摸界面并为P1 Live2D WebGL留接口；P0仍只要求PNG/短文本/HUD。Orange Pi的Chromium/WPE/WebKit等具体引擎不冻结，必须以WebGL、冷启动、RSS/线程、磁盘、功耗、温度、触摸和稳定性实测选择，并可能触发renderer独立服务复审。
- `64A-R1`：ViewModel只传类型化AssetRef，不传任意路径、URL或完整像素帧。AssetRef绑定kind、asset_id、角色包identity/version和内容摘要；renderer通过受校验manifest解析只读资产。Live2D按模型bundle处理，所有内部相对引用限制在bundle根；失败回退标准PNG。
- `65C-R1`：Host拥有页面、配置锁、interaction_mode、配对/校准/解绑/关机流程和操作结果；renderer只拥有滚动、按钮反馈、过渡、动画时间与Live2D混合。UiIntent必须经host接受后才改变业务状态，renderer重建不要求恢复动画精确帧。
- Live2D属于P1可选表现，不是P0门；公开发行或允许第三方/多模型扩展前设置独立许可复核门。

Renderer传输与视觉资产复用决策：

- `66A-R1`：只发送带schema/revision的最新完整ViewModel。仍有效的短暂表现以内嵌、带instance id/valid until的有界当前状态表达；不做patch历史和第二动画流。RenderStatus只用于健康/延迟诊断，不反压host。
- `67B-R1`：P0用loopback HTTP提供只读Vue bundle、WebSocket传ViewModel/UiIntent；页面重连取得最新快照。field只绑定本机并执行严格Host/Origin/CSP、启动会话凭证、白名单意图、view revision复核、消息大小/速率限制，禁止通配CORS、远程导航、任意文件接口和开发服务器。
- `68A-R1`：复用OCLive`portrait_catalog`、`visual_state_id`和`visual_presentation/performance_directive`。本项目验证后投影运行态AssetRef，不修改标准角色schema、不把模型放进枪械交互伴随包。未来外置Visual Pack只能作为新增AssetResolver provider，不得成为P0依赖。
- 自由度门：RendererPort DTO不绑定tokio watch/WebSocket；AssetRef不绑定唯一provider；未来替换传输、浏览器或资产来源不得改变双内核和Output Arbiter语义。
- P0范围门：只实现PNG/短文本/HUD/维护页、最新快照恢复和安全UiIntent。Live2D、外置Visual Pack、差量协议、可靠动画队列与远程控制继续留在P1/证据触发项。

OCLive Linux适配裁剪审查门（OPEN，进入Orange Pi部署前执行）：

- “删掉”默认指从四季宝Linux构建、镜像、systemd服务和运行时依赖闭包中排除，不等于从OCLive通用主仓删除源码。真正删除主仓能力必须另有跨宿主证据和独立评审。
- 对实际依赖逐项分成四类：直接复用的通用无头能力、由`ailive-gun-spirit-host`替换的桌面宿主能力、通过Cargo feature/独立服务按需启用的可选能力、完全不进入设备镜像的开发/桌面能力。
- 审查对象至少覆盖：Tauri/WebView/桌面IPC与托盘更新、Windows/macOS适配、NVIDIA/本地模型服务、语音与Live2D等可选adapter、HTTP管理面、角色包/记忆/SQLite、通用Input Scheduler、Resource Coordinator、日志/遥测和原生动态库。
- 不按目录名猜测。以目标板`cargo tree -e features --target aarch64-unknown-linux-gnu`、最终ELF动态依赖、镜像文件清单、启动服务清单、RSS/线程/磁盘/冷启动测量和离线冒烟测试形成“保留/替换/可选/排除”矩阵。
- 四季宝不复制一份精简OCLive源码。能复用的能力继续通过锁定版本/revision依赖；板卡路径、Output Arbiter、BLE、GPIO、电源与systemd留在本仓集成层，保持两个项目文档和代码所有权分离。

## 5. 故障链路

| 故障 | 降级 |
|------|------|
| FSR 抖动 | 节点本地迟滞/时间防抖 |
| FSR 开路 | 远端 EOL 预留 + open/released 台架比较；未装或不可分时相关事实 unknown，不伪装 released |
| ADC 长期极端/不变化 | fault flag；相关事实 unknown |
| BLE 断连/TTL 过期 | 相关 grip/motion unknown，不保留最后一次 held；角色/长期状态不受传感器最后值污染 |
| 重复/乱序 | sequence 去重，旧包丢弃，trigger 不重放 |
| edge event丢失 | 下一次CurrentState快照修正；记录丢包/sequence gap，不永久保留错误状态 |
| 重连事件积压 | 不重放瞬时动作；HELLO + fresh snapshot；仅诊断计数可回传 |
| 生命周期记录重复 | event_id去重；resume_consumed事务性落盘；角色最多消费一次 |
| 未登记/已撤销节点 | 不进入融合；仅在实体授权窗口显示pending候选；记录审计但不持续弹窗 |
| node_uid冲突或assembly变化 | 冲突节点隔离；assembly变化使旧校准失效并要求确认，不静默套用 |
| IMU安装方向错误/校准过期 | 自检/预览暴露；pose变unknown并提示重新确认，不用错误transform继续派生Ready |
| 冲击/跌落 | 诊断shock与机械检查提示；不翻译成trigger或火控动作 |
| 触摸误触/配置锁定 | Ready/运动时拒绝写操作；实体键保留背光、安全关机、返回和受控维护入口 |
| 屏幕贴墙/装包受压 | 海绵与刚性止挡先承力；按钮退入保护平面且不得被泡棉持续按下 |
| 节点 low/critical | UI 提示；先停非必要流量，保留核心事件；到关机点发 shutdown_pending，主机 checkpoint 后节点受控关机 |
| 主机欠压 | 有可信电源遥测时触发正常关机；P0 无遥测成品电源依赖可见电量提示与手动关机，不虚构保护；V1 监督电路闭环 |
| 屏幕失败 | 状态机继续，显示进程可重启 |
| MicroSD/数据库异常 | 启动完整性检查；从最后完整事务/导出恢复；不让传感日志无限写满系统盘 |
| 摄像头失败 | P1 退出画面回到器灵 UI，不阻塞主循环 |
| trigger 辅助件改变行程/手感 | 立即否决并拆除 |

故障注入：前/后/双节点断电、BLE 屏蔽、陈旧/乱序/重复包、ADC 0/满量程、主机重启、屏幕拔出、USB hub/camera 拔出和低电关机。

重启状态分层：

1. **角色连续状态（主机持久化）**：角色身份、记忆/成长、用户设置、会话摘要等由 Orange Pi/OCLive 通过周期 checkpoint 和 `shutdown_pending` 触发的追加 checkpoint 保存；不能只依赖最后一条 BLE 包。
2. **节点稳定配置（节点持久化）**：`node_id`、bonding、硬件/校准版本、已验证 calibration profile 和用户配置保留；仅在内容变更或受控关机且 dirty 时原子写入，避免每次采样磨损 Flash。
3. **传感器瞬时证据（重启清空）**：ADC/IMU 滤波窗口、engaged/released、motion、Held/Ready、去抖计时器和旧 sequence 全部失效；重连后先报告 unknown，完成自检和新鲜采样后再派生。这里“重置”不是删除校准，也不是用开机瞬间自动归零。
4. **诊断状态（追加记录）**：保存关机原因、固件版本、配置 CRC 和必要计数；每次启动产生新 `boot_id`，主机据此拒绝把重启前瞬时状态续接到重启后。

## 6. 机械链路与预算

受力：

- 握持：hand → sleeve/load spreader → FSR → rigid backing → grip/rail。电池/PCB/焊点不受主握持力。
- 屏幕碰撞：foam/support → pod skeleton → roll → pitch → yaw → rigid rail clamp。LCD/PCB/cell 不承力。
- 线缆：硬限位在线缆拉紧前生效；线缆不作为限位或结构件。

FSR 机械拓扑已于 2026-08-26 冻结，材料厚度和传感器长度仍待实测：

- 前部采用“可移动传感握片/导轨护片 + 后方下导轨电子盒”分离结构；FSR 和受力层随前握位置移动，XIAO/IMU/电池留在后方节点盒，通过 80/120/160 mm 三档局部隐藏软线比较。
- 后部 V0 采用原握把后背带纵向单条 FSR；100 次握持动作若出现明显盲区，再评审左右双侧，不预先增加第二 ADC 通道。
- 后握夹层顺序固定为 `hand → thin removable TPU/silicone sleeve → flexible load spreader → FSR → original rigid backstrap`。原握把直接充当刚性背板；扩散片位于 FSR 上方，不得靠过紧外套持续预压。
- FSR 尾线沿可拆握把套的隐藏线槽向下，经独立应变释放进入可拆底盒。底盒不得封死原电机底板、调节/检修与通风位置。
- V0 不切削、钻孔或永久粘接原握把。外层、扩散片和 FSR 可分别更换；握把套总增厚继续以 1.5–3 mm 为 `ESTIMATED` 目标。

单条后握 FSR 的验证与校准规则已冻结，指标仍需实物执行：

- 第一阶段由项目所有者完成 100 次独立握持—释放正样本：右手裸手、右手手套、左手裸手、左手手套各 25 次，每组混合轻握、正常握和较紧握。
- 另做 50 次开机负样本，覆盖桌面静置、轻触但未完整握持、拿取底盒、载体移动但未握后把和临时靠放。正常装包、运输和长期收纳按用户习惯整机物理关机，不把背包持续挤压列为开机核心验收工况。
- 单条 FSR 初始通过门：正样本总成功率 ≥95%；四个分组各 ≥90%；不存在固定握法的系统性盲区；50 次负样本误判 ≤1；释放不长期卡在 engaged。任一门失败先调整机械/校准，仍失败才升级左右双侧 FSR。
- 采用“装配基础校准 + 可选用户校准”。保存无压力基线、轻握/正常握范围、engaged/released 阈值和 calibration version；更换 FSR、扩散片或握把套后使旧校准失效。
- 节点启动时加载已保存校准，不得把开机瞬间 ADC 自动写成零点；用户可能在握持或受压状态下开机。启动快照可以报告当前状态，但重新标定必须由明确操作触发。
- 结构稳定后再邀请不同手型用户重复动作集，作为通用性门，不在 V0 结构频繁变化时提前扩大样本。

尺寸/质量占位：

| 部分 | 包络 | 质量 |
|------|------|------|
| 主机屏一体舱 | 约 100 × 65 × 30–38 mm | 225–345 g |
| 三轴/导轨夹具 | 折叠约 70 × 35 × 25 mm | 45–80 g |
| 前节点 | 65–80 × 24–26 × 9–11 mm | 18–28 g |
| 后握底盒 | 34–38 × 28–32 × 13–16 mm | 20–30 g |
| 后握薄套 | 增厚 1.5–3 mm | 含于后节点 |
| P1 摄像头/线 | 约 30 × 25 × 25 mm 起步 | 20–35 g |
| 完整增加 | 分散式 | 约 330–520 g，中心约 400 g |

使用 225/285/345 g 三档主机舱假体。移动舱 >350 g 或完整套件 >520 g 时触发结构/板屏/续航回退，不通过无限增加关节摩擦掩盖质量问题。

前 FSR 线试 80/120/160 mm 三档；后 FSR/trigger 约 20–40 mm。后握按上述 100 正样本 + 50 开机负样本门执行，有显著盲区才升级双侧。P1 相机在三轴附近使用可更换约 10–15 cm 级短跳线；V0 做 500 次、P1 做 1,000 次全角度循环。

## 7. 实现顺序

1. `C0`：schema/GATT/mock；1,000 条 observation 回放。
2. `H0`：Orange Pi 点 HDMI 屏、USB touch、本地 OCLive、自启动/watchdog、20 次冷启动和 2 h soak。
3. `N0`：前节点 USB 台架；FSR/IMU 各至少 100 次动作。
4. `N1`：前节点 BLE + protected LiPo；20 次断连重连和 8 h 会话回放。
5. `N2`：复制后节点；先 FSR，后外壳，最后辅助 trigger。
6. `F0`：双节点融合、故障注入、Wi-Fi/BLE/HDMI 共存 2 h。
7. `M0/M1`：质量/尺寸假体、三轴、天线、电池、力路径和线缆循环，再冻结外壳。
8. `R0`：RADIAN 断电装配；`R1` 通电静态；`R2` 至少三次场地 Alpha。
9. `P1`：UVC 640×480/30，测端到端延迟、CPU、温度、功耗、掉帧和线缆循环。

## 8. 实测记录

每项记录 `DATASHEET | ESTIMATED | MEASURED`、料号/批次、版本、带接头尺寸、裸重/装配重/重心、平均/峰值电流、温度、采样/滤波/校准、100 次动作结果、p50/p95、照片/日志/波形、keep/modify/reject 和受影响文档。

可上枪原型必须绑定主机 commit、前/后固件与校准版、schema、三套机械 revision、BOM/power revision 和 test report id；不得只写“最新版”。

## 9. 当前开放项

- G5后的P1+余弹遥测使用独立设计与测试表，不并入本文件的P0双节点拓扑；见`MAGAZINE_AMMO_ESTIMATION.md`和`test-worksheets/07-余弹融合与弹匣结构测试表.md`。
- Orange Pi + 3.5 HDMI 的真实功耗、4 h 电池厚度和移动质量。
- 节点 LiPo 的具体尺寸、保护、开关和充电口。
- FSR 408 长度、力扩散层、RM、滤波和断线诊断。
- trigger 微动/光电/删除的实物裁决。
- P1 USB 短跳线和线夹位置。
- 若超过重量硬门，2.8 HDMI、Lyra/DSI、减续航或重新分配电池的优先级；必须由 MEASURED 数据和用户手感决定。
