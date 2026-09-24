# A.I.Live-ai枪娘器灵文档入口

**状态**：CURRENT  
**最后核对**：2026-09-22  
**职责**：这是本仓唯一的文档导航入口。它说明先读什么、每类事实在哪里维护，以及哪些文件只是历史或上游参考。

## 1. 新参与者阅读顺序

**先按身份选入口**（沿用 OCLive 的阅读习惯）：

| 我是谁 | 第一篇 | 接着读 |
|---|---|---|
| 项目所有者（做决策） | `PROJECT_BASELINE.md` | `PROJECT_BOUNDARIES.md` · `ROADMAP.md` |
| 硬件现场执行者（第一次动手） | `HARDWARE_ENGINEERING_GUIDE.md` | `test-worksheets/README.md` · `ORANGE_PI_BRINGUP_WORKSHEET.md` |
| 工程 / 软件协作者 | `ARCHITECTURE.md` | `DEVELOPMENT_DISCIPLINE.md` · `schemas/README.md` |
| AI 或后续接手者 | `handoff/README.md` | 本文件§2 治理真源 · `TECHNICAL_DEBT.md` |

**完整阅读顺序**（不区分身份时）：

1. `PROJECT_BASELINE.md`：当前从什么硬件和软件状态开始，什么才算P0完成。
2. `PROJECT_BOUNDARIES.md`：四季宝、OCLive、角色内容和未来扩展各自负责什么。
3. `ROADMAP.md`：当前阶段、下一步和验收门。
4. `HARDWARE_ENGINEERING_GUIDE.md`：第一次做硬件时怎样安全、可重复地学习和调试。
5. `test-worksheets/README.md`：实物到货后的记录顺序。

发生冲突时不要按文件名或篇幅猜测，使用`DOCUMENTATION_SYSTEM.md`定义的权威顺序。

## 2. 治理与项目级真源

| 文档 | 唯一职责 |
|---|---|
| `PROJECT_BASELINE.md` | 当前产品、硬件、软件、证据和进入硬件开发的共同起点 |
| `PROJECT_BOUNDARIES.md` | In scope / Deferred / Out of scope及跨仓所有权 |
| `DOCUMENTATION_SYSTEM.md` | 文档分级、状态、冲突处理、变更联动和归档规则 |
| `DECISIONS.md` | 已作出的架构决策及其原因；不得把新决定只写进聊天或总览 |
| `DEVELOPMENT_DISCIPLINE.md` | 代码、契约、硬件、验证和交付纪律 |
| `TECHNICAL_DEBT.md` | 活跃风险、未知项和可验证的关闭条件 |

## 3. 产品与路线

| 文档 | 用途 |
|---|---|
| `ROADMAP.md` | S0–S6路线和G0–G5阶段门 |
| `PROTOTYPE_KIT.md` | kit-first原型边界及从桌面到上枪的顺序 |
| `UX_RESEARCH.md` | 只复核既有体验决定的轻量玩家问题 |
| `FRONTEND_DESIGN_BRIEF.md` | 800×480界面信息架构与首批状态稿 |
| `handoff/README.md` | 人类总览、AI交接文档及跨会话入口 |
| `ESP32_GUN_ASSISTANT.md` | **枪上轻量助手形态**（枪上只留一块 ESP32-S3 + 小屏、自由/预设双模式、上下行契约边界、预设触发范围、BOM 与重量）；随 ADR-063 的 PC 主机形态，状态 `CANDIDATE` |
| `PC_SIMULATION_BACKENDS.md` | **主机侧娱乐后端**（屏幕打靶 / 投影靶场 / 计分玩法、瞄准通道选型、延迟归属、安全与文案红线）；随 ADR-066，状态 `CANDIDATE` |

## 4. 架构、契约与软件

| 文档/目录 | 用途 |
|---|---|
| `ARCHITECTURE.md` | 感知内核、Host、OCLive和Output Arbiter职责与信息流 |
| `schemas/README.md`及仓库`schemas/` | Rust生成的线协议Schema和正反fixtures |
| `LINUX_ARM64.md` | Zero 3W目标Linux镜像、服务和板端bring-up |
| `LOCAL_3B_INFERENCE.md` | 3B及以下模型、角色上下文、速度/质量与训练进入条件 |
| `CONFIG_REFERENCE.md` | 生成器/Host配置参考，不定义产品范围 |

## 5. 硬件领域真源

| 文档 | 唯一职责 |
|---|---|
| `HARDWARE_IMPLEMENTATION_PLAN.md` | 信息、能源、接口、故障、机械五条链和当前综合BOM假设 |
| `ORANGE_PI_BRINGUP_WORKSHEET.md` | Zero 3W板卡、显示、资源、模型和热的原始实测入口 |
| `DISPLAY_SELECTION.md` | 屏幕与显示接口选择 |
| `DISPLAY_ASSEMBLY.md` | 三轴屏幕舱、线缆、受力和维护结构 |
| `TERMINAL_ENCLOSURE_VISUAL_V0_1.md` | 终端外观与层叠方向；不替代电气/热真源 |
| `MAIN_POWER.md` | 主机舱电源、充放电和关机能源链 |
| `POWER_BUDGET.md` | 前后BLE节点电池与功耗 |
| `SENSOR_OPTIONS.md` | FSR、IMU等传感器候选和选择依据 |
| `WIRELESS_SENSOR_NETWORK.md` | BLE节点拓扑、协议与降级 |
| `MECHANICAL_INTERFACE.md` | 通用导轨底座和载体机械接口 |
| `MAGAZINE_AMMO_ESTIMATION.md` | P1+只读余弹扩展；不属于当前P0 |

## 6. 验证与原始证据

- `test-worksheets/`保存到货、板屏、节点、电源、机械和体验测试表。
- `test-worksheets/runs/`保存**每次实机运行**的规范记录（运行身份、`MEASURED`/`UNKNOWN` 分级、判定、证据位置、下一步）。首轮：`test-worksheets/runs/2026-09-11-zero3w-first-bringup.md`。
- 测试照片、串口、功耗和温度原始数据进入每次测试的证据目录，不写进ADR正文。
- `DATASHEET`、`ESTIMATED`、`MEASURED`、`DECIDED`和`UNKNOWN`不得混用。
- 一次测试不能覆盖旧记录；新建run/日期并引用硬件批次、软件revision和仪器。

## 7. 工具、上游参考与历史

以下文件不定义四季宝产品需求：

- `BLUEPRINT_V2_POINTER.md`：OCLive角色蓝图的上游指针。
- `DEBUG_REFERENCE.md`、`PIPELINE_CUSTOM.md`：生成器/内核调试参考。
- `WELD_BENCH_REPORT.md`及英文版：无头Monolith性能模板。
- `ARMV7_VALIDATION.md`：旧Lyra/ARMv7回退路线的历史与兼容证据。
- `history/EXTERNAL_DOCUMENT_AUDIT_2026-09-04.md`：桌面个人文档与仓库SSOT的归并审计。
- `history/EXTERNAL_DOCUMENT_AUDIT_2026-09-18.md`：bring-up首轮后的新增扫描（测试包导出件复核与刷新、个人文档比对、新导入项、排除项与工件位置）。
- `history/EXTERNAL_DOCUMENT_AUDIT_2026-09-22.md`：导出件漂移复核（8 份重新导出、桌面独有采购价格归并入 `test-worksheets/采购核对清单.md`、排除项与后续动作）。

这些文件若与项目基线、ADR或领域SSOT冲突，不具有覆盖权。
