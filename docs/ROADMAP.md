# A.I.Live-ai枪娘器灵实施路线

本文是本仓路线 SSOT。阶段按验收门推进；后续想法进入 backlog，不直接扩大 P0 产品原型。注意：产品 P0/v0.1 与感知契约 v0.2 是不同版本层。

## 当前状态

- S0 仓库和基础协议：进行中。
- S1 器灵模拟器：进行中；Observation reducer、100轮命令行回放、类型化sensor输入缝和屏幕ViewModel已完成，键盘面板/PNG renderer未完成。
- S2 OCLive输入来源：已完成第一条契约闭环；兄弟仓已有兼容`TurnOrigin`与副作用隔离测试，本仓已有真实窄适配器。
- OCLive跨仓真实桌面接入：**暂缓**；接口、适配器和测试保持现状，等待项目所有者完成OCLive内核梳理后再恢复，不阻塞四季宝硬件身份核验与独立bring-up。
- Linux L0 纯 headless 依赖：已完成，默认依赖树不含 Tauri/WebView。
- Linux ARMv7 构建门：workspace迁移后已重验；标准 Release 已生成 22.19 MiB ELF32 ARM EABI5 hard-float 产物。
- 原型载体：森柏龙 RADIAN MODEL 1；执行 kit-first，不直接上枪开发。
- 真实硬件：首批硬件于2026-09-04到货，并于**2026-09-11完成首轮bring-up**——主机系统冷启动稳定（峰值≈0.8A→稳态0.36–0.41A@5V）、温度10分钟平台化（40.5–46.1°C）、WiFi（5GHz）与免密SSH远程通道可用、开机自诊断服务落地、显示驱动软件链通过（EDID曾读出、模式480×800、`/dev/fb0`建立）。**视频链未通过**：随附Mini HDMI线DDC通道间歇失效，接已知良好显示器仍EDID=0、无模式，已换线待复测；屏幕疑无触摸接口。逐件身份核验（RAM容量、microSD料号、屏幕型号、环境温度）与两个XIAO节点测试均未开始。模块化基线为Orange Pi Zero 3W 6GB/A733 + 3.5inch 480×800 HDMI屏；USB-C DP主动转HDMI、厂商Linux BSP、板载风扇、功耗和CPU量化3B及以下三个尺寸档仍待实测。证据见`test-worksheets/runs/2026-09-11-zero3w-first-bringup.md`，开放项见`TECHNICAL_DEBT.md`的GS-HW-002/003/004。Luckfox Lyra Zero W + 2.8inch DSI保留为重量/厚度触发后的紧凑回退。
- 当前传感拓扑：前下导轨与后握把两个区域 BLE 节点；详细五链路计划见 `HARDWARE_IMPLEMENTATION_PLAN.md`。
- 工程命名：仓库`oclive-四季宝器灵`；正式产品`A.I.Live-ai枪娘器灵`；新ASCII代码/协议前缀`ailive-gun-spirit`。
- 工程门禁：三个 workspace member 与依赖 allowlist 已自动检查；严格 Clippy、all-targets、doctest、Schema 漂移和`cargo audit`已收敛到`scripts/verify.ps1`。

## S0：规格冻结与仓库骨架

目标：用一张事件—输出表描述完整拿起到放回体验。

- [x] 建立独立仓库和 OCLive `robot-soul` 无头骨架。
- [x] 固定仓库边界、非目标和 v0.1 范围。
- [x] 写出 DeviceEvent / Visual OutputCue v0.1 JSON Schema探索稿；感知v0.1已降为未交付历史草案。
- [x] 记录通用导轨机械接口、无内部走线原则和传感器候选矩阵。
- [x] 写出 SensorObservation v0.1 探索稿；当前由 `node_id` / capability 支持前下导轨与后握把两个区域节点。
- [x] 冻结相机侧屏大小、屏幕后置主电池和三轴恒扭矩显示机构方向。
- [x] 将当前bring-up显示路线更新为「Orange Pi Zero 3W 6GB/A733 + USB-C DP主动转HDMI + 3.5inch HDMI屏一体舱」；Lyra Zero W + 2.8inch DSI保留为重量/厚度回退，板卡仍须通过Linux、显示、无线、功耗、热和本地3B实测后冻结。
- [x] 建立`PROJECT_BASELINE.md`、`PROJECT_BOUNDARIES.md`、`DOCUMENTATION_SYSTEM.md`和统一`docs/README.md`，完成桌面外部四季宝文档归并审计；旧导出不再与仓库SSOT双向维护。
- [x] 跑通标准 OCLive ARMv7 hard-float Release 最终链接，记录大小、ABI、动态依赖和 Windows 中文路径限制，并加入可复现脚本。
- [x] 将目标原型拓扑收敛为前下导轨 + 后握把两个区域 BLE 节点，并记录信息、能源、接口、故障、机械五条链路的估算与实测修正规则。
- [x] 冻结首个实现契约为强类型SensorObservation/PerceptionState/DeviceEvent v0.2；不为未交付v0.1制作兼容层。
- [x] 冻结最小三crate workspace边界与独立枪械交互伴随包目录/本地导入路线。
- [x] 冻结Rust契约真源+生成Schema、节点/Host双单调时间和显式Fact unknown规则。
- [x] 冻结0–1000非概率confidence、每Fact最多4个证据引用和伴随包多版本/回滚规则。
- [x] 冻结NodeHello版本交集协商、语义revision/同revision重发和声明式default+mode反应表。
- [x] 冻结Host概率门控、模式显式禁用、基础角色升级重验和四职责跨层ADR门禁。
- [x] 冻结协议违规分级隔离、P0未签名本机包信任边界和Host独立SQLite注册表。
- [x] 冻结v0.2物理Observation、Standby/Held/Ready状态和六个中性DeviceEvent闭集。
- [x] 冻结可恢复pack安装journal、field SQLite WAL+FULL耐久和分级有界诊断留存。
- [x] 冻结持久化白名单、field两级低电断电链和安静启动恢复门。
- [x] 将当前单package启动骨架迁移到`contracts + perception-core + host` workspace，并同步binary名称；systemd service待Linux镜像阶段落地。
- [x] 实现三份v0.2 Rust DTO、确定性生成schema、3个有效/7个无效样例和`cargo test`零漂移门，保留v0.1文件并明确标记历史状态。
- [x] 修正workspace真实成员边界、生产路径panic点、ID正确构造、Replay入口语义校验和同revision重新发布时间语义，并建立本地完整门禁。
- [ ] 为BLE codec建立Rust/节点固件共享golden vectors，覆盖boot/sequence、边界值、截断帧和版本拒绝。
- [ ] 实现NodeHello协商、无交集降级、单帧隔离与重复协议违规测试；阈值等待HostProfile裁决。
- [ ] 实现协议违规限速计数、临时隔离、持久protocol_fault与维护恢复；用fault injection回填HostProfile阈值。
- [x] 实现完整PerceptionState融合/revision发布：覆盖前后节点binding、后握优先Held、Ready、保守Standby、TTL、node boot、sequence重复/回退、用户姿态anchor和同revision证据刷新。
- [ ] 实现枪械交互伴随包目录/zip安全导入、校验、按pack/version原子安装、目录发现、活动绑定、版本碰撞拒绝和回滚。
- [ ] 实现可注入RNG的Input Scheduler反应门控，并测试disabled、cooldown、概率、语音合并和fail-open不互相越权。
- [ ] 实现基础角色版本重验、`inactive_incompatible_role`诊断与显式兼容候选切换。
- [ ] 实现伴随包规范化manifest/SHA-256、路径/链接/文件类型/展开限制及“未签名本地包”明确UI。
- [ ] 建立独立`ailive-gun-spirit-state.db`迁移、foreign keys、包索引/binding/audit表和隐私过滤诊断导出。
- [ ] 实现`.staging + install_operations + 同盘原子rename`安装状态机及启动崩溃恢复矩阵。
- [ ] 实现field WAL+FULL/checkpoint/WAL上限/quick_check和SQLite一致性备份，并执行逐阶段断电故障注入。
- [ ] 实现分类诊断配额、field原始传感默认关闭、维护采集自动过期和带脱敏清单的显式导出。
- [ ] 实现Host/OCLive持久状态所有权白名单，验证瞬时Fact、滤波、BLE会话和单调cooldown不会跨重启恢复。
- [ ] 实现Shutdown Coordinator状态机与预生成表达截止时间；P0先完成safe-to-cut人工切电，field电源板到位后接可信低电输入、完成握手和主rail切断。
- [ ] 实现启动恢复门、baseline不发事件、节点超时Degraded、LifecycleContext一次消费和clean/unclean恢复回放测试。
- [ ] 为 AN94 固定把玩/射击两种模式的事件—输出映射。
- [ ] 扩充v0.2跨契约不变量和回放测试；生成Schema零漂移、基础语义校验及首批正反fixtures已完成。
- [ ] 为 `node.status.power_source` 增加 `lipo_1s` 并补兼容测试。

验收门 G0：节点观察、六个语义事件、一个屏幕输出协议和两种模式没有未定义的核心状态。

## S1：器灵模拟器

目标：在 Windows 开发机上不用硬件完成器灵闭环。

- [x] 实现`PerceptionReplay`会话门：新Host boot安静建baseline、同revision允许发布/证据时效刷新但分类值与unknown原因等语义必须相同、revision倒退/同revision语义漂移/非法状态拒绝。
- [x] 实现`ailive-gun-spirit-sim`命令行mock，默认回放Standby→Held→Ready→control→重启→放低→Standby→Unknown并输出JSON Lines。
- [x] 增加标准生命周期、Host重启、非法revision、非法命令和禁止事件回放测试。
- [x] 把模拟器输入改为真实`SensorObservation`，经纯reducer生成完整`PerceptionState`与`DeviceEvent`；不再直接构造最终载体状态。
- [x] 实现Host侧最小`InputScheduler → SensorTurnRequest/DeviceContext`类型缝，sensor输入不伪装成用户文本；真实OCLive投递留给S2。
- [x] 实现`OutputArbiter → ScreenViewModel`投影：当前revision角色提示、常规状态和节点离线SystemCue有确定优先级。
- [x] 自动连续回放100次标准序列，断言每轮回到Standby且离线后进入System层，无状态卡死。

- [ ] 增加键盘/测试面板，模拟拿起、举起、辅助触点、放回、模式切换和断网。
- [x] 实现SensorObservation→`Standby ⇄ Held ⇄ Ready`完整融合；只读primary control保持独立Fact/Event，不建立Active或击发状态。
- [ ] 完成S1其余协议行为：reducer的TTL、sequence和过期已完成；NodeHello、跨连接事件去重、冷却和持久日志回放仍未完成。
- [ ] 用 AN94 `visual_state_id` 驱动 PNG。
- [x] 用本地类型投影生成即时屏幕状态，不接语音或触觉输出；当前输出JSON ViewModel，PNG renderer另列。
- [x] 连续回放100次标准序列并由集成测试逐轮断言结果。
- [x] 建立`FRONTEND_DESIGN_BRIEF.md`，冻结800×480首轮信息架构、九个状态稿和ViewModel映射；视觉风格与renderer仍未冻结。

验收门 G1：无重复回合、无状态卡死；断网时仍能完成拿起—放回反馈。

## S2：OCLive 输入来源契约

目标：传感器回合不会伪装成用户发言或污染角色状态。

- [x] 在 OCLive 主仓实现兼容的`TurnOrigin = user | sensor | system`；现有HTTP/Tauri载荷不增加可选origin字段。
- [x] 由OCLive内部根据origin推导副作用策略，外部客户端不能请求低持久化回合。
- [x] sensor/system跳过用户情绪插件与用户聊天状态写入，包括聊天、记忆、事件、好感、关系、人格、连续性和虚拟时间提交。
- [x] 设备审计继续归本仓Host，未写入OCLive聊天存储。
- [x] 增加跨边界回归：同一会话sensor回合零提交，随后普通user回合仍写事件、聊天和角色情绪。
- [x] 本仓`OcliveSensorAdapter`把有界中性上下文投递为类型化sensor回合，并把OCLive回复/视觉状态投影为`RoleCueSource::Oclive`。

来源与副作用契约已经接通，但默认命令行模拟器仍故意使用`desktop_fixture`，这样无本地模型也能稳定回放。S2完成不等于S3通过；真实adapter接到可操作桌面Host流程、超时/取消/迟到/失败测量和PNG/Web renderer暂缓，等待OCLive内核梳理完成。它不阻塞四季宝硬件到货核验、单板bring-up、节点协议和电源测试。

验收门 G2：同一传感器事件重复运行后，聊天、长期记忆、好感和人格无非预期变化；普通用户对话不回归。

## S3：桌面纵向闭环

目标：模拟器通过真实 OCLive Host 调用 AN94。

- 即时反馈先发生，Fast 动态回复后到达。
- `visual_state_id` 与本地 HUD 状态进入统一屏幕仲裁。
- 记录事件进入、即时屏幕反馈、首字和动态画面接管时间。
- 模拟超时、失败、取消和迟到回复。

当前已完成本仓半闭环和真实OCLive窄调用能力：即时`PerceptionState/SystemCue`和桌面fixture `RoleCue`进入同一个Output Arbiter，过期RoleCue及SystemCue抢占有单测；真实adapter可生成`RoleCueSource::Oclive`。尚未完成的是把真实调用接入可操作桌面流程、PNG/Web renderer、超时/取消/迟到处理和时延采样，因此G3不能宣称通过。界面准备入口见`FRONTEND_DESIGN_BRIEF.md`。

验收门 G3：连续运行 2 小时；即时反馈规划目标 p95 ≤150ms；模型失败不导致 UI 卡死或设备状态丢失。

## S4：单机硬件原型

目标：先做与载体完全分离的桌面套件；ARM Linux 上无需人工 SSH 或自建服务器即可完成传感器—本地 OCLive—屏幕闭环。

平台 bring-up 与系统服务细节见 `LINUX_ARM64.md`。

套件边界、本地/可选网络能力分工与上枪顺序见 `PROTOTYPE_KIT.md`。

最小硬件：P0 Orange Pi Zero 3W 6GB/A733、板载散热器/小风扇、USB-C DP主动转HDMI、3.5inch 480×800 HDMI IPS电容屏、前下导轨XIAO nRF52840 Sense + FSR/IMU节点，以及后握把XIAO + FSR节点。两个节点先用USB/桌面电源完成协议和校准，再分别加入受保护1S LiPo；CR2032只保留给未来专用低功耗节点，不能接入XIAO充电路径。主机桌面电源先行，完成显示、内存、无线、功耗和热测试后再冻结一体式三轴主机舱主电池。状态判断不依赖磁性底座。本地3B是可选角色表达实测项，不是P0即时闭环的硬依赖。

- 启动 OCLive Host 并加载角色包。
- 接屏幕、IMU、按钮、Grip Node 和 watchdog。
- 先在 Windows mock 按 800 × 480 验证角色 UI，再在 Orange Pi 上通过 HDMI 点亮样屏并识别 USB 触摸，完成户外/偏振/保护层、20 次冷启动、四档背光功耗和板屏堆叠测试。
- 前后节点只发送低频 SensorObservation；主机按 node_id/capability 融合前握、后握与运动证据，不上传压力/IMU 原始流。
- 本地加载 OCLive Host、角色包、状态/记忆与 SQLite；外部 LLM API 仅是可选表达能力。
- 在制作Orange Pi镜像前完成OCLive Linux依赖裁剪审查：用目标架构feature依赖树、ELF动态依赖、服务/文件清单和实测资源建立“保留/宿主替换/按需启用/镜像排除”矩阵；从镜像排除不等于删除OCLive主仓源码。
- 按`ORANGE_PI_BRINGUP_WORKSHEET.md`回填板卡/镜像身份、冷启动、内存/线程、功耗、温度、显示、无线、事件延迟、关机恢复和资源快照；P0协调器仅观察，不根据未测估算执行准入。
- P0 Web闭环只实现最新完整ViewModel、loopback HTTP/WS安全桥、OCLive image catalog复用和PNG fallback；用mock transport/AssetResolver证明契约不绑定WebSocket与单一资产来源，不提前实现Live2D或外置Visual Pack。
- 实测RSS、`MemAvailable`、冷启动、温度、事件延迟和两小时稳定性；冻结门按Zero 3W 6GB实机建立，不沿用Zero 2W 2GB或Lyra 512MB数值。另按`LOCAL_3B_INFERENCE.md`对约3B、1.5–1.7B、0.5–0.6B三个尺寸档测试base/有界角色上下文、常驻、短句回复和连续压力；训练只在基线归因后进入。
- 实测主机屏灭/屏亮/Wi-Fi/BLE/本地 OCLive/可选网络回合功耗，按 4 小时 Alpha 与 8 小时设计目标计算电池。
- 断网重启仍可进入本地模式。
- 1,000 条 DeviceEvent 无丢失且事件到本地画面 p95 ≤100 ms；Wi-Fi + BLE + 屏亮连续两小时无驱动崩溃。
- 样屏与板型通过后，用 225/285/345 g 一体式主机舱假体验证三轴力矩、导轨固定、收纳泡棉和 P1 USB 短跳线；HDMI 不跨轴，再冻结电芯、模块布局和 RADIAN 导轨适配。

验收门 G4：自启动可用；连续 2 小时无崩溃、事件风暴和明显过热降频；前后节点分别使用受保护 1S LiPo 完成 8 小时会话回放，低电重连无 brownout；主机电池完成 4 小时完整负载 Alpha，边充边用和低电关机正常。

## S5：实体结构与场地 Alpha

目标：套件闭环通过后，先做 RADIAN MODEL 1 的非功能安装适配，再上枪验证它不只是一次性新奇玩具。

- 实测 RADIAN MODEL 1 导轨槽位、瞄具/操作净空、重心和屏幕视角。
- 以刚性夹具 + 三轴屏幕舱实现多导轨位置展开；任一屏幕转轴调整都不得改变固定 IMU 的载体姿态。
- 采用通用电子核心 + 可更换皮卡汀尼底座；快拆、非承力，首版不接内部火控、扳机或供弹机构。
- 先用断电套件检查装配与操控，再进行通电静态测试，最后进入场地测试。
- 至少三次真实场地测试。
- 记录误触、漏触、重启、断连和主动互动次数。
- 原型阶段按`UX_RESEARCH.md`把已冻结且能实际展示的体验决定拿给几名wargame爱好者简单复核；不规定正式样本量、评分或A/B流程，也不让单一意见直接冻结方案。

建议验收门 G5：

- 核心事件识别成功率 ≥95%。
- 明显误触发 ≤1 次/小时。
- 连续操作时屏幕不冻结、不残留过期状态。
- 不妨碍正常握持、瞄具和安全操作。
- 发热、松脱、夹手或遮挡必要操作等问题已经关闭或明确阻断下一阶段；重复出现的体验问题已尝试复现。
- “愿意再次使用”只作为方向信号，不单独作为发布门。

## S6：增强候选

只有 G5 通过后才恢复排期：

1. 第三个及更多 BLE 功能节点（只有双节点实测不足时增加）。
2. 拉栓、上弹、循环等确定性只读机械事件。
3. 独立弹匣余量扩展：以可信机械循环短计数为连续估算，以磁性随动件 + 外部霍尔阵列为绝对范围锚点；先按`MAGAZINE_AMMO_ESTIMATION.md`验证一个智能弹匣，不修改P0 v0.2闭集，不控制火控/电机/供弹。
4. 触觉反馈。
5. ASR、TTS 与手机侧车语音能力。
6. 更多角色和器灵场景。

相机、录像、拐角观察、AI 视觉、逐颗精确弹药计数和 Live2D 必须独立立项，不进入器灵核心。第3项只承诺粗粒度范围融合；在没有逐弹通过证据时，循环数不得对外表述为实际射出颗数。
