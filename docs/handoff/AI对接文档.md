# AI 对接文档（跨会话进度对齐）

**用途**　供项目所有者、DeepSeek、GPT、Claude Code 等接续同一份进度，回答「现在在哪、下一步做什么、谁在跟、证据在哪里」。技术决定、实现和验收分别由 [项目基线](../PROJECT_BASELINE.md)、[ADR](../DECISIONS.md)、[硬件计划](../HARDWARE_IMPLEMENTATION_PLAN.md)、[技术债](../TECHNICAL_DEBT.md)及实测记录维护。 [文档体系](../DOCUMENTATION_SYSTEM.md)

**更新规则**　开工先核对 §1 的仓库状态与 §5 的检查点。每次实质推进后更新 §1、对应检查点及 §6 最新一行。先把新技术决定或实测写入对应来源，再回填此处的摘要和链接。收工可直接使用 §11 的模板。

**最后核对**　2026-11-13 20:41（北京时间）。

## 1. 状态快照

| 项目 | 最近核对结果 |
| --- | --- |
| 仓库 | `E:\OCLive\oclive-四季宝器灵`，远端 `linkaiheng2233-cyber/oclive-skippy-plan`。相邻 OCLive 仓为 `E:\OCLive\oclivenewnew`。 |
| 分支与提交 | 本地 `docs/personal-document-archive-20261030`，核对基点 `8b71e0d`；相对 `origin/main` `5417492` **领先 12、落后 0**。远端只读查询未返回同名分支。本文及本轮同步的文档是此基点上的本地工作区改动。 |
| 基线 tag | 已验证的最近基线为 `baseline/GS-P0-BL-2026-09-23`。11 月新基线待 `GS-DEP-001` 处置和门禁复核。 [基线记录](../PROJECT_BASELINE.md) |
| 工程门禁 | `8b71e0d` 配相邻 OCLive `90677e53`，复跑 `./scripts/verify.ps1`。fmt、workspace 边界、Clippy、**42 项测试**、doctest、Schema 零漂移通过。`cargo audit` 查出 1 项漏洞并退出 1，另有 3 条工具报告为 allowed 的警告。 [命令和结果](../TECHNICAL_DEBT.md#1-审查基线) |
| 开放审计项 | `GS-DEP-001` / `RUSTSEC-2026-0459`。当前 Windows Host 依赖链经相邻仓 `oclive_sqlx`、`sqlx-sqlite 0.8.6` 引入 bundled `libsqlite3-sys 0.30.1`，修复路径见 §4。 [技术债台账](../TECHNICAL_DEBT.md#2-当前台账) · [RustSec 公告](https://rustsec.org/advisories/RUSTSEC-2026-0459.html) |
| 板卡 | 晶彩已点屏，基础触摸和 USB 表情控制通过。图库串口顺序通过，画面和按钮待现场验收。2026-11-13 16:45 最后选择出厂 `app0`，沐沐图库留在 `app1`，出厂画面待现场确认。 [到货与实机记录](../test-worksheets/01-硬件到货清单.md) |
| 阶段 | P0-A 是当前主线。板上共享感知内核、USB 状态闭环、外接 IMU 和 Wi-Fi 会话均待验证。固定角布局、后握薄套和电池位置是设计候选。P0-B 保留后置。 [实施路线](../ROADMAP.md) |

接手时可用 `git status --short --branch`、`git rev-parse --short HEAD` 和 `git rev-list --left-right --count origin/main...HEAD` 复核。最后一条按「落后、领先」输出，对比的是本地记录的远端引用。

## 2. 一句话现状

**晶彩小屏已经工作，Rust v0.3 感知在桌面通过回归。下一步由 DeepSeek 推进共享内核的 CP1 桌面对拍，现场可以并行验沐沐图库和断电假体。** 桌面对拍通过后还需 ESP32-S3 目标编译、链接与板上回放，才有板上运行证据。Windows 台架程序先做表情回传，正式 OCLive 桌面接入继续按路线恢复。 [ADR-069](../DECISIONS.md#adr-069p0-a-由-esp32-s3-运行感知内核) · [实施路线](../ROADMAP.md)

## 3. 已定决策

| 决定 | 内容与出处 |
| --- | --- |
| ADR-063 / D1–D6 | P0-A 用 PC 宿主，调试 USB-C，运行走 Wi-Fi STA。小屏固定在导轨，语音后置。外部只读触点及 IMU 候选见形态决定，首轮仍以 IMU 为先。 [D1–D6](../ESP32_GUN_ASSISTANT.md#91-已裁决2026-09-23项目所有者) |
| ADR-065 / 067 | 节点按能力集配置。v0.3 用 `GripContact.grip` 区分前后握，`AngularRateSample` 走独立流。 [ADR-067](../DECISIONS.md#adr-067节点模型泛化落地单板形态强制契约-v03角速率改走独立指向流) |
| ADR-066 | 主机侧娱乐功能另有路线及信息边界。 [主机娱乐设计](../PC_SIMULATION_BACKENDS.md) · [项目边界](../PROJECT_BOUNDARIES.md) |
| ADR-068 | 晶彩 JC3248W535 带壳一体屏，悍迪 191 为首台载体。先做桌面套件。 [ADR-068](../DECISIONS.md#adr-068首版采用晶彩板屏一体方案成本优先amoled-按需评估) |
| ADR-069 | ESP32-S3 运行感知内核和本地反馈。Windows PC 本地跑 OCLive，经快照与类型化事件联机。USB 和 Wi-Fi 共享消息语义。 [ADR-069](../DECISIONS.md#adr-069p0-a-由-esp32-s3-运行感知内核) |

## 4. 等待决定的开放项

| 编号 | 待定事项 | 当前接续方式 |
| --- | --- | --- |
| O1 | `GS-DEP-001` 与 11 月基线 tag | 优先在 OCLive 兄弟仓评估依赖升级，或在适用构建范围内评估隔离。若提出带期限的定向例外，需保留影响面、处置人与到期日期，并由所有者裁决。门禁状态以实测为准。 [技术债台账](../TECHNICAL_DEBT.md#2-当前台账) |
| O2 | 开发分支名与合入路线 | 现有分支可接着做本地工作。改名、合入和 PR 路线由所有者安排。 |
| O3 | 共享内核的板端集成方式 | 先试 CP1，再用 CP1b 判定与 Arduino/LVGL 的集成成本。桌面与板端证据齐后再选集成路线。 |
| O4 | 同步远端与新 tag | 待所有者安排推送和协作基线。外部会话若只拿到远端 `main`，需要补充当前提交与本地变更。 |

## 5. 下一步计划

软件线 **CP1 → CP1b / CP2 → CP5**。CP2 的协议草案可以和目标构建并行。现场线 **CP3 的现有图库验收、CP4 的断电假体**可以先做，IMU 则接在最小固件与线束核对之后。

| 检查点 | 状态与工作 | 通过判据 |
| --- | --- | --- |
| **CP1 · 桌面等价** | **待开始，首选软件任务。** 用单一感知实现对照当前 `std` 与候选 `no_std` 路径。以 `contracts`/`perception-core` 为范围，核对标识符、固定容量集合与验证错误，时间继续由调用方注入。`esp` feature、定长 ID、`heapless` 为实现候选。先利用现有夹具和回放。 | 在独立的 Cargo 调用里验证两种配置，核对 feature 树。对同一输入逐帧比较 `PerceptionState`、事件类型及顺序，覆盖未安装传感器、过期、重启与重复输入。保留当前 Host 的 9 帧回归、v0.3 fixtures 及 Schema 零漂移。 [回归测试](../../crates/ailive-gun-spirit-host/tests/golden_frames.rs) · [Cargo feature 规则](https://doc.rust-lang.org/cargo/reference/features.html#feature-unification) |
| **CP1b · ESP32-S3 目标** | 待 CP1 形成可复现结果。验证目标编译和最小 C ABI 或独立 ESP-IDF 集成，接现有驱动前保留可运行的图库镜像。 | 记录工具链、依赖、链接结果与镜像大小。上板对照同一回放输入，记录内部堆与 PSRAM，并复测显示和触摸。 [固件与分区约束](../HARDWARE_IMPLEMENTATION_PLAN.md#013-p0-a-固件与分区约束2026-11-13-评审补记) |
| **CP2 · 最小联机语义** | 待实施，台架先走 USB。定义快照、事件、表情回传和设备启动会话映射。帧头和启动标识的具体格式待验证，首轮可静态配置能力集，正式 `NodeHello` 与 `AssetRef` 见 `GS-CONTRACT-004`。 | 主机测试覆盖首连、重连、新启动、同 `revision` 重发、过期回复、截断和坏帧。初连/重连先建快照基线；接入 `AngularRateSample` 时按类型分派，保持独立账本。 [ADR-067](../DECISIONS.md#adr-067节点模型泛化落地单板形态强制契约-v03角速率改走独立指向流) · [ADR-069](../DECISIONS.md#adr-069p0-a-由-esp32-s3-运行感知内核) |
| **CP3 · 现有图库与首颗 IMU** | 图库串口顺序通过、现场待验。用原型镜像和同板备份按说明启动 `app1`，依次点 `Next`、`Next`、`Prev`。之后以 USB 供电核对 P3/P4 和 IMU 模块。 | 图库三次画面与 `TOUCH`/`MOOD` 回执一致。IMU 核对地址/芯片 ID、轴向、抬放与静止、停止采样后转 `unknown`。前后握未接保持 `unknown`，完整握持流程用模拟输入覆盖。 [图库验收](../../firmware/jc3248w535/README.md#实机验收) · [感知试验表](../test-worksheets/P0-A-晶彩板屏台架与装配.md#2-usb-供电下的感知) |
| **CP4 · 固定角假体与供电** | 方案候选，现场待验。先用带壳屏纸样和等尺寸/质量电池假体试装，比较屏幕后与固定侧腔。可单独跑独立 5 V 功耗测量。 | 量出实际包络、活动件净空及后握盖线槽路径，记录握持和插拔空间、平均功耗与峰值。电池另走 P5/成品 5 V 路线的实测门。 [装配及供电试验表](../test-worksheets/P0-A-晶彩板屏台架与装配.md) |
| **CP5 · USB → Wi-Fi 闭环** | 待板端回放与消息语义可用。先让 Windows 测试程序收状态并回表情 ID，再改用独立供电和局域网。 | 断开 PC 后板端保留本地反馈。重连先取最新快照，新事件可用，重复消息及旧回复不会触发表达。记录链路延迟和恢复时间。 [实施路线](../ROADMAP.md) |

## 6. 进度日志（新→旧）

| 日期 | 谁 | 做了什么 | 结论与证据 |
| --- | --- | --- | --- |
| 2026-11-13 20:41 | GPT | 复跑门禁，核对 Git、分区备份和现有源码。补全 CP1/CP1b、现场并行项与交回模板，并校正现行基线的旧口径。 | 受检 `8b71e0d`。42 项测试通过，审计仍为 1 项。 [门禁证据](../TECHNICAL_DEBT.md#1-审查基线) |
| 2026-11-13 20:30 | DeepSeek | 建立本文件，与远端 `main` 合并，修补 4 项传递依赖漏洞，登记 `GS-DEP-001` 和四项硬件/固件约束。 | 合并 `44a9849`，对接稿 `8b71e0d`。 [技术债](../TECHNICAL_DEBT.md#2-当前台账) · [评审补记](../HARDWARE_IMPLEMENTATION_PLAN.md#013-p0-a-固件与分区约束2026-11-13-评审补记) |
| 2026-11-13 晚间 | GPT | 整理固定侧 IMU、后握薄套与单路供电候选，纠正 P5 图纸针号，建立试验表。 | 设计候选，实物数据待填。 [P0-A 试验表](../test-worksheets/P0-A-晶彩板屏台架与装配.md) |
| 2026-11-13 16:45 | 前序会话 | 图库烧入 `app1`，验证串口换图与双向循环，选择出厂 `app0` 恢复启动。 | 图库实体按钮与出厂画面待现场观察。 [实机记录](../test-worksheets/01-硬件到货清单.md#p0-a-沐沐情绪切换2026-11-13) |
| 2026-11-13 下午 | 前序会话 | Arduino-ESP32 3.0.2 与 LVGL 8.3.11 验证点屏、基础触摸和 USB 表情回传。 | [到货验收表](../test-worksheets/01-硬件到货清单.md#p0-a-一体屏首次上电2026-11-13) |
| 2026-10-12 | 前序会话 | 契约 v0.3 与节点模型泛化，保留 9 帧 golden 回归。 | [ADR-067](../DECISIONS.md#adr-067节点模型泛化落地单板形态强制契约-v03角速率改走独立指向流) |
| 2026-09-29 | 前序会话 | Zero 3W 上跑通无头 Host，以 mock LLM 验一个角色回合。 | [P0-B 历史实测](../test-worksheets/runs/2026-09-29-zero3w-host-headless-bringup.md) |

## 7. 证据入口

| 想核对什么 | 从哪里读 |
| --- | --- |
| 软件当前源码与回归 | [Rust 契约](../../crates/ailive-gun-spirit-contracts/src/lib.rs) · [感知内核](../../crates/ailive-gun-spirit-perception-core/src/lib.rs) · [桌面模拟器](../../crates/ailive-gun-spirit-host/src/bin/simulator.rs) · [9 帧回归](../../crates/ailive-gun-spirit-host/tests/golden_frames.rs) |
| 板卡身份、点屏和图库 | [到货与实测](../test-worksheets/01-硬件到货清单.md) · [固件构建和验收](../../firmware/jc3248w535/README.md)。16 MB Flash / 8 MB PSRAM，图库 1,537,328 B，应用槽 2 MiB。 |
| 接线和 Flash 备份结论 | [原厂接口核对](../HARDWARE_IMPLEMENTATION_PLAN.md#011-原厂手册核对2026-11-09) · [固件与分区约束](../HARDWARE_IMPLEMENTATION_PLAN.md#013-p0-a-固件与分区约束2026-11-13-评审补记)。`ffat` 的空白结论来自 11 月 13 日备份，素材迁移待容量结果。 |
| 摆位、握持与电池 | [机械布局](../MECHANICAL_INTERFACE.md#p0-a悍迪-191-原型布置候选) · [传感器设计](../SENSOR_OPTIONS.md#p0-a单板传感器设计候选) · [供电](../MAIN_POWER.md#p0-a晶彩一体板供电候选) · [试验表](../test-worksheets/P0-A-晶彩板屏台架与装配.md) |

## 8. 项目边界和实物条件

- 载体只做可拆外置配件，保持与内部火控、扳机机构、供弹机构和载体供电隔离。感知只表达观察事实。外部辅助触点及主机侧娱乐遵循既有范围。 [项目边界](../PROJECT_BOUNDARIES.md)
- 实物 P5 针位、U2 充电规格与电芯匹配核实前，开发期走单路 USB-C，P5 空置。电芯要求自带 PCM，充电需有人看守。GPIO5 电压饱和需要标定和实测。 [供电设计](../MAIN_POWER.md#p0-a晶彩一体板供电候选)
- 拟接线、编译结果、桌面回放和实机观察要分别注明证据。整片出厂备份、设备 MAC 和本地网络信息留在本机；外部评审提供所需的脱敏日志、示意或源码。 [实测记录规则](../test-worksheets/README.md)

## 9. 需要所有者现场补的证据

| 编号 | 需要记录 | 对应检查点 |
| --- | --- | --- |
| E1 | U2 顶标近照或厂商料号与充电规格。 | CP4 内置电池路线 |
| E2 | P5 的板端 GND 针位、插座朝向、插头实际极性。 | CP4 内置电池路线 |
| E3 | 悍迪 191 左右侧与顶部带尺照片，活动件净空、戴手套时后握接触区及候选线槽路径。 | CP4 假体 |
| E4 | 板屏原壳长宽厚、带接头包络和重量、两处候选电池空间。 | CP4 假体与电池 |
| E5 | 图库三次按键的现场画面及对应串口回执。 | CP3 图库 |

## 10. 会话分工建议

| 参与者 | 下一轮工作 |
| --- | --- |
| DeepSeek | 先推进 CP1，再提交 CP1b、CP2 的实现结果或待选方案。按 §11 回填改动与证据。 |
| GPT 或下一会话 | 接续硬件资料、台架和装配记录，复核 DeepSeek 的改动与门禁。 |
| 项目所有者 / 现场 | 完成 CP3 图库点触与 CP4 实物测量，确认需裁决的事项。 |

分工是当前接续建议。新会话开始时注明实际负责项和所在分支；并行编辑同一区域时先核对现有 diff。

## 11. 会话交回模板

```text
时间（北京时间）／负责会话：
检查点与状态（待开始／进行中／待实机／通过／阻塞）：
仓库、分支、起始提交、当前提交或未提交改动：
改动文件与已落地结果：
验证命令、环境、退出码与证据路径（远端会话附脱敏摘要）：
未测事项／失败原因／需要所有者决定或操作：
下一步精确动作：
```

给 DeepSeek 的开场可以直接复制「请按 §1 对齐当前版本，先推进 §5 的 CP1 桌面对拍。完成后按 §11 回填实际改动、验证结果、证据和下一步。缺少本机源码或实物数据时，列出所需的最小文件或现场记录。」项目历史与早期讨论见 [11 月 13 日一次性交接稿](2026-11-13-p0a-deepseek.md)。
