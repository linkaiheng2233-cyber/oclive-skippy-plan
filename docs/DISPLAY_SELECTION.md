# 屏幕选型与冻结门

## 1. 当前结论

屏幕必须先于屏幕舱外壳、主电池和最终关节力矩冻结。v0.1 的首选工程基线是：

- 2.8 英寸、原生 240 × 320，装机默认横向 320 × 240。
- IPS、无触摸、户外高亮，目标 800–1,000 cd/m²；优先带抗眩光表面。
- 4-wire SPI 或 8080 并口；首个 Linux 原型使用 SPI，主板与显示背板留在屏幕后方。
- 角色表情 + 极简 HUD，不以视频、网页或桌面环境为负载目标。
- 以 Newhaven `NHD-2.8-240320AF-CSXP-F Rev1B` 为首选样屏，但只有样品通过本文件的 D1 验收后才锁料号。

首选样屏横放后的裸屏外廓约为 69.2 × 50 × 3.39 mm，能够把屏幕舱正面控制在约 74–78 × 54–58 mm 的工程区间。此前 `≤75 × 55 mm` 保留为美学目标，不再作为尚未看过实屏时的刚性承诺。

## 2. 为什么不是先选 HDMI/DSI 开发模块

3.5 英寸成品小屏模块的外形和裸屏差异很大：

| 候选 | 分辨率 | 亮度 | 模块/裸屏外廓 | 接口 | 当前判断 |
|------|--------|------|-----------------|------|----------|
| Newhaven NHD-2.8-240320AF-CSXP-F Rev1B | 240 × 320 | 1,000 cd/m² | 50 × 69.2 × 3.39 mm | SPI / 8080 | v0.1 首选样屏 |
| Newhaven NHD-3.5-640480EF-MSXP | 640 × 480 | 950 cd/m² | 76.9 × 63.9 × 3.2 mm | 4-lane MIPI DSI | 高分辨率升级候选 |
| Riverdi RVT35HHBNWN00 | 320 × 240 | 1,000 cd/m² | 76.9 × 63.9 × 8.77 mm；44 g | SPI/QSPI + BT817Q | 显示卸载备选，但同分辨率更大更厚 |
| Waveshare 3.5inch 480×800 LCD | 480 × 800 | 300 cd/m² | 88.87 × 52.56 × 7.15 mm | HDMI + USB/I²C touch | 桌面 bring-up 可用，不进入上枪外壳 |
| Waveshare 3.5inch DSI LCD (E) | 640 × 480 | 180 cd/m² | 成品触控模块 | Raspberry Pi DSI | 约 0.5 W，但户外亮度不足 |
| Newhaven NHD-3.5-HDMI-HR-RSXP | 640 × 480 | 950 cd/m² | 约 92 × 84.9 × 15.8 mm | HDMI/USB | 高亮且易接 Linux，但只适合台架 |

HDMI 最容易在任意 Linux 主机点亮，却把桥接芯片、连接器、PCB 和约 2.3 W 的显示模块功耗一起带进屏幕舱；小型 DSI 成品模块功耗低，但常见 180–300 cd/m² 版本不能代表 wargame 户外体验。最终产品应围绕裸屏和自己的显示/电源背板集成，成品 HDMI 屏只承担软件台架角色。

## 3. 首选样屏的工程含义

`NHD-2.8-240320AF-CSXP-F Rev1B` 的官方数据为：

- 外廓：50 × 69.2 × 3.39 mm；横放后为 69.2 × 50 mm。
- 可视区：44.2 × 58.6 mm；横放后为 58.6 × 44.2 mm。
- 240 × 320、IPS、全视角、无触摸、抗眩光、1,000 cd/m²。
- ST7789VI，3/4-wire SPI 或 8/16-bit 8080-II，40-pin 0.5 mm FFC。
- LCD 典型约 3.3 V / 8 mA，背光约 3.1 V / 160 mA；满亮显示部分的典型量级约 0.52 W，不包含 Linux 主板和 DC/DC 损耗。

这会直接冻结以下方向：

1. 默认 UI 画布为 320 × 240 landscape、RGB565；PNG/表情资产必须在这个画布上先验收。
2. v0.1 不做触摸。模式/维护使用实体键、调试接口或后续管理端，不让屏幕保护与手套操作受触摸层牵制。
3. 屏幕后增加一块显示背板：40-pin FFC、SPI/模式配置、复位、可选 TE、背光恒流/PWM、ESD 与测试点。
4. 背光不能直接由 GPIO 或电阻粗放供电；按 160 mA 级恒流与 PWM 调光设计，并实测 25/50/75/100% 档位的亮度、功耗和温升。
5. 裸屏无安装孔，必须使用刚性 carrier 和边缘支撑；前方增加可更换抗刮/低反射保护片，泡棉只压在非有效区和外壳承力边。

## 4. Linux 显示路线

应用继续保持无桌面环境：

```text
VisualCue
  → 320×240 scene renderer
  → dirty-rectangle / frame scheduler
  → DisplaySink
       ├── mock window/file sink (Windows/Linux test)
       ├── Linux SPI MIPI-DBI sink (primary hardware path)
       └── DRM/KMS sink (optional board/kernel path)
  → ST7789VI panel
```

- 首轮 SPI 使用 4-wire 模式，避免 9-bit SPI 控制器兼容性问题。
- 一帧 RGB565 为 153,600 bytes。32 MHz SPI 的纯线速理论上限约 38.4 ms/帧（约 26 fps）；实际帧率受控制器、复制和面板时序限制，必须实测，不以理论值承诺动画帧率。
- UI 采用状态画面、局部 HUD 和脏矩形刷新，不要求持续 30/60 fps。即时状态切换目标仍是事件进入到首个可见像素 p95 ≤150 ms。
- Linux 内核存在通用 MIPI-DBI SPI DRM 路径，但具体板卡内核、设备树、初始化序列和背光仍需集成；本仓保留直接 SPI backend，避免把项目成败绑死在某个发行版的显示 overlay 上。
- 如果 320 × 240 的真实 UI 评审失败，再进入 3.5 英寸 640 × 480 MIPI DSI。不能仅因参数更高就预先承担更大的正面外廓、4-lane DSI 转接和板级驱动风险。

## 5. 屏幕舱初始堆叠

以首选样屏为基线，从正面到后壳为：

```text
0.5–1.0 mm replaceable low-reflection protector
→ perimeter foam bumper, 0.8–1.5 mm proud of protector
→ recessed 69.2 × 50 × 3.39 mm LCD
→ rigid LCD carrier + 40-pin FFC strain relief
→ display/power carrier PCB
→ Linux compute board + antenna keep-out
→ protected 1S battery pocket
→ puncture-resistant rear cover + roll joint load path
```

暂用三套厚度假体 `18 / 22 / 26 mm` 验证观感、收纳和碰撞；这不是产品厚度承诺。实际厚度由开发板、连接器、电芯和安全间隙相加决定。前脸和厚度必须分开管理：裸屏可以很薄，但完整 Linux 屏幕舱不会等同于相机裸侧屏。

## 6. 采购与冻结顺序

### D0：不等硬件的软件门

- Windows mock 与所有角色图按 320 × 240 横屏渲染。
- 检查最小字号、表情辨识度、网络/电量/模式 HUD 占用和黑屏降级。
- 如果核心画面必须依赖 640 × 480 才能成立，立即触发 D2，不制作 2.8 英寸外壳。

### D1：2.8 英寸样屏门（推荐）

采购建议：1 块官方 breakout 用于台架，2 块 `NHD-2.8-240320AF-CSXP-F Rev1B` 裸屏用于背板与结构验证。到货先核对 Rev1B；Rev1A 的接口和亮度不能混用。

通过条件：

- 正午日照、树荫和室内三种环境均拍照记录；在预计观察距离和护目镜下表情/HUD 可辨。
- 分别测试普通透明片与低反射保护片；检查偏振护目镜横竖方向是否出现不可接受的变暗或黑屏。
- 25/50/75/100% 背光下记录输入功耗、屏面亮度、主板和电芯温升。
- 连续刷新 2 小时无花屏、撕裂、卡死和内存增长；事件到首个可见像素 p95 ≤150 ms。
- 冷启动、异常断电恢复、屏灭/唤醒和 1,000 次 FFC/关节等效弯折后仍正常。
- 装入保护片、泡棉、carrier 和背板后的实测外廓、厚度、质量、重心进入机械台账。

D1 通过后，才能冻结屏幕舱前脸、显示背板和新的三档完整舱体配重。

### D2：高分辨率升级门

仅在以下任一条件成立时评估 3.5 英寸 640 × 480 MIPI DSI：

- 320 × 240 下核心角色表情或必须存在的 HUD 无法辨识。
- SPI 实测刷新延迟无法满足即时反馈，且局部刷新仍不能解决。
- Linux 主板已证明能稳定驱动目标 4-lane MIPI 面板，且 76.9 × 63.9 mm 裸屏和更大外壳通过 RADIAN 包络检查。

## 7. 样屏否决项

- 户外画面需要长期满亮仍不可辨，或保护片反射抵消高亮优势。
- 与常用偏振护目镜组合时关键姿态黑屏。
- 显示背板、主板与电池无法在不压迫电芯的前提下进入可接受包络。
- SPI 刷新阻塞传感器/状态机，事件首像素延迟超标且无法通过独立线程、DMA 或局部刷新修正。
- 采购版本不可控、生命周期不可接受，或同料号修订导致接口/亮度不兼容。

## 8. 官方资料

- [Newhaven NHD-2.8-240320AF-CSXP-F 产品页](https://newhavendisplay.com/2-8-inch-ips-tft-without-touchscreen/)
- [Newhaven NHD-2.8-240320AF-CSXP-F 规格书](https://newhavendisplay.com/content/specs/NHD-2.8-240320AF-CSXP-F.pdf)
- [Newhaven 2.8 英寸 Rev1A → Rev1B Transition Guide](https://newhavendisplay.com/content/docs/NHD-2.8-240320AF-CSXP-F_TransitionGuide.pdf)
- [Newhaven NHD-3.5-640480EF-MSXP 产品页](https://newhavendisplay.com/3-5-inch-ips-tft-mipi-interface-without-touchscreen/)
- [Newhaven NHD-3.5-640480EF-MSXP 规格书](https://newhavendisplay.com/content/specs/NHD-3.5-640480EF-MSXP.pdf)
- [Riverdi RVT35HHBNWN00 规格书](https://download.riverdi.com/RVT35HHBNWN00/DS_RVT35HHBNWN00_Rev.1.7.pdf)
- [Waveshare 3.5inch 480×800 LCD](https://www.waveshare.com/wiki/3.5inch_480x800_LCD)
- [Waveshare 3.5inch DSI LCD (E)](https://www.waveshare.com/wiki/3.5inch_DSI_LCD_%28E%29)
- [Newhaven NHD-3.5-HDMI-HR-RSXP 规格书](https://newhavendisplay.com/content/specs/NHD-3.5-HDMI-HR-RSXP.pdf)
- [Linux MIPI-DBI DRM helper](https://docs.kernel.org/5.19/gpu/drm-kms-helpers.html)
