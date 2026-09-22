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

最新收尾已在当前工作树运行 `./scripts/verify.ps1`，全部步骤 PASS：三个 manifest 格式、workspace/依赖边界、严格 Clippy、29 项 all-targets 测试（含100轮生命周期门）、doctest、Schema 零漂移和 `cargo audit`（298个依赖、0个漏洞级命中）。该结论仍受上方“脏工作树、未绑定提交 SHA”限制；表格中的14项只是本轮工程审查开始时的历史基线。

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
| GS-REMOTE-001 | P2 | IN PROGRESS · remote created, first push pending | **2026-09-18**：GitHub 远端已建立（`linkaiheng2233-cyber/oclive-skippy-plan`，**公开/开源**），本机已通过代理打通 `github.com:443`；首次推送与分支保护待完成 | 完成 main + 基线 tag 推送并启用分支保护；依赖获取策略（GS-CI-001）冻结后再启用 CI；此前不声称“CI 已绿” |
| GS-HW-001 | P0 | OPEN · needs Zero 3W hardware | 主板已由Zero 2W 2GB切换为Zero 3W 6GB/A733，但USB-C DP主动转HDMI、厂商BSP、GPU/NPU、板载风扇、电源峰值和3B/1.5–1.7B/0.5–0.6B质量速度均无本项目MEASURED证据 | 完成OPZ-D01、T01、P01、LM01–LM05；保存板卡/镜像/转接/每档模型身份和原始日志，再决定本地模型分流、训练、RAG、电源、外壳风道与驱动能力 |
| GS-HW-002 | P0 | OPEN · blocked by cable, re-test pending | 首轮bring-up证明系统/网络/远程通道可用，但**视频链未通过**：随机附带的Mini HDMI↔HDMI线DDC通道间歇失效——接已知良好显示器同样EDID读0字节、无模式、无`/dev/fb0`；HPD可连通但反复跳变。原线已退货换品牌线 | 换线后四项判据全部通过：HPD连续2分钟稳定`connected`、EDID非0字节、出现屏幕真实分辨率、`enabled`与`/dev/fb0`存在；随后补测USB-C DP Alt路线作为第二视频路径。原始数据见`test-worksheets/runs/2026-09-11-zero3w-first-bringup.md` |
| GS-HW-003 | P1 | OPEN · touch capability unverified | 当前3.5英寸屏只有HDMI与USB-C（标注`only power`）两个接口，**很可能不提供触摸**；`OPZ-D02`无输入通道可测，直接影响P0的触摸交互门与屏幕选型 | 确认该屏是否支持触摸；不支持则按`DISPLAY_SELECTION.md`更换为带USB触摸的屏，并复测`OPZ-D02`裸手/手套/边缘误触 |
| GS-HW-004 | P2 | OPEN · board identity incomplete | 首轮只核验到主机名、OS、内核、设备树与根分区UUID；**RAM容量（报告6GB）、microSD料号、屏幕型号、环境温度均未在系统内核验**，运行记录中保持`UNKNOWN` | 下一轮采集`free -m`、`lscpu`、`lsusb -t`、`df -h`、`systemd-analyze`、本轮warning/error清单，回填`ORANGE_PI_BRINGUP_WORKSHEET.md`§2与§4 |
| GS-DEV-001 | P2 | OPEN · toolchain not yet reproducible | 本轮为了建立远程通道与自诊断，对工作镜像做了4处修改（netplan WiFi、authorized_keys、opi-diag服务、armbianEnv参数）并重新拼接分区烧录；该流程目前只存在于会话记录与脚本中，未固化为可复现步骤 | 把镜像改造与远程调试流程写成受版本控制的文档与脚本（含Cygwin debugfs读写ext4、分区拼接、便携OpenSSH客户端版本要求、密钥管理），并在新镜像上做一次端到端复现验证 |
| GS-DOC-002 | P2 | OPEN · 已刷新，但漂移是结构性的 | 09-18 审计声称「已完成重新导出」后，导出件**再次落后**：仓库源文档在导出动作之后又被修改，导出件必然随之过期。2026-09-22 复核实测 8 份过期（个人文档 5 + 到货测试包 3），已全部重新导出并逐份 SHA-256 验证一致；桌面 1 份独有采购价格已归并入 `test-worksheets/采购核对清单.md`。见 `history/EXTERNAL_DOCUMENT_AUDIT_2026-09-22.md` | 把「重新导出」固化为**可重放的单一脚本**（文件映射表 + 逐份 SHA-256 比对 + 自动刷新 `导出说明.md`），并在导出源变更后触发；**关闭条件是脚本存在且被实际重放，不是「导出过一次」** |

## 3. 不作为技术债的未实现项

以下是正常路线缺口，不应包装成“代码质量问题”：NodeHello/BLE codec、伴随包完整Input Scheduler门控、把真实OCLive adapter接入可操作桌面Host流程、RendererPort/PNG或Web UI、SQLite状态库、systemd与实机驱动。Observation reducer、最小sensor调度缝、类型化OCLive窄适配器、Output Arbiter和JSON ViewModel已实现；后续只有违反既定契约或长期无验收路径时才转入本台账。

## 4. 需实机而不能在本轮关闭

功耗/峰值、内存/温度、屏幕亮度与背光控制、触摸、Wi-Fi/BLE 共存、节点续航与 brownout、FSR 阈值、IMU 坐标/校准、三轴力矩/线缆寿命、整枪重量/重心和场地误判，只能由测试工作表产生 `MEASURED` 证据。本轮可以审查测试设计，不能把估算改名为通过。

## 5. 维护规则

- 新债必须给出文件/命令证据、能力链影响与可判定关闭条件；印象只记为待查，不直接标 P0/P1。
- 每个 ID 只有一条权威状态。验证历史保留在该行或版本控制，不复制第二张状态表。
- `Done · local verified` 不等于远端/实机验证。提交后绑定 SHA；硬件项再绑定实物 revision 和原始记录。
