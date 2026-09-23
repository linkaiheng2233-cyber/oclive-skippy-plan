# Zero 3W 首次 bring-up 运行记录（2026-09-11）

**状态**：EVIDENCE / 第一轮实机运行  
**唯一职责**：记录 2026-09-11 夜 Zero 3W 主机首次上电、系统、网络、远程通道与视频链的实测结果、判定与遗留项。  
**适用阶段**：S4 单机硬件原型 bring-up 第一轮  
**上游**：`../PROJECT_BASELINE.md`、`../test-worksheets/README.md`、`../ORANGE_PI_BRINGUP_WORKSHEET.md`  
**下游**：`../TECHNICAL_DEBT.md`（新增项）、`../ORANGE_PI_BRINGUP_WORKSHEET.md`（回填）、`../PROJECT_BASELINE.md`（事实基线）  
**证据词表**：`MEASURED` / `UNKNOWN` / `REPORTED_RECEIVED`，按 `../DOCUMENTATION_SYSTEM.md` 第 3 节使用。

## 1. 运行身份（本记录绑定的硬件与软件）

| 项目 | 值 | 证据级 |
|---|---|---|
| 日期 / 时段 | 2026-09-11，约 11:20–12:10（本地） | — |
| 操作者 | 项目所有者（现场操作）+ AI（远程分析、记录、脚本） | — |
| 主板 | Orange Pi Zero 3W（所有者在售页面报告 6GB） | 板型 `MEASURED`；**RAM 容量未在系统内核验核 → `UNKNOWN`** |
| 主机名 | `orangepizero3w` | `MEASURED` |
| OS | Armbian 26.8.1 trixie（Debian 13.6） | `MEASURED`（`/etc/os-release`） |
| 内核 | 6.6.98-vendor-sun60iw2 | `MEASURED`（`uname -r`） |
| 设备树 | `allwinner/sun60i-a733-orangepi-zero3w.dtb` | `MEASURED`（kernel cmdline + dmesg） |
| 根分区 UUID | `932dec6a-307e-4174-8694-c4a1ef18eef4` | `MEASURED`（`/boot/armbianEnv.txt`） |
| 镜像 | `Armbian_26.8.1_Orangepizero3w_trixie_vendor_6.6.98_minimal.img.xz` | `MEASURED` |
| 镜像 SHA-256 | `915054fed84758a78b09e0d5166f7e308c2185067faa77402031c6ad9366c632` | `MEASURED`（下载后与解包后两次校验一致） |
| 解包后 img | 1,870,659,584 字节；MBR 单分区 type 0x83，起始 32 MiB，大小 1752 MiB，卷标 `armbi_root` | `MEASURED`（分区表解析 + e2fsck） |
| microSD | 32GB（品牌、料号、耐久等级未记录） | `UNKNOWN` |
| 主机供电 | 台式数控电源 **5.00V / 限流 3A**，经香蕉头→USB-C 直通线 | `MEASURED` |
| 屏幕供电 | 5V 墙充（屏幕 USB-C 口标注 "only power"） | `MEASURED`（USB 测试仪 0.13A） |
| 屏幕 | **3.2 英寸** HDMI IPS 模块（尺寸由所有者 2026-09-23 更正；原记 3.5 英寸属误记。型号、控制板、面板、触摸型号均未核验） | `REPORTED_RECEIVED` / `UNKNOWN` |
| 视频线 | Mini HDMI↔HDMI 直连线（本轮判定故障，已退货） | 判定见 §3.1 |
| 环境温度 | 未记录 | `UNKNOWN` |
| 仪器 | 台式数控电源、USB 电压/电流测试仪、（现场）手机拍照未留存 | — |

## 2. 通过项（`MEASURED`）

### 2.1 系统启动链

| 观测 | 结果 |
|---|---|
| 冷启动电流 | 峰值 ≈0.8A → 稳态 **0.36–0.41A @5.00V**（≈1.8–2.1W） |
| 限流影响 | 限流设 0.5A 时**启动即欠压停机**（电流 0A、电压维持 5V）；改为 3A 后正常。工作电流远低于需求上限 |
| 红色指示灯 | 规律闪烁（稳定节奏） |
| 首次启动自动扩展根分区 | **1752 MiB → 29472 MiB（占满整卡）** → 证明 引导→内核→rootfs 全链完整执行 |
| 启动稳定性 | 连续运行 ≥10 分钟无重启、无异常 |

### 2.2 温度与散热

| 观测 | 结果 |
|---|---|
| 8 个 thermal zone | 40.5–46.1 °C（空闲/轻载） |
| 10 分钟观察 | **温度平台化**，不再爬升 |
| 散热 | 已装散热片；板载 2-PIN 风扇为温控式，启动短暂运转后停转 |
| 判定 | 被动散热足够；A733 小面积芯片"烫手感"属常态，非故障 |

### 2.3 网络与远程通道（本轮最大可用成果）

| 观测 | 结果 |
|---|---|
| WiFi | SSID `<已隐去>`，**5GHz 信道 44（80MHz）**，`wlan0` = `<内网地址>`/24 |
| wlan0 MAC | `<已隐去>` |
| 网络标识说明 | SSID、内网地址与 `wlan0` MAC 在公开仓中隐去；原始值保存在仓库外工件目录的 `bringup-toolchain/network-identifiers.txt`，需要复现时从该文件取用 |
| SSH | `ssh.service` 启用；**免密公钥登录成功**（公钥已注入镜像） |
| 远程能力 | 可远程执行 dmesg / DRM 状态 / 温度 / 网络 / 文件操作 → **后续调试无需重烧卡** |
| 开机自诊断 | systemd 服务自动采集 `/root/opi-diag.log`（16,696 B）与 `/root/dmesg-full.txt`（115,637 B） |

### 2.4 显示驱动（软件侧）

| 观测 | 结果 |
|---|---|
| DRM 连接器枚举 | `card0-HDMI-A-1`、`card0-DP-1`、`card0-Writeback-1` |
| 带屏冷启动（HPD 高） | 0.6s 检测 `connect` → 1.2s HDMI 启用 → 设置模式 **480×800** → 创建 `/dev/fb0` → fbcon 绑定 |
| EDID | **曾成功读取 256 字节**；面板原生 480×800，DTD 像素时钟 34.86 MHz，HTOTAL 700 / VTOTAL 830 |
| 判定 | **板卡 HDMI 输出、驱动与帧缓冲链路正常** |

## 3. 未通过项

### 3.1 视频链：HPD/DDC 不稳定 → 判定为线缆故障

| 时间 | 事件 |
|---|---|
| 冷启动（小屏已通电） | 0.6s `connect` → 55s `hpd disconnect` → 黑屏 |
| 重插板端插头 | 162s `connect`，约 60s 后再次 `disconnect` |
| 白/黑闪烁测试期间 | 388s connect → 450s disconnect → 456s connect（反复跳变） |
| 90 秒接触测试 | 全程 `disconnected`，无一次 connect |
| **换接电脑显示器（已知良好设备）** | `connected` 但 **EDID = 0 字节**、`enabled = disabled`、无任何分辨率、无 `/dev/fb0` |

**判定**：HDMI 的 **DDC（SDA/SCL）通道断或接触不良**。

依据：
1. 换成已知良好的显示器后**同样**读不到 EDID、拿不到分辨率 → 排除小屏嫌疑；
2. HPD 能连通并响应插拔 → 线缆未完全断，HPD 针正常；
3. 此前曾成功读取 256 字节 EDID 并成功设置模式、建立 fb0 → **排除板卡与驱动**，属间歇性接触故障。

**处理**：原线已退货，更换品牌线（绿联）后待复测。

### 3.2 触摸能力存疑

该屏幕仅有 HDMI 与 USB-C（`only power`）两个接口，**没有触摸数据接口** → 该屏很可能**不提供触摸**。影响 `OPZ-D02` 与屏幕选型门；需在复测时确认，并据此决定是否更换屏幕。

### 3.3 未核验项（保持 `UNKNOWN`）

- 主板 RAM 容量（需 `free -m` 核验是否 6GB）
- microSD 品牌 / 料号 / 耐久等级
- 屏幕型号、控制板、面板批次
- 环境温度与热像数据
- USB 触摸链路（受 3.2 阻塞）
- USB-C DP Alt 输出路线（本轮未使用）

## 4. 本轮对工作镜像的改造（可复现）

为建立远程通道与自诊断能力，本轮对镜像做了 4 处修改（工具链：Cygwin `debugfs`/`e2fsck` 读写 ext4 + 重新拼接分区 + balenaEtcher 烧录）：

| 修改 | 目的 |
|---|---|
| `/etc/netplan/30-wifi.yaml` | 预置 WiFi，开机自动联网 |
| `/root/.ssh/authorized_keys` | 注入调试公钥，免密登录 |
| `/root/opi-diag.sh` + `opi-diag.service` + `multi-user.target.d` drop-in | 开机自动采集显示/无线/温度/内核日志 |
| `/boot/armbianEnv.txt` 追加 `video=HDMI-A-1:800x480@60`；删除 `/root/.not_logged_in_yet` | 强制目标分辨率；跳过首次登录向导 |

**工具链注意**：Windows 自带 OpenSSH 9.5 与服务端 OpenSSH 10.0 的 hostbound 签名不兼容，需使用便携版 OpenSSH 10.x 客户端；私钥若带密码短语会静默失败。

## 5. 与 OPZ 门的对应

| 门 | 本轮状态 |
|---|---|
| `OPZ-B01` 冷启动 | **部分通过**——系统冷启动成功且稳定，但无可用画面（受视频链阻塞），不能判定整门通过 |
| `OPZ-D01` HDMI 显示 | **未通过**——阻塞原因为线缆 DDC 故障，非板卡 |
| `OPZ-D02` USB 触摸 | **未测**——屏幕疑无触摸接口 |
| `OPZ-P01` 输入功耗 | **部分数据**——空闲 0.36–0.41A @5V；各负载档与背光档未测 |
| `OPZ-M01/T01` 内存/温度稳定性 | **部分数据**——10 分钟温度平台化；两小时负载与内存水位未测 |
| 其余 OPZ 门 | 未开始 |

## 6. 证据与工件位置

**大工件统一放在仓库外的工件目录**（按 `DEVELOPMENT_DISCIPLINE.md`§9，二进制不入仓）：
`E:\OCLive\oclive-四季宝-artifacts\`

| 内容 | 位置 | SHA-256 |
|---|---|---|
| 改造版镜像（含 WiFi/SSH/诊断） | `bringup-2026-09-11\Zero3W-已配置WiFi与诊断.img` | `238499B76146B86FD9FFF50949C266EEC83CFBE7D405325F8CF8578F436BA0F6` |
| 原始镜像（解包后） | `bringup-2026-09-11\Armbian_26.8.1_Orangepizero3w_trixie_vendor_6.6.98_minimal.img` | `D4BB7C8BD1BCF06908107AC803F6D6ABDBF28D23D7F36E542E4C8E47077C0BCB` |
| 原始压缩包（下载源） | 同目录 `.img.xz` | `915054FED84758A78B09E0D5166F7E308C2185067FAA77402031C6AD9366C632`（与下载时一致） |
| 调试脚本与密钥归档 | `bringup-toolchain\`（21 个文件：诊断/监测/填色脚本、SSH 密钥、known_hosts） | — |
| 板端日志 | `/root/opi-diag.log`、`/root/dmesg-full.txt`（在 SD 卡内，可离线读取） | — |
| 便携 SSH 客户端（工作副本） | `E:\WSL\openssh\ssh.exe`（10.0p2；Windows 自带 9.5 不兼容服务端 hostbound 签名） | — |
| 桌面调试记录（外部副本） | `<用户主目录>\Desktop\Zero3W调试记录-2026-09-11.md`（已标 `SUPERSEDED`，内容归并入本文档） | — |

## 7. 下一步（按顺序）

1. **换线后复测视频链四项判据**：HPD 连续 2 分钟稳定 / EDID 非 0 / 出现屏幕真实分辨率 / `enabled` + `/dev/fb0` 存在。
2. **补齐板卡身份**：`free -m`、`lscpu`、`lsusb -t`、`df -h`、`systemd-analyze`、`dmesg` warning 清单，回填 `../ORANGE_PI_BRINGUP_WORKSHEET.md`。
3. **确认触摸能力**（或改用带触摸的屏幕），据此更新 `../DISPLAY_SELECTION.md`。
4. 视频链与触摸通过后，再进入 Host 与最低 UI（`OPZ-E01`/`OPZ-V01` 序列）。
5. 本轮所有 `UNKNOWN` 项在下一轮运行记录中转为 `MEASURED` 或明确 `QUARANTINED`。
