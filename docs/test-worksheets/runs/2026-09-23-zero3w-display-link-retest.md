# Zero 3W 视频链复测运行记录（2026-09-23）

**状态**：EVIDENCE / 关闭 `GS-HW-002`  
**唯一职责**：记录换用品牌 Mini HDMI 线后的视频链四项判据复测结果、原始证据与遗留项。  
**上游**：`../PROJECT_BASELINE.md`、`2026-09-11-zero3w-first-bringup.md`、`../TECHNICAL_DEBT.md` 的 `GS-HW-002`  
**下游**：`TECHNICAL_DEBT.md`（关闭该债）、`ORANGE_PI_BRINGUP_WORKSHEET.md`（`OPZ-B01`/`OPZ-D01` 回填）、`PROJECT_BASELINE.md`（事实表）、`handoff/AI_SESSION_HANDOFF_2026-09-23.md`

## 1. 运行身份

| 项目 | 值 | 证据级 |
|---|---|---|
| 日期 / 操作者 | 2026-09-23 上午 / 项目所有者（现场）+ AI（远程） | — |
| 主板 | Orange Pi Zero 3W，主机名 `orangepizero3w` | `MEASURED` |
| OS / 内核 | Armbian 26.8.1 trixie / `6.6.98-vendor-sun60iw2` | `MEASURED` |
| 根分区 UUID | `932dec6a-307e-4174-8694-c4a1ef18eef4`（与镜像一致） | `MEASURED` |
| 镜像 | `Zero3W-已配置WiFi与诊断.img`（含 WiFi/SSH 公钥/开机诊断/`video=` 参数） | `MEASURED` |
| **视频线** | **绿联 Mini HDMI↔HDMI**（替换首轮判定故障的随附线） | `MEASURED` |
| 屏幕 | **3.2 英寸** HDMI IPS 模块（尺寸由所有者 2026-09-23 更正，原记 3.5 英寸属误记；型号 `UNKNOWN`；仅 HDMI + `only power` USB-C 两口） | `UNKNOWN` |
| 主机供电 | 台式数控电源 **5.00V / 限流 3A** | `MEASURED` |
| 屏幕供电 | 5V 墙充 | `MEASURED` |
| 板端网络 | `192.168.2.184/24`，`wlan0` MAC `10:4d:05:af:a4:06`（与首轮一致） | `MEASURED` |
| 板端热区 | 42.2 °C（`thermal_zone0/5`） | `MEASURED` |
| 环境温度 | 未记录 | `UNKNOWN` |

## 2. 复测前的一段异常（记录在案）

复测开始时出现「指示灯规律闪 + **0.08A**、不发热」，一度误判为供电电压问题。最终确认为 **SD 卡不是镜像卡**：SoC 停在 boot ROM/bootloader 循环，找不到可启动介质。换回镜像卡后立即恢复 **5V / 0.4A** 正常启动。

**可复用判据**（与首轮一致）：

| 供电读数（5V 下） | 含义 |
|---|---|
| 0 A | 输出未开或线未接好 |
| **0.06–0.10 A + 规律闪灯** | 无可启动介质（卡不对/未插好） |
| **0.36–0.41 A + 闪灯** | 系统正常启动 |

## 3. 四项判据结果（全部通过）

| # | 判据 | 通过标准 | 实测 | 结论 |
|---|---|---|---|---|
| 1 | HPD 稳定性 | 120 秒内稳定 `connected`，跳变 ≤1 | `connected`，变化 1 次（连上后不再跳） | ✅ |
| 2 | EDID | 非 0 字节 | **256 字节**（首轮为 0） | ✅ |
| 3 | 真实分辨率 | 出现屏幕真实模式 | `480x800`、`1024x768`、`800x600`、`800x480`、`640x480`（共 5 个） | ✅ |
| 4 | 帧缓冲 | `enabled` + `/dev/fb0` | `enabled`；`fb0` virtual `480,1600`、stride 1920、bpp 32；fbcon 已绑定 | ✅ |

## 4. 关键原始证据

- **内核驱动日志**：`0.639s drm hdmi detect: connect` → `1.173s edid parse block0/block1 finish` → `drm get edid support modes: 5` → `drm hdmi mode set: 480*800` → `hdmi drv has been enable!` → `drm hdmi atomic enable`。
- **EDID 头 32 字节**：`00 ff ff ff ff ff ff 00 30 ae 86 10 01 01 01 01 22 15 01 03 80 0a 10 78 …`（厂商码 `30ae`、产品 `1086`、EDID 1.3）——证明面板确实回报了 EDID，而非驱动伪造。
- **面板 DTD 仍报错**：`dw edid parse dtd timing pixel clock[0KHz] invalid!` ×5。厂商驱动对此宽容，最终用 established timings + cmdline 强制模式取得 5 个可用模式。
- **cmdline 含** `video=HDMI-A-1:800x480@60`（首轮注入）。
- **当前生效模式是面板原生 `480×800`（竖屏）**：`800x480` 虽在模式列表内，但未被优先选中。
- 完整回显存档：`E:\OCLive\oclive-四季宝-artifacts\bringup-2026-09-11\display-link-retest-2026-09-23.txt`

## 5. 判定

1. **首轮"线缆 DDC 通道故障"的假设被证实**：更换品牌线后 EDID 从 `0` → `256` 字节、HPD 由反复跳变变为稳定。
2. **`GS-HW-002` 关闭**：视频链四项判据全部通过。
3. 板卡 HDMI 输出与厂商驱动在首轮已被证明正常；本轮补齐"物理链路可用"这一半。

## 6. 遗留项

1. **方向（新）**：面板原生 `480×800` 竖屏，而项目要求横向 `800×480` 使用。**不能靠改 mode 解决**（EDID 首选即 480×800）——控制台用 `fbcon=rotate:1`，未来图形 UI 由 renderer 旋转。需一次重启做视觉复核。
2. **触摸能力**仍未确认（`GS-HW-003`）。
3. **板卡身份**（RAM 容量、microSD 料号、屏幕型号、环境温度）仍为 `UNKNOWN`（`GS-HW-004`）。
4. 本轮未做 20 次冷启动与两小时 soak（`OPZ-B01`/`OPZ-M01` 的完整门）。

## 7. 下一步

1. **视觉确认**：确认屏幕现在是否显示控制台文本（本轮链路已通、fbcon 已绑定）。
2. **横向使用**：在 `/boot/armbianEnv.txt` 增加 `fbcon=rotate:1`（或 `3`，取决于实际装配方向）后重启，复核视觉效果。
3. 之后进入 **Host 与最低 UI**（`OPZ-E01`/`OPZ-V01` 序列），并补齐 `GS-HW-004` 的身份采集。
