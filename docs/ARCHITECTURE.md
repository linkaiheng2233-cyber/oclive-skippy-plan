# 架构边界

## 1. 目标

本项目由两个平级薄核协作。枪械感知内核把连续、嘈杂的物理信号转换成稳定载体事实；OCLive内核把事实与角色状态结合并生成表达。枪械感知内核是wargame发射器/枪械交互的特化，不是OCLive角色内核的特化，也不是硬件驱动集合或第二套角色编排器。

```text
Raw sensors
  │  GPIO / ADC / I2C / SPI / mock input
  ▼
Node / driver adapters
  │  filter / debounce / edge / aggregate
  ▼
SensorObservation v0.2
  │  BLE GATT or local adapter
  ▼
Gun Perception Kernel
  │  registry / freshness / fusion / deterministic state machine
  ├──────────────→ PerceptionState ──→ immediate system cue / diagnostics
  │
  └──────────────→ DeviceEvent v0.2
                         │  narrow integration bridge
                         ▼
                    OCLive Kernel
                         │  character / memory / turn / expression
                         ▼
                       RoleCue ───────→ output arbiter
```

两个内核源码与依赖分离：感知内核位于本仓独立纯Rust crate，不依赖OCLive；OCLive也不依赖感知内核。P0由同一个`ailive-gun-spirit-host`进程部署，只有最外层bridge/composition root可以同时引用双方并执行显式值转换。两个内核互相不知道，不能共享内部对象、注册表引用或可变状态；边界数据必须可序列化、可记录并可回放。进程隔离只在实测证明需要独立崩溃域、升级或资源调度后增加。

当前物理输入由两个区域 BLE 节点承担：前下导轨节点包含载体参考 IMU 与前握 FSR，后握把节点包含后握 FSR 与后续只读 trigger 辅助输入。区域内部允许隐藏短线，区域之间不拉线。一体式显示主机舱具有独立 yaw/pitch/roll 三轴，屏幕与 Linux 主机一同转动；载体参考 IMU 固定在前下导轨节点，任何调屏动作都不得进入载体 motion 观察，否则会污染 Held/Ready 判定。HDMI 视频线完全留在移动舱内。当前硬件、电源、机械和故障链路 SSOT 见 `HARDWARE_IMPLEMENTATION_PLAN.md`。

## 2. 枪械感知内核边界

感知内核拥有：

- 节点身份、能力、binding与校准引用的有效性。
- observation去重、乱序、TTL、boot_id与断连unknown。
- 多传感证据融合、载体状态转换与故障降级。
- 持续`PerceptionState`和离散`DeviceEvent`的确定性输出。

感知内核不拥有：

- BLE扫描、BlueZ bond store、GPIO/ADC/I2C/SPI/DRM和数据库驱动。
- 角色包、记忆、人格、情绪、Prompt、模型调用和回复文本。
- 准星、弹道、目标识别、波箱、电机、供弹、火控或任何`actuator.*`接口。

## 3. 状态机

v0.2载体状态闭集：

```text
Standby ⇄ Held ⇄ Ready
```

- `Standby`：握持释放并持续静止；不要求放回固定底座。
- `Held`：设备被拿起，允许一次唤醒反馈。
- `Ready`：设备被举起或进入准备姿态。
- 屏幕休眠、进程生命周期和省电属于Host，不是感知carrier state。
- 只读辅助触点是独立Fact与DeviceEvent；它不把carrier推进为`Active`，也不证明机构完成动作。

状态机必须处理重复、乱序、过期和断线重连，不得依赖 LLM 才能完成转换。

## 4. 输入与双输出契约

感知边界采用三份强类型契约，而不是“一个通用 envelope + 任意 payload”：

1. `SensorObservation`：节点或本地适配器实际观察到什么。每一种 observation kind 都有封闭的类型化 payload、单位、取值范围和unknown/quality语义；前后区域由稳定node identity、binding和capability区分。
2. `PerceptionState`：感知内核当前相信什么。它是带revision、freshness、confidence/source evidence和显式unknown的完整持续快照，供重连、自愈、即时UI、诊断和回放断言使用。
3. `DeviceEvent`：有意义的设备状态何时发生转换。它只由感知状态机产生带event id、前后状态、发生时间和因果revision的离散边沿，不承载原始ADC/IMU或任意JSON。

SensorObservation v0.2 P0 variant闭集为：

- `GripContact { contact: engaged | released, strength_milli }`。
- `MotionClass { motion: idle | moving }`。
- `OrientationEstimate { gravity_mg_x, gravity_mg_y, gravity_mg_z, quality_milli }`：节点滤波后的低频重力方向估计，不含yaw/目标方向；Host结合载体坐标和用户calibration派生pose。
- `AuxControlContact { control_id: primary_trigger, contact: engaged | released }`：只读辅助触点，不属于actuator或机构完成确认。
- `ShockObserved { severity_milli }`：只进诊断/机械检查提示，禁止映射为trigger/fire DeviceEvent。

电量、固件、校准版本和故障码属于NodeStatus；unknown由Host基于NodeStatus、TTL、断连和校准有效性派生，节点不发送“unknown observation”。正常运行不上传ADC原始流、完整IMU波形或任意厂商payload。

PerceptionState v0.2 P0字段闭集为：`carrier_state: Fact<Standby|Held|Ready>`、`front_grip: Fact<Contact>`、`rear_grip: Fact<Contact>`、`motion: Fact<Idle|Moving>`、`pose: Fact<Raised|Lowered>`、`primary_control: Fact<Contact>`以及有界`node_health`摘要。`interaction_mode`、屏幕睡眠、角色状态和原始shock不在该状态内。

DeviceEvent v0.2 P0 kind闭集为：`carrier.held_entered`、`carrier.held_exited`、`carrier.ready_entered`、`carrier.ready_exited`、`control.primary.engaged`和`control.primary.released`。每项携带event id、发生时间和因果PerceptionState revision；同revision多个边沿由Input Scheduler合并。节点在线/低电、模式、shock和生命周期走状态/SystemCue/诊断，不存在`device.triggered`、`weapon.fired`或actuator事件。

现有`schemas/*v0.1.schema.json`是从未交付的探索草案，只保留为设计历史，不是兼容承诺。首个实现基线为v0.2；直接编写严格schema、有效/无效样例和Rust类型，不为未发布v0.1制作兼容adapter。`mode.changed`属于Host交互策略，节点连接/低电/系统故障属于PerceptionState/SystemCue，不进入角色语义DeviceEvent。节点断连时相关事实必须变为`unknown`，不得无限沿用最后一次握持状态；周期PerceptionState不得被桥接成重复角色回合。

契约真源位于`ailive-gun-spirit-contracts`的Rust类型。JSON Schema由同一类型确定性生成并提交仓库，CI重新生成后要求零差异，同时用有效/无效fixture验证语义不变量。BLE紧凑二进制、Web JSON和诊断日志只是显式codec/投影，不各自建立业务契约；BLE codec必须通过跨语言golden vectors证明与Rust DTO一致。

BLE连接先通过`NodeHello`协商契约版本：节点报告明确的`supported_contract_versions`及对应capabilities，Host从双方显式支持集合选择最高共同版本，不能根据SemVer或未知字段猜兼容。没有交集时节点保持可见但进入`incompatible_protocol`，任何观察都不进入融合。协商后出现未知variant/非法字段时只隔离该帧并记录协议故障；在有界窗口内重复违规达到HostProfile阈值后隔离节点。不得把未知variant作为任意字符串转发到感知核心或OCLive。

P0协议违规采用分级隔离，初值全部标记`ESTIMATED`：单条非法应用帧丢弃并限速记账；滚动10秒内3条非法应用帧则断开并隔离30秒；连续3个连接周期都触发隔离后进入持久`protocol_fault`，需要维护页明确确认才重新启用。超过长度上限、协商后仍使用错误版本等硬违规可直接终止本次连接。BLE链路层重传、RSSI差、普通断连和未通过链路完整性的数据不计入应用协议违规。次数、窗口和隔离时间属于HostProfile，须经fault injection与实机数据覆盖，不进入跨版本契约。

节点时间不尝试组成全局时钟。每条观察携带`node_uid + boot_id + sequence + node_monotonic_ms`，只用于同一节点同一boot内去重、排序和检测回退；Host接收时补充`host_boot_id + received_monotonic_ms`，感知核心使用Host单调时间、TTL和融合窗口决定跨节点新鲜度。不同节点的`node_monotonic_ms`禁止直接比较，wall-clock只可用于人类诊断，不进入状态转换。节点重启必须更换boot_id并允许sequence归零。

PerceptionState的每项可失效事实使用显式`Fact<T>`，而不是nullable值或“最后值+全局stale”：

```text
Fact<T> =
  Known { value, observed_at, fresh_until, confidence, source_refs }
  | Unknown { reason, since_revision }
```

P0 `UnknownReason`闭集为`never_observed | node_offline | stale | uncalibrated | sensor_fault`。Known必须有value且不得有unknown reason；Unknown必须有reason且不得把last-known value当作当前值。诊断可在独立字段保留last-known摘要，但融合、bridge和角色上下文只能消费当前Fact。

Known中的`confidence_milli`使用`0..=1000`整数表示当前规则与校准下的证据强度/判定余量，不是统计概率，UI不得显示成“87%概率”。阈值按fact kind和calibration version配置；Unknown不携带confidence。每个Fact最多携带4个有界`source_ref`，字段仅含`observation_id`、`node_uid`、`capability`和`relation: supports | contradicts`。完整Observation只留在有界诊断/回放缓冲区，PerceptionState不嵌入原始压力或IMU数据。

PerceptionState只有在Fact分类值/Unknown原因、节点健康或其它权威语义改变时才递增revision；单纯收到数值不同但未改变分类的Observation不产生新revision。新的同值Observation仍可在同revision快照中刷新`observed_at/fresh_until/confidence/source_refs`证据包络；Replay比较分类值、Unknown原因、节点健康和身份/revision，接受包络刷新但更新其最新快照。该刷新不得产生DeviceEvent、桥接角色回合或伪造语义变化。新订阅、重连、renderer恢复或Host健康心跳可以重发相同revision的最新完整快照；心跳周期属于HostProfile/实测参数，不写入契约。DeviceEvent仍只由状态转换产生并引用因果revision。

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

截至2026-08-28，OCLive兄弟仓已提供公开`TurnOrigin = user | sensor | system`和Rust内部`process_message_with_origin`/stream入口。普通HTTP、Tauri与现有`SendMessageRequest`仍隐式固定为`user`，外部载荷不能自行选择低持久化来源；sensor/system可读取角色、近期上下文并生成回复/视觉状态，但会绕过用户情绪插件及所有用户聊天状态提交。本仓只有`ailive-gun-spirit-host::OcliveSensorAdapter`同时认识双方类型，并始终用类型化`TurnOrigin::Sensor`投递。

## 5. OCLive桥接与输出边界

屏幕由`ailive-gun-spirit-host`的Output Arbiter唯一控制，两个内核都不能直接访问renderer或显示设备：

```text
PerceptionState / LifecycleEvent ─→ SystemViewState / SystemCue ─┐
                                                                  ├→ Output Arbiter → Renderer → Screen
OCLive ─────────────────────────────────────────────→ RoleCue ─────┘
```

共同前端的概念层为：

1. `RoleStage`：角色表情、短文本与情绪表现，是正常画面的主体。
2. `StatusBar`：电量、节点、网络与模式等简明、持续的系统事实。
3. `SystemOverlay`：安全关机、维护确认、配置锁定和必须处理的故障，优先级最高。

`schemas/output-cue.v0.1.schema.json`定义语义VisualCue，而非像素或框架组件。语音与触觉不进入v0.1。Output Arbiter按以下规则决定执行：

1. 丢弃已经超过 `ttl_ms` 的迟到输出。
2. 高优先级视觉状态可以打断低优先级。
3. 即时本地反馈先执行；动态回复到达后只能更新仍然有效的角色槽位，不能覆盖系统Overlay。
4. 网络失败不回滚已经完成的设备状态转换。
5. 通过`related_event_id/context_revision`拒绝已经跨越状态边界的旧RoleCue。

桥接方向在P0保持单向：感知内核产生事实，集成宿主把允许触发角色互动的DeviceEvent投影为类型化sensor turn；OCLive返回RoleCue。OCLive不能回写物理事实，角色表达也不能成为感知状态转换的证据。

`interaction_mode`属于host交互策略，不属于两个内核。它可改变哪些事件进入角色回合以及前端呈现方式，但不能改变感知事实和安全语义。新感知事件只有存在显式bridge映射与测试时才进入OCLive；未映射事件保留在系统状态/诊断，不能以任意字符串穿透到Prompt。

bridge只为有意义的状态转换建立角色回合，按event_id/revision去重并支持合并、冷却和过期丢弃。周期快照、重复状态、抖动和重连同步只修正PerceptionState，不重复触发角色表达。即时SystemCue不经过OCLive。

未来多输入调度位于OCLive侧通用Input Scheduler，而不进入枪械感知内核：

```text
Voice/Text ───────────────────────→ PrimaryInput ─┐
                                                   ├→ Input Scheduler → OCLive main turn
DeviceEvent ─→ bridge ─→ DeviceContext ───────────┘

若无显式输入：通过门控的DeviceEvent可成为sensor PrimaryInput。
若有语音/文字：显式输入为PrimaryInput，DeviceContext为AuxContext。
```

感知内核不知道语音状态或情绪词表。角色包在OCLive侧定义中性设备事实到情绪偏置、回复概率、冷却和表现的映射。VAD开始时取消尚未提交的sensor turn并合并上下文；ASR partial不提交回复，final transcript才可形成主回合。过期sensor RoleCue由context revision/event id/TTL拒绝。安全SystemCue绕过对话调度直接进入Output Arbiter。

sensor turn在角色调度前使用约500–800 ms的ESTIMATED合并窗口，窗口不影响即时UI/安全通道。Aux DeviceContext为有界快照：当前载体状态、模式、节点健康/电量与最近约2秒最多4个语义变化；不包含原始采样或无限历史。

设备情绪反应策略不进入OCLive标准角色包。项目使用独立、版本化、引用基础角色身份的“枪械交互伴随包”（Gun Interaction Companion Pack），由专用loader/adapter对接Input Scheduler。它使用`ailive.gun-spirit.interaction/1`格式标识并安装到`content/gun-interaction-packs/<pack_id>/<version>/gun-interaction.json`；基础角色仍由OCLive角色目录独立管理。

用户可以从电脑选择伴随包目录或压缩包导入。专用importer先复制到临时区，验证manifest/schema、路径边界、大小、内容摘要、pack identity/version和`base_role { id, version_req }`，再原子安装到本项目受管目录并登记。它不直接执行导入源、不扫描任意外部目录、不写回基础角色包，也不复用OCLive标准角色包loader假装二者是同一种包。扩展缺失或不兼容只关闭专属主动反应，不阻止基础角色加载，也不改变两个内核的依赖边界。

P0本地导入不要求发布者数字签名，也不得把SHA-256描述成作者身份认证。每个包必须有规范化内容清单与SHA-256，importer拒绝绝对/越界/重复规范化路径、符号链接/硬链接/设备文件、超限文件数/展开大小及摘要不符；用户必须明确确认本机目录或压缩包。由于P0包禁止代码、脚本、远程URL、任意Prompt、密钥和模型资产，严格声明式schema+完整性校验是当前信任边界。未来接市场或第三方公开分发时，Ed25519发布者签名、信任库、换钥和吊销必须另立ADR与schema版本，不能把当前未签名包显示为“可信发布者”。

同一pack id允许多个SemVer版本并存；本机binding registry独立保存每个基础角色当前启用的`pack_id + version`。导入新版本先完成验证和原子安装，只有用户明确确认或受控升级流程成功后才切换活动指针，旧版本保留用于回滚。相同id/version/digest重复导入幂等成功；相同id/version但digest不同返回稳定`version_collision`，禁止静默覆盖。卸载旧版本和垃圾回收必须是显式维护操作，且不得删除当前活动版本。

P0 reactions使用封闭声明式表：每个已登记DeviceEvent id至多一条`default`规则，并可按已登记interaction mode提供字段覆盖。确定性合并顺序为`default → current mode override → Host安全/用户上限`；缺少模式覆盖时直接使用default。规则只允许schema列出的情绪偏置、主动回复概率、冷却和视觉语义ID等数据，不支持条件表达式、优先级规则链、JavaScript、脚本或任意Prompt片段。未知event/mode/visual id在导入验证阶段拒绝或按明确optional策略禁用，不能运行时猜测。

主动sensor turn的概率门控只属于Host Input Scheduler。执行顺序固定为：事件有效性/去重/TTL → default+mode反应解析 → `active_reply`状态 → Host安全与用户上限 → cooldown → `probability_milli`抽取 → 创建或跳过sensor turn；之后仍服从语音优先和合并窗口。测试注入可复现RNG，生产使用Host会话随机源并记录decision id、有效概率和结果。`confidence_milli`是感知证据强度，`probability_milli`是调度抽样概率，两者类型名、所有者和日志必须分开。

模式可用类型化`active_reply: disabled | probabilistic{probability_milli}`明确禁止独立sensor turn。disabled不删除PerceptionState、DeviceEvent、System UI、诊断或中性DeviceContext，也不阻止该事实在用户语音/文字回合中作为辅助上下文；它只表示“不得由此事件主动开启角色回合”。OCLive和感知内核均无权改写该Host策略。

四条职责基线是架构门禁：

1. 枪械感知内核只产出可验证物理事实与状态转换，不认识角色概率、冷却或台词。
2. 枪械交互伴随包只声明受限反应策略，不执行代码、不拥有调度状态。
3. Host/Input Scheduler拥有模式解析、上限、冷却、概率、合并、取消和回合投递，不解释角色人格。
4. OCLive只结合角色包、记忆、上下文生成理解与表达，不读取原始硬件，也不决定物理事实。

任何新增功能若跨越以上职责，必须在实现前写出理由、替代方案、耦合与回退影响并新增ADR；不得以局部便利为由顺手跨层。Output Arbiter的唯一前端所有权继续作为独立既有红线。

活动binding精确固定`pack_id + pack_version`。每次加载或基础角色升级后，Host重新校验伴随包的`base_role.id + version_req`；仍兼容则继续，失配则标记`inactive_incompatible_role`并fail-open启动基础角色。Host可以列出已安装兼容候选并提供一键确认，但不得自动改绑到“最新兼容”版本，也不得忽略版本范围。

已安装包索引、活动binding、禁用原因和导入/切换审计保存在A.I.Live Gun Spirit Host拥有的独立SQLite状态库，逻辑名`ailive-gun-spirit-state.db`；实际路径由平台HostProfile决定，例如Linux field位于受权限保护的应用数据目录。它与OCLive聊天、角色记忆和关系数据库分离。包文件保持不可变，数据库只保存identity/version/digest/受管相对路径、binding和有界审计；切换/回滚使用事务，开启foreign keys和schema migrations。维护页/诊断包只导出经过隐私过滤的只读JSON摘要，不暴露原始导入绝对路径。

包导入以受管根目录内的`.staging/<operation_id>`和SQLite`install_operations`组成可恢复发布协议：先把外部来源复制到与最终目录同一文件系统的staging，验证后记录installing，再原子rename到不可变版本目录并补记installed；活动binding另行事务切换。启动时按journal、目录存在性与digest把中断安装收敛为继续、完成、missing/disabled或隔离孤立目录，不自动信任扫描到的文件，也不依赖跨文件系统rename。

field状态库固定WAL+FULL、foreign keys和明确busy timeout；checkpoint在待机/维护窗口受控执行并限制WAL增长，非正常启动做恢复检查。development可以显式使用NORMAL，但发布/断电测试必须以field配置为准。运行中备份使用SQLite一致性备份，不单独搬运主DB文件。诊断分为安装/绑定/安全审计、协议/节点故障、资源/renderer和显式维护原始窗口四类，各自按时间/条数/容量有界轮转；field默认不记录原始传感流，Host永不复制OCLive对话、原始语音和角色记忆。

持久化使用显式白名单而不是“序列化整个运行时”。Host拥有节点信任/注册、pack/binding、已确认校准、设备配置和Lifecycle记录；OCLive拥有角色身份、记忆/关系、连续摘要和预生成生命周期表达。PerceptionState、滤波/去抖、BLE会话、boot/sequence、抽样和单调时钟cooldown均为瞬态，重启后从Unknown和新会话重建。新字段必须先声明owner与恢复语义才允许落盘。

启动由Host执行`Booting → StorageRecovery → OCLiveContinuityRestore → NodeRearm → SensorBaseline → Ready|Degraded`恢复门。初次新鲜观察只建立baseline，Unknown到首次Known不发DeviceEvent；armed后真实边沿才发事件。节点超时则相关Fact保持Unknown并进入Degraded，不阻塞基础角色和维护UI。clean/unclean shutdown只作为一次有消费位的LifecycleContext进入OCLive，不穿过感知内核、不补播旧动作。

产品field低电由Shutdown Coordinator编排：Low先卸载可选重负载；Critical Reserve禁止新写操作/新角色回合，使用预生成表达，限时提交OCLive与Host关键状态并poweroff；可控PMIC/监督MCU/锁存开关在完成信号或实测硬超时后切断主rail。P0台架可用人工在safe-to-cut后切电验证同一软件次序，但field成品不得依赖Linux halt、普通充电宝或电芯保护板自行完成真正断电。

前端技术与两个内核解耦。桌面模拟器、Linux DRM或轻量Web renderer都只消费Output Arbiter生成的最终ViewModel；更换屏幕与渲染方案不改变感知核心或OCLive。触摸事件先进入host交互策略，不能由页面直接修改内核内部状态。

800×480画布、九个首轮状态画板、组件变体和评审方式见`FRONTEND_DESIGN_BRIEF.md`。该文档允许现在开始低保真设计，但其中安全边距、触控尺寸、动画和性能目标在样屏到货前均为PROVISIONAL。

P0 renderer与`ailive-gun-spirit-host`同进程，但通过`RendererPort`可拆端口消费版本化、可序列化的完整ViewModel快照。端口使用有界最新值语义，慢renderer跳过旧revision而不堆积画面；触摸以独立`UiIntent`返回host。renderer不持有内核对象，实体键与安全关机不依赖renderer。实测若证明需要故障隔离，只把端口传输替换为本地IPC和独立systemd服务，不改变Output Arbiter及两个内核。

前端方向为Web，但具体Linux浏览器/WebView引擎尚未定板。ViewModel只携带语义与受校验`AssetRef`，renderer通过AssetManifest解析本地PNG或未来Live2D模型bundle；任意路径、远程URL和逐帧Cubism参数不跨边界。Host拥有页面、配置锁、校准/配对/关机等权威流程，renderer只拥有滚动、过渡、动画时间和Live2D混合等可丢弃表现。WebGL/模型失败必须回退PNG和System UI。

P0 RendererPort使用最新完整快照：每个Envelope具有schema、revision和当前完整ViewModel，短暂表现作为有界、带instance id/有效期的当前状态；不依赖patch历史或可靠动画队列。传输实现为loopback HTTP静态Vue包 + WebSocket JSON，但DTO和端口不依赖该传输，未来可换Native Bridge或本地IPC。loopback按不可信本地客户端防护：field只绑定本机、严格Host/Origin/CSP、启动级会话凭证、白名单UiIntent、大小/速率/版本限制，禁止远程导航、任意文件API与开发服务器。

AssetResolver优先消费OCLive标准`portrait_catalog / visual_state_id / performance_directive`，验证角色根内相对路径和封闭catalog后生成运行态AssetRef；枪械交互伴随包不携带模型。未来若证据支持独立Visual Pack，只增加resolver provider，不改变感知、OCLive、Output Arbiter或ViewModel语义。

## 6. Rust工作区边界

仓库已于2026-08-27从单package启动骨架迁移为最小三crate workspace：

```text
crates/
  ailive-gun-spirit-contracts/        # DTO、版本与schema；纯值类型
  ailive-gun-spirit-perception-core/  # 确定性融合与状态机
  ailive-gun-spirit-host/             # OCLive桥、BLE/Linux/Web与组合根
web/                                  # Vue renderer
schemas/                              # 跨语言JSON Schema与样例
```

依赖只允许`perception-core → contracts`以及`host → contracts + perception-core + OCLive`。contracts不得依赖OCLive、BlueZ、文件系统、数据库、HTTP或async runtime；perception-core不得依赖OCLive和平台驱动；只有host/composition root可以同时引用感知侧与OCLive公开类型。三个crate仍编译为一个`ailive-gun-spirit-host`进程，拆crate不是拆服务。

当前实现已经建立三份v0.2 Rust DTO、生成Schema、正反fixtures、纯`SensorObservation → PerceptionState` reducer、`PerceptionReplay`会话门、Host侧`InputScheduler/DeviceContext`窄缝、真实`OcliveSensorAdapter`、`OutputArbiter/ScreenViewModel`和`ailive-gun-spirit-sim`桌面夹具。Reducer按binding限制前后节点能力，使用Host单调时间处理TTL，按node boot清空旧能力，拒绝同boot sequence回退/碰撞，并执行后握优先Held、前握+moving、后握+raised Ready、双明确释放+idle稳定窗Standby及保守Unknown。姿态分类器当前只使用桌面profile提供的raised/lowered anchor；真实迟滞、过渡区和阈值仍必须由用户校准与实机动作集替换。

测试强制保证Host换boot时安静建baseline、Unknown不猜测边沿、同revision可刷新证据包络但不可改变分类语义、非法状态/revision倒退拒绝、primary control与carrier独立、SystemCue抢占RoleCue、过期RoleCue不覆盖新状态，并连续完成100轮标准序列。OCLive侧另有回归测试证明sensor回合不写聊天、记忆、事件、好感、关系、人格/连续性且随后普通user回合仍按旧行为持久化。本仓真实适配器已能把OCLive返回投影为`RoleCueSource::Oclive`，但默认CLI仍使用明确的`desktop_fixture`以保持无模型可重复测试；尚缺把真实调用接入可操作桌面面板、时延/取消/失败演练和PNG/Web renderer，因此仍不能宣称G3完整闭环通过。

仓库slug固定为`oclive-四季宝器灵`，正式产品展示名为`A.I.Live-ai枪娘器灵`；新crate、binary、service和schema使用ASCII技术前缀`ailive-gun-spirit`/`ailive.gun-spirit`。旧`oclive-skippy-spirit`、`skippy-host`和Skippy命名只作为迁移期历史标识，不能继续生成新公共契约。

依赖门禁：感知内核的`Cargo.toml`不得出现任何`oclive_*`依赖；OCLive仓不得引用本仓crate；只有集成宿主模块允许同时导入双方公开类型。核心测试必须能在不启动OCLive、BLE和Linux设备的情况下用回放数据完成。

## 7. OCLive资源协调器在硬件宿主中的边界

OCLive当前的Resource Coordinator可以作为未来硬件宿主的资源准入控制面，但它不是Linux意义上的CPU时间片调度器，也不执行冻结蓝图中的`steps[]`。它根据HostProfile、系统RAM/CPU拓扑、适配器资源估算、租约、优先级、老化、超时和可逆抢占来回答“某个可选重任务现在能否启动、是否应降级或等待”。

当前实现不会根据实时CPU利用率、频率或温度主动限流，也不设置CPU affinity、nice、cgroup或线程上限；因此不能把它描述成实时硬件调度器。Linux内核和systemd仍负责进程、线程、启动与恢复。

以下常驻链路不得进入可拒绝或可抢占的资源租约：

- BLE接收、数据新鲜度与节点健康。
- 感知融合、故障降级和受控关机。
- POWER/MAINT实体按键与最低限度System UI。

未来ASR、TTS、相机分析、复杂renderer和后台内容生成等可选重任务可以由本仓集成层注册为owner命名的HostExtension资源适配器。枪械感知内核本身保持零OCLive依赖；只有composition root通过OCLive提供的`ResourceAdapterRegistrar`注册适配器。Orange Pi专用资源策略属于本仓的A.I.Live Gun Spirit HostProfile，不进入蓝图、标准角色包或枪械交互伴随包。P0已经冻结为只观察资源快照、不执行拒绝/排队/抢占；P1可选重任务是否接入仍需实机数据。

## 8. 仓库分工

本仓：

- 独立、不依赖OCLive的枪械感知内核crate。
- SensorObservation、PerceptionState、DeviceEvent 与 Visual OutputCue schema。
- BLE 功能节点配对、状态同步和主机侧多传感器融合。
- 器灵状态机和屏幕状态仲裁。
- 桌面模拟器与事件回放。
- ARM Linux 服务、GPIO/IMU 与屏幕适配。
- systemd、日志导出、实机基准和结构文件。
- 本项目专用枪械交互伴随包格式、loader、A.I.Live Gun Spirit HostProfile和硬件资源策略文档。

OCLive 主仓：

- 已实现`TurnOrigin` / 回合副作用通用契约，并保持HTTP/Tauri普通聊天兼容。
- 通用回合编排、角色包和视觉状态。
- 已实现sensor回合不污染聊天、记忆、事件、关系与人格的跨边界回归测试。
- 通用Input Scheduler、Output/Host扩展契约与Resource Coordinator能力；不保存四季宝专用阈值、节点结构或角色扩展格式。

角色资产：

- AN94、MP5 角色包留在 OCLive 角色包 SSOT 或市场。
- 本仓只保存硬件映射、缓存清单和明确授权的预渲染产物。
