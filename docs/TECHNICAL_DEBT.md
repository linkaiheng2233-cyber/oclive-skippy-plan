# A.I.Live-ai枪娘器灵技术债与审查台账

**SSOT 范围**：本文件只记录可复现的工程风险、技术债、门禁缺口与关闭条件；不复制路线或硬件参数。  
**最后更新**：2026-09-22  
**状态**：Current  

## 1. 审查基线

本轮审查基于 `HEAD f3c5642` 加未提交工作树。由于工作树在审查前已经包含大量项目变更，本地结果不能绑定为 `f3c5642` 的发布证据；合并/提交后应在冻结 SHA 上重跑 `scripts/verify.ps1`。

初始命令事实：

| 检查 | 初始结果 | 处置 |
|------|----------|------|
| `cargo test --workspace --all-targets --locked` | PASS，14 项测试 | 保留并扩展 |
| `cargo test --workspace --doc --locked` | PASS，当前无 doctest 示例 | 纳入完整门禁 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | FAIL，fixture 有 `single_match` | 已修复并纳入脚本 |
| `cargo fmt --all -- --check` | FAIL，越界命中兄弟 OCLive 的格式差异 | 改为三个 manifest 逐包检查 |
| Schema `--check` | PASS | 保留 |
| `cargo audit` | PASS，298 个依赖、0 个漏洞级命中 | 保留；锁文件变化后必跑 |
| `cargo metadata --no-deps` | 声称三 crate，实际四个 workspace member | `vendor` 已显式 exclude |

**2026-09-22 更新（`GS-P0-BL-2026-09-22` 基线门禁）**：已在**干净工作树、绑定提交 `55d10e6`** 上重跑 `./scripts/verify.ps1`，全部步骤 PASS——三个 manifest 格式、workspace/依赖边界（3 members，依赖 allowlist 匹配）、严格 Clippy、**30 项 all-targets 测试**（含 100 轮生命周期门；比上轮记录多 1 项）、doctest、Schema 零漂移、`cargo audit`（298 依赖 / 0 漏洞级命中，1 条允许的 `chacha20 0.10.1` yanked 警告）。**这解除了此前"脏工作树、未绑定提交 SHA"的证据限制**，是本仓第一条绑定冻结提交的完整门禁记录。

历史限制说明（保留）：上轮结论基于 `HEAD f3c5642` 加未提交工作树，只能证明当前工作树，不能冒充该 SHA 或远端 CI 证据。表格中的 14 项是那一轮工程审查开始时的历史基线。

## 2. 当前台账

| ID | 优先级 | 状态 | 可复现事实 | 完成条件 |
|----|--------|------|------------|----------|
| GS-CONTRACT-001 | P1 | Done · local verified | Replay 原先用完整结构体相等判断同 revision 重发，`produced_at_monotonic_ms`或同值新Observation带来的证据时效刷新会误报`SameRevisionChanged`，与ADR-051冲突 | 比较分类值/Unknown原因/health/identity/revision；接受并保存同revision证据包络刷新但不发边沿；语义漂移仍拒绝且有测试 |
| GS-CONTRACT-002 | P1 | Done · local verified | `PerceptionReplay::push` 原先不执行 `ContractValidate`，Rust 内可构造跨字段非法状态并建立 baseline | 入门先校验；非法状态不覆盖 replay；测试锁定 |
| GS-CONTRACT-003 | P2 | Done · local verified | 四个 ID newtype 的内部 `String` 公开且构造器不校验，非法 ID 可进入生产路径后才被额外校验发现 | 字段私有；受校验构造/反序列化；Schema 长度约束不漂移 |
| GS-BOUNDARY-001 | P1 | Done · local verified | Cargo 自动把 `vendor/oclive_monolith_builtin` 纳入 workspace，实际成员数为四，与架构 SSOT 冲突 | 显式 exclude；门禁断言恰好三个成员与依赖 allowlist |
| GS-QUALITY-001 | P1 | Done · local verified | 基础验证未运行 Clippy/doctest/audit，且 `cargo fmt --all` 会越界检查兄弟仓 | `scripts/verify.ps1` 固化逐包 fmt、边界、Clippy、测试、doctest、Schema、audit |
| GS-DOC-001 | P1 | Done · static verified | 多份现行专题文档仍把 Lyra+DSI、单 Grip Node、CR2032 写成 P0，与 ADR-022/028 及硬件 SSOT 冲突 | 专题已收敛为 Orange Pi+HDMI、双节点、受保护 1S LiPo；旧 ADR/ARMv7证据明确为历史或回退 |
| GS-CI-001 | P1 | OPEN · needs repository decision（已有可行候选） | Host 通过 `../oclivenewnew` 路径依赖 OCLive；独立远端 checkout 无法复现，直接加普通 CI 会失败。**2026-09-18 更新**：兄弟仓已在 GitHub（`linkaiheng2233-cyber/oclivenewnew`，本机 HEAD `c6a56bb0`），因此"钉住 rev 的 git 依赖"已从纸面变成可执行选项 | 在（a）`git = "https://github.com/linkaiheng2233-cyber/oclivenewnew.git", rev = "<冻结SHA>"`、（b）submodule、（c）同工作流双 checkout、（d）受控 vendor 中选一条并记录 ADR；随后绑定冻结 SHA 的 CI。选择前不创建 Actions workflow |
| GS-CROSS-001 | P1 | Done · cross-compiled | workspace迁移后已用Arm GNU 14.2.Rel1生成22.19 MiB ELF32 ARM EABI5 hard-float Host，动态依赖为glibc基础四项 | 产物/命令/SHA-256已写入`ARMV7_VALIDATION.md`；仍明确不等同Lyra实机通过 |
| GS-MAINT-001 | P2 | Observe · partially improved | contracts `lib.rs` 约600行；新增reducer后已把profile与测试拆到子模块，reducer生产文件约870行但仍按ingest→resolve→render单向排列 | NodeHello/codec加入前先按实际导航证据决定是否再拆session/fusion；任何拆分必须零语义并跑完整门禁 |
| GS-OCLIVE-001 | P1 | Done · cross-repo verified | 兄弟OCLive已增加兼容`TurnOrigin`并由内部推导副作用策略；sensor/system不写用户聊天、长期记忆、好感、关系、人格、连续性、虚拟时间或用户情绪。本仓`OcliveSensorAdapter`只通过公开类型化入口投递有界中性上下文 | 跨仓测试已验证sensor零提交后普通user回合仍正常写入；本仓adapter单测验证中性机器可读上下文。默认CLI保留desktop fixture不影响该债关闭 |
| GS-CAL-001 | P1 | OPEN · needs measured calibration | 桌面reducer使用profile提供的raised/lowered重力anchor做最近点分类，只足以验证数据流；尚无实机过渡区、迟滞、稳定时间和安装transform数据 | XIAO/IMU到货后按校准动作集记录MEASURED样本，冻结transform/阈值/迟滞/稳定窗，补边界与抖动回放测试；此前不得宣传姿态准确率 |
| GS-REMOTE-001 | P2 | IN PROGRESS · main + 基线 tag 已推送，分支保护未启用 | **2026-09-18**：GitHub 远端已建立（`linkaiheng2233-cyber/oclive-skippy-plan`，**公开/开源**），本机已通过代理打通 `github.com:443`。**2026-09-22 实测**：`main` 已快进推送至 `0df2361`（`66ae211..0df2361`），注释 tag `baseline/GS-P0-BL-2026-09-22`（tag 对象 `20c8915` → commit `0df2361`）已推送，旧基线 tag 完好；`gh api .../branches/main/protection` 返回 **404 `Branch not protected`**，即**分支保护尚未启用** | 剩余条件只剩**启用分支保护**（required status checks 待 GS-CI-001 冻结后再定，可先加禁止强推/删除）；CI 仍须等依赖获取策略冻结，**此前不声称"CI 已绿"** |
| GS-HW-001 | P0 | OPEN · needs Zero 3W hardware | 主板已由Zero 2W 2GB切换为Zero 3W 6GB/A733，但USB-C DP主动转HDMI、厂商BSP、GPU/NPU、板载风扇、电源峰值和3B/1.5–1.7B/0.5–0.6B质量速度均无本项目MEASURED证据 | 完成OPZ-D01、T01、P01、LM01–LM05；保存板卡/镜像/转接/每档模型身份和原始日志，再决定本地模型分流、训练、RAG、电源、外壳风道与驱动能力 |
| GS-HW-002 | P0 | **Done · measured（2026-09-23 复测通过）** | 首轮bring-up证明系统/网络/远程通道可用，但**视频链未通过**：随机附带的Mini HDMI↔HDMI线DDC通道间歇失效——接已知良好显示器同样EDID读0字节、无模式、无`/dev/fb0`；HPD可连通但反复跳变。原线已退货换品牌线 | **已关闭**：换用绿联线后四项判据全部通过——HPD 120 秒稳定`connected`（变化 1 次）、EDID `256` 字节、模式列表 5 个（含 `480x800`/`800x480`）、`enabled` + `/dev/fb0`（fbcon 已绑定）。证据见`test-worksheets/runs/2026-09-23-zero3w-display-link-retest.md` |
| GS-HW-005 | **P1** | OPEN · panel incompatible, replacement decided | 当前 **3.2 英寸**（所有者 2026-09-23 更正尺寸；此前误记为 3.5 英寸）480×800 HDMI 屏模块（EDID 名 `HDMI480x800HH`，克隆 EDID；品牌/控制板型号待核，见 `GS-HW-004`）**在板端不出图**：板端 HPD/EDID/模式/CRTC/平面/fb0 全部正常，屏在 PC 上可正常显示，驱动以 `VIC 0 + hdmi14 vsif` 发送非 CEA 模式 → 判定为**屏侧 HDMI 兼容性**（见 ADR-064）。另：面板原生为竖屏，横向 800×480 需 `fbcon=rotate`/renderer 旋转 | ⓪ 免费试 DVI 信令（命令行 `video=<conn>:480x800@60D`，跳过信息帧，2 分钟；**提案·未实测**，判读见交接 §2.6——须比对 dmesg 的 `vsif` 行是否变化，否则结果无效）；① 先验证 `USB-C DP Alt → 主动式 DP→HDMI` 路线（基线原定路线，30–80 元）；② 换"能作为通用 HDMI 接收端工作、EDID 为标准 EDID"的屏，通过四项判据 + 横向落地；③ 或改用 SPI 小屏由 MCU 直驱。运行记录见 `test-worksheets/runs/2026-09-23-zero3w-panel-no-image.md` |
| GS-HW-006 | P2 | OPEN · vendor driver limits block display debugging | 本厂商驱动的两个限制：① `modetest -s <conn>@<crtc>:<mode>` 会导致 SSH 断开并让板子崩溃重启（实测两次）；② 内核 `drm_kms_helper` 无 `edid_firmware` 参数、EDID 覆盖功能未编入，`video=` 也不覆盖 EDID 首选模式 | 显示调试改为"只通过开机命令行生效的模式策略"；把这两条限制写入交接文档踩坑清单（已完成）；若后续需要运行期换模式，改用重新引导或 renderer 侧旋转 |
| GS-HW-003 | P1 | OPEN · touch capability unverified | 当前屏（**3.2 英寸** IPS LCD 模块，所有者 2026-09-23 更正）只有 HDMI 与 USB-C（标注 `only power`）两个接口，**很可能不提供触摸**；`OPZ-D02` 无输入通道可测，直接影响 P0 的触摸交互门与屏幕选型。**已撤回的推断**：此前拿"微雪 3.5 英寸 480×800 触摸款"做对比，得出"疑似非原厂件"——该推断建立在错误的型号前提上，**撤回**。仍成立的事实：本件 USB-C 只标 `only power`；其 EDID 为克隆件（厂商码 `LEN`，见 `GS-HW-005`），但这在廉价 HDMI 驱动板上常见，不能单独作为"非原厂"证据 | 确认该屏是否支持触摸（问卖家 + 实测）；不支持则按 `DISPLAY_SELECTION.md` 更换为带 USB 触摸的屏，并复测 `OPZ-D02` 裸手/手套/边缘误触。**处置口径（所有者 2026-09-23 决定）**：先等 `USB-C DP→HDMI` 转接头验证，**若该路径能点亮则不追究退货**；只有确认不可用且仍在退换期时才发起退货 |
| GS-HW-004 | P2 | OPEN · board identity incomplete | 首轮只核验到主机名、OS、内核、设备树与根分区UUID；**RAM容量（报告6GB）、microSD料号、屏幕型号、环境温度均未在系统内核验**，运行记录中保持`UNKNOWN` | 下一轮采集`free -m`、`lscpu`、`lsusb -t`、`df -h`、`systemd-analyze`、本轮warning/error清单，回填`ORANGE_PI_BRINGUP_WORKSHEET.md`§2与§4 |
| GS-DEV-001 | P2 | OPEN · toolchain not yet reproducible | 本轮为了建立远程通道与自诊断，对工作镜像做了4处修改（netplan WiFi、authorized_keys、opi-diag服务、armbianEnv参数）并重新拼接分区烧录；该流程目前只存在于会话记录与脚本中，未固化为可复现步骤 | 把镜像改造与远程调试流程写成受版本控制的文档与脚本（含Cygwin debugfs读写ext4、分区拼接、便携OpenSSH客户端版本要求、密钥管理），并在新镜像上做一次端到端复现验证 |
| GS-DOC-002 | P2 | **Done · local verified**（2026-09-22） | 「重新导出」已固化为**可重放的单一脚本**：`scripts/export-docs.ps1` + 数据文件 `scripts/export-map.tsv`（22 份映射，含目标根与清单文件名元数据）。脚本做逐份 SHA-256 比对、提供 `-Check` 干跑模式、并自动重写 `导出说明.md` 的清单块。**已实际重放并做漂移注入测试**：人为改坏 `00-从这里开始.md` 后 `-Check` 报 `STALE` 且退出码 1，正式运行修复并复检归零（`mapped=22 stale=0`）。此前 09-18 的「已导出一次」按本条自身的关闭条件不算关闭 | 已满足。**后续仓库 `docs/` 变更后重跑脚本即可**；若将来要自动化，在提交钩子或 CI 中调用 `-Check` 模式 |
| GS-QUALITY-002 | P2 | OPEN · 脚本依赖本机 PowerShell 环境 | **症状一（编码，2026-09-22 实测）**：默认 Windows 控制台（代码页 **936/GBK**、`$OutputEncoding=20127`）下调用 `scripts/verify.ps1`，会在第 4 步 `check-workspace-boundaries.ps1` 的 `$metadataJson \| ConvertFrom-Json` 处抛 `ArgumentException` 并整体退出码 1。根因是 `cargo metadata` 输出 UTF-8，中文仓库路径被按 GBK 解码成非法 JSON。加 `[Console]::OutputEncoding=UTF8` 后立即 PASS 且完整门禁 PASS。**症状二（模块路径，同日实测）**：本机 `PSModulePath` 含 PowerShell 7 模块目录且排在 PS5.1 自身 `v1.0\Modules` 之前，导致 Windows PowerShell 5.1 下 **12 个常用 cmdlet 中唯独 `Get-FileHash` 不可解析**（`New-Object`/`Get-Date`/`Select-Object` 等其余 11 个正常）。`export-docs.ps1` 已改用 .NET SHA256 规避。**附带约束**：PS5.1 把无 BOM 的 `.ps1` 按 ANSI 解码，故本仓 `.ps1` 一律保持 ASCII-only（4 个既有脚本 0 汉字），CJK 只放在以显式 UTF-8 读取的数据文件里（如 `export-map.tsv`） | 让涉及编码或哈希的脚本**不依赖本机环境状态**：控制台编码脚本内自固定、哈希走 .NET、CJK 走数据文件、`.ps1` 保持 ASCII-only。关闭条件：新增本节的复现说明，并在非 UTF-8 控制台与 PS5.1 下各留一次通过记录。**在此之前，门禁结论必须注明运行控制台与 PowerShell 版本** |
| GS-ARCH-001 | **P1** | OPEN · **方向已冻结为 `ADR-065`；代码未动**，落地以 ADR-063 形态裁决为前置（2026-09-23 静态核验） | **感知内核在结构上无法表达「一块板承担全部感知」**（ADR-063 / `ESP32_GUN_ASSISTANT.md` 的单 ESP32 形态会直接撞上）：① `PerceptionProfile::try_new`（`reducer/profile.rs:113`）**强制两个不同** `NodeUid`，相同即 `ProfileError::DuplicateNodeBinding`；② `NodeRole` 是 `reducer.rs:51` 的**私有两值枚举**，`node_role()` 按 UID 硬分派；③ `(role, observation)` 绑定表是**白名单**（`reducer.rs:495–522`），越界组合返回 `CapabilityNotBound`——因此一块节点**不能同时**上报前握接触 + 后握接触 + 姿态估计 + 扳机触点。附带缺口：**表达侧（下行）类型一个都还没有**——`AssetRef`、表达指令、模式（自由/预设）、能力协商/hello 均只在文档（`64A-R1`）而未进 `contracts` Rust 真源 | 把「两个固定角色」泛化为**「节点 → 能力集映射」**（**已于 `ADR-065` 冻结方向与三条硬约束**），reducer 按能力分派并同时落地能力协商（推荐）；不采用「单节点冒充两个 `node_uid`」（伪造拓扑、破坏诊断）。**不需要硬件**，可在桌面用模拟器完成（约 1–2 天，`ESTIMATED`）；关闭条件：新 ADR + `contracts` 增补能力协商与表达侧类型 + reducer 泛化 + 回放/降级测试与 fixtures 同步，并跑完整 `./scripts/verify.ps1` |

## 3. 不作为技术债的未实现项

以下是正常路线缺口，不应包装成“代码质量问题”：NodeHello/BLE codec、伴随包完整Input Scheduler门控、把真实OCLive adapter接入可操作桌面Host流程、RendererPort/PNG或Web UI、SQLite状态库、systemd与实机驱动。Observation reducer、最小sensor调度缝、类型化OCLive窄适配器、Output Arbiter和JSON ViewModel已实现；后续只有违反既定契约或长期无验收路径时才转入本台账。

## 4. 需实机而不能在本轮关闭

功耗/峰值、内存/温度、屏幕亮度与背光控制、触摸、Wi-Fi/BLE 共存、节点续航与 brownout、FSR 阈值、IMU 坐标/校准、三轴力矩/线缆寿命、整枪重量/重心和场地误判，只能由测试工作表产生 `MEASURED` 证据。本轮可以审查测试设计，不能把估算改名为通过。

## 5. 维护规则

- 新债必须给出文件/命令证据、能力链影响与可判定关闭条件；印象只记为待查，不直接标 P0/P1。
- 每个 ID 只有一条权威状态。验证历史保留在该行或版本控制，不复制第二张状态表。
- `Done · local verified` 不等于远端/实机验证。提交后绑定 SHA；硬件项再绑定实物 revision 和原始记录。
