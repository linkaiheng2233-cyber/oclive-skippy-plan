# A.I.Live-ai枪娘器灵开发纪律

**SSOT 范围**：本文件负责本仓代码、契约、文档、硬件证据、验证与交付纪律；不定义产品路线、硬件参数或 OCLive 通用内核行为。  
**最后更新**：2026-08-27  
**状态**：Current  

本纪律以兄弟仓 OCLive 的 `AGENTS.md`、`handoff/AI_CHANGE_BOUNDARIES.md`、`handoff/AI_VERIFICATION_PROTOCOL.md` 与 `handoff/RECURRING_OPTIMIZATION_PLAYBOOK.md` 为基础，按本仓“三 crate + 实体硬件”范围裁剪。继承的是防回退、单一真源、能力闭环和证据纪律，不复制 OCLive 的发行版、六槽或 Tauri 专有规则。

## 1. 四条职责基线

| 层 | 只负责 | 禁止 |
|----|--------|------|
| `ailive-gun-spirit-contracts` | 纯 DTO、版本、语义校验、生成 Schema | OCLive、BLE/Linux、文件系统、数据库、HTTP、async runtime |
| `ailive-gun-spirit-perception-core` | 确定性事实融合、状态、边沿、回放门 | 角色、Prompt、概率/冷却、OCLive、平台驱动 |
| `ailive-gun-spirit-host` | 组合根、驱动适配、OCLive bridge、Input Scheduler、Output Arbiter、生命周期 | 把硬件事实写回 OCLive 内核；让角色输出成为感知证据 |
| OCLive 兄弟仓 | 角色、记忆、关系、通用回合与表达 | 读取 ADC/FSR/IMU/BLE 原始数据；拥有四季宝硬件阈值 |

依赖只允许：

```text
perception-core → contracts
host → contracts + perception-core + OCLive public crates
```

任何功能要跨越上表，先写 ADR：说明原因、未采用的替代方案、耦合、兼容与回退；不得顺手耦合。`scripts/check-workspace-boundaries.ps1` 是当前自动门禁。

## 2. 改能力，不只改文件

动生产者、契约或宿主行为前，先写最小影响链，没有的节点明确写“无”：

```text
输入/物理量
  → producer/adapter
  → wire DTO / schema / event
  → validation / trust / permission
  → perception / host consumer
  → state / fallback / persistence
  → renderer / OCLive bridge
  → tests / replay / hardware worksheet
```

完成声明必须列出：已改节点、已核对无需改的节点、兼容/降级行为、边界两侧测试。改动文件少不能证明能力闭环已完成。

## 3. 代码纪律

1. 先用 `rg` 查已有 DTO、helper、状态和测试，再新增；同一事实只有一个实现。
2. 用封闭 `enum`、穷尽 `match`、显式 `Option/Result` 和受校验构造器让错误尽量在编译/解码阶段暴露。
3. 生产路径禁止 `.unwrap()` / `.expect()`；测试可在测试模块或测试 crate 顶部局部允许，禁止全仓放宽。
4. `unsafe` 全仓禁止。`todo!`、`unimplemented!`、`dbg!` 不进入可交付代码。
5. 同一逻辑出现第二份复制时优先抽 helper；不为“以后可能”提前增加 trait、泛型、服务或第四个核心 crate。
6. 只收敛当前触及路径的明显重复、死代码和失效注释；大重构必须有量化证据与独立验收。
7. 新增公开 DTO、trait、crate、re-export 或示例后必须跑 doctest；`cargo test --all-targets` 不能替代 `cargo test --doc`。

## 4. 契约与 Schema 纪律

- Rust 类型是 v0.2 感知契约唯一真源；`schemas/*.v0.2.schema.json` 是生成产物，禁止手改。
- 正确顺序：修改 contracts Rust 类型与语义校验 → 补有效/无效 fixture → 生成 Schema → 零漂移检查 → 修改 producer/consumer。
- 每个输入同时规定：版本、单位、范围、身份、时钟域、sequence/boot 语义、unknown/过期语义和故障行为。
- 非法 ID、越界数值和未知 variant 应在构造或反序列化阶段拒绝；跨字段不变量由 `ContractValidate` 再防一层。
- `PerceptionState.revision` 只随权威语义变化递增。同 revision 可因重连/心跳重新发布，允许发布时间变化，但其它权威 payload 必须相同；重发永不产生 `DeviceEvent`。
- Breaking 变更必须新版本化，不静默改变已承诺字段语义；未交付的 v0.1 历史草案不建立伪兼容层。

## 5. 硬件开发纪律

任何硬件变更同时检查五条链路：

| 链路 | 必答问题 |
|------|----------|
| 信息 | 物理量如何变成可验证观察、事实和事件？ |
| 能源 | 电池如何经过保护、转换、分配并安全关断？ |
| 接口 | 每段机械、电压、引脚、协议、单位和语义是什么？ |
| 故障 | 断线、低电、误判、损坏时如何进入 unknown/降级？ |
| 机械 | 力、重心、应变、线缆弯折和维护路径如何传递？ |

数据标签只有三种：

- `DATASHEET`：厂商资料，尚未验证到本装配。
- `ESTIMATED`：计算、假体或待测起点。
- `MEASURED`：绑定料号/批次、硬件与固件 revision、软件 commit、方法、环境和原始证据。

不得把网图、商品页、纸面 mAh/功耗或一次成功运行写成 `MEASURED`。实测覆盖估算时保留“旧假设 → 数据 → 新裁决 → 影响范围”，通过 ADR 修改路线。

电气审查至少做两遍：接线前按引脚/电压/极性/公共地/保护路径审图；首次上电限流、分模块、先测电源轨再接负载。锂电、充电、欠压、边充边用和机械防穿刺未验证前不进入场地 Alpha。

### 外部原型体验询问

- 当前只把`UX_RESEARCH.md`筛出的既有体验决定拿给几名 wargame 爱好者简单复核；只问本次有假体、界面或原型能展示的项目，可以口述代填，不要求评分、计时、样本分层或正式统计。
- 爱好者只补充体积/重量感、握持、屏幕、交互、安装、校准和角色反馈体验，不直接裁决电源安全、协议、BLE信任、内核职责、数据持久化或火控边界。
- 重复出现的问题优先复现，特殊握法或左手意见保留；产品化后再设计正式用户研究。

## 6. 文档与决策纪律

- 文档分责先查 `docs/handoff/README.md`；已有 SSOT 就扩展原文，不平行新建第二份真源。
- 当前路线只写在 `docs/ROADMAP.md`；架构只写在 `docs/ARCHITECTURE.md`；硬件五链路与参数只写在 `docs/HARDWARE_IMPLEMENTATION_PLAN.md`；决策只写在 `docs/DECISIONS.md`；技术债只写在 `docs/TECHNICAL_DEBT.md`。
- 既有决策到外部体验问题的映射只写在`docs/UX_RESEARCH.md`；简短记录使用`docs/test-worksheets/06-爱好者用户体验反馈表.md`。
- 候选/专题文档只保存比较方法和该主题细节，当前结论用一行链接指向 SSOT，不复制整套参数表。
- 旧 ADR 保留历史，但必须标记 `Superseded by ADR-xxx`；不得把历史段落当当前采购或实现指令。
- 新建或改变文档职责时，同轮更新 `docs/handoff/README.md` 的文档分责。
- 中文 UTF-8 无 BOM；仓库文档不用个人绝对路径作运行契约；秘密、Token、`.env`、原始隐私日志不得提交。

## 7. 测试与验证档位

### 开发切片

先跑受影响包的单测/fixture/回放。契约改动至少跑 contracts、perception-core 与 Schema 漂移；Host bridge 改动至少覆盖感知侧和 OCLive 侧边界。

### 本地完整门禁

```powershell
./scripts/verify.ps1
```

该脚本依次检查：三个包各自格式、workspace/依赖边界、Clippy `-D warnings`、all-targets、doctest、Schema 漂移与 `cargo audit`。离线或仅做快速开发时可显式使用 `-SkipAudit`，但改 `Cargo.lock`、发版或交付结论时不得跳过审计。

禁止在本仓运行 `cargo fmt --all` 作为格式门禁：路径依赖会让 rustfmt 越界检查兄弟 OCLive。必须按三个本仓 manifest 逐包格式化/检查，脚本已经固化该行为。

### 硬件门禁

无硬件时只可声称 mock/contract/replay 通过。涉及功耗、温度、射频、显示、触摸、机械、续航、恢复和场地体验的结论，必须在 `docs/test-worksheets/` 与 `docs/ORANGE_PI_BRINGUP_WORKSHEET.md` 留证。

## 8. 证据与汇报纪律

| 级别 | 可声称内容 | 必须证据 |
|------|------------|----------|
| L0 | 观察、方向、待查假设 | 明确标“未核实” |
| L1 | 单点可复现事实 | 命令/文件位置、日期、HEAD；脏工作区须注明 |
| L2 | 计数、比较、优先级 | 口径、排除规则、输出摘要、SSOT 对照 |
| L3 | 技术债/门禁/架构决策 | L2 + 影响链 + 验收与回退 |

脏工作区的本地通过只能证明“当前工作树”，不能冒充某个 commit 或远端 CI 证据。报告必须区分：已实现、已本地验证、需实机、需远端冻结 SHA。

## 9. Git、依赖与本地安全

- 不覆盖用户已有改动，不用 `git reset --hard` / `git checkout --` 清理工作区。
- 提交按单一能力闭环组织，建议 Conventional Commits；不把硬件资料缓存、构建产物、`.env`、个人绝对路径或原始诊断包提交。
- `Cargo.lock` 变化后跑 `cargo audit`；漏洞与例外若出现，必须进入 `docs/TECHNICAL_DEBT.md`，写清可达性、处置和复查日期，不能只 ignore。
- 本仓当前通过兄弟目录路径依赖 OCLive；在远端 CI/异机开发策略冻结前，不创建必然失败的 GitHub workflow。需要远端协作时先决定 OCLive 依赖如何可复现获取。
- 外部 LLM/API 只可接收经 Host 明确允许的语义上下文；原始传感流、BLE 密钥、绝对导入路径、对话/记忆与隐私日志默认不外发。

### 基线、分阶段提交与单一权威

- **唯一权威处**：本仓 `E:\OCLive\oclive-四季宝器灵` 是四季宝的唯一权威。桌面、聊天导出、`个人文档` 目录和现场填写包中的同名文件一律只是导出件或历史副本；不得在权威处之外继续演化，也不得建立第二份代码副本。发现重复副本时先归并或隔离，并在`docs/history/`留下审计记录。
- **基线**：里程碑状态打基线提交并附注释 tag，命名 `baseline/<BaselineID>`，BaselineID 与`docs/PROJECT_BASELINE.md`的`Baseline ID`一致。基线一经打上，后续任何结论都要能表达为“相对基线的差异”。
- **分阶段提交**：一次提交只承载一个可回滚的变更单元。工作区/构建迁移、契约与 Schema、感知内核、Host、门禁脚本、治理文档、硬件领域文档、实机证据各自独立提交；禁止把跨领域改动压成一个巨型提交，也不要把无关格式化混进功能提交。
- **提交前门禁**：阶段提交至少跑受影响包的窄测；基线提交、改动 `Cargo.lock`、发版或对外交付前必须跑 `./scripts/verify.ps1`。
- **不入仓的大工件**：系统镜像、原始证据包、模型权重等大文件放在仓库外的 `E:\OCLive\oclive-四季宝-artifacts\`；仓库只记录路径、SHA-256 与获取方式，不提交二进制本体。
- **提交信息**：`<type>(<scope>): <summary>`；`type` 取 `feat/fix/docs/chore/refactor/test`，`scope` 用 crate 名或领域名；正文写清能力链影响与证据级别（L0–L3）。

## 10. 收尾清单

- [ ] 能力链每一段已改或明确无需改。
- [ ] Rust 类型、Schema、fixtures、producer、consumer 同步。
- [ ] 正常、unknown、断线/过期、重启、重复/乱序和降级路径有测试。
- [ ] 新依赖没有越过三 crate 边界；`Cargo.lock` 已审计。
- [ ] 文档只改对应 SSOT；新职责已登记。
- [ ] 所有硬件数字有 `DATASHEET/ESTIMATED/MEASURED` 标签。
- [ ] 涉及用户体验的改动记录了原话和需复现问题，且未越权覆盖安全/契约/架构门。
- [ ] 完整门禁通过；脏工作区与未完成实测已明确披露。
