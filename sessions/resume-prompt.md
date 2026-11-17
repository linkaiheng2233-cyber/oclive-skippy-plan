# 新机器开工引导语（复制粘贴给 DSH）

> 用途：在另一台电脑克隆本仓库后，用这段话让 DSH 快速对上进度。按需删改。

---

## 第零段：先确认真实目录布局（**两个仓**）

本仓 Rust workspace 依赖兄弟仓 `../oclivenewnew`。克隆时必须两个仓做兄弟：

```powershell
./scripts/bootstrap-clone.ps1          # 一键
# 或手动：
git clone -b docs/personal-document-archive-20261030 https://github.com/linkaiheng2233-cyber/oclive-skippy-plan.git E:\OCLive\oclive-四季宝器灵
git clone -b fix/chat-pro-rc9-feedback-20261106 https://github.com/linkaiheng2233-cyber/oclivenewnew.git E:\OCLive\oclivenewnew
```

## 第一段：让 DSH 先读文件

```
这是 E:\OCLive\oclive-四季宝器灵 仓库（GUN SPIRIT / 四季宝器灵，A.I.Live 枪娘器灵）。

请按顺序读：
1. AGENTS.md（会自动加载，仍需确认红线与治理规则）
2. docs/handoff/AI对接文档.md —— 现在在哪、下一步做什么
3. docs/PROJECT_BASELINE.md —— 当前事实基线
4. docs/ROLE_KERNEL_INTEGRATION.md 与 docs/DECISIONS.md 里的 ADR-073 —— 三块分离设计
5. sessions/2026-11-17-P0A-全套设计.md 的**最后一章**（前情摘要，不必全读）

读完后用不超过 15 行总结：当前基线、已完成、下一步、阻塞项。
```

## 第二段：确认环境

```
请检查环境并报告：
- git status -sb（应当干净；**必须在工作分支** docs/personal-document-archive-20261030 上，
  默认分支 main 不含本项目进度）
- git log --oneline -3
- ./scripts/verify.ps1 -SkipAudit 是否 PASS（cargo audit 需要网络，失败不算回归，见 GS-DEP-002）
- Rust / Node / lv_font_conv 是否可用（板端构建需要 espup + Arduino CLI，见 docs/P0A_FIRMWARE_PLATFORM.md）
```

## 第三段：继续开发（当前下一步）

```
按 docs/ROLE_KERNEL_INTEGRATION.md §8 继续 **S1：GameSignal 类型 + 模拟一局生成器**。

约束（务必遵守）：
- GameSignal / RoleCue 放新 crate（ailive-gun-spirit-role-bridge），**不要**加进 contracts
- 游戏事实不得转成 SensorObservation / DeviceEvent；C→B 只影响表现，不得进命中判定
- 每阶段做完停下，等所有者复核后再进下一阶段
- 门禁用 ./scripts/verify.ps1 -SkipAudit；涉及 crates 改动要跑完整测试
```

## 常用事实（避免重复问）

| 项 | 值 |
|---|---|
| 板卡 | 晶彩 JC3248W535（ESP32-S3，16 MB flash / 8 MB OPI PSRAM），右 RESET / 左 BOOT |
| 固件构建 | `./scripts/flash-p0a.ps1`（= `build.ps1 -BlobAssets -WithKernel -WithWifi`） |
| 现状 | 契约 v0.4；M1–M5 上板验证；Wi-Fi 自动发现；屏幕 OS 两页；发行版运行器 |
| 阻塞 | **IMU 已到货（GY-601N1，芯片型号待 `IMU PROBE` 确认）**；待焊排针/转接线（SH1.25 4P → 杜邦母）；发射器选型待定（20 号后） |
| 技术债 | `GS-DEP-003`：`KSYNTH` 合成观测必须在 IMU 到货后删除 |
| 基线 tag | `baseline/GS-P0-BL-2026-11-14b` |



