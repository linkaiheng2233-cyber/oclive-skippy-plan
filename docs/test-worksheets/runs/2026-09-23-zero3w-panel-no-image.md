# Zero 3W 小屏出图排查（2026-09-23 下午）

**状态**：EVIDENCE / 未解决，已排除多个假设  
**唯一职责**：记录"链路判据全通过但面板不出画面"这一现象的排查过程、被证伪的假设、以及对厂商驱动的两个重要限制。  
**上游**：`2026-09-23-zero3w-display-link-retest.md`（链路四项判据通过）  
**下游**：`../TECHNICAL_DEBT.md`（`GS-HW-005` 扩展）、`../DISPLAY_SELECTION.md`（选型风险）

## 1. 现象

链路参数全部正常，但屏幕只有**一条竖线**（背光会随模式变化亮起）：

| 已确认正常 | 值 |
|---|---|
| HPD | `connected` 稳定 |
| EDID | 驱动内部解析成功：`edid parse block0/block1 finish`、`drm get edid support modes: 5` |
| 模式选择 | `drm hdmi mode set: 480*800`、`hdmi drv has been enable!`、`drm hdmi atomic enable` |
| CRTC / 平面 | `crtc[99] mode: "480x800": 60 34860 480 500 510 700 800 801 803 830`；`plane[92] crtc=DE-0 fb=163` |
| 帧缓冲 | `/dev/fb0`，virtual `480,1600`，fbcon 已绑定 |

**面板行为**：背光随模式切换亮起（说明收到了信号并尝试锁定），但画面只有一条竖线。

## 2. 关键交叉证据（改变结论方向）

**同一块小屏在 PC 上能正常显示桌面**（项目所有者实测，使用另一根标准 HDMI 线）。  
⇒ 面板、面板 EDID、面板所需时序**本身可用**；问题出在"板子 → 面板"这一侧。

## 3. 被证伪或被排除的假设

| 假设 | 结论 | 依据 |
|---|---|---|
| 线缆 DDC 故障 | 已排除 | 换绿联线后 EDID/HPD 全部正常（见上一份运行记录） |
| 分辨率/模式未设置 | 已排除 | CRTC 与平面均在按面板 EDID 的原生时序扫描输出 |
| `video=` 参数未生效导致模式错 | **确认为真但非根因** | 驱动忽略 `video=`，优先用 EDID 首选模式；即使用 EDID 原生模式仍无画面 |
| 面板 EDID 是错的（克隆 EDID） | 部分成立 | 面板名为 `HDMI480x800HH`，DTD 图像尺寸字段是垃圾（480×800 **mm**）、DTS 解析报 `pixel clock[0KHz] invalid`；但面板在 PC 上可用，说明其实际所需时序可被 PC 满足 |
| 用 EDID 覆盖强制微雪官方时序可解决 | **无效（功能不存在）** | 本内核 `drm_kms_helper` **没有 `edid_firmware` 参数**（只有 `drm_fbdev_overalloc`/`fbdev_emulation`/`poll`），EDID 覆盖未编入内核；自制的 `edid/hdmi480x800-waveshare.bin` 未被加载 |

## 4. 本轮吃到的两个驱动限制（重要，勿重复踩）

1. **不能用 `modetest` 改模式**：对连接器执行 `modetest -s 146@99:<mode>` 会让 **SSH 会话断开并使板子崩溃重启**（实测两次，一次直接重启）。厂商驱动的 atomic commit 路径不安全。→ 想换模式只能改内核命令行并在**开机时**生效。
2. **不能靠 `video=` 覆盖 EDID 首选模式**：驱动在 1.16s 按 EDID preferred 选定 `480*800`，命令行里的 `video=HDMI-A-1:800x480@60` 只作为额外模式（`type: userdef`）存在，不会被优先选中。

## 5. 当前最可能的根因（待验证）

驱动日志有一行值得注意：

```
sunxi hdmi select vic 0 use hdmi14 vsif
```

`VIC 0` 表示该模式**不在 CEA 标准模式表内**（480×800 本就是非 CEA 模式），驱动改用 **HDMI 1.4 vendor-specific infoframe** 发送。廉价屏的 HDMI 接收芯片常在这种"非标准模式 + 厂商信息帧"组合下无法正常同步；而 PC 显卡对同一模式的处理路径不同，因此 PC 能显示、板子不能。

其他同向可能：板子 HDMI 输出对这块屏的**信号裕量偏紧**（TMDS 幅度/预加重），加一级中继通常可解。

## 6. 下一步（按性价比）

| 方案 | 成本 | 能判定什么 |
|---|---|---|
| **① 板子接 PC 显示器（用同一根绿联线）** | 免费 | 决定性：显示器出画面 → 板子与线都没问题，问题在小屏兼容性；显示器也不出 → 板子输出或线有问题 |
| ② 板子 → **USB-C DP Alt → HDMI 主动式转接头** → 小屏 | 30–80 元 | 绕开 Mini HDMI 与厂商 HDMI 驱动路径，同时验证 DP 通道 |
| ③ 板子 → **HDMI 分配器/中继（带均衡）** → 小屏 | 20–40 元 | 若为信号裕量问题，通常可直接解决 |
| ④ 走 ADR-063 的 **PC 主机形态**：屏幕直接由 PC 驱动 | 0 元 | 与"PC 前置"形态天然一致——板端显示问题不再阻塞该形态 |

## 7. 证据位置

- 链路四项判据原始回显：`E:\OCLive\oclive-四季宝-artifacts\bringup-2026-09-11\display-link-retest-2026-09-23.txt`
- 自制 EDID（微雪官方时序，未被内核加载）：`...\bringup-toolchain\edid-480x800-waveshare.bin`
- 板端备份：`/boot/armbianEnv.txt.bak`（含 `edid_firmware` 参数的版本保留在位，无害）
