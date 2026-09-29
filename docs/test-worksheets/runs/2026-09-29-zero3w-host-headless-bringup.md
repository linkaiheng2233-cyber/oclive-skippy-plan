# Zero 3W 无头 Host 首次上板运行记录（2026-09-29）

**状态**：EVIDENCE / 第二轮实机运行（本仓**首次在真实硬件上运行 OCLive 无头 Host**）  
**唯一职责**：记录交叉编译产物在 Orange Pi Zero 3W 上的部署、启动、API 验证与资源实测，以及本轮暴露的部署问题。  
**适用阶段**：S4 单机硬件原型 bring-up 第二轮  
**上游**：`../PROJECT_BASELINE.md`、`../runs/2026-09-23-zero3w-display-link-retest.md`、`docs/LINUX_ARM64.md`  
**下游**：`../TECHNICAL_DEBT.md`（新增项）、`../ORANGE_PI_BRINGUP_WORKSHEET.md`（回填 §2）、`../PROJECT_BASELINE.md`（事实表）  
**证据词表**：`MEASURED` / `UNKNOWN` / `REPORTED_RECEIVED`，按 `../DOCUMENTATION_SYSTEM.md` 第 3 节使用。

## 1. 运行身份

| 项目 | 值 | 证据级 |
|---|---|---|
| 日期 / 时段 | 2026-09-29，板端 `07:10–07:30 UTC`（≈ 本地 15:10–15:30 CST） | — |
| 操作者 | 项目所有者（现场接线、供电、换线）+ AI（远程构建、部署、验证、记录） | — |
| 主板 | Orange Pi Zero 3W（A733） | `MEASURED` |
| 主机名 | `orangepizero3w` | `MEASURED`（`hostname`） |
| OS / 内核 | Armbian 26.8.1 trixie（Debian 13.6）／ 6.6.98-vendor-sun60iw2 | `MEASURED`（沿用 09-11 身份） |
| **RAM** | **总计 5,844 MiB ≈ 6 GB** | **`MEASURED`（`free -m`，本轮首次核验）** |
| CPU | 8 核 = 6× Cortex-A55 + 2× Cortex-A76，1 socket，最高 1,794 MHz / 最低 416 MHz | `MEASURED`（`lscpu`） |
| 架构 | `aarch64`，32/64-bit capable | `MEASURED`（`lscpu`） |
| 存储 | `/dev/mmcblk1p1`，29 G，已用 1.6 G（6%） | `MEASURED`（`df -h`） |
| microSD 料号 | `name=SD`、`manfid=0x0000fe`、`oemid=0x3432`、`date=11/2025`、`serial=0x14` | `MEASURED`；**厂商 id 非注册值，属无牌/白牌卡** |
| 温度 | 开机空闲 **30.0–30.3 °C**；Host 运行中 **49.5 °C** | `MEASURED`（thermal_zone） |
| 主机供电 | 台架 5.00 V / 限流 3 A | `MEASURED`（沿用 09-11；本轮实测 0.41 A 稳态） |
| 屏幕供电 | 5 V 墙充，4.8 V / 0.13 A | `MEASURED`（所有者现场读数） |
| 产物 | `aarch64-unknown-linux-gnu` release | `MEASURED` |
| 产物大小 / SHA-256 | **25,857,760 B（24.66 MiB）** / `de9e58a24fe5bcf451d8a1b9c5c4fa22c34b91d9cedc3d5f0014c32aaf1dbee1` | `MEASURED`（两侧分别计算，一致） |
| 工具链 | Arm GNU Toolchain 14.2.Rel1（Build arm-14.52）14.2.1 | `MEASURED`（与 `ARMV7_VALIDATION.md` 同一版本） |
| 构建报告 | `tmp/aarch64-report-20260929.json`（本地忽略目录） | `MEASURED` |
| 动态依赖 | `libgcc_s.so.1` / `libm.so.6` / `libc.so.6` | `MEASURED`（`readelf -d`） |
| 部署路径 | `/root/ailive-gun-spirit-host` + `/root/roles` + `/root/migrations` | `MEASURED` |
| 会话 | `session_id=smoke-20260929`，`role_path=/root/roles/default` | `MEASURED` |

## 2. 通过项（`MEASURED`）

### 2.1 构建与部署

| 观测 | 结果 |
|---|---|
| 交叉构建 | `./scripts/build-aarch64.ps1` 成功；**1 m 49 s**（与 09-01 的 108 s 吻合） |
| ELF 校验 | `elf64=true`、`machine_aarch64=true`；`readelf` 头 `7f 45 4c 46 02` + `b7 00`（EM_AARCH64） |
| 产物比 09-01 大 236 KB | 25,621,760 → 25,857,760 B；**因为构建会一并编译兄弟仓 `oclivenewnew`，该仓自 09-01 后已推进**；这也是 `GS-CI-001`「不可复现」的直接体现 |
| 传输完整性 | 板端 `sha256sum` 与本地一致（`de9e58a2…`） |

### 2.2 板上运行

| 观测 | 结果 |
|---|---|
| 启动 | 进程正常运行；`INFO oclive_api: HTTP API listening http://127.0.0.1:8420` |
| **启动到监听耗时** | **90 ms**（日志时间戳 `07:27:47.954906Z` → `07:27:48.045217Z`） |
| 常驻内存 | **RSS 15.25 MiB**（`ps` RSS 15,616 KB），VSZ 427 MB，**7 线程** |
| 空闲 CPU | 0.1–0.5 % |
| 角色目录 | `INFO oclive_roles: find_roles_dir: OCLIVE_ROLES_DIR -> /root/roles` |
| 迁移 | `INFO oclive_migrate: using bundled migrations near executable/cwd dir=/root/migrations` |
| 插件扫描 | `directory plugins scanned count=0 ids=[]` |
| `/health`（无认证路由） | 返回 `ok` |
| 受保护路由（无 token） | **HTTP 401** —— 认证确实生效 |
| 受保护路由（带 token） | 通过认证进入 handler |
| **完整角色回合** | **成功**。`POST /chat` 返回完整结构：`reply`、`emotion{neutral:1.0}`、`relation_state=Acquaintance`、`favorability_current=50.0`、`events[{event_type:Ignore,confidence:0.35}]`、`scene_id=default`、`schema=16`、`api_version=1` |

**结论**：`contracts + perception-core + host` 三 crate 与 OCLive 内核的**全部代码路径在目标硬件上可加载、可执行**；角色加载、情绪、关系状态、事件分析、Prompt 组装与回复生成整链贯通。本轮使用 `OCLIVE_HTTP_API_MOCK_LLM=1`（mock LLM），**未验证真实模型 / 网络回合**。

### 2.3 板卡身份核验（`GS-HW-004` 部分关闭）

| 项 | 结果 |
|---|---|
| RAM 容量 | **5,844 MiB → 6 GB 属实**（此前一直为 `UNKNOWN`） |
| CPU 拓扑 | 6×A55 + 2×A76，8 逻辑核，与 A733 规格一致 |
| microSD | 白牌（`manfid=0x0000fe` 非注册值），2025 年第 11 周生产 |
| 存储余量 | 27 G 可用 |
| 屏幕型号 / 环境温度 | 仍 `UNKNOWN`（屏幕未接板端、环境温度未记录） |

## 3. 本轮暴露的问题

### 3.1 SQLite 迁移未打包（**阻断级**，已现场绕过）

首次启动直接失败：

```
[OCLIVE_API_SERVER_FAILED] Database migration failed: no SQLite migrations directory found; tried:
  embedded E:\OCLive\oclivenewnew\kernel\crates\oclive_kernel_host/migrations (missing or no .sql);
  /root/migrations ; /root/resources/migrations ; /root/roles/migrations ; /root/roles/resources/migrations ;
  monorepo root not found from 3 anchor(s)
```

**根因**：迁移文件未内嵌二进制，内核在**运行时**按路径探测，而**编译期烙入的是构建机的 Windows 路径**（`E:\OCLive\…`），在 Linux 目标上必然不存在。

**现场处置**：把兄弟仓 `oclive_kernel_host/migrations/`（40 个 `.sql`）部署到探测路径之一 `/root/migrations`，随即通过。

**根治方向**：OCLive 侧改为编译期内嵌（`include_str!` / sqlx 宏），或由本仓构建流程显式把迁移目录随产物打包。**已登记为 `GS-HW-007`。**

### 3.2 ext4 `commit=120`：传文件后必须 `sync`（**实测踩中**）

`findmnt` 显示根分区挂载参数为 `rw,relatime,errors=remount-ro,commit=120` —— **日志仅每 120 秒强制提交一次**。

实测时间线：07:21 传输 25 MB 产物 → 07:23 所有者为换线断电 → **重启后文件全部消失**（`/root` 目录 mtime 仍为 09-23）。重新传输并显式 `sync` 后正常。

**部署纪律**：向本板传输任何持久文件后**必须 `sync`**，否则短时断电即丢失。这条同样适用于未来的镜像/系统服务部署。

### 3.3 HTTP API 的两个安全默认（**符合预期，记录备查**）

1. **强制 token**：`OCLIVE_API_TOKEN` 未设置时直接拒绝启动；存在开发逃逸开关 `OCLIVE_API_ALLOW_UNAUTHENTICATED=1`，**仅限隔离的本地开发**。本轮用 32 字节随机 token，存 `/root/.oclive_api_token`（mode 600）。
2. **仅绑定 `127.0.0.1`**：外部无法直连。需要跨机访问时走 SSH 隧道或显式改绑定。

### 3.4 接口契约（HTTP 层与桌面 DTO 不同名）

```
POST /chat
Header: x-oclive-api-token: <token>
Body:   {"role_path": "/root/roles/default", "message": "你好", "session_id": "..."}
```

**注意**：HTTP 层用 **`role_path`**（角色**目录路径**）与 **`message`**，不是桌面 DTO 的 `role_id` / `user_message`。首次调用连续踩了三次参数名错误。

### 3.5 构建脚本缺陷（**已修复**）

`scripts/build-aarch64.ps1` 与 `scripts/build-armv7.ps1` 都调用 `Get-FileHash`。本机 Windows PowerShell 5.1 下该 cmdlet **不可解析**（`GS-QUALITY-002` 症状二），导致**构建成功但脚本在生成报告时崩溃、报告 JSON 永不落盘**（首次运行时 `tmp/aarch64-report-20260929.json` 未生成，产物本身正常）。

**已修复**：两个脚本改用 .NET `SHA256`，与 `export-docs.ps1` 的既有做法一致；两个脚本保持 ASCII-only。重跑后报告正常生成。

## 4. 未验证项（保持 `UNKNOWN`）

- 真实 LLM / 网络回合（本轮为 mock）
- 断网启动与降级
- 屏幕渲染链（无头运行，未接显示输出）
- 触摸（屏幕疑无触摸接口，`GS-HW-003`）
- 长时间稳定性与 20 次冷启动（`OPZ-B01`）
- 1,000 条 DeviceEvent 回放与画面 p95（`OPZ-E01`）
- 感知节点（XIAO / FSR / IMU 尚未接入）

## 5. 下一步

1. **把迁移打包问题根治**（`GS-HW-007`）：或在 OCLive 侧内嵌，或在构建流程中随产物打包 → 否则每次部署都要手工补 `/root/migrations`。
2. **把部署流程写成可重放脚本**并并入 `GS-DEV-001`（含 `sync` 纪律、token 生成、路径约定）。
3. 显示链：ADR-064 的 ⓪ DVI 信令实验（`video=HDMI-A-1:480x800@60D`）已写入板端 `/boot/armbianEnv.txt`，**待接 HDMI 后重启验证**；注意有效性判据是 `dmesg` 的 `vsif` 行是否变化。
4. 上述完成后进入 `OPZ-E01` / `OPZ-V01` 序列（Host 与最低 UI）。
