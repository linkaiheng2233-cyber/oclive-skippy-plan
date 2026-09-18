# A.I.Live-ai枪娘器灵文档体系

**状态**：CURRENT  
**最后核对**：2026-09-11  
**目标**：让项目所有者、后续AI和工程协作者能找到唯一真源，区分决定、估算、实测、总览和历史。

## 1. 权威顺序

发生冲突时按以下顺序处理，但任何覆盖都要回写相关文档：

1. 安全停止条件与明确禁止边界。
2. 已接受ADR和`PROJECT_BOUNDARIES.md`。
3. 领域SSOT，例如电源、显示、传感、机械或协议文档。
4. 带完整身份和条件的最新`MEASURED`原始证据。
5. `PROJECT_BASELINE.md`与`ROADMAP.md`中的当前汇总。
6. 人类/AI交接总览。
7. 工具参考、历史草案、聊天记录和外部导出。

实测发现旧决定错误时，不能只在工作表中静默覆盖ADR；应保存证据、新增或修订ADR，再同步领域SSOT、基线和路线。

## 2. 六种文档角色

| 类别 | 回答的问题 | 示例 |
|---|---|---|
| Baseline | 我们现在从哪里开始，完成是什么 | `PROJECT_BASELINE.md` |
| Boundary/ADR | 为什么这样做，哪些事不能顺手做 | `PROJECT_BOUNDARIES.md`、`DECISIONS.md` |
| Domain SSOT | 某一领域具体怎样设计 | `MAIN_POWER.md`、`WIRELESS_SENSOR_NETWORK.md` |
| Plan/Gate | 下一步是什么，怎样通过 | `ROADMAP.md` |
| Evidence | 实际测到了什么 | `test-worksheets/`及证据目录 |
| Overview/History | 帮助理解背景，不直接覆盖当前工程 | `handoff/`、上游参考、历史审计 |

一个参数只能有一个领域SSOT。其它文档应链接它并写摘要，不复制可独立漂移的完整表格。

## 3. 状态和证据词表

- `CURRENT`：当前维护入口。
- `ACCEPTED`：已经作出并仍有效的决定。
- `CANDIDATE`：允许进入比较，尚未冻结。
- `DEFERRED`：有价值但不在当前阶段。
- `HISTORICAL`：只保存推演/兼容背景。
- `SUPERSEDED`：已由明确的新文档或ADR覆盖。
- `REPORTED_RECEIVED`：用户确认到货但工程身份未核验。
- `DATASHEET`：来自指定厂商资料。
- `ESTIMATED`：计算或经验估算。
- `MEASURED`：带条件、仪器、测点、批次和revision的实测。
- `UNKNOWN`：证据不足；不得以默认值冒充。

## 4. 新文档最小头部

新工程文档至少写明：标题、状态、最后核对日期、唯一职责、适用阶段和上游/下游链接。测试记录还必须写硬件批次、软件revision、仪器、环境和证据目录。

## 5. 变更联动

| 变更 | 必须同步 |
|---|---|
| 产品范围/禁止项 | 边界、ADR、基线、路线、README/AGENTS |
| 主板/屏幕/节点拓扑 | ADR、硬件综合计划、对应领域SSOT、采购/到货表、路线 |
| 协议字段 | Rust类型、语义校验、fixtures、生成Schema、架构、ADR |
| 实测推翻估算 | 原始工作表、技术债、领域SSOT、必要时ADR/基线 |
| 阶段完成 | ROADMAP、PROJECT_BASELINE、handoff状态摘要 |
| 只增加想法 | 先判断边界；未排期内容不得直接改P0文档 |

## 6. 目录策略

本轮采用“先建立逻辑分层、暂不批量移动”的策略。当前工作树包含大量尚未形成干净提交的代码和文档变化，立即移动全部文件会制造重命名噪声和断链风险。`docs/README.md`从现在起提供唯一导航；新文档按其类别放置，待形成可追踪基线提交后，才能以单独重构提交物理迁移目录并自动检查链接。

建议未来物理目录为：

```text
docs/
  governance/   # baseline、boundaries、ADR、discipline
  product/      # roadmap、UX、frontend、handoff
  architecture/ # architecture、Linux、model
  hardware/     # power、display、sensor、wireless、mechanical
  validation/   # worksheets与证据索引
  reference/    # 上游/生成器工具参考
  history/      # 外部旧快照审计和已取代方案
```

迁移要求：单独提交、只做路径和链接变化、不同时修改技术含义、运行全仓链接扫描与`./scripts/verify.ps1`。

**迁移状态（2026-09-18）**：**明确推迟**。当前工作树仍有大量未提交变更（三 crate 迁移、schemas、治理文档与证据记录，见`TECHNICAL_DEBT.md`§1的脏工作树限制），尚不满足“干净基线提交后再迁移”的前置条件。本轮只维护逻辑分层、`docs/README.md`单一导航与 §9 目录职责表；物理迁移待基线提交后作为**独立的纯路径提交**执行。

## 7. 外部文档归并规则

- 内容与仓库相同：记录哈希和canonical路径，不再复制。
- 仓库版本更新：外部副本标记`SUPERSEDED`，仓库版本继续维护。
- 属于OCLive：留在OCLive仓或个人文档，不导入四季宝。
- 跨越项目禁止边界：在审计中说明排除原因，不进入backlog。
- 只有仓库不存在的四季宝独有内容才导入；导入后先标`UNREVIEWED_IMPORT`，完成边界/事实审查后才能成为SSOT。

本轮归并结果见`history/EXTERNAL_DOCUMENT_AUDIT_2026-09-04.md`与`history/EXTERNAL_DOCUMENT_AUDIT_2026-09-18.md`。

## 8. 与 OCLive 文档体系的对应

OCLive 主仓的组织方式是“按身份入口 + 目录职责表 + 索引不复制内容 + 双语镜像 + archive 存历史”。四季宝沿用同一套阅读与归属习惯，但按**单产品硬件宿主**的规模裁剪：

| 维度 | OCLive 主仓 | 四季宝本仓 | 说明 |
|---|---|---|---|
| 入口 | `creator-docs/README.md`、`getting-started/DOCUMENTATION_INDEX.md` | `docs/README.md`（唯一入口） | 两边都要求先按身份选入口 |
| 身份划分 | 普通用户 / 角色包创作者 / 插件作者 / 主仓开发者 / 维护者 | 项目所有者、硬件现场执行者、工程与 AI 协作者 | 四季宝当前没有外部用户与插件作者 |
| 索引职责 | 索引只回答“去哪里读” | 同 | 不复制架构、契约、进度或测试表 |
| 目录分层 | `human-docs/`、`creator-docs/`、`handoff/`、`handoff/archive/`、`*-en/` | 逻辑分层已定（见 §6），物理迁移待干净基线提交 | 本仓暂不批量移动文件 |
| 历史 | `handoff/archive/ARCHIVE_PROJECT_HISTORY.md` | `docs/history/` | 历史与现行严格分离，历史不得作为现行真源 |
| 双语 | 关键契约 `X.md` + `X.en.md` | 暂只保留 `WELD_BENCH_REPORT.en.md` | 四季宝暂不要求全量双语；出现对外交接件时再补镜像 |

**跨仓引用规则**：四季宝需要引用 OCLive 契约时，用链接指向主仓 SSOT 并写摘要，不在本仓复制一份会漂移的正文；反之 OCLive 主仓只保留跨宿主通用的内核契约，不收录四季宝的阈值、结构、HostProfile 与枪械扩展格式。

**沿用 OCLive 的四条纪律**（与主仓含义一致）：

1. **权威顺序**先于篇幅与文件名（见 §1）。
2. **状态词表**必须显式标注，不靠上下文猜测（见 §3）。
3. **索引不复制内容**：一处参数只有一个领域 SSOT，其它文档链接并写摘要。
4. **不新建第二份总览**：找不到位置时先修正入口或扩展既有 SSOT。

## 9. 目录职责表（目标态）

物理迁移完成后，各目录的唯一职责如下（对应 §6 建议布局）：

| 目录 | 只负责 |
|---|---|
| `governance/` | 基线、边界、ADR、纪律与文档体系本身 |
| `product/` | 路线与阶段门、原型边界、体验复核、前端信息架构、交接总览 |
| `architecture/` | 感知内核/Host/OCLive 职责与信息流、Linux 目标镜像、本地模型 |
| `hardware/` | 能源、显示、传感、无线、机械、外观等领域 SSOT |
| `validation/` | 工作表、每次运行的规范记录与证据索引 |
| `reference/` | 生成器/内核工具参考、上游指针 |
| `history/` | 外部文档归并审计、已取代方案与历史快照 |

迁移执行条件与要求见 §6；迁移本身只做路径与链接变化，不同时修改技术含义。
