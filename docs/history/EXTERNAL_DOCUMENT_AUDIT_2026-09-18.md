# 外部四季宝相关文档归并审计（2026-09-18）

**状态**：HISTORICAL AUDIT  
**扫描位置**：`C:\Users\13603\Desktop\A.I.Live-ai枪娘器灵-硬件到货测试包`、`C:\Users\13603\Desktop\Zero3W系统镜像`、`C:\Users\13603\Desktop\个人文档`、桌面根目录、`E:\OCLive` 周边目录  
**规则**：不删除桌面文件；仓库已有相同或更新 SSOT 时不制造第二份可编辑副本；哈希用于确认本次扫描对象，不代表内容真实性。  
**前置**：`EXTERNAL_DOCUMENT_AUDIT_2026-09-04.md`（已处理当时桌面 `个人文档` 的七个文件）；本文是 2026-09-11 首轮 bring-up 之后、于 2026-09-18 执行的新增扫描。

## 1. 归并结论概览

| 类别 | 数量 | 处理 |
|---|---:|---|
| 与仓库逐字节一致（无需复制） | 9 | 记录 canonical 路径 + 哈希，桌面副本仅作填写/导出件 |
| 仓库版本更新（外部副本过期） | 11 | 外部副本标记 `SUPERSEDED`，仓库版本继续维护 |
| 仓库不存在的四季宝独有内容 | 1 | **已导入**（见 §3） |
| 属于其它项目或跨越禁止边界 | 1 | `EXCLUDED`，不进入 backlog |
| 非文档工件（镜像/密钥/脚本） | — | 见 §5，不进入文档体系 |

## 2. 逐文件比对（外部副本 ↔ 仓库 SSOT）

### 2.1 硬件到货测试包（`Desktop\A.I.Live-ai枪娘器灵-硬件到货测试包`）

| 外部文件 | 外部 KB | 仓库 SSOT | 仓库 KB | 结论 | 外部副本 SHA-256 前缀 |
|---|---:|---|---:|---|---|
| `00-从这里开始.md` | 3.50 | `test-worksheets/README.md` | 4.00 | 仓库更新 → 外部 `SUPERSEDED` | `18DF6C689035` |
| `01-硬件到货清单.md` | 3.20 | `test-worksheets/01-硬件到货清单.md` | 3.60 | 仓库更新 → `SUPERSEDED` | `DE8D4416E535` |
| `02-主机与屏幕测试表.md` | 2.60 | `test-worksheets/02-主机与屏幕测试表.md` | 4.30 | 仓库更新 → `SUPERSEDED` | `9AD92A418FDB` |
| `03-无线传感节点测试表.md` | 4.00 | `test-worksheets/03-…` | 4.00 | 逐字节一致 | `7CF82D0696CE` |
| `04-主机电源与关机测试表.md` | 3.00 | `test-worksheets/04-…` | 3.00 | 逐字节一致 | `D578B7F6F6B4` |
| `05-机械与场地测试表.md` | 3.40 | `test-worksheets/05-…` | 3.40 | 逐字节一致 | `060167746207` |
| `06-OrangePi详细系统测试表.md` | 16.10 | `ORANGE_PI_BRINGUP_WORKSHEET.md` | 19.80 | 仓库更新 → `SUPERSEDED` | `510DC59FDB9B` |
| `07-硬件实施计划-参考.md` | 58.60 | `HARDWARE_IMPLEMENTATION_PLAN.md` | 60.00 | 仓库更新 → `SUPERSEDED` | `C5AF67019A50` |
| `08-项目人类阅读版-参考.txt` | 64.70 | `handoff/四季宝-人类阅读版.txt` | 71.50 | 仓库更新 → `SUPERSEDED` | `0E8050627890` |
| `09-爱好者用户体验反馈表.md` | 2.00 | `test-worksheets/06-爱好者用户体验反馈表.md` | 2.00 | 逐字节一致 | `85845E5523CE` |
| `10-既有决策体验复核清单-参考.md` | 4.70 | `UX_RESEARCH.md` | 4.70 | 逐字节一致 | `E6614960A5A1` |
| `11-余弹融合与弹匣结构测试表.md` | 5.10 | `test-worksheets/07-…` | 5.10 | 逐字节一致 | `0DFD5FE421C0` |
| `12-余弹融合扩展设计-参考.md` | 13.10 | `MAGAZINE_AMMO_ESTIMATION.md` | 13.10 | 逐字节一致 | `935F6B5E4EEC` |
| `采购核对清单.md` | 8.50 | `test-worksheets/采购核对清单.md` | 8.80 | 仓库更新 → `SUPERSEDED` | `3D1966B5AE04` |
| `证据/README.md` | 1.70 | 无（现场辅助件） | — | `EXPORT_ONLY`：仅指导现场归档，不进入文档体系 | — |

**说明**：该目录是 bring-up 现场的**填写/导出包**。按既定规则，桌面副本可现场填写，完成后回填仓库；本次比对确认包内文件均为仓库 SSOT 的导出或更旧版本，**不产生新的仓库内容**，唯一新增内容见 §3。

### 2.2 桌面 `个人文档`（09-04 审计后的再次核对）

| 外部文件 | 外部 KB | 仓库 SSOT | 仓库 KB | 结论 | 外部副本 SHA-256 前缀 |
|---|---:|---|---:|---|---|
| `四季宝-人类阅读版.txt` | 64.70 | `handoff/四季宝-人类阅读版.txt` | 71.50 | 仓库更新 → `SUPERSEDED` | `0E8050627890` |
| `四季宝-智能战术配件系统-ai阅读版.txt` | 246.30 | `handoff/四季宝-智能战术配件系统-ai阅读版.txt` | 259.00 | 仓库更新 → `SUPERSEDED` | `176760E8D479` |
| `A.I.Live-ai枪娘器灵-采购清单-完整版.md` | 5.40 | `test-worksheets/采购核对清单.md`（不同结构） | 8.80 | 仓库已重构 → `SUPERSEDED` | — |
| `A.I.Live-ai枪娘器灵-技术债与审查台账.md` | 5.40 | `TECHNICAL_DEBT.md` | 7.10 | 仓库更新 → `SUPERSEDED` | `FB96C239F872` |
| `A.I.Live-ai枪娘器灵-既有决策体验复核清单.md` | 4.70 | `UX_RESEARCH.md` | 4.70 | 逐字节一致 | `E6614960A5A1` |
| `A.I.Live-ai枪娘器灵-开发纪律.md` | 9.80 | `DEVELOPMENT_DISCIPLINE.md` | 9.80 | 逐字节一致 | `508E13757B3F` |
| `A.I.Live-ai枪娘器灵-余弹融合扩展设计.md` | 13.10 | `MAGAZINE_AMMO_ESTIMATION.md` | 13.10 | 逐字节一致 | `935F6B5E4EEC` |
| `水弹智能瞄准-结构思维实验记录.md` | 43.00 | 无 | — | `EXCLUDED`（原文声明独立于本仓；自动瞄准跨越项目禁止边界） | — |

其余 `个人文档` 内容（`OCLive-*`、开发故事、开发者/用户简介、聊天导出、路演材料、PPTX、其它项目文件）属于 OCLive、个人背景或其它项目，**不导入四季宝工程仓**。

## 3. 新导入项（仓库此前不存在的四季宝独有内容）

| 外部文件 | 字节 | SHA-256 | 导入位置 | 说明 |
|---|---:|---|---|---|
| `Zero3W调试记录-2026-09-11.md` | — | — | `test-worksheets/runs/2026-09-11-zero3w-first-bringup.md` | 首轮 Zero 3W bring-up 的现场记录。已按证据角色重写为规范运行记录：补齐运行身份表、`MEASURED`/`UNKNOWN` 分级、OPZ 门对应、证据位置与下一步。外部桌面副本保留为只读导出件。 |

导入后状态：`EVIDENCE`（已由项目所有者现场数据 + 远程实测交叉确认，不再是 `UNREVIEWED_IMPORT`）。

## 4. 排除项与理由

| 内容 | 理由 |
|---|---|
| `水弹智能瞄准-结构思维实验记录.md` | 独立于四季宝的另一个项目构思；其"视觉辅助瞄准/枪管随动"跨越本仓明确禁止边界（`PROJECT_BOUNDARIES.md`）。**不进入 backlog，不建立仓库副本。** |
| OCLive 路演材料（`OCLive-路演项目计划书.md`、`OCLive路演术语与概念说明.docx`、`A.I.Live-240800483林凯恒.pptx`、架构简图 svg/png） | 属于 OCLive 项目表达与商业材料，不是四季宝工程 SSOT。 |
| `gpt6的内核工作记录.txt`、`oclive的近期讨论.txt`、`codex对话关于oclive.txt`、`OCLive-工作交接-对接GPT6.md`、`Claude Code探索与OCLive启发…` | OCLive 内核梳理与聊天导出，归 OCLive 仓或个人档案。 |
| 个人材料（`开发者简介.txt`、`用户简介.txt`、`开发故事.txt`、`接单需求统计…`、`情绪引擎方案A对接…`、`AI我的世界（游戏）PC.md`、`deepseek人设指导.txt`、`AI聊天有感…`） | 个人/其它项目，与四季宝工程无关。 |
| 内测包目录与 zip、`ChatGPT_Image_*.png`、快捷方式 | 构建产物或素材，不是文档。 |

## 5. 非文档工件（记录位置，不进入文档体系）

**统一位置**：`E:\OCLive\oclive-四季宝-artifacts\`（仓库外；按 `DEVELOPMENT_DISCIPLINE.md`§9，二进制与证据大文件不入仓）

| 工件 | 位置 | 处置 |
|---|---|---|
| 改造版镜像（含 WiFi/SSH/诊断） | `bringup-2026-09-11\Zero3W-已配置WiFi与诊断.img`（1784 MB，SHA-256 `238499B7…36BA0F6`） | **已从桌面迁入 E 盘工件目录**；桌面仅留快捷方式 |
| 原始镜像 + 压缩包 | 同目录 `…minimal.img`（`D4BB7C8B…077C0BCB`）与 `.img.xz`（`915054FE…9366C632`，与下载源一致） | 同上 |
| 调试脚本 / SSH 密钥 / known_hosts | `bringup-toolchain\`（21 个文件） | 已归档；工作副本仍在 `E:\WSL\` |
| 便携 OpenSSH 客户端 | `E:\WSL\openssh\` | 保留为工具链（Windows 自带 9.5 与服务端 10.0 不兼容） |
| 桌面快捷方式 | `C:\Users\13603\Desktop\Zero3W镜像与工件.lnk` → 工件目录 | 便于现场快速取镜像烧录 |

桌面 `Zero3W系统镜像` 目录已清空并删除，桌面不再存放二进制工件。工件路径与校验值同时记入 `test-worksheets/runs/2026-09-11-zero3w-first-bringup.md`§6。

## 6. 后续动作

1. 桌面副本从此只作为**导出/填写件**：修改完成后必须回填仓库，不得双向各自演化。
2. ✅ **已于 2026-09-18 完成重新导出**：桌面测试包内 `00`–`12`、`采购核对清单.md` 已按仓库现行 SSOT 刷新，另补两份现场参考 —— `13-Zero3W首轮运行记录-2026-09-11.md`（← `test-worksheets/runs/2026-09-11-zero3w-first-bringup.md`）与 `14-硬件工程入门-参考.md`（← `HARDWARE_ENGINEERING_GUIDE.md`）。刷新前包内 7 份旧版本（含刚回填实测数据的 `06`）已全部替换，导出动作与源路径记录见包内 `导出说明.md`。
3. 桌面 `Zero3W系统镜像`（≈3.7 GB 工件）建议迁出桌面到工件目录；是否迁移待项目所有者决定。
4. 桌面 `Zero3W调试记录-2026-09-11.md` 已在文件头标注 `SUPERSEDED`，指向仓库版运行记录；内容不再单独维护。
5. 本轮新导入的运行记录已驱动 `PROJECT_BASELINE.md`、`ORANGE_PI_BRINGUP_WORKSHEET.md`、`TECHNICAL_DEBT.md` 的联动更新。
