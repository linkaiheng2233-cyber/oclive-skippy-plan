# A.I.Live-ai枪娘器灵文档包

这个目录集中保存用于理解项目和跨会话交接的文档：

仓库slug为`oclive-四季宝器灵`，正式产品名为`A.I.Live-ai枪娘器灵`；旧“四季宝器灵 / Skippy Spirit”只作为历史代号保留。

- `四季宝-人类阅读版.txt`：面向项目所有者，说明产品是什么、为什么这样设计、当前路线和实测方法。
- `四季宝-智能战术配件系统-ai阅读版.txt`：面向后续 AI/开发者，保存更完整的背景、约束、接口、决策和接手上下文。
- `AI_SESSION_HANDOFF_2026-09-23.md`：**跨会话 AI 交接**（最新一份）。含接手第一步、显示现状（**视频链已通、当前屏在板端不出图**）与复测/分诊方法、环境与访问（代理/SSH/工件路径）、已确立硬件事实、开放项清单、纪律摘要与踩坑清单。**新会话接手请先读这一份。**

## 阅读顺序

1. 新参与者先读人类阅读版，建立产品和结构认知。
2. 执行开发或继续规划时，再读 AI 阅读版。
3. 实际实施以仓库内的工程 SSOT 为准：
   - `../PROJECT_BASELINE.md`
   - `../PROJECT_BOUNDARIES.md`
   - `../DOCUMENTATION_SYSTEM.md`
   - `../HARDWARE_IMPLEMENTATION_PLAN.md`
   - `../ARCHITECTURE.md`
   - `../ROADMAP.md`
   - `../DECISIONS.md`
   - `../DEVELOPMENT_DISCIPLINE.md`
   - `../TECHNICAL_DEBT.md`
   - `../UX_RESEARCH.md`
   - `../MAGAZINE_AMMO_ESTIMATION.md`（G5后独立的P1+只读余弹扩展）
   - `../TERMINAL_ENCLOSURE_VISUAL_V0_1.md`（终端机视觉语言、屏幕/主板、板载散热风扇与一体蜂窝后盖的工程占位）
   - `../LOCAL_3B_INFERENCE.md`（Zero 3W 6GB本地3B及以下三个尺寸档的质量、速度、角色短句、RAG/训练顺序与实测门）
   - `../../schemas/`
4. 硬件到货后从`../test-worksheets/README.md`开始填写，并配合`../ORANGE_PI_BRINGUP_WORKSHEET.md`保存实测证据；**每次实机运行的结果写在`../test-worksheets/runs/`**（首轮见`2026-09-11-zero3w-first-bringup.md`）；外部爱好者体验评审先读`../UX_RESEARCH.md`。

如果总览文档和工程 SSOT 冲突，以版本更新较晚且带有实测证据的工程 SSOT 为准，并在同一轮修改中回写两份总览文档。

## 维护规则

- 不再依赖桌面或聊天附件中的副本；仓库内文件是后续维护入口。
- 硬件数据标明 `DATASHEET`、`ESTIMATED` 或 `MEASURED`，不得把估算写成实测。
- 变更硬件拓扑、接口契约、安全边界或阶段门时，同时更新两份总览文档和对应工程 SSOT。
- 文档使用 UTF-8、无 BOM。

## 文档分责

| 文档 | 唯一职责 |
|------|----------|
| `../PROJECT_BASELINE.md` | 当前产品、硬件、软件、证据和进入硬件开发的共同起点 |
| `../PROJECT_BOUNDARIES.md` | 四季宝/OCLive/内容资产/P0/P1/项目外边界 |
| `../DOCUMENTATION_SYSTEM.md` | 文档权威顺序、状态、变更联动与外部归并规则 |
| `../ROADMAP.md` | 当前阶段、验收门与后续顺序 |
| `../ARCHITECTURE.md` | 组件职责、信息流与跨内核边界 |
| `../HARDWARE_IMPLEMENTATION_PLAN.md` | 硬件五链路、当前参数、估算与实测覆盖 |
| `../DECISIONS.md` | ADR 历史与当前裁决 |
| `../DEVELOPMENT_DISCIPLINE.md` | 开发、契约、文档、验证与安全纪律 |
| `../TECHNICAL_DEBT.md` | 可复现风险、技术债与关闭条件 |
| `../UX_RESEARCH.md` | 已冻结体验决定到wargame爱好者简短复核问题的映射与边界 |
| `../MAGAZINE_AMMO_ESTIMATION.md` | P1+弹匣余量双源融合、机械分支、未来契约与故障降级；不修改P0闭集 |
| `../TERMINAL_ENCLOSURE_VISUAL_V0_1.md` | 终端机纯视觉方向、板载散热风扇、一体蜂窝后盖、让位凹槽和待实测项；不替代热/电气SSOT |
| `../LOCAL_3B_INFERENCE.md` | 本地3B及以下尺寸档、运行边界、短角色台词、未来语音分层、RAG/训练顺序与质量/速度实测方法 |
| `../test-worksheets/` | 硬件到货后的原始实测与逐人用户体验记录入口 |
| `../test-worksheets/runs/` | 每次实机运行的规范记录：运行身份、`MEASURED`/`UNKNOWN` 分级、判定、证据位置与下一步；不替代工作表本体 |
| 本目录两份总览 | 面向人类/AI 的项目解释与交接，不替代工程 SSOT |

## 仓库状态

本目录所在仓库已经是独立本地 Git 仓库，并已配置公开远端 `origin`（`linkaiheng2233-cyber/oclive-skippy-plan`，MIT）与基线 tag；**CI 仍未启用**——需先按`../TECHNICAL_DEBT.md`的 GS-CI-001 冻结 OCLive 依赖获取方式，再绑定 CI。

截至2026-09-18，本仓已完成可重复的桌面半闭环：双节点`SensorObservation → PerceptionState → DeviceEvent → typed sensor request → desktop fixture RoleCue → Output Arbiter → ScreenViewModel`，并通过100轮生命周期测试。兄弟OCLive已增加类型化sensor回合来源和副作用隔离，本仓已有真实窄适配器；默认CLI仍保留确定性desktop fixture。

硬件侧于2026-09-11完成**首轮 bring-up**：Zero 3W 主机系统冷启动稳定（峰值≈0.8A→稳态0.36–0.41A@5V）、温度10分钟平台化（40.5–46.1°C）、WiFi（5GHz）与免密 SSH 远程通道可用、开机自诊断服务落地、显示驱动软件链验证通过（EDID曾读出、模式480×800、`/dev/fb0`建立）。**2026-09-23 复测：视频链通过**——首轮随附 Mini HDMI↔HDMI 线 DDC 间歇失效（接已知良好显示器仍 EDID=0、无模式）已由换绿联线解决，四项判据全通（HPD 稳定、EDID 256 字节、5 个模式、`/dev/fb0`），`GS-HW-002` 关闭；但**当前屏（**3.2 英寸** IPS LCD 模块，尺寸由所有者 2026-09-23 更正）在板端仍不出图**（只有一条竖线，判定为屏侧 HDMI 兼容性 `GS-HW-005`，对策见 ADR-064：先验证 USB-C DP→HDMI 主动转接，再考虑换屏或 SPI 小屏）；屏幕疑无触摸接口。逐件身份核验（RAM容量、microSD料号、屏幕型号、环境温度）与两个 XIAO 节点测试均未开始。当前禁止在身份未核验前组合上电。证据见`../test-worksheets/runs/`的三份记录（`2026-09-11-zero3w-first-bringup.md` 首轮、`2026-09-23-zero3w-display-link-retest.md` 视频链复测、`2026-09-23-zero3w-panel-no-image.md` 屏不出图），开放项见`../TECHNICAL_DEBT.md`的 GS-HW-003/004/005/006、GS-DEV-001、GS-CI-001。

本地模型仍按约3B、1.5–1.7B和0.5–0.6B三个尺寸档作为可失败角色表达候选。OCLive真实桌面接入暂缓，等待内核梳理完成；四季宝当前继续独立推进硬件基线、节点契约、显示/电源/热和本地状态链。P1+余弹融合只完成设计记录，不改变当前P0顺序。详见AI阅读版17.71–17.77、`../PROJECT_BASELINE.md`、`../ROADMAP.md` S1–S4/S6和`../TECHNICAL_DEBT.md`。
