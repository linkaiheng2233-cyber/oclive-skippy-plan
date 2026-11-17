# 会话记录目录

本目录保存**跨会话、跨机器**的对话记录，让在另一台电脑上克隆仓库后能直接接上进度。

## 这里有什么

| 文件 | 内容 |
|---|---|
| `2026-11-17-P0A-全套设计.md` | 2026-11-14 → 11-18 这段长会话的可读摘录（最新导出：1237 轮）。涵盖：板端点亮→双向联调、M1–M5、契约 v0.4、屏幕 OS、发行版 MVP、三块分离（ADR-073）、**IMU 到货与 P3/P4 接线核对** |
| `resume-prompt.md` | **新机器开工时直接粘给 DSH 的引导语** |
| `export-session.mjs`（在 `scripts/`） | 导出工具：从 `~/.dsh/sessions/.../session.v4.jsonl.zstd` 生成 Markdown |

## 为什么不是原始会话文件

DSH 的会话是 `session.v4.jsonl.zstd`（多帧拼接、约 7 MB 压缩 / 39 MB 解压），里面包含：

- 本机绝对路径（`C:\Users\<用户名>\...`）
- 网络标识（Wi-Fi SSID、板子 MAC）
- 大量工具参数（命令、文件内容）

本仓库是**公开**的，因此入库的是**脱敏后的可读摘录**：只保留对话文字，SSID / MAC / 用户名已替换为 `[已脱敏]`，工具参数不计入。

> 如果将来需要"逐字节还原会话"，请把原始 `.zstd` 放到**私有位置**（私有仓库或网盘），不要放进这个公开仓库。

## 在另一台电脑上接续（推荐流程）

1. **克隆到与原来相同的路径**（可选但更省事）：

   ```powershell
   # 必须在 clone 时指定工作分支（默认分支 main 不含本项目进度）
   git clone -b docs/personal-document-archive-20261030 `
     https://github.com/linkaiheng2233-cyber/oclive-skippy-plan.git E:\OCLive\oclive-四季宝器灵
   ```

   已经克隆过也可以在仓库里切：
   ```powershell
   git checkout -b docs/personal-document-archive-20261030 origin/docs/personal-document-archive-20261030
   ```

   基线 tag 可直接检出：`git checkout baseline/GS-P0-BL-2026-11-14b`

2. 在该目录打开 DSH（新会话即可）。`AGENTS.md` 会被自动加载。

3. 把 `sessions/resume-prompt.md` 的内容粘进去，或直接说：

   > 读取 `docs/handoff/AI对接文档.md` 与 `docs/PROJECT_BASELINE.md`，再读 `sessions/2026-11-17-P0A-全套设计.md` 的最后一章；
   > 按 ADR-073 与 `ROLE_KERNEL_INTEGRATION.md` 继续 S1（GameSignal 类型 + 模拟生成器）。

4. **开工前先确认**（这是仓库的跨会话约定）：

   ```powershell
   git status -sb
   git log --oneline -3
   ./scripts/verify.ps1 -SkipAudit
   ```

## 导出方法（以后新增会话时）

```powershell
# 1. 找到当前工作区最近的会话文件
$dir = "$env:USERPROFILE\.dsh\sessions\--E-OCLive-oclive-~56DB~5B63~5B9D~5668~7075--"
$src = (Get-ChildItem $dir -Recurse -File -Filter 'session.v4.jsonl.zstd' |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName

# 2. 准备脱敏清单（一行一个，勿入库）
#    Wi-Fi SSID / 板子 MAC / 本机用户名 等

# 3. 导出
node scripts/export-session.mjs $src 'sessions/<日期>-<主题>.md' --redact <脱敏清单.txt>
```

## 注意

- 本目录的文件**会进入公开仓库**，提交前务必检查脱敏清单是否生效。
- 会话记录是**过程证据**；技术结论以 `docs/` 下的领域文档与 ADR 为准，二者冲突时以文档为准。


