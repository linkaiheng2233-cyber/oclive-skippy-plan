# oclive-四季宝器灵

正式产品名为 **A.I.Live-ai枪娘器灵**。本仓是 OCLive 的第一个实体角色宿主：把角色包、记忆、情绪和场景编排接入物理事件，让设备能感知用户拿起、举起、操作只读辅助触点和放回，并通过小屏表情与状态作出符合人格的回应。“四季宝器灵 / Skippy Spirit”只作为灵感来源与早期工程代号保留，不再作为正式产品名或新协议命名空间。

当前状态：**三crate软件地基、v0.2类型化感知契约、SensorObservation确定性融合、Input Scheduler窄缝、屏幕ViewModel仲裁和OCLive类型化sensor回合窄适配器已经落地；命令行桌面夹具可连续回放100轮。用户已确认首批硬件到货，但实物身份、完好、电气参数和兼容性尚未逐件核验，实体板卡尚未bring-up。下一硬件里程碑是完成到货清单并只对Orange Pi单板做受控首次上电；软件仍继续真实OCLive桌面Host与PNG/Web renderer闭环。**

首台原型载体固定为 **森柏龙 RADIAN MODEL 1**。实施顺序是桌面套件 → 套件外壳/导轨适配 → 上枪验证，不在软件和电子闭环完成前修改或连接载体内部机构。

套件机械目标是「一套电子核心 + 可更换导轨安装件」：首发覆盖带标准皮卡汀尼导轨的 wargame 载体，其它安装标准通过独立转接件扩展。传感器默认不走机内线；前下导轨 BLE 节点提供固定载体参考 IMU 与前握感知，后握 BLE 节点提供后握与只读辅助触点；状态判断不依赖磁性放置底座。

显示机构采用三轴一体式显示主机舱：yaw/pitch/roll 三个恒扭矩转轴负责调姿；屏幕、Linux 小板、壳内短距视频/触摸连接和完整受保护 5 V 电源共同装入移动舱。固定载体参考 IMU 在独立前下导轨节点中，不随屏幕转动。当前模块化bring-up基线为 Orange Pi Zero 3W 6GB（Allwinner A733）+ 3.5inch 480×800 HDMI IPS电容屏，P0 不跨轴走电气线；USB-C DP Alt Mode 到 HDMI 的主动转接必须纳入包络、功耗和冷启动验证。Luckfox Lyra Zero W + 2.8inch DSI只保留为移动质量、厚度或功耗不达标时的紧凑回退。屏幕边缘缓冲框在收纳贴合时先承力，LCD保持内缩。

## 项目分层

| 层 | 职责 | 所在位置 |
|----|------|----------|
| Gun Perception Kernel | 节点注册、校准、数据时效、多传感融合、载体状态机与故障降级 | 本仓独立Rust crate，不依赖OCLive |
| OCLive Core | 人格、记忆、情绪、场景、角色包与回合编排 | 兄弟仓 `../oclivenewnew`，不依赖感知内核 |
| A.I.Live Gun Spirit Host | 组合两个内核、DeviceEvent桥接、屏幕仲裁、ARM Linux与硬件适配 | 本仓应用层 |

本项目不做智能火控、弹道计算、自动瞄准、武器控制或军警用途。产品原型 P0/v0.1 只验证「器灵是否真的活了」；感知数据契约从 v0.2 起步，两种版本号不是同一层。

## v0.1 闭环

```text
模拟/实体传感器
  → 节点/驱动层滤波、去抖、边沿检测
  → 低频 SensorObservation
  → 枪械感知内核融合
  → PerceptionState + DeviceEvent
  → 即时确定性反馈
  → 必要时经窄桥接触发 OCLive 回合
  → 角色回复与屏幕输出仲裁
```

感知内核与OCLive是两个平级、单向协作的薄核：前者只回答“物理上发生了什么”，后者只回答“角色如何理解和表达”。P0在同一个`ailive-gun-spirit-host`进程运行，但两个内核源码互相零依赖、互相不知道；只有最外层bridge通过版本化值契约组合它们，不共享内部对象或可变状态。

屏幕由`ailive-gun-spirit-host`的Output Arbiter统一控制：感知侧贡献系统状态和确定性提示，OCLive贡献角色表情与回复，仲裁器按角色主体、常驻状态栏和高优先级系统Overlay三层组合成最终画面。两个内核都不直接操作显示设备或前端组件。

交互模式同样由`ailive-gun-spirit-host`管理。新感知能力可以独立加入本地状态，只有bridge提供显式映射时才触发OCLive；角色回合只响应有意义的状态转换，并经过事件去重、合并、冷却和过期检查，周期传感快照不会让角色重复说话。

长期职责门禁固定为：感知内核只产事实，枪械交互伴随包只声明策略，Host/Input Scheduler只执行调度，OCLive只完成角色理解与表达。任何新增功能若必须跨越这些边界，须先新增ADR说明理由、替代、耦合与回退，不能顺手跨层。

未来加入语音后，OCLive侧Input Scheduler以语音/文字作为PrimaryInput，把近期中性DeviceContext作为AuxContext合入同一个主对话；没有显式输入时，门控后的DeviceEvent才可独立开启sensor turn。角色包负责把设备事实解释成情绪，感知内核不认识ASR、角色或OCLive情绪词表。安全SystemCue不参与对话排队，始终由host立即处理。

枪械交互反应不会写入OCLive标准角色包格式。本项目使用独立、版本化并引用基础OCLive角色身份的“枪械交互伴随包”，由专用loader对接；它可以像OCLive角色包一样从电脑选择目录或压缩包导入，但会安装到独立的受管目录，不写入基础角色包。伴随包缺失或不兼容时基础角色仍可运行，只关闭枪械专属主动反应。

本项目专用的感知契约、枪械交互伴随包、硬件HostProfile、阈值和结构文档只保存在本仓，不写入OCLive通用文档。OCLive现有Resource Coordinator未来可为ASR、TTS、相机分析、复杂renderer等可选重任务提供准入和降级，但不接管常驻感知、安全关机或Linux CPU调度；P0是否接入须由Orange Pi实测决定。

v0.1 只包含：

- 一个角色：AN94。
- 两种模式：把玩、射击。
- 四组核心体验：拿起、举起、只读辅助触点按下/释放、放回。
- 一套状态证据：前/后握 FSR 判断握持，前下导轨固定 IMU 判断移动、静止与举起；两处明确释放 + 持续静止进入待机。
- PNG 表情与极简 HUD。
- 断网时仍完整可用的本地屏幕反馈。

ASR、TTS、扬声器、预渲染音频、触觉输出、相机、只读AI视觉、心率、精确弹药计数、Live2D、第三个及更多BLE功能节点均在MVP之后评审。Zero 3W 6GB新增CPU量化3B及以下本地模型候选；到板后按约3B、1.5–1.7B、0.5–0.6B三个尺寸档做相同对话、角色上下文、速度、质量和功耗实测，不进入即时感知和安全关键路径。

## 当前骨架

本仓最初由OCLive `robot-soul`工厂模板生成，现已迁移为最小Rust workspace，并继续使用`oclive_kernel_host`纯无头依赖：

- `crates/ailive-gun-spirit-contracts`：v0.2 DTO、语义校验和确定性生成JSON Schema。
- `crates/ailive-gun-spirit-perception-core`：不依赖OCLive的Observation reducer、TTL/boot/sequence、状态融合和确定性边沿逻辑。
- `crates/ailive-gun-spirit-host`：Input Scheduler、DeviceContext、Output Arbiter、桌面夹具、标准无头OCLive Host及未来BLE/Linux/Web组合根。
- `roles/default`：RobotSoulPack 烟测夹具，不是正式 AN94 资产。
- `schemas/`：生成的v0.2感知契约、正反fixtures和v0.1历史草案。
- `docs/`：架构、路线和决策记录。

当前 `Cargo.toml` 通过相对路径依赖同级 `../oclivenewnew`，适合本地协同开发，但独立远端 checkout 尚不可复现。远端仓/CI 建立前必须先完成[技术债 GS-CI-001](docs/TECHNICAL_DEBT.md)的依赖获取决策。

## 远端与权威

| 项 | 值 |
|---|---|
| **唯一权威** | 本仓（开发机上 `E:\OCLive\oclive-四季宝器灵`） |
| GitHub 远端 | `origin` = `https://github.com/supermumucoming/oclive-skippy-plan.git`（**公开 / 开源**） |
| 当前基线 | tag `baseline/GS-P0-BL-2026-09-18`，Baseline ID 见 `docs/PROJECT_BASELINE.md` |
| OCLive 兄弟仓 | `https://github.com/linkaiheng2233-cyber/oclivenewnew.git`（公开；本仓按同级目录路径依赖，clone 时放到同级目录即可） |
| 大工件 | 仓库外 `E:\OCLive\oclive-四季宝-artifacts\`（镜像、证据包、模型权重不入仓） |
| 许可 | 本仓代码 MIT（见 `LICENSE`）；角色图像、声音与文本资产另行声明，不自动继承代码许可 |

**本机网络提示**：本开发机直连 `github.com:443` 不通，需走本机代理；已配置 `git config --global http.https://github.com.proxy http://127.0.0.1:7897`（gh 命令需临时设置 `HTTPS_PROXY` 环境变量）。

**公开仓库的脱敏约定**：SSID、内网地址、`wlan0` MAC 等本机网络标识不写入仓库，统一存放在仓库外工件目录 `bringup-toolchain/network-identifiers.txt`；文档中只保留频段/信道等非标识信息。

**独立克隆的已知限制**：远端 clone 后必须在同级目录放一份 `oclivenewnew` 才能编译（路径依赖）。仓库当前**不提供 GitHub Actions**：按 `docs/DEVELOPMENT_DISCIPLINE.md`§9，依赖获取方式冻结前不创建必然失败的 CI；`GS-CI-001` 记录了解法候选（钉住 rev 的 git 依赖 / submodule / 受控 vendor），且兄弟仓已在 GitHub，候选 (a) 已可执行。

**桌面与导出件不是权威**：桌面测试包、个人文档目录和现场填写件都只是导出副本；改动一律回填本仓，见 `docs/history/EXTERNAL_DOCUMENT_AUDIT_2026-09-18.md`。

OCLive 不是云端薄客户端：Host、角色包、记忆/状态、SQLite、DeviceEvent、器灵状态机和屏幕仲裁都在板端运行。P0 可以使用本地 mock，也可以把动态表达接到外部 LLM API；网络不可用时动态文本可以降级，但设备必须能启动，拿起、举起、辅助触点和放回的完整闭环必须继续工作。当前不要求自建服务器。

## 启动无头宿主

```powershell
$env:OCLIVE_HTTP_API_MOCK_LLM = "1"
$env:OCLIVE_ROLES_DIR = "$PWD\roles"
cargo run -- --port 8420
```

默认 API 端口为 `8420`。生成器的完整配置参考见 `CONFIG_REFERENCE.md`。

## 无硬件生命周期回放

```powershell
cargo run -p ailive-gun-spirit-host --bin ailive-gun-spirit-sim
```

默认依次回放baseline、Held、Ready、辅助触点按下/释放、Host重启、放低、待机和Unknown。每个命令先产生前/后节点`SensorObservation`或时钟推进，再经过reducer、quiet baseline、`PerceptionState`、完整`DeviceEvent`、类型化`sensor`输入请求、桌面角色夹具和Output Arbiter，最终以JSON Lines输出完整状态与`screen_view_model`。也可在命令后显式给出`baseline held ready control-on control-off lower standby unknown reboot`序列。

`role_bridge.source=desktop_fixture`且`oclive_dispatched=false`是有意的默认模拟器标记：OCLive兄弟仓已经具有类型化sensor回合来源，本仓也已有窄适配器，但默认命令行仍保持无模型、确定性回放。不得因此改用伪用户文本；把真实适配器接入可操作桌面流程、超时/取消和renderer仍属于S3。

## 路线入口

- [统一文档入口](docs/README.md)
- [项目开发基线](docs/PROJECT_BASELINE.md)
- [项目边界](docs/PROJECT_BOUNDARIES.md)
- [文档体系与冲突规则](docs/DOCUMENTATION_SYSTEM.md)
- [硬件工程学习指南](docs/HARDWARE_ENGINEERING_GUIDE.md)
- [人类与 AI 交接文档包](docs/handoff/README.md)
- [架构边界](docs/ARCHITECTURE.md)
- [实施路线](docs/ROADMAP.md)
- [关键决策](docs/DECISIONS.md)
- [开发纪律与完整门禁](docs/DEVELOPMENT_DISCIPLINE.md)
- [技术债与审查台账](docs/TECHNICAL_DEBT.md)
- [Wargame 爱好者既有决策体验复核清单](docs/UX_RESEARCH.md)
- [P1+ 弹匣余量双源融合设计](docs/MAGAZINE_AMMO_ESTIMATION.md)
- [800×480 前端设计准备包](docs/FRONTEND_DESIGN_BRIEF.md)
- [Linux ARMv7/ARM64 适配路线](docs/LINUX_ARM64.md)
- [ARMv7/Luckfox 可行性实测](docs/ARMV7_VALIDATION.md)
- [Orange Pi Zero 3W 6GB 实机测试填空表](docs/ORANGE_PI_BRINGUP_WORKSHEET.md)
- [本地3B及以下角色表达评估](docs/LOCAL_3B_INFERENCE.md)
- [硬件到货测试包入口](docs/test-worksheets/README.md)
- [爱好者原型体验简表](docs/test-worksheets/06-爱好者用户体验反馈表.md)
- [P1+ 余弹融合与弹匣结构测试表](docs/test-worksheets/07-余弹融合与弹匣结构测试表.md)
- [RADIAN MODEL 1 套件路线](docs/PROTOTYPE_KIT.md)
- [通用导轨机械接口](docs/MECHANICAL_INTERFACE.md)
- [三轴显示主机舱与导轨机构](docs/DISPLAY_ASSEMBLY.md)
- [终端机外观与层叠结构 V0.1](docs/TERMINAL_ENCLOSURE_VISUAL_V0_1.md)
- [屏幕选型与冻结门](docs/DISPLAY_SELECTION.md)
- [一体式主机舱电池与充放电路线](docs/MAIN_POWER.md)
- [传感器候选与实测计划](docs/SENSOR_OPTIONS.md)
- [BLE 无线功能传感器网络](docs/WIRELESS_SENSOR_NETWORK.md)
- [无线节点电池与功耗预算](docs/POWER_BUDGET.md)
- [感知契约版本状态](schemas/README.md)（v0.2已由Rust类型生成；v0.1仅为历史草案）
- [SensorObservation v0.2](schemas/sensor-observation.v0.2.schema.json)
- [PerceptionState v0.2](schemas/perception-state.v0.2.schema.json)
- [DeviceEvent v0.2](schemas/device-event.v0.2.schema.json)
- [SensorObservation v0.1 历史草案](schemas/sensor-observation.v0.1.schema.json)
- [DeviceEvent v0.1 历史草案](schemas/device-event.v0.1.schema.json)
- [OutputCue v0.1 Schema](schemas/output-cue.v0.1.schema.json)

## 本地完整验证

```powershell
./scripts/verify.ps1
```

脚本只格式检查本仓三个 package，不会使用`cargo fmt --all`越界修改兄弟 OCLive；随后检查 workspace 依赖边界、Clippy、all-targets、doctest、Schema 漂移和 RustSec 审计。

## 许可证

本仓新增代码使用 MIT。OCLive 兄弟仓仍遵循其 Apache-2.0 许可证；正式角色图像、声音与文本内容分别声明授权，不自动继承本仓代码许可证。
