# pipeline.ocblueprint（v2/v3/v4）

角色包 **SSOT** 为 `pipeline.ocblueprint`；新 Stable 包用 `schema_version: 4`，v2 保持兼容，v3 仅为冻结的双核 Beta。

权威文档（oclivenewnew 仓库）：

- [ROLE_PACK_SPEC.md](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/main/creator-docs/role-pack/ROLE_PACK_SPEC.md)
- [V1_TO_V2_MIGRATION.md](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/main/creator-docs/role-pack/V1_TO_V2_MIGRATION.md)

校验：`cargo run -p oclive-cli -- pack validate <角色根>`（按声明版本精确分派）。

编排运行时以宿主 `process_message` 为准，**不**使用蓝图 `steps[]` DSL。

## 四季宝边界

- 枪械感知内核、枪械交互伴随包和硬件资源策略不进入OCLive角色蓝图或标准角色包schema。
- 蓝图只表达角色/宿主需要的能力与provider偏好，不直接分配CPU、RAM、GPU，也不管理硬件节点生命周期。
- Orange Pi专用预算与降级规则写入本仓A.I.Live Gun Spirit HostProfile；本仓集成层按需向OCLive Resource Coordinator注册可选重任务适配器。
- 资源协调的通用定义以OCLive仓的`handoff/KERNEL_SCHEDULER_RESCOPE.md`与`creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md`为准，本仓只记录四季宝的适配决策，不复制通用规范。
