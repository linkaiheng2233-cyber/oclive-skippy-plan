# AI 会话交接（2026-09-23）

**状态**：CURRENT HANDOFF  
**读者**：在 Claude Code / 其它 agent 中接手四季宝工作的 AI 与协作者  
**唯一权威**：`E:\OCLive\oclive-四季宝器灵`；公开远端 `https://github.com/linkaiheng2233-cyber/oclive-skippy-plan`  
**上一份交接**：本文替代聊天记录；工程事实只以仓库 SSOT 为准，本文只做导航与状态摘要。

---

## 0. 30 秒速览

- **项目**：四季宝器灵 = A.I.Live-ai枪娘器灵。把 OCLive 角色接入导轨设备事件与小屏输出的实体宿主（P0 原型；仅娱乐/把玩/角色陪伴）。
- **当前基线**：`GS-P0-BL-2026-09-22`（文档权威收敛基线）。硬件事实与 09-18 基线相同。
- **代码/文档**：三 crate Rust workspace + v0.2 契约与生成 Schema + 无头 Host；`./scripts/verify.ps1` 在绑定提交 `55d10e6` 的干净工作树上全绿（30 项测试）。
- **硬件**：Zero 3W 首轮 bring-up 完成——**系统/网络/远程通道通过**；**视频链已于 2026-09-23 复测通过**（换绿联线解决 DDC 故障：HPD 稳定、EDID 256 字节、5 个模式、`/dev/fb0`）。
- **立刻要做的第一件事**：👉 ① 把显示落到**横向 800×480**（面板原生是竖屏，`GS-HW-005`）；② 采集板卡身份（`GS-HW-004`）；然后进入 Host 与最低 UI（§2 末）。
- **三条硬约束**：① 唯一权威是本仓，桌面/导出件只读；② 分阶段提交 + 提交前跑门禁；③ 不实现火控/瞄准/武器控制类功能。

---

## 1. 接手第一步：核对状态（不要跳过）

```powershell
cd E:\OCLive\oclive-四季宝器灵
git log --oneline -12
git tag -l 'baseline/*'
git status --short
```

预期：工作树干净；存在基线 tag `baseline/GS-P0-BL-2026-09-18` 与 `baseline/GS-P0-BL-2026-09-22`；HEAD 至少为 `1f1fbbc`（若更新，先读新提交的 message）。

**必读四份文档（按序）**：

| 顺序 | 文档 | 读它能知道 |
|---|---|---|
| 1 | `../README.md`（docs 导航） | 身份入口、每类事实在哪 |
| 2 | `../PROJECT_BASELINE.md` | 当前事实基线、门禁证据、下一步 |
| 3 | `../test-worksheets/runs/2026-09-23-zero3w-display-link-retest.md` | **最新**实机记录：视频链复测通过（`GS-HW-002` 关闭） |
| 4 | `../test-worksheets/runs/2026-09-11-zero3w-first-bringup.md` | 首轮 bring-up 记录（系统/网络/远程通道、判定依据、证据位置） |
| 5 | `../TECHNICAL_DEBT.md` | 开放项与关闭条件 |

---

## 2. 视频链：已于 2026-09-23 复测通过（`GS-HW-002` 关闭）

### 结论（勿重复测）

换用**绿联 Mini HDMI↔HDMI 线**后四项判据全部通过：

| 判据 | 实测 |
|---|---|
| HPD 稳定性 | 120 秒稳定 `connected`，变化 1 次 |
| EDID | **256 字节**（首轮为 0） |
| 分辨率 | 5 个模式：`480x800`/`1024x768`/`800x600`/`800x480`/`640x480` |
| 帧缓冲 | `enabled` + `/dev/fb0`（`480,1600`、stride 1920、bpp 32），fbcon 已绑定 |

首轮"随附线缆 DDC 间歇故障"的判定被证实并由换线解决。证据：`runs/2026-09-23-zero3w-display-link-retest.md` 与仓库外 `bringup-2026-09-11\display-link-retest-2026-09-23.txt`。

**当前遗留**：面板原生 **480×800 竖屏**，项目要求横向 800×480；`800x480` 在模式列表内但未被优先选中，需 `fbcon=rotate`（控制台）与 renderer 旋转（未来 UI），见 `GS-HW-005`。

### 复测方法（保留作回归用：四项判据）

| 判据 | 通过标准 |
|---|---|
| 1 | HPD 连续 120 秒稳定 `connected`，跳变 ≤1 次 |
| 2 | `/sys/class/drm/card0-HDMI-A-1/edid` 读到 **128/256 字节**（不是 0） |
| 3 | `modes` 出现屏幕真实分辨率（期望 480×800 或 800×480） |
| 4 | `enabled` = `enabled` 且 `/dev/fb0` 存在 |

### 一键复测脚本

```powershell
pwsh -File 'E:\OCLive\oclive-四季宝-artifacts\bringup-toolchain\verify-display-link.ps1'
# 板端地址不同时： -BoardIp 192.168.x.x
```

脚本逐项打印判据与总分诊结论。

### 失败分诊（按首轮证据推出）

| 现象 | 结论与下一步 |
|---|---|
| EDID 仍为 0 且换线无效 | 问题在**屏幕侧** → 改走 USB-C DP Alt（**主动式**转接头）路线 |
| EDID 非 0 但屏幕仍黑 | **分辨率/时序不匹配** → 强制 `video=HDMI-A-1:800x480@60`（已在镜像 `/boot/armbianEnv.txt`）或换屏 |
| HPD 仍跳变 | **接触问题** → 检查插到底、换转接头、避开劣质线 |

### 复测通过后必须做的事（按纪律）

1. 新建 `docs/test-worksheets/runs/<日期>-<主题>.md`（运行身份 + 判据结果 + 证据）；
2. 回填 `docs/ORANGE_PI_BRINGUP_WORKSHEET.md`（`OPZ-B01`/`OPZ-D01`）与 `docs/PROJECT_BASELINE.md` 事实表；
3. 更新 `docs/TECHNICAL_DEBT.md` 的 `GS-HW-002` 状态（证据 + 关闭条件）；
4. **分阶段提交**并推送；需要时更新基线 tag。

---

## 3. 环境与访问（本机）

| 项 | 值 |
|---|---|
| 仓库 | `E:\OCLive\oclive-四季宝器灵`（唯一权威） |
| 远端 | `origin` = `https://github.com/linkaiheng2233-cyber/oclive-skippy-plan`（公开，MIT） |
| **GitHub 网络** | 直连 `github.com:443` **时通时不通**；已配置 `git config --global http.https://github.com.proxy http://127.0.0.1:7897`（本机 Clash 类代理）。**推送失败时的两种绕行**：① 代理不稳（`schannel: failed to receive handshake` / `unexpected disconnect`）→ 临时直连 `git -c http.https://github.com.proxy= push origin main`；② 直连不通 → 保持代理默认配置直接 `git push`。`gh` 命令需临时 `$env:HTTPS_PROXY='http://127.0.0.1:7897'`（`api.github.com` 通常可直连）。**推送后务必核对** `git rev-parse --short main` 与 `origin/main` 一致 |
| 板子 | Orange Pi Zero 3W 6GB（A733），Armbian 26.8.1 trixie，内核 6.6.98 vendor |
| 板端网络标识 | **不在仓库内**：`E:\OCLive\oclive-四季宝-artifacts\bringup-toolchain\network-identifiers.txt`（SSID / 内网地址 / MAC） |
| SSH 登录 | `E:\WSL\openssh\ssh.exe -i E:\WSL\ssh\id_opi root@<板端地址>`（**必须用便携 10.x**，见 §7） |
| 复测/调试脚本 | `E:\OCLive\oclive-四季宝-artifacts\bringup-toolchain\`（含 `verify-display-link.ps1`、诊断/监测脚本） |
| 镜像与工具链工件 | 同目录 `bringup-2026-09-11\`（含 WiFi/SSH/诊断的改造版镜像 + 校验值） |
| 本地代理端口 | `127.0.0.1:7897`（Clash 类；git/gh 未自动继承） |

**找板子**（上电约 60 秒后 WiFi 才重连）：

```powershell
1..254 | ForEach-Object -Parallel { $ip="192.168.2.$_"; $c=New-Object System.Net.Sockets.TcpClient; try { $t=$c.ConnectAsync($ip,22); if ($t.Wait(400) -and $c.Connected){$ip} } catch {} finally {$c.Close()} } -ThrottleLimit 64
```

### ⚠️ 判定板子是否真的在线：必须用 ARP 或应用层证据

**本机代理（TUN 模式，Mihomo/gvisor 栈）会伪造 ICMP 与 TCP 握手**：`ping` 会收到回包、端口看起来"开放"、SSH 甚至能"握手成功"却永远收不到 banner。因此：

| 方法 | 可信度 |
|---|---|
| `ping` 回包 | ❌ 可能被代理代答 |
| 端口"开放"（包括随机端口） | ❌ 可能被代理在本地接住 |
| **ARP 表里有该 IP 的 MAC** | ✅ 二层信息，代理伪造不了 |
| **应用层证据**：`uname -a`、`/proc/cmdline`、`dmesg`、`iw dev` 等真实回显 | ✅ 决定性 |
| 路由器后台 DHCP 客户端列表 | ✅ 独立第三方 |

推荐顺序：**先看 ARP（`arp -a`）→ 再做 SSH 并核对 `uname -a`/`/proc/cmdline`**；只用 `ping` 判定在线是本项目已经踩过的坑。

**做局域网/板卡调试前**：关闭代理的 TUN 模式，或把 `192.168.0.0/16` 加入代理直连（bypass）列表；否则会出现"时通时不通 + 假在线"。

**另一个隐患**：本机有线（`192.168.2.170`）与无线（`192.168.2.171`）**同时挂在同一网段**，两条默认路由指向同一网关。遇到"时通时不通"时优先怀疑它。

---

## 4. 硬件已确立事实（勿重复测）

| 项目 | 结论 |
|---|---|
| 冷启动 | 峰值 ≈0.8A → 稳态 **0.36–0.41A @5.00V**（≈1.8–2.1W）；**限流设 0.5A 会导致启动欠压停机**（用 3A） |
| 温度 | 8 个 thermal zone **40.5–46.1°C**，10 分钟平台化；温控风扇在温度稳定时停转 |
| 系统 | 首次启动自动扩展根分区（1752 MiB → 29472 MiB）证明启动链完整 |
| 远程通道 | WiFi（5GHz 信道 44）+ **免密 SSH** + 开机自诊断服务（`/root/opi-diag.log`）→ 后续调试不必重烧卡 |
| 显示驱动 | HDMI/DP/Writeback 连接器正常枚举；能读 EDID、能设模式、能建 `/dev/fb0` |
| 视频链 | **已通过（2026-09-23 复测）**：HPD 稳定、EDID 256 字节、5 个模式、`/dev/fb0`（fbcon 绑定） |
| 触摸 | 屏幕仅 HDMI + `only power` USB-C，**疑无触摸**（`GS-HW-003`） |
| 未核验 | RAM 容量、microSD 料号、屏幕型号、环境温度（`GS-HW-004`） |

---

## 5. 开放项（按优先级）

| ID | 优先级 | 一句话 |
|---|---|---|
| `GS-HW-002` | ~~P0~~ | ✅ **已关闭（2026-09-23）**：换绿联线后四项判据通过 |
| `GS-HW-005` | P2 | 面板原生竖屏，横向 800×480 待 `fbcon=rotate`/renderer 旋转落地 |
| `GS-HW-001` | **P0** | 板卡实测门（OPZ-B01 20 次冷启动、T01/P01、本地模型档位）尚未开始 |
| `GS-CI-001` | P1 | OCLive 依赖获取策略未冻结（候选：钉 rev 的 git 依赖已可执行）；冻结前**不加 CI** |
| `GS-CAL-001` | P1 | IMU 姿态校准缺实机样本（需节点到货） |
| `GS-HW-003` | P1 | 触摸能力存疑，影响 `OPZ-D02` 与屏幕选型 |
| `GS-REMOTE-001` | P2 | 远端已建、首次推送已完成；**分支保护待启用** |
| `GS-HW-004` | P2 | 板卡/配件身份未核全 |
| `GS-DEV-001` | P2 | 镜像改造与远程调试流程未固化为可复现文档/脚本 |
| `GS-QUALITY-002` | P2 | Windows 控制台 936/GBK 编码导致脚本中文输出异常（脚本已按 ASCII 规避） |
| `GS-MAINT-001` | P2 | `contracts/reducer` 文件规模与拆分时机（观察中） |

**另一项待决策（非债务）**：2026-09-18 之前的提交历史里含本机路径/网络标识（当时未脱敏）。当前工作树已干净（本轮又修掉 `history/EXTERNAL_DOCUMENT_AUDIT_2026-09-22.md` 里的学号+真名）。是否重写历史需项目所有者决定——注意重写会改变 SHA，而 `PROJECT_BASELINE.md` 已绑定 `55d10e6` 等 SHA。

**新增候选决策（2026-09-23，见 `DECISIONS.md` ADR-063）**：项目所有者提出「把 Host 放到 PC、枪侧只留传感器 + ESP32 级 MCU」的桌面形态。该提案把形态拆为 **P0-A 桌面瘦客户端**（PC 作唯一 Host，可完全不用 HDMI）与 **P0-B 独立主机舱**（保留但后置），两者共享 v0.2 契约与 `perception-core`。**若被接受**：视频链（`GS-HW-002`）将不再位于关键路径，P0 主线与采购清单需同步更新。**裁决前不要**改形态、不要追加 P0-B 采购；先做提案里列的四项待办（ESP32 与 XIAO 的 FSR ADC 对比、Wi-Fi 回传丢包与 p95、形态裁决、体验复核）。

---

## 6. 工作纪律（必须遵守）

1. **唯一权威**：本仓是四季宝唯一权威；桌面/导出件/个人文档只是副本。现场填写件用 `scripts/export-docs.ps1` 重新导出，填写后回填仓库。
2. **分阶段提交**：一次提交一个可回滚单元；Conventional Commits（`feat/fix/docs/chore/refactor/test` + scope）；不把无关改动混进同一提交。
3. **提交前门禁**：改动至少跑受影响包窄测；基线、改 `Cargo.lock`、交付前跑 `./scripts/verify.ps1`。
4. **大工件不入仓**：镜像、证据包、模型权重放仓库外 `E:\OCLive\oclive-四季宝-artifacts\`；仓库只记路径与 SHA-256。
5. **文档单点 SSOT**：一个参数只有一个领域真源；其余文档链接 + 摘要。冲突按 `DOCUMENTATION_SYSTEM.md`§1 权威顺序处理。
6. **实机必留证**：每次上机在 `docs/test-worksheets/runs/<日期>-<主题>.md` 留运行身份、证据分级、判定与下一步；同时回填工作表与技术债。
7. **证据分级**：`DATASHEET`/`ESTIMATED`/`MEASURED`/`UNKNOWN` 不得混用；脏工作树结论不得冒充绑 SHA 证据。
8. **产品边界（硬红线）**：不实现或宣传智能火控、弹道计算、自动瞄准、武器控制或军警用途；P0 不接发射器内部火控/扳机/供弹机构；相机是 P1 且只做娱乐观察。

---

## 7. 踩坑清单（按这些做能省半天）

1. **SSH 客户端版本**：Windows 自带 OpenSSH 9.5 与板端 10.0 的 hostbound 签名不兼容（表现为"服务器接受公钥但仍 Permission denied"）→ 用 `E:\WSL\openssh\ssh.exe`（10.0p2 便携版）。
2. **ssh-keygen 引号陷阱**：`-N '""'` 会把 `""` 当成密码短语，导致密钥静默不可用；用 `-N ''`，或事后 `ssh-keygen -p -P '""' -N ''` 去除。
3. **限流不是油门是保险丝**：0.5A 会掐断启动峰值；零 W 但 5V 正常 = 板子没启动；"电流顶格 + 电压跌落"才是危险。
4. **便宜 HDMI 线 DDC 通道易断**：HPD 通但 EDID=0 是典型症状；用已知良好显示器交叉验证可定位线缆。
5. **厂商 HDMI 驱动忽略 DRM debugfs `force`**：不要指望强制连接器状态，改走换线/EDID 覆盖。
6. **GitHub 直连不通**：必须走本机代理（§3）。
7. **Cygwin `debugfs` 改 ext4 镜像**：`write` 前必须先 `cd` 到目标目录，否则文件不会链进目录树；文件名易打错（曾把 `armbianEnv.txt` 打成 `arbmianEnv.txt` 留下悬空项，需 `e2fsck -fy` 清理）。
8. **板子断电后**约 60 秒才重连 WiFi；扫不到就再等。
9. **PowerShell 控制台编码**：默认 936/GBK 会让脚本内中文乱码（`GS-QUALITY-002`）；脚本保持 ASCII、中文放外部 UTF-8 映射文件。
10. **Git CRLF 警告**：本仓文档含中文，提交时 CRLF/LF 警告无害，不要为它批量改行尾。
11. **代理伪造在线（2026-09-23 实测）**：TUN 代理会代答 ICMP、在本地完成 TCP 握手，导致 `ping` 通、端口"开放"、SSH"握手成功但无 banner"等假象。判定设备在线**只能**用 ARP 表或应用层回显（`uname -a` 等）。调试局域网前先关 TUN 或把 `192.168.0.0/16` 设为直连。
12. **双网卡同网段**：本机有线与无线同时在 `192.168.2.x`，两条默认路由同网关；出现"时通时不通"优先怀疑此项。
13. **不要把"能 ping 通"当成 bring-up 证据**：首轮 bring-up 的结论建立在应用层回显上（`/proc/cmdline` 含注入参数、dmesg 驱动日志、`iw dev` 的 SSID、HPD 随物理插拔变化）以及**离线读卡**看到的根分区扩展，因此不受本次代理假象影响。任何后续连接都应同样以应用层证据归档。

---

## 8. 2026-09-18 那轮已完成（供追溯）

- 三 crate workspace 迁移、v0.2 契约与生成 Schema、perception-core、无头 Host + 模拟器 → **11 个分阶段提交**；
- 修复 `rustls` 0.23.43 → 0.23.45（RUSTSEC-2026-0285）；
- 建立文档体系（基线/边界/文档体系/纪律/技术债/路线/硬件 SSOT/测试包/runs/历史审计）；
- 开源前脱敏 + 建立 GitHub 公开远端并推送 `main` 与基线 tag；
- 后续会话在 09-22 完成：桌面采购价格归并、外部文档归并审计、导出机制脚本化（`scripts/export-docs.ps1`，关闭 `GS-DOC-002`）、确认 `GS-P0-BL-2026-09-22` 基线。

---

## 9. 给接手 agent 的交接要求

1. **先核对状态再动手**（§1）；不要重跑 §4 已确立的实测。
2. **实机动作前先确认板子在线**，并说明使用的供电与限流设置。
3. **任何产品边界、契约、安全红线的改动先写 ADR**（`docs/DECISIONS.md`），不得顺手跨层。
4. **每次实机后写 `runs/` 记录并回填**工作表/基线/技术债，然后分阶段提交并推送。
5. **报告纪律**：区分"已实现 / 已本地验证 / 需实机 / 需绑 SHA"。不能把估算说成实测，也不能把未复测的结论说成通过。
