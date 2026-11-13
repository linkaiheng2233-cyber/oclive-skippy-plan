# 四季宝器灵近期工作总结与 DeepSeek 接手文档

> 本文记录 11 月 13 日的早期交接。接手时先读 [AI 对接文档](AI对接文档.md)获取当前提交、门禁、下一步检查点及回填方式。

**状态**　交接快照，最后核对于 2026-11-13（北京时间）。  
**适用阶段**　A.I.Live-ai枪娘器灵 P0-A 桌面原型。  
**用途**　供项目所有者回顾近期工作，也可单文件交给 DeepSeek 做工程评审。项目决定、实测与后续验收仍由各自来源维护。  
**上游**　[项目边界](../PROJECT_BOUNDARIES.md)、[项目决定](../DECISIONS.md)、[硬件实施计划](../HARDWARE_IMPLEMENTATION_PLAN.md)、[实机工作表](../test-worksheets/01-硬件到货清单.md)。  
**续跑入口**　[当前基线](../PROJECT_BASELINE.md)、[实施路线](../ROADMAP.md)、[接手索引](README.md)。

## 先读这里

项目给 Wargame 玩具载体做一套可拆的小屏角色配件。当前用 **晶彩 JC3248W535 带壳一体屏**做台架原型，后续在**悍迪 191**上试装。板上的 ESP32-S3 负责感知和即时显示，Windows 电脑负责本地 OCLive 角色运行。调试先走 USB-C，正常联机计划走局域网 Wi-Fi。套件独立供电的验证从 5 V USB 开始。

目前实机已经跑通**点屏、基础触摸和 USB 表情控制**。七张沐沐图片已打入固件，电脑经串口验证了换图顺序和双向循环，**图库画面与触屏按钮仍待现场验收**。样机最后一次操作已切回出厂程序启动项。下一段是把现有 Rust 感知内核放到 ESP32-S3 上回放，先用模拟输入把板端状态和 PC 表情反馈接通，再接首颗 IMU 并复测 Wi-Fi。感知移植、传感器、无线及真实 OCLive 联机均待实施或实测。

## 近期工作与证据

| 时间 | 工作进展 | 交接时的证据与余项 |
| --- | --- | --- |
| 09-23 至 09-29 | 确定 PC 宿主形态为 P0-A 主线。Orange Pi Zero 3W 曾运行 Linux 无头 Host，以 mock LLM 完成一个角色回合。 | 这是后置 P0-B 的历史证据，显示链仍有未通过项。Linux 小板、HDMI 适配和独立主机舱由该路线继续保留。 [Orange Pi 无头 Host 实测](../test-worksheets/runs/2026-09-29-zero3w-host-headless-bringup.md) · [显示复测](../test-worksheets/runs/2026-09-23-zero3w-display-link-retest.md) |
| 10-12 | Rust 感知内核完成单节点能力集泛化。协议升到 v0.3，同一节点用 `GripContact.grip` 区分前握、后握。`AngularRateSample` 是独立消息。 | 现有桌面模拟器有 9 帧逐字段回归证据，既有命令行回放已具备。`NodeHello` 版本与能力协商及正式 `AssetRef`/表达下行仍在 `GS-CONTRACT-004`。 [ADR-067](../DECISIONS.md#adr-067节点模型泛化落地单板形态强制契约-v03角速率改走独立指向流) · [技术债](../TECHNICAL_DEBT.md) |
| 10-30 | 历史交接稿移出项目源码，保留当前基线、路线、ADR、领域文档和实测工作表为阅读入口。 | 本文记录本阶段进度。架构与协议以相关决定、代码和证据为准。 [文档体系](../DOCUMENTATION_SYSTEM.md) |
| 11-09 | 锁定晶彩 GUITION JC3248W535 系列一体板。所有者确认 ¥112 对应完整 ESP32-S3、3.5 英寸电容触摸 IPS 屏和原装壳套餐，无赠品。已归档厂商规格书、使用手册、V1.0 原理图、结构图/DXF、Arduino LVGL 示例和 BSP 快照。 | 厂商规格对应 `JC3248W535C_I_Y`，原生 320×480，原型以 480×320 横屏使用。淘宝参数栏的 320×240 和宣传图的 `JC4832W535` 与原厂资料冲突。实物完整型号后缀和 PCB 版本待读丝印。屏幕尺寸上限为 3.5 英寸，¥112 仅为板屏套装价格。LILYGO T4-S3 保留为更小的 AMOLED 候选，分体 ESP32-S3 加屏幕后置。 [ADR-068](../DECISIONS.md#adr-068首版采用晶彩板屏一体方案成本优先amoled-按需评估) · [硬件实施计划 §0.1](../HARDWARE_IMPLEMENTATION_PLAN.md) |
| 11-13 到货 | 出厂界面上电可见，所有者实际使用了自带触摸测试，主观评价屏幕观感较好。Windows 识别 `VID_303A&PID_1001`，当时为 `COM3`。读取确认 ESP32-S3 芯片 revision v0.2、16 MB Flash、8 MB PSRAM。 | 触摸和显示为实际观察，亮度、户外可读性、整机尺寸和功耗仍待测。芯片 revision 与待核对的 PCB revision 是两个字段。 [实机工作表](../test-worksheets/01-硬件到货清单.md) |
| 11-13 基础原型 | 以 Arduino-ESP32 3.0.2、LVGL 8.3.11 和原厂 AXS15231B 屏幕驱动跑通。582,048 B 应用烧录及回读校验通过，电脑发出 `idle`、`curious`、`happy` 收到回执，现场确认画面和点击计数，串口收到 `TOUCH 1` 至 `3`。 | 此次验收对应基础表情与触摸原型。 [实机工作表](../test-worksheets/01-硬件到货清单.md) · [固件说明](../../firmware/jc3248w535/README.md) |
| 11-13 沐沐图库 | 复用 OCLive Chat Pro 的 `mumu` 立绘（源快照 `90677e53`），构建时打包成七张 260×260 RGB565 图片并写入 Flash。固件预留 `Prev` 和 `Next`，串口已验证两方向的首尾循环和七张顺序。 | 平静、开心、兴奋、害羞、疑惑、难过、生气。当前镜像 1,537,328 B，`app1` 容量 2,097,152 B，镜像剩余空间 559,824 B。编译报告 70,704 B 全局变量为静态占用。180 秒监听收到 0 条图库点击记录，期间也没有现场操作确认，画面及按钮功能待复验。 [实机工作表](../test-worksheets/01-硬件到货清单.md) |
| 11-13 架构与收尾 | ADR-069 接受板上感知、PC 本地角色、Wi-Fi 联机的方向。16:45 按所有者要求切回出厂 `app0`，引导程序、分区表和 `app0` 均与整片备份吻合，启动选择写入回读校验通过。 | USB 重枚举后 `COM3` 返回，沐沐原型握手已停止响应。出厂画面待现场观察。沐沐应用仍保存在 `app1`，板上感知与联机还在设计阶段。 [ADR-069](../DECISIONS.md#adr-069p0-a-由-esp32-s3-运行感知内核) · [实机工作表](../test-worksheets/01-硬件到货清单.md) |

## 当前架构与硬件约束

| 位置 | 要承担的职责 |
| --- | --- |
| ESP32-S3 | 传感器采样、滤波与去抖，生成板内 `SensorObservation`。复用 Rust `PerceptionReducer` 融合状态、推进超时并推导变化，使用本地表情资源提供即时反馈。缺失或过期的传感信息、缺少有效校准的姿态保持 `unknown`。 |
| USB-C / Wi-Fi | 同一组语义先在 USB 联调，再用于本地网络。计划由 ESP32 以 2.4 GHz Wi-Fi STA 加入现有路由器，主动建立到指定 PC IP 的 TCP 长连接。上行是 `PerceptionState` 快照与类型化事件，下行先验证现有表情 ID。消息封装、连接校验和超时仍待设计与实测。 |
| Windows PC | 最小台架程序先收状态、回表情 ID。后续由 Host 校验来源、新鲜度和重复事件，再通过既有 sensor origin 适配进入 OCLive，由 PC 处理角色、记忆、表达及输出仲裁。OCLive 在本地运行，板上即时反馈与 PC 回合独立。 |

初连、重连以板端当前快照建立基线，之后处理新的实时事件。用设备启动标识、事件标识和 `revision` 区分重启、重复数据和过期表情回复，分别记录设备单调时钟与 PC 接收时间。重复快照不触发角色回合。掉线时板端继续感知和本地反馈，PC 将远端状态标记为过期。正式语义消息仍待实现，现有 `PING`、`STATE`、`STEP`、`GET STATE`、`TOUCH` 和 `MOOD` 属于台架串口调试。 [架构决定](../DECISIONS.md#adr-069p0-a-由-esp32-s3-运行感知内核) · [固件说明](../../firmware/jc3248w535/README.md)

- **先接一颗六轴 IMU。** ICM-42688-P 3.3 V I²C 模块优先评估，QMI8658 为候选。按原理图，P3、P4 接同一组 3.3 V、GND、GPIO17/18；选一个插座走一条短四芯线，台架先核对针序、SDA/SCL、上拉、芯片 ID 和方向。IMU 固定在导轨底座参考侧，若屏幕可翻转，IMU 留在固定侧。 [硬件实施计划 §0.1.2](../HARDWARE_IMPLEMENTATION_PLAN.md#012-单板传感器与电脑无线连接)
- **握持后接。** 薄后握套可试一片 FSR，GPIO6 为候选 ADC，GPIO7 留给可能的第二片。安装阈值、预压、戴手套和走线待实测。IMU 单独可验快照中的运动、静止和校准姿态。当前规则要求后握持和抬起姿态才能判 `Ready`，`Standby` 要求前后握明确松开且静止达到时长。握持传感器缺失时保持 `unknown`。v0.3 的 `DeviceEventKind` 包含载体握持、就绪及主控触点变化，首轮用模拟回放覆盖这些事件。 [状态决策源码](../../crates/ailive-gun-spirit-perception-core/src/reducer.rs) · [协议类型](../../crates/ailive-gun-spirit-contracts/src/lib.rs) · [硬件实施计划](../HARDWARE_IMPLEMENTATION_PLAN.md)
- **先独立台架，再上载体。** 悍迪 191 使用可拆、非磁性机械底座，线留在附件壳或附件盖线槽内。实物测含插头厚度、活动件净空和按钮可达性。先独立 5 V USB 供电跑无线，再实测电池口极性、充电与供电路径、亮屏联网功耗和装壳余量。电池型号、续航和最终安装方案待定。 [机械接口](../MECHANICAL_INTERFACE.md) · [硬件实施计划](../HARDWARE_IMPLEMENTATION_PLAN.md)
- **本轮范围。** 先做 Wargame 玩具载体的外置感知和角色显示，保持附件独立。发行版、语音、摄像头、远端 BLE 节点、Linux 一体舱及载体内部接口后置。项目边界将火控、自动瞄准、弹道和供弹控制排除在外。 [项目边界](../PROJECT_BOUNDARIES.md) · [实施路线](../ROADMAP.md)

## 恢复工作顺序

1. **现场补验图库。** 用已归档图库应用及同板出厂备份，按固件说明进入下载模式和选择应用。样机 `app0` 在 `0x10000`，测试 `app1` 在 `0x210000`。安装脚本检查出厂区域，校验 `app1` 回读后切换启动项；`--factory` 可恢复出厂启动项。`COM3` 是当时端口，开测时按枚举结果填写。运行 `./firmware/jc3248w535/test-usb.ps1 -Port COM3 -CheckGallery -WatchSeconds 45 -MinimumTaps 3`，在 `Calm 1 / 7` 按 `Next`、`Next`、`Prev`，预期显示 `Happy 2 / 7`、`Excited 3 / 7`、`Happy 2 / 7`，同时记录三对 `TOUCH` 和 `MOOD`。 [构建与实机命令](../../firmware/jc3248w535/README.md)
2. **共享感知内核做最小板端回放。** 从现有桌面回放取可重复输入，先验证支持 `std` 的 ESP-IDF Rust 目标（候选 `xtensa-esp32s3-espidf`），将同一输入在桌面和板上逐项对比状态、`unknown`、超时及事件次序。现有 `PerceptionReplay` 推导变化类型，完整 `DeviceEvent` 的 ID 和封装在桌面模拟器生成，板端需定义与既有契约的映射。记录应用镜像大小、内部堆和 PSRAM 的运行峰值及触摸响应，再决定与 Arduino/LVGL 固件如何集成。 [感知回放](../../crates/ailive-gun-spirit-perception-core/src/lib.rs) · [桌面模拟器](../../crates/ailive-gun-spirit-host/src/bin/simulator.rs) · [回归测试](../../crates/ailive-gun-spirit-host/tests/golden_frames.rs)
3. **用 USB 跑最小 PC 闭环。** 以板端模拟输入产生快照和事件，由 Windows 测试程序读取并回一条表情 ID，确认图库切换。验证重复快照、掉线、重启、过期回复，以及事件与状态版本的对应关系。 [协议类型](../../crates/ailive-gun-spirit-contracts/src/lib.rs) · [Host](../../crates/ailive-gun-spirit-host/src/lib.rs)
4. **接入一颗 IMU，再用 Wi-Fi 重做闭环。** 先核对芯片 ID、轴向、两姿态校准、运动/静止与过期。维持独立供电，切断并恢复 PC 会话，验证板端仍有本地反馈、PC 以新快照恢复、旧事件与过期回复被丢弃。记录恢复用时、状态链路延迟分布和丢失情况。 [硬件实施计划 §0.1.2](../HARDWARE_IMPLEMENTATION_PLAN.md#012-单板传感器与电脑无线连接)
5. **证据齐了再加装配与角色联动。** 用电流、温升和装配数据选电池和外壳，再评估 FSR 接入与实际安装。正式跨仓桌面联调按现行路线暂缓，待所有者完成 OCLive 内核梳理后，利用现有 `oclive_adapter` 核对 sensor 来源和角色副作用边界。Windows 台架测试程序可先独立推进。 [OCLive 适配器](../../crates/ailive-gun-spirit-host/src/oclive_adapter.rs) · [实施路线](../ROADMAP.md)

## 接手阅读索引

上传这份 Markdown 即可先做架构评审。表中的相对链接用于本地仓库，外部会话需要核查源码或实测时可按行补对应文件。工程仓库为 `E:/OCLive/oclive-四季宝器灵`，OCLive 通用内核和正式角色资产在兄弟仓 `E:/OCLive/oclivenewnew`。

| 优先级 | 读什么 | 解决什么问题 |
| --- | --- | --- |
| 1 | [ADR-068/069](../DECISIONS.md#adr-068首版采用晶彩板屏一体方案成本优先amoled-按需评估)、[项目边界](../PROJECT_BOUNDARIES.md)、[ESP32 方案](../ESP32_GUN_ASSISTANT.md) | 当前选型、板端与 PC 职责、本轮范围。老文档出现 v0.2、旧板屏和 P0-B 双 BLE 拓扑时，以 ADR-067 至 ADR-069、v0.3 Rust 类型与当前硬件实施计划为准。 |
| 2 | [到货与实机工作表](../test-worksheets/01-硬件到货清单.md)、[硬件实施计划 §0.1](../HARDWARE_IMPLEMENTATION_PLAN.md)、[固件 README](../../firmware/jc3248w535/README.md) | 证据、原厂资料索引、实物接口、应用分区和验证方法。 |
| 3 | [协议](../../crates/ailive-gun-spirit-contracts/src/lib.rs)、[状态机](../../crates/ailive-gun-spirit-perception-core/src/reducer.rs)、[回放](../../crates/ailive-gun-spirit-perception-core/src/lib.rs)、[桌面模拟器](../../crates/ailive-gun-spirit-host/src/bin/simulator.rs) | 明确板上复用范围、`ingest` / `tick`、状态到事件及设备启动会话到现有契约的映射。 |
| 4 | [晶彩显示入口](../../firmware/jc3248w535/Jc3248W535/display_app.cpp)、[固件构建脚本](../../firmware/jc3248w535/build.ps1)、[素材打包脚本](../../firmware/jc3248w535/pack-portraits.py)、[OCLive 适配器](../../crates/ailive-gun-spirit-host/src/oclive_adapter.rs) | 核查现有 Arduino/LVGL 入口、素材体积和 PC 角色边界。 |

**版本基准。** 交接时分支为 `docs/personal-document-archive-20261030`，基准提交 `603eb40`。11 月方案文档改动和 `firmware/jc3248w535/` 仍在本地工作区，按此提交克隆只能得到历史基准。给 DeepSeek 做代码级评审时，需另外附上上表对应的当前源码与文档。

**当前 Rust 门禁。** 交接时运行 `scripts/verify.ps1`，三个包的格式与 workspace 边界检查通过。Clippy 在 `--locked` 下报告 `Cargo.lock` 需要更新，脚本在此终止，其后的测试、Schema 检查和审计尚未重验。 [验证脚本](../../scripts/verify.ps1) · [技术债与既有门禁记录](../TECHNICAL_DEBT.md)

**本地工件。** 原厂资料入口为 `E:/OCLive/oclive-四季宝-artifacts/hardware/jc3248w535/2026-11-09/README.md`。11 月 13 日同层日期目录下，`prototype/touch-test.log` 记录首版触摸回传，`mumu-gallery/` 含图库应用、构建日志、串口记录与 `mumu-assets.json`，`factory-backup/` 含本板整片备份与恢复日志。外部评审需要复核时优先提供脱敏日志或素材清单。整片 Flash 是本地恢复工件。 [实机工作表与 SHA-256](../test-worksheets/01-硬件到货清单.md)

## 可以直接发给 DeepSeek 的评审问题

```text
请根据附件评审四季宝器灵 P0-A 的下一轮实现，并给出一个最小可验收的实施顺序。

1. 现有 Rust 感知内核使用 std，目标是同一套状态机在 Windows 和 ESP32-S3 上回放。请给出最小板端验证方案，并比较如何与当前 Arduino-ESP32/LVGL 固件集成。图库镜像为 1,537,328 B，应用槽为 2,097,152 B。请列出要测的固件大小、内部内存和 PSRAM 指标、通过条件及失败后的可逆备选。目标或库的兼容性请标注依据和待验证项。
2. 请给出 USB 和局域网 TCP 共用的最小消息语义。覆盖初连/重连快照、启动会话、事件标识和 revision、重复/过期判定、PC 表情回传，说明与现有 v0.3 DeviceEvent 的映射。指出最小台架能静态配置什么，哪些适合留到正式 NodeHello 和 AssetRef 类型。
3. 首轮只有一颗 IMU，握持数据保持 unknown。请给出可在台架复现的输入和预期快照，覆盖运动、校准姿态、观测过期、PC 掉线、重启和本地反馈。用模拟握持输入补齐现有 DeviceEvent 的验收。

请按实施建议、所需改动位置、逐步验收、待测假设组织回答。当前约束是沿用晶彩样机与沐沐图库、保持外置配件形态、PC 本地运行 OCLive，首轮范围为板端感知和桌面闭环。附件中带路径的文件可能只存在本地，请将未拿到的源码和硬件证据明确列为待核对项。
```
