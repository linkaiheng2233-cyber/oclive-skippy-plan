# 关键决策记录

## ADR-001：四季宝使用独立仓库

状态：Accepted（2026-08-24）

OCLive 主仓保持通用角色运行时；四季宝的状态机、模拟器、ARM 宿主和硬件驱动放在独立仓库。跨宿主都需要的输入来源契约回到 OCLive 主仓实现。

原因：避免硬件依赖污染通用内核，同时保留本地 path dependency 的快速协同开发。

## ADR-002：器灵是平台层，四季宝是首个宿主

状态：Accepted（2026-08-24）

「器灵」表示物理事件与角色表达之间的通用适配层；「四季宝」只负责 wargame 实体形态。v0.1 不为机器人、车辆和智能家居提前设计复杂插件系统。

## ADR-003：禁止用文本前缀代替输入来源

状态：Accepted（2026-08-24）

`[系统事件]` 可以作为给模型看的渲染文本，但不能承担安全或持久化语义。回合必须有类型化 `user | sensor | system` 来源；sensor 默认不进入用户情绪、记忆、关系和人格演化。

## ADR-004：即时层不等待 LLM

状态：Accepted（2026-08-24）

屏幕醒来、固定视觉确认和状态切换由本地状态机立即执行。OCLive 动态回复是后到的角色表达，失败或迟到不得破坏物理状态。

## ADR-005：v0.1 只验证器灵闭环

状态：Accepted（2026-08-24）

v0.1 使用 AN94、两种模式、四个核心动作、PNG 和极简 HUD。ASR、TTS、扬声器、预渲染音频、触觉、心率、相机、AI 视觉、精确弹药计数、Live2D、多 BLE 节点和手机本地模型全部后置。

## ADR-006：原型硬件由实测决定

状态：Accepted（2026-08-24；开发/部署分层由 ADR-020 更新）

保留「大板开发、小板部署」，但不在桌面闭环前按历史价格锁定 Pi/Orange Pi、电池和外壳。S4 先用桌面电源，实测 RSS、温度、相机/屏幕需求后再冻结板卡。

## ADR-007：v0.1 只打通传感器、屏幕与主机

状态：Accepted（2026-08-24）

第一条硬件纵向路线固定为「传感器 → DeviceEvent → 器灵状态机 → OCLive → 屏幕」。语音和触觉均不进入首轮实现；只有这条路线在 Windows 模拟器、Linux x86_64 和目标 ARMv7/ARM64 实机上依次通过后，才恢复其它输出通道。

## ADR-008：首台原型载体固定为 RADIAN MODEL 1

状态：Accepted（2026-08-24）

首台上枪验证使用森柏龙 RADIAN MODEL 1。实体尺寸、导轨槽位、瞄具/操作净空和重心以开发者手中实物测量为 SSOT。v0.1 仅外挂安装，不连接内部火控、扳机或供弹机构。

## ADR-009：kit-first、cloud-first，但板端不是哑终端

状态：Partially superseded by ADR-021（2026-08-25）

先把开发板、屏幕、IMU、按钮和 Grip Node 做成与载体无关的桌面套件，通过软件、电子和 Linux 稳定性验收后才设计外壳与导轨夹具。LLM 重推理走云端；板端仍负责物理状态、屏幕即时反馈、轻量 OCLive Host、网络降级和系统生命周期。

## ADR-010：通用电子核心 + 可更换安装底座

状态：Accepted（2026-08-24）

套件不围绕 RADIAN 外形做一次性电子设计。主机、屏幕和通用传感器组成统一电子核心，首发底座兼容标准皮卡汀尼导轨；其它安装标准通过独立底座/转接件扩展。RADIAN MODEL 1 是第一套适配验证，不是唯一目标载体。

## ADR-011：无内部走线为默认传感器边界

状态：Accepted（2026-08-24）

发射器内部空间紧张，基础款不要求拆壳或内部走线。v0.1 使用主模块内置 IMU 和物理按钮，以及一个 BLE 自供电 Grip Node。外部有线仅用于台架采样，载体内部火控信号只可能成为未来专用适配器，不进入通用套件契约。

## ADR-012：外部传感器采用 BLE 功能节点

状态：Superseded in topology detail by ADR-022（BLE功能节点原则仍有效）

状态：Accepted（2026-08-24）

远端传感器按功能组成节点，而不是每颗传感器一个节点。v0.1 只实现一个压力式 Grip Node，Shoulder Node 在握持节点验收后再评审。节点完成采样、去抖、迟滞与校准，只发送低频 `SensorObservation`；导轨主机融合握持与运动证据后才生成 `DeviceEvent`。节点断连使相关事实进入未知态，不能沿用最后一次握持状态。

原因：无线节点满足不拆壳、不走机内线和跨载体迁移；单独的观察层则避免 BLE 节点直接耦合角色状态机，也为多传感器交叉验证保留边界。

## ADR-013：下场状态不依赖磁性底座

状态：Accepted（2026-08-24）

v0.1 删除霍尔 + 放置点磁体和 `dock.entered/exited`。Standby 由 grip released + motion idle 的稳定时间窗产生；Held/Ready 由握持压力和主机 IMU 组合判断。Grip Node 断连时握持进入未知态，状态机使用更长的静止超时降级，不能假装仍在握持。

原因：wargame 下场没有固定放置点，挂载、跑动和临时放置都必须在同一套自主状态逻辑中工作。

## ADR-014：Grip Node 下场默认使用可更换 CR2032

状态：Superseded by ADR-022 / ADR-028（2026-08-26）

状态：Accepted（2026-08-24）

Grip Node 台架阶段使用桌面电源；下场 Alpha 默认使用 CR2032 一次电池，CR1632 仅作为尺寸不足时的紧凑备选。v0.1 不为每个节点增加磁吸充电、USB-C 或位于握把受力区的软包锂电。节点上报毫伏值与 normal/low/critical 状态，不显示未经整节点放电测试支持的线性百分比。

原因：CR2032 可在场地快速更换，220 mAh 标称容量比同厚度 CR1632 的 140 mAh 留有更大余量；代价是 20 mm 直径和射频峰值压降，必须通过储能设计、低温/低电测试和实际重连波形验证。

## ADR-015：主机使用屏幕后置 1S 可充电池

状态：Partially superseded by ADR-021（2026-08-25；电池留在主机舱内，1S/2S 按目标板重新实测）

Linux 主机、屏幕、固定 IMU 和 BLE Central 由屏幕后方的受保护 1S 可充锂电包供电，经带 NTC、输入保护和真正 power-path/load-sharing 的充电管理接入主板。USB-C 边充边用不应导致重启。具体电芯容量与屏幕舱厚度必须在屏亮/屏灭、Wi-Fi、BLE 和本地 OCLive/可选网络回合功耗实测后冻结；4 小时为首个场地 Alpha 目标，8 小时为设计目标。

该主电池与 Grip Node 的 CR2032 是两套独立电源。v0.1 不做磁吸充电、双电池热插拔或未经验证的线性电量百分比。

原因：屏幕后方提供了与相机侧屏相近的平面面积，可以集中主板、显示、电池、充电口与天线设计；但 Linux 平均功耗决定能量和厚度，不能从开发板尺寸反推续航。

## ADR-016：显示机构使用刚性导轨座 + 三轴恒扭矩屏幕舱

状态：Accepted（2026-08-24；主板布局由 ADR-021 更新）

屏幕正面采用约 3.0 英寸相机侧翻屏尺度，yaw/pitch/roll 三轴负责从不同导轨位置调整视角。导轨夹具刚性锁定，不使用沿导轨滑移产生阻尼；三个转轴使用恒扭矩/预紧摩擦并带硬限位，收纳使用机械卡位而非磁吸。屏幕边缘设置高回弹缓冲框，使贴合平面时泡棉先接触，LCD 与电池不承力。

载体参考 IMU 必须位于固定导轨根部，不能跟随屏幕舱旋转。屏幕舱优先容纳主板与电池以缩短显示排线；移动质量和固定主板方案通过等质量假体、转轴力矩与跨轴线缆循环对比后最终冻结。

原因：三轴能兼容顶部/侧面、靠前/靠后的安装位置，又能保持屏幕可读和贴身收纳；固定传感器根部则避免调屏动作污染持枪姿态判断。

## ADR-017：三轴采用串联摩擦关节，不采用微型气弹簧

状态：Accepted（2026-08-24；初始质量/力矩档由 ADR-021 延续实测冻结）

三轴实现固定为：导轨根部圆盘 yaw 关节 → 25–35 mm 短连杆 → 双耳 pitch 关节 → 屏幕后盖中心 roll 圆盘关节。每轴由肩轴、止推片、摩擦片、碟簧/波簧、锁紧螺母和独立硬限位组成；导轨夹具不参与运动。V0 初始力矩区间分别为 yaw 0.10–0.18 N·m、pitch 0.18–0.30 N·m、roll 0.06–0.12 N·m，以 120/180/220 g 三种屏幕舱假体和三倍静态安全系数校准。

Sugatsune HG-MF08/15/25 的 0.0784/0.147/0.245 N·m 微型自由停铰链只作为 V0 扭矩与人体工学基准；最终下场机构使用可防尘、防松、可维护的定制摩擦关节。普通电脑显示器臂的气弹簧、同时锁定两个方向的球头和过重的相机魔术臂均不进入产品结构。

原因：屏幕舱质量远小于桌面显示器，摩擦关节已足以实现“推到哪里停在哪里”；三轴独立预紧和限位比球头更容易控制手感、线缆与收纳包络。

## ADR-018：屏幕先于外壳冻结，v0.1 首选 2.8 英寸高亮 SPI 裸屏

状态：Superseded by ADR-019 / ADR-020（2026-08-25）

v0.1 以 2.8 英寸、原生 240 × 320、装机横向 320 × 240、无触摸、800–1,000 cd/m²、4-wire SPI 裸屏为工程基线。首选样屏为 Newhaven `NHD-2.8-240320AF-CSXP-F Rev1B`：横放外廓约 69.2 × 50 × 3.39 mm、1,000 cd/m²，满亮显示部分典型量级约 0.52 W。样屏必须通过户外、偏振护目镜、保护片、刷新延迟、功耗、温升和两小时稳定性测试后才能锁料号、显示背板与屏幕舱前脸。

屏幕后自制显示背板承担 40-pin FFC、4-wire SPI、模式配置、复位/TE、背光恒流与 PWM、ESD 和测试点。Linux 应用保持无桌面环境，以 320 × 240 RGB565、局部刷新和 `DisplaySink` 适配器驱动；可使用直接 SPI 或经实机验证的通用 MIPI-DBI DRM 路径。

3.5 英寸 640 × 480、950 cd/m² 的 4-lane MIPI DSI 裸屏只在 320 × 240 UI 无法辨识、SPI 延迟不达标或其它 D1 否决项成立时进入 D2。常见 180–300 cd/m² HDMI/DSI 成品模块只用于桌面 bring-up，不定义上枪外壳。

原因：该屏型在相机侧屏正面包络内同时给出户外亮度、无触摸的耐用边界和约半瓦显示负载；选择 SPI 会增加一块显示背板和 Linux 适配工作，但比把宽大、低亮或高功耗 HDMI 成品模块直接塞进三轴屏幕舱更符合最终形态。屏幕决定外廓、功耗、保护层、UI 分辨率和接口，因此必须先看实屏再冻结机械。

## ADR-019：屏幕与全尺寸 Raspberry Pi 组成一体式 DSI 主机舱

状态：Superseded by ADR-020（2026-08-25）

v0.1 把屏幕、全尺寸 Raspberry Pi B 型板包络、显示接口、散热件和可充电主机电池放进同一个三轴移动主机舱。首选样屏改为 Waveshare `2.8inch DSI LCD`（480 × 640）；使用 Raspberry Pi 原生 DSI 和壳内短排线，不让 DSI/HDMI 视频线跨 yaw/pitch/roll 三轴，也不在产品外观面暴露视频插头、转接板或裸 PCB。

主板机械基线为 Raspberry Pi 4B/5 共用的 85 × 56 mm 级包络。Pi 4B 与 Pi 5 的最终选择仍由 S4 板端负载、屏亮/屏灭功耗、散热、供电峰值和冷启动实测冻结；主云端架构不因追求参数默认选择 Pi 5。屏幕不必单独覆盖整块 PCB，完整前脸和过渡壳体负责遮蔽主板，以圆角、斜面、收腰、内藏接头和检修盖形成与载体统一的配件外观。

固定 IMU 继续位于导轨根部，经藏入中空轴/封闭内槽的低速耐弯链路连接主机舱。跨轴链路只承载根部传感器的小电流供电和低速数据；显示、主板和主电池彼此不跨轴。日常外露接口原则上只保留带防尘结构的充电/维护口与必要实体键。

全尺寸主板提高移动质量，因此 ADR-017 的 120/180/220 g 假体和旧初始力矩不再适用。新台架使用 250/350/450 g 三档完整舱假体，力矩按实测重心和三倍静态安全系数重新计算；350 g、40 mm 力臂的 pitch 三级安全系数约需 0.41 N·m。若重量破坏操控，先减小板卡、散热和电池质量，不通过无限增大关节摩擦掩盖问题。

原因：把屏幕和主机放在一起，能把最显眼、最脆弱的高速视频线缩成壳内短连接；屏幕成为设备正面，主板和接头被统一外壳遮蔽，明显改善把玩手感、抗勾挂能力和视觉完整性。代价是体积、重量和散热上升，因此屏幕亮度、主板型号、电池和关节必须作为完整堆叠共同冻结。

## ADR-020：全尺寸 Pi 只开发，CM4 小载板进入一体式主机舱

状态：Superseded by ADR-021（2026-08-25）

保留 ADR-019 的核心拓扑——屏幕与主机在同一个移动舱、DSI 只走壳内短线——但撤销“85 × 56 mm 全尺寸 Raspberry Pi B 型板进入上枪外壳”。全尺寸 Pi 4B 只作为桌面开发与故障诊断平台；上枪集成基线改为 55 × 40 mm Raspberry Pi CM4，首件结构原型使用 Waveshare `CM4-NANO-B`（56 × 41 mm、15-pin DSI）。

CM4-NANO-B 的 RJ45、USB-A、3.5 mm 音频、mini-HDMI 和 40-pin GPIO 便于 bring-up，却会增加厚度和形成“接口塔”。因此它只定义 K2/K3 集成原型，不定义产品外壳。样屏、软件、电源和传感器闭环通过后，再评审一块精简载板，只保留 CM4 连接器、5 V/电源管理入口、15/22-pin DSI、维护/烧录 USB、固定 IMU 低速链路、必要实体键与测试点；不把桌面端口永久背在屏幕后。

首选计算模块为带无线和 eMMC 的 CM4，主云端负载优先评估 2–4 GB RAM / 16 GB eMMC 档。CM5 虽与 CM4 同为 55 × 40 mm 级，但其供电与散热需求更高，且小载板兼容性必须逐板确认；v0.1 不因尺寸相同默认升级 CM5。

集成舱重新使用 160/220/280 g 三档假体。以 220 g、pitch 轴到重心 35 mm 计算，三倍静态安全系数约需 0.23 N·m；各轴力矩仍以完整堆叠实测冻结。若现成 NANO-B 的高大接口导致厚度不可接受，不通过放大外壳迁就，而是提前进入精简载板。

原因：全尺寸 Pi 加屏幕、散热和电池会形成宽厚方盒，既削弱屏幕对主板的视觉遮蔽，也提高三轴末端质量。CM4 + 小载板能把计算部分真正藏进 2.8 英寸屏幕后，同时保留 Raspberry Pi OS、DSI 和开发生态；两级开发板路线兼顾开发便利与成品把玩体验。

## ADR-021：本地 OCLive + ARMv7 数据门，Lyra Zero W 成为 P0 实机候选

状态：Accepted route / Proposed board freeze（2026-08-25）

撤销“必须用 Raspberry Pi/CM4”和“主云端”的当前基线。OCLive Host、角色包、记忆/状态、SQLite、DeviceEvent、器灵状态机和显示仲裁全部在 Linux 主机本地运行；外部 LLM API 只是可选的动态表达提供者，不能成为启动、事件闭环或断网运行的前提。

P0 实机候选改为 Luckfox Lyra Zero W + Waveshare `2.8inch DSI LCD`。厂商数据为三核 Cortex-A7 1.2 GHz、512 MB DDR3L、2-lane MIPI DSI、256 MB SPI NAND + TF、2.4 GHz Wi-Fi 6、Bluetooth 5.2/BLE；Luckfox 官方 DSI 文档的适配清单包含该 2.8 英寸屏，并提供 `luckfox-config`、DRM `modetest`、触控测试和背光 sysfs 路径。

代码侧已经不是纸面兼容：标准 Release 以 Rust `armv7-unknown-linux-gnueabihf` 和 Arm GNU 14.2.Rel1 完成最终链接，产物为 23,371,692 bytes（22.29 MiB）的 ELF32 ARM EABI5 hard-float PIE；动态依赖仅 `libgcc_s.so.1`、`libm.so.6`、`libc.so.6` 和 `ld-linux-armhf.so.3`。同一代码的 Windows x64 mock 空载工作集为 11.0 MiB，但这不能替代 512 MB ARM 实机 RSS、页缓存、DSI、BLE 和网络共存数据。

因此当前只冻结“购买/借测顺序”，不冻结量产板：

1. P0：Lyra Zero W，因 DSI/无线集成和小板拓扑最符合壳内短线；必须通过 `ARMV7_VALIDATION.md` 的 512 MB 实机门。
2. P1：Orange Pi Zero 2W，官方配置为四核 Cortex-A53、1–4 GB LPDDR4、30 × 65 mm、Wi-Fi 5/BT 5.0 和 mini-HDMI；内存余量更高，但视频接头/线缆隐藏更差。
3. P2：Raspberry Pi 5 或已有 Pi 4，只承担高余量开发、故障定位或 P0/P1 失败后的回退，不再因生态默认进入三轴舱。

Luckfox Pico/Pico Ultra 的单 Cortex-A7 与 128/256 MB 不进入候选；不能通过删掉本地 OCLive、角色记忆或安全生命周期来迁就更小板卡。CM4 路线保留为历史备选，除非实机数据证明其价格、体积和功耗重新有优势。

依据：`ARMV7_VALIDATION.md`、Luckfox Lyra Zero W 官方规格、Luckfox Lyra DSI 官方适配文档和 Orange Pi Zero 2W 官方规格。

## ADR-022：Orange Pi/HDMI 模块化 bring-up 与双区域 BLE 节点

状态：Accepted current implementation route（2026-08-26；覆盖 ADR-021 的板屏采购优先级，以及 ADR-012/014 的单 Grip Node/CR2032 原型基线）

当前首轮模块化 bring-up 使用 Orange Pi Zero 2W 2GB + 3.5inch 480×800 HDMI IPS 电容屏，主板、屏幕、主电池和显示/触摸接口共同位于三轴移动主机舱。Luckfox Lyra Zero W + 2.8inch DSI 保留为移动质量、厚度或功耗触发后的紧凑回退，不再是当前第一采购优先级。

传感器拓扑由单 Grip Node 改为两个区域 BLE 节点：前下导轨节点负责前握 FSR、固定载体参考 IMU 和运动/震动；后握把节点负责后握 FSR 与独立只读 trigger 辅助输入。节点按区域无线，区域内部允许隐藏短线；不为每颗传感器单独增加 MCU/电池。顶部不新增第三传感节点。

XIAO 原型节点使用受保护可充 1S LiPo；ADR-028随后把容量占位细化为前节点和后节点两档，并成为数值真源。CR2032/CR1632 只保留为未来专用低功耗节点，不得接入 XIAO 可能执行充电的电池路径。

P1 摄像头采用前端 UVC MJPEG、护木隐藏 USB 主线和三轴附近可更换短跳线；摄像头接 Orange Pi，不接显示面板。P0 全枪传感闭环通过前不接摄像头。

原因：单节点无法同时保持前握自由度、后握/trigger 短线和载体参考 IMU 的合理位置；两个区域节点把长线变为区域内部短线，同时复用相同 BLE 核心和固件。Orange Pi/HDMI 作为标准接口模块更利于第一轮本地 OCLive、触摸和后续 UVC bring-up，但其重量/功耗必须用硬门约束。

## ADR-023：五链路设计、估算标签与实测覆盖规则

状态：Accepted（2026-08-26）

所有硬件设计与评审必须同时维护：信息链路、能源链路、接口契约、故障链路和机械链路。当前工程细节与阶段门以 `HARDWARE_IMPLEMENTATION_PLAN.md` 为 SSOT。

数据证据等级固定为：

- `DATASHEET`：厂商资料，尚未在本装配中验证。
- `ESTIMATED`：纸面计算、尺寸盒、功耗/阈值/时序占位。
- `MEASURED`：绑定料号、批次、硬件/固件/软件版本、方法和原始证据的实测。

实测覆盖估算时不删除历史，而是追加“原假设 → 实测 → 新裁决 → 影响范围”。可上枪原型必须绑定主机 commit、前/后节点固件与校准、schema、三套机械 revision、BOM/power revision 和测试报告。

当前质量硬门：主机舱使用 225/285/345 g 三档假体，移动舱超过 350 g 时先减电池/接口/外壳或切紧凑板屏，不通过无限增加关节摩擦掩盖；完整附件预计 330–520 g，超过 520 g 时重新评审结构。功耗、电池、尺寸、阈值和 TTL 在硬件到货前均保持 `ESTIMATED`。

原因：硬件没有覆盖物理现实的完整编译器；只有把能源、信息、接口、故障和受力路径与版本化实测绑定，下一位开发者或 AI 才能复现问题，而不会把旧估算当作已验证事实。

## ADR-024：双节点状态语义采用后握优先、Ready 置信度和保守待机

状态：Accepted（2026-08-26；用户确认 `1A / 2C / 3A`）

Held 的进入规则冻结为：后握 `engaged` 可直接进入；只有前握 `engaged` 时，还需要固定 IMU 的 motion moving 证据。进入 Held 后，只要前后任一握持仍明确 `engaged`，即使载体转为静止也保持 Held，避免静止持枪误睡眠。

Ready 继续使用一个设备状态，不复制成两个事件名，但内部保留置信度：后握 `engaged` + `motion.raised` 形成较低/中置信度 Ready；前后握均 `engaged` + `motion.raised` 形成高置信度 Ready。这样支持单手举起，同时把完整双手握持视为更强证据。后握为 `released` 或 `unknown` 时不产生 Ready。

Standby 只有在前后握都明确 `released`，且 `motion.idle` 持续达到实测时间窗时成立。断连、TTL 过期或快照无效产生的 `unknown` 不得冒充 `released`。为了屏幕省电可以另设更保守的显示降级计时器，但不能因此生成确定的放下/待机语义事件。

角度、压力阈值、运动阈值、去抖、Standby 等待时间和 confidence 数值尚未冻结，继续标记 `ESTIMATED`，由台架动作集与场地 Alpha 覆盖。

原因：后握是操作意图最强的证据，前握与运动组合能识别从前部拿取而不让静态外物压住 FSR 造成误判；Ready 置信度兼顾单手把玩与双手完整握持；保守处理 unknown 可防止掉线被解释成用户放下。

## ADR-025：前握传感件与电子盒分离，后握使用可拆单条 FSR 夹层

状态：Accepted（2026-08-26；用户确认 `4A / 5A / 6A`）

前部采用可移动传感握片/导轨护片与后方下导轨电子盒分离的结构。FSR、外层和力扩散结构随前握位置移动，XIAO、固定参考 IMU 和节点电池留在后方长薄电子盒；两者只通过区域内部可隐藏、可更换并带应变释放的短软线连接。

后握 V0 在原握把后背带使用纵向单条 FSR。先以一个 ADC 通道完成至少 100 次握持动作集；只有裸手/手套、左右手、不同手型或握力出现显著盲区，才升级左右双侧传感器，不在取得证据前增加通道和线束。

后握受力叠层固定为：`手掌 → 可拆薄 TPU/硅胶外套 → 略有弹性的力扩散片 → FSR → 原握把刚性后背面`。原握把充当刚性背板，扩散片位于 FSR 上方，将偏置掌压传给传感条。外套不得靠持续预压换取灵敏度；FSR 尾线沿握把套隐藏线槽进入可拆底盒，底盒保留原电机底板、调节/检修和通风净空。

V0 不切削、钻孔或永久粘接原握把。材料、扩散片刚度、FSR 型号/长度、线长和 1.5–3 mm 总增厚仍为待实测参数。

原因：分离前握传感件能保留前手位置自由和后方长薄电子盒；单条后背 FSR 是最小可验证后握方案；把原握把作为背板可显著降低结构复杂度，同时让传感层全部可逆、可换和不进入原机构。

## ADR-026：单条后握 FSR 使用分组动作门和版本化可选校准

状态：Accepted（2026-08-26；用户确认 `7A / 8A / 9C`）

后握单条 FSR 第一阶段动作门由 100 次独立握持—释放正样本组成：右手裸手、右手手套、左手裸手、左手手套各 25 次，每组混合轻握、正常握和较紧握。另做 50 次开机负样本，覆盖静置、轻触、拿取底盒、未握后把的载体移动和临时靠放。结构稳定后再用不同手型复验，不在 V0 结构频繁变化时提前扩大样本。

单条方案的初始通过门为：正样本总成功率 ≥95%，四个分组各 ≥90%，没有固定握法的系统性盲区，50 次负样本误判 ≤1，释放后不长期卡在 engaged。未通过时先调整扩散片、套体预压和校准；仍未通过才升级左右双侧 FSR，不能只靠降低阈值掩盖盲区。

校准采用“装配基础校准 + 可选用户校准”。节点保存无压力基线、轻握/正常握范围、engaged/released 阈值和 calibration version。更换 FSR、扩散片或握把套后旧校准失效。启动时加载已保存校准，不得把开机瞬间 ADC 自动当作新零点；重新标定只能由用户明确触发。

正常装包、运输和长期收纳时按用户习惯整机物理关机，因此背包持续挤压不进入开机核心验收工况。开机后的轻触、临时靠放和误拿底盒仍保留为负样本，避免日常短暂停放造成频繁虚假 Held。

原因：分组门能防止总平均值掩盖左手、手套或轻握盲区；先修机械再增传感器可控制复杂度；版本化可选校准兼顾即开即用与更换材料后的适配，同时避免受压开机污染基线。

## ADR-027：FSR V0 使用简单分压、远端 EOL 预留和盒内两针接口

状态：Accepted（2026-08-26；用户确认 `10A / 11C / 12B`）

FSR V0 使用简单电阻分压接入 XIAO nRF52840 SAADC，不在 P0 增加运放或带电位器比较器模块。电路起点为 `3V3 → (FSR || optional R_EOL) → ADC tap → RM 47 kΩ → GND`；RM、R_EOL、ADC gain/reference/acquisition time 和滤波参数全部通过台架数据冻结。

在每个传感夹层的 FSR 远端预留 EOL 高阻电阻位置，接口仍为两线。EOL 放在被诊断短线与连接器的传感器一侧，使正常 released 具有小幅基准电压，线路开路时 ADC 回到近零；若把 EOL 放在节点 PCB 上，中间断线后节点仍能看到电阻，不能完成诊断。台架比较未安装与多个候选高阻值后再决定是否装及阻值。

EOL 只用于细化线路开路诊断，不宣称覆盖短路、机械持续预压或 ADC stuck-high；这些故障继续结合极值、不变性、运动上下文和故障注入判断。无法可靠区分时相关事实进入 unknown，不得冒充 released。

前后 FSR 短线在节点盒内部使用微型两针、防呆、抗振锁定连接器，并在传感侧和节点侧分别做应变释放。具体连接器系列、线规和插拔寿命待尺寸假体冻结。FSR 尾片是否可焊必须遵循实际料号连接方式，不默认直接对聚合物尾片加热。

原因：简单分压已经满足二值握持语义；远端 EOL 只增加一个可选电阻即可把开路从释放区间中分离；预留而不提前冻结阻值能用实测平衡诊断裕量、ADC采集与功耗；盒内连接器让握片可替换又不增加外观接头。

## ADR-028：节点采用同电气规范、两种尺寸候选的可拆1S电池匣

状态：Accepted（2026-08-26；用户确认 `13A / 13-R2 / 14B / 15A`）

前后 BLE 节点的续航目标统一为连续 8 h 真实会话回放后仍保留约 20% 容量。电池均为受保护可充 1S LiPo，统一标称 3.7 V/充满 4.2 V、极性、防呆、保护和电量语义；不强制两种电池匣机械互换。

前节点保留 200–260 mAh 大尺寸候选，后节点保留 120–160 mAh 小尺寸候选。最终选择由节点平均/峰值电流、8 h 回放、低温、重连、充电温升、装配包络和握持手感决定；实测若小电池有余量则缩小，不足则在相应机械上限内增大，不能把占位容量当成采购定案。

电芯、PCM、接点和外壳组成独立硬壳电池匣，采用非磁性滑轨/卡扣方向；P0 可先用带检修盖的可拆托盘。当前 `ESTIMATED` 包络为：后匣约 35–38 × 16–20 × 8–10 mm、6–9 g，前匣约 36–42 × 22–24 × 8–11 mm、8–12 g；相对固定软包每节点预计增加 3–6 cm³ 和 2–5 g，必须用假体覆盖。

前后节点均使用凹入式物理硬开关。P0 可在电池正极串联开关，关断时物理断开电池。P0 使用 XIAO 自带 USB-C 作为维护、烧录和装机直充入口，不另造充电板；充电时节点停用并把硬开关置于允许电池连接的位置。实际充电电流、充满时间、温升以及 USB 接入时节点是否启动必须实测并明确记录。

两种电池匣从 P0 起统一预留非磁性、机械防反的 `BAT+ / BAT−` 充电触点基准和充电座包络，但 P0 不要求完成充电座。V1 再开发 USB-C 输入、专用 1S CC/CV 管理的独立充电座，让拆下的大小电池匣可用同一电气定义充电；机械定位可做分档或适配块，不使用磁吸定位。

不得复用可接入常见 7.4/11.1 V wargame 电池的 Mini-Tamiya、T-Plug/Deans、JST-RCY 或 XT30 作为节点1S通用入口。可以借用可拆换体验，不能制造过压误插路径。

原因：相同电气契约降低充电、固件和维护分叉，两种机械尺寸避免为了共用电池把后握底盒做大；可拆硬壳匣适合下场快速换电并保护软包电芯；容量保持数据驱动可避免过度堆电池。P0 复用 XIAO USB-C 能优先验证传感器和 BLE，提前保留触点与包络又不会堵死 V1 的场外快速换电体验。

## ADR-029：节点电量采用板载ADC、受控关机与分层恢复

状态：Accepted（2026-08-26；用户确认 `16A`，接受修改后的 `17A / 18A`）

P0 使用 XIAO nRF52840 Sense 板载电池分压/ADC 路径读取电压，固件按官方要求控制 P0.14 并从 P0.31 采样。对外提供真实 `battery_mv` 与 `normal / low / critical` 粗粒度状态，不提供未经完整放电曲线验证的精确百分比。电量采集在固件内抽象为可替换 provider，协议允许未来专用 fuel gauge 增加带来源/有效位的 SOC 与健康度字段；载板保留 I2C 可达性和待选扩展空间，P0 不装额外电量计。

低电策略为渐进降级和有余量受控关机：low 只提醒；critical 先停止原始调试流、高频日志等非必要任务，保留握持/运动结论和 BLE 心跳；滤波电压持续低于实测关机点后发送 `shutdown_pending`，Orange Pi 限时完成 checkpoint 并确认，节点记录最小关机原因后进入低功耗关机。确认超时也必须在截止时间内关机，不能一直等待直到 brownout 或 PCM 保护切断。

关机阈值、确认时间和余量不按纸面百分比冻结。硬件到货后在低温、BLE 峰值、重连和老化电芯工况测得 brownout/PCM 边界，再反向确定 `low/critical/shutdown` 阈值与迟滞。最差工况连续 10 次均须完成最终通知、主机 checkpoint 和节点关机，且关机后仍有可测电压余量。再次启动由换电/充电后切换物理开关或复位触发，不依赖电压回升自动唤醒。

恢复数据严格分层：角色身份、记忆/成长、用户设置和会话摘要等连续状态由 Orange Pi/OCLive 周期持久化，并在收到 `shutdown_pending` 时追加 checkpoint；节点只持久化 node_id、bonding、硬件/校准版本、已验证 calibration profile、配置 CRC 与关机原因。节点每次冷启动产生新 `boot_id`，ADC/IMU 滤波、grip、motion、Held/Ready、去抖计时和旧 sequence 全部清空为 unknown，取得新鲜样本后再派生。传感器“重置”不等于删除校准，更不能用可能受压的开机采样自动归零。

原因：板载测量可先把硬件复杂度降到最低，provider/协议预留又允许实测不足时平滑升级；提前于保护板断电的受控关机为数据落盘留出确定窗口；把长期角色状态、稳定节点配置和瞬时传感证据分开，既保持角色连续性，又不会复活重启前已经失真的握持状态。

## ADR-030：主机P0采用4小时轻量目标、成品5V电源和手动安全关机

状态：Accepted（2026-08-26；用户确认 `19A / 20A / 21A`）

主机 P0 以连续 4 h 真实场地负载为硬门，先选择能满足门槛的最小/最轻电源；不足时在同一固定 5 V 输出契约下换更大容量，不预先为 8 h 目标堆叠电池。按当前 3.5–5 W 平均功耗、85% 转换效率和 80% 可用能量计算，4 h 所需标称能量约 20.6–29.4 Wh，保持 `ESTIMATED`，由 Orange Pi、屏幕亮度、触摸、BLE、Wi-Fi 和 OCLive 实测覆盖。

P0 使用完整、不拆壳、不改电芯的成品电源/充电宝。固定 5 V 输出能力以 3 A 级别起测，禁止 PD 诱骗到 9/12 V；经受保护的 5 V 分配点分别向 Orange Pi 与屏幕供电，不允许屏幕反向代供主板。P0 充电时关机，不承诺 pass-through。自动休眠、低电切断、按键、电压跌落、静态耗电和峰值行为均为验收项；不合格就更换整件，而不是拆壳修改。

P0 安全关机采用人工可验证闭环：长按/按下关机请求键，OCLive 原子 checkpoint，Linux 正常 shutdown，随后以独立指示告诉用户可以切断主 5 V，最后操作凹入式硬开关。Orange Pi Zero 2W 的扩展接口具有 power-button 能力，但具体引脚与 Linux 行为仍须按所用板卡 revision 和镜像实测。

普通成品充电宝通常不会给 Linux 提供可信低电遥测，因此 P0 不宣称自动欠压保存。只有所选电源确实提供并验证了遥测/告警才接入；否则 V1 再加入低功耗监督 MCU、power latch/load switch 与 Linux ACK，完成自动低电 checkpoint、延迟切断和彻底断电。定制 1S/2S 主电池架构同样推迟到 P0 功耗、温升、体积和三轴手感数据齐全后裁决。

原因：先用完整成品电源可在不承担锂电设计风险的情况下获得真实 Wh、峰值和热数据；4 h 轻量门避免电池吞噬屏幕后包络与三轴手感；手动安全关机先保护 Linux 文件系统，接口预留则允许 V1 自动化而不推翻 OCLive 的 checkpoint 语义。

## ADR-031：P0采用一体移动电源舱、背光分级和高耐久MicroSD

状态：Accepted（2026-08-26；用户确认 `22A / 23A / 24A`，要求写清后续优化方向并先跑通）

P0 将完整成品 5 V 电源与 Orange Pi、屏幕共同放在三轴移动主机舱内，换取内部短线和完整外观。移动质量按屏、板、电源、外壳、接头、线缆和关节从动件总和计算，先以 225/285/345 g 假体测试；350 g 是硬退出门。超过时先减电池容量、壳体和非必要接口或改紧凑件，仍不通过才把电池移到固定导轨底座，并为跨三轴的单根 5 V 柔性线设计弯曲半径、应变释放和硬限位。不得通过无限提高摩擦补偿超重。

P0 不让 Linux suspend，采用背光三级策略：active 正常亮度、无交互后 dim、Standby 后 backlight_off；主机、BLE 和触摸控制器继续运行。30 s、120 s 和 ≤300 ms 唤醒只是 `ESTIMATED` 初值，握持、运动、触摸、按键与关键角色事件均可唤醒。候选 HDMI 屏必须实测软件可控背光/PWM或可靠控制引脚；输出黑色图像不等于关闭背光，也不通过频繁切整块 HDMI 板电源实现 P0 省电。

背光后续优化按数据触发：先测各亮度功耗、户外可见度、唤醒延迟和误唤醒；若仍不能达到 4 h，再依次评审环境光传感器、CPU governor、USB/摄像头电源域和更深低功耗。Linux suspend 只有 BLE、HDMI、触摸和应用状态恢复都有重复证据后才可进入，不作为默认升级。

P0 使用高耐久或工业级 MicroSD，ext4 保留日志，OCLive 以 SQLite 原子事务保存角色连续状态，并在周期、关键状态变化和安全关机时 checkpoint。FSR/IMU 高频原始样本默认只进入内存/有界调试缓冲，不持续写卡；系统和应用日志轮转并设置总量上限，提供角色/配置导出与恢复。

存储后续优化同样由故障注入决定：在副本镜像与专用测试卡上验证异常断电、日志写满、数据库恢复和换卡；若出现不可接受的启动失败、状态回退或磨损，升级 USB SSD。只有 SSD 的 USB 占用、体积和功耗也不可接受时，才重新评审带 eMMC 的主板，不因规格表更高而提前换板。

原因：一体舱最接近目标把玩和外观体验，但350 g门保护三轴手感；背光控制通常比整机休眠更容易获得稳定收益；高耐久MicroSD是Zero 2W最短软件路径，通过事务、写入节流和故障测试可先验证OCLive，而三条路线均保留由实测触发的升级出口。

## ADR-032：节点采用实体授权、逐设备密钥和无蓝图能力注册表

状态：Accepted（2026-08-26；用户确认 `25A / 26A-R1 / 27A-R1`）

P0 通过凹入式实体维护键开启限时配对界面，一次只确认一个候选节点；约60秒窗口仅是 `ESTIMATED` 初值，成功、取消或超时后立即关闭。触摸界面用于显示唯一识别码、命名与确认，但不能在没有实体授权动作时让陌生设备进入配对。

BLE信任采用逐设备bonding：每个节点与主机有独立长期密钥。新增节点只向Linux/BlueZ bond store和可信身份白名单增加一项，不重新生成旧节点密钥或整张列表；撤销节点也只删除对应bond和注册状态。OCLive节点注册表不保存BLE长期密钥，只保存身份、信任和能力元数据。只有主机信任库存储丢失、整机重置或确认泄露时才全量重配。P0不使用组密钥或额外应用层加密，V1可增加二维码/OOB身份而不改变上层模型。

节点注册表参考OCLive注册表的“登记实际存在能力”原则，但明确不使用角色蓝图、发射器蓝图或理想配件配置。记录至少包括 `node_uid`、显示名、`pending/trusted/revoked`、capabilities、可选安装bindings、固件/schema版本、`assembly_id`、calibration引用和last_seen。`node_uid`跨重启稳定且不能只依赖可随机变化的BLE地址；具体生成/预配方案待固件安全评审。`boot_id`每次冷启动变化；`assembly_id`绑定节点板、传感器与机械夹层，变化后旧校准失效。

安装bindings只陈述当前事实，例如 `grip.front`、`grip.rear`、`carrier.motion`，不要求所有节点存在。缺少某能力时系统降级为unknown或使用较少证据，不能让角色包或OCLive启动失败。角色包只消费稳定DeviceEvent，不感知FSR型号、节点厂商、BLE地址或具体枪型拓扑。

原因：逐设备密钥允许单独扩展、替换和撤销节点；BlueZ与应用注册表分责避免在业务数据库复制秘密；无蓝图能力注册表可适配已经存在的发射器与配件生态，同时保留解释压力和运动所需的最小现实安装语义。

## ADR-033：BLE采用edge加快照、节点本地融合和不重放重连

状态：Accepted（2026-08-26；用户确认 `28A / 29A / 30A`；补充关机生命周期语义，交互时机待下一批）

正常BLE链路采用即时edge event与周期CurrentState snapshot并行：握持、释放、motion变化和关键fault立即通知，完整当前状态约1 Hz兜底。快照至少携带node_uid、boot_id、sequence、单调时间、当前状态位、电池、fault和confidence；具体Characteristic与二进制布局版本化。edge丢失时下一快照修正，不让主机永久卡在旧状态。

节点本地以较高频率采样、滤波和派生结论，BLE只低频传语义。FSR 25–50 Hz、IMU 26–52 Hz、快照1 Hz、电池30–60 s和维护原始流最高约10 Hz均为 `ESTIMATED`；最终由平均/峰值功耗、事件p95延迟、漏报和误报实测冻结。正常下场不持续上传原始压力或IMU流。

断连期间不排队重放握持、运动或trigger等瞬时动作。TTL到期后相关事实unknown；自动重连发送HELLO和新鲜快照。boot_id相同表示链路恢复，变化表示节点重启，旧滤波、sequence和瞬时状态全部作废。断线期间变化次数可以作为诊断计数回传，但不注入角色状态机。0.5/1/2/5秒封顶退避只是测试起点。

用户提出在保存数据中加入关机语义，使角色可以主动交互。该需求采用结构化LifecycleEvent而非自由文本关键词：至少记录event_id、host/node scope、reason、requested_at、clean、pre_shutdown_handled和resume_consumed。Orange Pi持久化last_shutdown，节点只存自身最小关机原因。瞬时动作不补发，但未消费的关机生命周期记录允许在重启后消费一次，随后以原子事务标记，避免每次启动重复触发。

生命周期交互的确切时机（关机前、重启后或两者）、表达来源和不同reason的角色化程度留给下一批。无论选择如何，受控关机不能无限等待网络或动态模型，必须在保留电量截止时间前结束。

原因：edge保证及时，快照提供自愈，本地融合降低BLE功耗；拒绝重放避免角色对已结束动作迟到响应；关机属于可持久化生命周期事实而非旧传感动作，结构化且一次性消费可同时实现角色连续性和幂等。

## ADR-034：关机生命周期采用双阶段反馈、预生成表达缓存和诊断隔离

状态：Accepted（2026-08-26；用户确认 `31A / 32A-R1 / 33A`）

同一LifecycleEvent允许两个不同阶段各反馈一次：`pre_shutdown`在关机前立即反馈，`recovered`在下一次恢复后反馈。两阶段共享event_id但分别保存`pre_shutdown_consumed`与`resume_consumed`；任何反馈失败或超时都不阻止受控关机。未留下clean marker时，下次启动派生`unclean_restart`。

关机现场禁止发起AI或网络请求。系统在正常电量和空闲阶段异步预生成少量角色化短句，按reason_class与phase缓存；缓存不足、角色context revision变化或长期未刷新时再补充。关机时只选择近期生成且最近未使用的合格缓存，失败立即使用固定本地兜底。缓存项至少记录phrase_id、reason_class、phase、text、context_revision、generated_at、last_used_at和used_count；约每类3条为ESTIMATED。所有候选必须通过长度、格式、角色边界和控制指令过滤，表达缓存不进入角色长期记忆。

reason采用稳定枚举`manual / low_battery / maintenance / update / thermal / storage_fault / unclean_restart`并允许版本化扩展。角色投影只包含规范化reason、友好scope和phase；底层diagnostic_code、数据库错误、文件路径、堆栈与硬件日志只进入系统诊断，不进入Prompt或角色记忆。Orange Pi保留last_shutdown与有界生命周期历史，最近32条或90天仅为ESTIMATED起点。

原因：提前生成兼顾角色个性与关机确定性，固定兜底消除模型不可用风险；双阶段让角色既能告别又能解释恢复；规范化语义和诊断隔离避免角色把技术报错当作自身认知或对话内容。

## ADR-035：IMU采用载体坐标、用户姿态校准和非击发冲击语义

状态：Accepted（2026-08-26；用户确认 `34A / 35A-R1 / 36A`）

系统规范右手载体坐标为`+X向前、+Y向左、+Z向上`。节点外壳使用不对称机械定位，IMU芯片轴通过assembly绑定的`mount_transform`映射到载体坐标。重力无法确定枪口方向，因此不自动猜安装朝向；assembly或binding变化后旧transform/姿态profile失效并要求用户确认。

举起语义由用户通过类似手柄陀螺仪的向导自行校准，而非使用全产品固定角度。校准明确分为传感器静止bias/self-test、安装方向transform和用户行为profile三层。快速流程建议静止3秒、自然放低2秒、自然举起2秒、举放5次并预览，全部为ESTIMATED。程序从样本确定重力方向、pitch/roll分布、过渡范围、迟滞和稳定时间，用户接受后才启用。

同一装配允许多个命名profile，绑定`node_uid + assembly_id + binding`以及calibration_version。提供预览、接受、取消、上一版回滚和重置。原始IMU只在用户显式进入校准/维护模式时临时录制，生成profile后默认丢弃或显式导出；正常运行禁止后台静默学习和持续落盘。这里的自定义只改变人体工学阈值，输出仍是稳定`pose.raised/lowered/unknown`，不让任意用户标签污染协议。

P0 IMU只提供`motion.idle/moving`、`pose.raised/lowered/unknown`和诊断`shock.observed`。冲击不生成`device.triggered`，不宣称击发识别，也不进入弹道、目标或火控能力。Ready仍由后握、可选前握和raised等多证据融合，raised单独不等于Ready。

原因：用户选择校准点能适应不同发射器、安装位置和姿态；分层profile避免把芯片偏置、机械方向和行为阈值混成不可维护的数字；禁止静默学习保证行为可复现；冲击保持诊断语义则避免碰撞和奔跑被误当击发。

## ADR-036：P0采用双实体键、角色优先界面和运行态配置锁

状态：Accepted（2026-08-26；用户确认 `37A-R1 / 38A-R1 / 39A-R1`）

主机舱设置POWER与MAINT两个独立实体键，均采用凹入、高阻尼/高操作力并避开受压边。POWER短按管理背光、长按请求安全关机；MAINT短按承担返回/最小确认、长按请求维护入口。触摸是正常交互主路径，但UI锁定、触摸失败或显示异常时，实体键仍必须能完成安全关机和进入受控维护。

屏幕借助外圈海绵贴合枪身或平面保护，但泡棉与刚性止挡必须先于LCD和按钮承力；按钮退入保护平面，泡棉不得持续压键。高阻尼只是一层防误触，不能替代机械凹入、位置保护和软件长按。进入配对/校准建议要求非Ready、静止、MAINT长按约3秒及二次确认，时序为ESTIMATED。

P0主界面角色优先，状态栏只显示主机电量、节点健康、网络与模式；详细电压、RSSI、原始传感器和版本放维护页。Ready时进一步去除无关文字，禁止准星、弹道、目标识别和射击参数。接口预留`interaction_mode`，以后可增加把玩/下场模式，但P0不提前复制两套页面或状态机。

运动或Ready时锁定配对、解绑、校准和设置写操作，只保留背光、安全关机、返回和查看提示。关键修改使用大触摸目标、长按或二次确认；锁定状态可经实体键操作。约12mm触摸目标、高操作力和具体长按时间均需戴手套、贴墙、装包及碰撞测试覆盖。

原因：两个明确按键比多击手势更可恢复；海绵与刚性止挡使贴枪保护不转化为按键误触；角色优先保持产品定位，运行态锁定保护现场使用；只预留模式契约而不实现双模式，给未来自由度但不提前过度设计。

## ADR-037：枪械感知内核与OCLive作为独立薄核协作

状态：Accepted（2026-08-26；用户确认 `40A / 42A`，将 `41A`细化为独立crate的`41A-R1`，并确认同进程零依赖的`43B-R1`）

本项目建立独立于OCLive的“器灵枪械感知内核”（暂用工程名 Gun Perception Kernel）。它是面向wargame发射器/枪械交互的特化内核，而不是OCLive角色内核的硬件分支：感知内核拥有节点身份与能力、安装binding、校准引用、时效性、去重、证据融合、载体状态机和故障降级；OCLive拥有角色包、记忆、情绪、回合编排和角色回复。两者以稳定契约合作，互不拥有对方的内部状态。

感知内核在本仓实现为独立纯Rust crate，不依赖任何OCLive crate，也不直接依赖BlueZ、GPIO、I2C、SPI、SQLite、DRM或具体板卡。mock、BLE和Linux设备访问留在adapter层；集成宿主是composition root，可以同时依赖感知内核与OCLive并完成映射。P0采用同一个`skippy-host`进程，避免IPC、双服务生命周期与额外部署成本；进程内仍只有bridge/composition root可以同时引用双方类型，两个内核源码互相零依赖、互相不知道。只有实测出现必须隔离的崩溃、独立升级或资源调度需求时才拆进程。

内核输入为版本化`SensorObservation`及显式配置/时间，输出同时包含持续`PerceptionState`和离散`DeviceEvent`。PerceptionState用于重连恢复、即时UI、诊断和测试断言；只有状态转换产生的DeviceEvent经过bridge策略投影后进入OCLive，周期快照不得反复触发角色回合。OCLive不接触ADC值、原始IMU、BLE地址、厂商驱动和阈值，也不能回写“正在握持/举起”等物理事实。

P0依赖方向固定为`hardware/mock adapters → gun perception kernel → integration bridge → OCLive kernel → role cue`。bridge执行显式值转换，不把感知内核对象、注册表或状态机引用直接交给OCLive，也不把OCLive角色对象交给感知内核。OCLive任务失败时，感知、状态机、维护提示、安全关机与确定性即时反馈继续运行；若整个进程退出则由systemd统一恢复。感知事实unknown时，OCLive不得凭角色逻辑补造确定状态。当前合作链为只读观察链，不定义`actuator.*`、波箱、电机、供弹或火控控制接口。

可以借鉴OCLive的薄核、版本化契约、能力注册、确定性转换与回放测试，但不复制角色蓝图、人格、记忆、场景编排、模型调用或市场插件结构。只有一个真实适配器之外出现第二个独立复用方、或实测需要进程级故障隔离时，才评审拆出更多crate/进程/仓库。

原因：两个平级薄核让“感知”和“思考”可以独立验证、替换与演化；独立crate与依赖规则给出编译期边界，同进程集成避免过早承担发布、IPC和多服务运维成本；双输出契约同时满足事实自愈和低频角色互动；单向只读边界保持电子故障只损失观察能力的产品红线。

## ADR-038：共同前端由Skippy Host输出仲裁器统一控制

状态：Accepted（2026-08-26；用户确认 `45A-R1`）

屏幕不是任何一个内核的私有输出。枪械感知内核只输出`PerceptionState / DeviceEvent`等事实，OCLive只输出角色侧`RoleCue`；两者都不直接访问DRM、framebuffer、WebView、触摸页面或具体显示驱动。`skippy-host`内的Output Arbiter是唯一最终屏幕所有者，负责把双方贡献组合成共同前端。

host将感知事实与生命周期投影为确定性`SystemViewState/SystemCue`，将OCLive的表情、文字和情绪表现视为`RoleCue`候选。P0前端逻辑分为三个概念层：`RoleStage`作为角色主体；`StatusBar`持续显示主机电量、节点、网络与模式；`SystemOverlay`显示安全关机、维护确认、配置锁定和必须处理的故障。系统Overlay优先于角色层，普通状态栏不得无故遮蔽角色主体。

Output Arbiter按来源、优先级、TTL、可选`related_event_id`与`context_revision`裁决。即时确定性反馈先显示；迟到、过期或所关联状态已经变化的RoleCue丢弃，不能让旧回复覆盖新状态。OCLive任务失败时继续显示系统UI与本地角色兜底；显示进程/renderer失败不改变感知状态，恢复后从最新SystemViewState重建画面。

两个内核只提供语义，不规定前端技术、像素位置、CSS、动画系统和具体屏幕驱动。renderer可以从桌面模拟器换成Linux DRM、轻量Web前端或其它实现，而不修改感知与角色内核。触摸输入同样先进入host的交互/维护策略，不允许页面直接修改内核内部对象。

原因：唯一输出所有者消除双方抢屏和优先级冲突；三层组合让角色与设备状态能够同时表达；TTL和事件关联阻止慢回复穿越状态边界；语义与渲染分离允许更换屏幕和前端实现而不破坏双内核自由度。

## ADR-039：交互模式归宿主、扩展显式桥接、角色回合按状态转换门控

状态：Accepted（2026-08-26；用户确认 `44A / 46A / 47A`）

`interaction_mode`由`skippy-host`交互策略层管理，只允许通过明确用户操作或受控维护流程切换。模式可以改变前端信息密度、触摸权限、角色主动程度以及哪些DeviceEvent可触发角色回合，但不能改写SensorObservation、PerceptionState、融合阈值事实、安全关机和故障降级。OCLive可把当前模式作为回复上下文读取，不能由角色回复或模型工具自行切换。

感知能力按本地版本化契约独立扩展。bridge采用显式映射表和测试把特定DeviceEvent投影为OCLive的类型化sensor turn；没有映射的新事件继续存在于PerceptionState、系统UI、审计和诊断中，默认不触发角色、不作为自由字符串进入Prompt，也不迫使OCLive同步发布。桥接映射可以独立增加、禁用和版本化，但不能把原始ADC/IMU或诊断细节绕过边界交给角色。

角色主动互动只由有意义的状态转换触发。即时SystemCue始终本地执行，不等待OCLive；bridge按event_id与状态revision去重，并为同类事件提供短窗口合并、冷却、进行中回合抑制和过期丢弃。周期PerceptionState、BLE快照、压力抖动、重复Held/Ready和重连后的当前状态同步不得自动形成重复角色回合。合并窗口与冷却时长不在纸面冻结，先通过回放统计漏响应、重复响应和主观打扰程度。

原因：宿主拥有模式可同时约束输入与前端而不污染物理事实；显式桥接让两个内核可以分别扩展且保持可审计；状态转换门控既保留器灵主动性，又避免高频传感事实把OCLive变成事件打印机。

## ADR-040：主对话采用显式输入优先、感知辅助和角色包情绪映射

状态：Accepted（2026-08-26；用户确认 `48A / 49A / 50A`，强调感知内核与OCLive相互不认识）

未来语音、文字和传感主动互动由OCLive侧的通用Input Scheduler统一编排。枪械感知内核始终只发布中性PerceptionState/DeviceEvent，不知道ASR是否存在、用户是否正在说话、当前对话状态或角色身份；OCLive也不依赖感知crate、不读取节点/阈值/原始采样。integration bridge把枪械事件显式投影为OCLive可消费的通用、版本化ContextSignal/DeviceContext值，两侧不共享内部对象。

每次角色回合只允许一个PrimaryInput，并携带有界AuxContext。没有显式语音/文字时，通过ADR-039门控的有意义DeviceEvent可作为`sensor` origin的PrimaryInput，形成纯枪械主动互动；存在语音/文字时，显式用户输入成为PrimaryInput，近期握持、姿态、运动和节点健康等中性DeviceContext作为辅助上下文合并进同一个主对话回合，不再为相同传感变化另开第二次回复。

角色包可在OCLive侧声明设备事件反应策略，例如情绪偏置、主动回复概率、模式差异、冷却和表现偏好。感知内核不得模仿或依赖OCLive情绪词表，不得输出`character.excited/alert`等角色结论；同一`device.picked_up`可由不同角色包解释为期待、冷淡、紧张或不主动回复。

对话通道采用显式用户输入高于环境sensor chatter的优先级。VAD/语音采集开始后，尚未提交的sensor turn取消并将相关事实合并到语音AuxContext；sensor turn已在生成时，若支持则取消，否则其RoleCue必须通过context_revision、related_event_id和TTL复核，失效即丢弃。ASR partial只更新临时输入和上下文窗口，不触发正式回复；ASR final transcript才可提交PrimaryInput。具体合并等待、取消截止和上下文范围留待语音阶段回放实测。

运行/安全优先级与对话优先级分轨：critical low battery、shutdown、维护确认、配置锁和故障SystemCue始终由host/Output Arbiter立即处理，不能被语音或生成中角色回合压住；是否把其友好摘要附加到主对话是次要行为，不得延迟保护动作。

原因：PrimaryInput+AuxContext消除语音与动作争抢回合；角色包拥有情绪映射保持人物自由；通用ContextSignal和显式bridge维持双内核零依赖；安全通道分轨避免“语音优先”被错误扩展为压制设备保护。

## ADR-041：输入采用短合并窗口、有界上下文和独立枪械角色扩展包

状态：Accepted（2026-08-26；用户确认 `51A / 52A`，并将 `53A`修订为`53A-R2`）

通过ADR-039门控的sensor事件先进入约500–800 ms的对话合并窗口。窗口内检测到VAD开始或显式文字输入时，事件不单独形成sensor turn，而是进入该用户回合的AuxContext；窗口结束仍无显式输入时才允许成为sensor PrimaryInput。该范围为ESTIMATED，必须通过回放比较抢话、响应迟滞与合并成功率。SystemCue、屏幕唤醒、低电、关机和维护不经过该等待。

DeviceContext采用有界语义快照，初始包含当前Held/Ready/Standby、interaction_mode、节点健康/电量等级、最近约2秒内最多4个语义变化，以及各项event time、confidence与context_revision。2秒和4项为ESTIMATED。上下文不包含原始ADC/IMU、音频、BLE地址、诊断堆栈和无限历史；超限时保留当前状态与较新/较高重要度变化，而不是截断安全事实。

`device_reactions`明确不属于OCLive标准角色包规范。它属于项目所有者二次开发、面向枪械宿主的独立角色扩展格式（暂称Gun Interaction Profile/Pack），使用自己的schema/version并引用基础OCLive角色身份。专用loader/adapter读取扩展策略并对接OCLive主对话；OCLive通用角色包、角色内核和市场格式不因为本项目增加枪械字段。

枪械扩展只描述中性设备事件到角色侧策略的映射，例如适用interaction_mode、情绪偏置、主动回复概率、冷却与表现简洁度。基础角色的人格、记忆、通用Prompt和资产仍以OCLive标准角色包为SSOT，不在扩展包复制。扩展缺失、版本不兼容或校验失败时，基础角色必须继续加载；对应设备事件默认不主动触发专属反应，但仍可作为通用DeviceContext进入显式用户回合。

原因：短合并窗口减少角色与用户抢话；有界上下文既保留当前身体状态又限制Prompt噪声；独立二次开发格式保护OCLive标准角色包的通用性，并允许枪械宿主针对角色自由扩展而不形成标准绑架。

## ADR-042：枪械角色扩展采用独立伴随包、宿主加载和失效开放

状态：Accepted（2026-08-26；用户确认 `54A / 55A / 56A`，并要求四季宝与OCLive文档严格分仓）

枪械交互策略使用独立伴随包，而不是修改、复制或派生覆盖OCLive标准角色包。伴随包暂称Gun Interaction Pack，以稳定`base_role_id`、兼容版本要求及可选基础内容摘要引用已安装角色；它只携带device_reactions、模式差异与枪械前端表现策略。基础人格、记忆、通用Prompt、关系和通用资产仍以OCLive标准角色包为唯一SSOT。正式包名、后缀和目录形态后续冻结。

专用loader/validator属于本仓`skippy-host`的OCLive integration adapter。它负责发现伴随包、验证独立schema、解析基础角色引用，再把允许的策略投影为OCLive通用Input Scheduler/角色回合可消费的值。OCLive标准角色包loader、schema、编写器和市场契约不增加枪械字段；OCLive内核也不依赖本仓类型。

扩展缺失、损坏、校验失败、基础角色不匹配或版本不兼容时采取fail-open：基础角色继续启动，枪械专属主动反应关闭，维护页显示稳定兼容性原因并保留诊断。不得猜测未知字段、把其它角色扩展套给当前角色、回退到不兼容旧schema或阻止感知/安全链路。

文档所有权严格分仓：本项目的感知内核、枪械包格式、硬件HostProfile、机械/电气/传感路线和实测记录只在`oclive-四季宝器灵`维护；OCLive主仓只维护真正跨设备通用的角色内核、输入契约和资源协调能力。需要引用OCLive机制时使用链接与代码锚点，不向主仓复制四季宝决策，也不把OCLive长篇SSOT复制进本仓。

原因：伴随包能复用基础角色而不产生人格分叉；宿主专用loader实现“本二次开发版可识别”而不污染标准；fail-open保证内容扩展失败不阻断角色和硬件；文档分仓避免两个项目再次形成隐性共同SSOT。

## ADR-043：复用通用资源协调器，P0只观察，P1按重量混合部署

状态：Accepted（2026-08-26；用户授权按建议冻结 `57A / 58A / 59C-R1`）

P0不复制、重写或模仿一套硬件专用Resource Coordinator。`skippy-host`直接复用OCLive通用协调器的资源快照/诊断能力，但不让它拒绝、排队或抢占P0任务。BLE、感知融合、节点健康、实体键、安全关机和最低限度System UI始终绕过资源准入；Linux/systemd继续负责实际线程、进程和恢复。

本仓现在只建立Orange Pi Zero 2W实机测量与HostProfile字段骨架，数值全部留待MEASURED数据。桌面默认RAM/CPU余量不得复制成板卡结论。启用强制准入前必须验证每个可选任务的RSS/线程/启动峰值、超时、释放回收、兜底和协调器故障时的关键链路独立性。

P1只为ASR、TTS、相机分析、复杂renderer和后台生成等可选重任务注册资源适配器。轻量、纯Rust且崩溃风险低的任务可保留进程内；携带原生运行时、独立模型、不可控线程池或较大故障域的任务可作为systemd服务。具体归类由实测逐项决定，不提前把所有能力强制塞进同一进程或全部拆进程。

这里的“硬件特化”仅发生在四季宝HostProfile、adapter估算、优先级/降级策略和Linux服务形态；通用Resource Coordinator仍由OCLive维护。枪械感知内核对OCLive的模仿仅限薄核边界、版本化契约和确定性测试思想，不模仿OCLive资源调度器，也不依赖它。

原因：复用通用控制面避免第二套协调器语义漂移；P0观察先取得真实2GB板数据；P1混合部署允许按故障域与资源回收证据决定边界，同时确保任何协调或角色故障只损失可选表现，不损失感知和安全链路。

## ADR-044：资源诊断采用维护摘要加导出包，HostProfile分开发与场地

状态：Accepted（2026-08-26；用户确认 `60C / 61B`；renderer部署随后由ADR-045裁决）

P0维护页只显示用户可理解的有界健康摘要，例如主机可用内存、温度、节点在线数、网络、存储健康和最近稳定故障原因。后台保存经过容量限制和轮转的诊断历史，用户显式请求时生成完整诊断包，包含板卡/OS/内核/程序版本、近期资源快照、节点连接生命周期、启动/关机原因和系统日志摘要；默认排除角色私密对话、原始音频和持续原始传感流。导出失败、低存储或安全关机时不得阻塞关键链路。

Orange Pi使用development与field两套HostProfile。两者共享同一个板卡资源基线、协议版本和关键安全边界；development允许详细日志、维护原始流、诊断接口和测试服务，field使用有界日志、关闭原始流与无关服务、收紧现场功耗/资源策略并保持角色界面优先。Profile切换必须显式显示当前身份、记录审计并通过受控重启生效，不能与把玩/下场`interaction_mode`混为一谈。

两套Profile必须由同一schema生成或校验，共享关键门测试，并有差异清单；禁止手工复制后长期漂移。field交付物不得因为development通过测试就视为通过，development新增能力也不得自动进入field。

原因：摘要+诊断包兼顾现场自查与回家复现；双Profile允许开发调试工具不污染稳定交付，但通过共享基线、显式差异和分别冒烟控制配置分叉风险。

## ADR-045：P0 Renderer同进程但使用可拆消息端口，按实测决定进程隔离

状态：Accepted（2026-08-26；用户确认 `62C`）

P0 renderer与`skippy-host`同进程部署，不提前承担第二服务、IPC握手和额外常驻内存；但Output Arbiter不得直接依赖DRM、framebuffer、WebView或具体绘图库。双方通过本仓定义的`RendererPort`交换可序列化、版本化、不可变的完整ViewModel快照。进程内实现优先使用“只保留最新值”的有界通道，慢renderer可以跳过已过期revision，不能让画面队列无限增长或反向阻塞感知。

ViewModel envelope至少携带schema_version、revision、单调生成时间、当前RoleStage/StatusBar/SystemOverlay及必要TTL/关联信息；renderer只画已由Output Arbiter裁决的结果，不持有两个内核、注册表或内部状态引用。触摸通过反向`UiIntent`窄契约交给host；旧触摸、重复触摸和renderer重启前的动作不重放。实体POWER/MAINT和安全关机不依赖renderer。

renderer在独立task或专用线程中执行，具体方式按显示库阻塞特性实测。host记录last_requested/last_presented revision、呈现耗时、跳过revision、错误/重初始化次数、心跳和可选缓存量；可恢复错误保持最近有效画面并重试，task退出由host监督重建并重新投递最新完整快照。原生段错误、abort和全进程OOM在P0仍由systemd重启整个host，不能宣称已有崩溃隔离。

是否升级为独立`skippy-renderer`服务由Orange Pi实测触发：renderer导致host崩溃/长阻塞/OOM，资源无法可靠归因或释放，事件到本地画面p95门被破坏，需要独立更新/重启，或引入Chromium/大型原生图形运行时等显著扩大故障域时，重开62B评审。若延迟、RSS/线程、温度、两小时稳定性和故障注入均通过，则维持同进程。拆分时只替换RendererPort传输适配器，不修改感知内核、OCLive或Output Arbiter语义。

原因：该方案用较低P0成本验证真实显示栈，同时把未来进程隔离限制为可替换传输层；明确退出门避免“可拆”沦为永远不验证的口号，也避免没有硬件数据时先制作完整IPC系统。

## ADR-046：Web前端、类型化资产引用与Host权威交互状态

状态：Accepted（2026-08-26；用户确认 `63C / 64A / 65C`，按实现边界修订为`63C-R1 / 64A-R1 / 65C-R1`）

Renderer技术方向冻结为Web前端，以Vue/TypeScript/HTML/CSS承载800×480角色舞台、状态栏、维护页和触摸交互，并为后续Live2D Cubism SDK for Web的TypeScript/WebGL路径留接口。P0仍只要求PNG表情、短文本和极简HUD，Live2D属于P1可选表现；禁用WebGL、模型加载失败或资源不足时必须回退PNG/System UI。Orange Pi具体浏览器/嵌入引擎、kiosk方式和是否触发ADR-045的独立进程退出门不冻结，必须比较冷启动、RSS/线程、磁盘、WebGL、触摸、温度和两小时稳定性。

Host不向renderer发送任意文件路径、URL或逐帧像素。ViewModel使用类型化`AssetRef`，至少包含asset kind、稳定asset_id、包identity/version、内容摘要及可选variant。Renderer只能通过已验证AssetManifest在只读角色资产根/受控缓存内解析。PNG可直接引用单资产；Live2D引用一个经过清单校验的模型bundle，内部`.model3.json`、纹理、motion/expression/physics等相对引用必须保持在bundle根内。缺失、摘要不符、越界或不兼容统一回退标准PNG并产生稳定诊断。

Host拥有功能页面、interaction_mode、配置锁、配对/校准/解绑/关机流程、操作授权和已确认配置结果；renderer只拥有滚动像素、按钮按压效果、页面过渡、动画时间、Live2D motion/expression混合和其它可丢弃的瞬时表现。Renderer以`UiIntent`请求动作，只有host接受并发布新ViewModel后业务状态才改变。重建renderer时恢复当前权威页面、角色表现语义和状态，不要求恢复动画的精确帧。

角色表现契约保持语义化，例如presentation_kind、expression_id、motion_id、intensity、loop、priority、TTL与related_event_id；Host和OCLive不每帧发送Cubism参数，也不认识WebGL对象。基础角色资产仍以OCLive角色资产SSOT为准，Gun Interaction Pack只选择设备事件下的表现偏好，不复制Live2D模型包。

Live2D接入另设发布许可门。开发验证可以先进行，但公开发行、允许加载多个第三方模型或形成可扩展角色生态前，必须按当时Live2D官方SDK Publication License与Expandable Application规则复核，不把开发期下载/验证权等同于无条件再分发权。

原因：Web前端最能复用现有Vue能力并承接Live2D WebGL；类型化资产引用保持ViewModel轻量且防止路径/缓存失控；Host权威+renderer瞬时表现既能安全恢复又允许流畅动画；引擎与许可留门避免在没有板卡和发行形态前冻结高成本假设。

## ADR-047：最新完整ViewModel、loopback Web桥与OCLive视觉资产复用

状态：Accepted（2026-08-27；用户确认 `66A-R1 / 67B-R1 / 68A-R1`，要求本地安全、保持未来自由并先跑通）

RendererPort P0只传最新完整ViewModel快照，不建立差量patch或第二条可靠动画事件流。Envelope包含schema标识、单调revision、生成时间和当前RoleStage/StatusBar/SystemOverlay/InteractionState；仍有效的短表现以有界`presentation_instance_id + valid_until`留在快照内。进程内通道只保留最新值，慢renderer跳过旧revision；host不等待逐帧ACK，只从独立RenderStatus读取last_presented_revision、延迟与错误诊断。协议以可序列化DTO定义，不依赖tokio watch、WebSocket或某个进程形态，未来可替换传输。

Web P0使用仅本机loopback的HTTP静态资源服务和WebSocket JSON桥：静态Vue产物只读，连接/刷新后立即发送最新完整快照；页面只可发送白名单UiIntent，携带会话、client sequence和所见view revision，由host重新验证当前锁、模式和授权。本地进程不自动可信：field Profile禁止监听非loopback地址、通配CORS、远程导航和开发服务器，要求严格Host/Origin/CSP、每次启动随机会话凭证、消息大小/频率限制、不可预测端口或等价保护，并禁止诊断日志泄漏凭证。安全实体键和关机不依赖Web连接。

角色视觉复用OCLive现有`portrait_catalog`、`visual_state_id`、`visual_presentation`和`performance_directive`，不在Gun Interaction Pack复制PNG/Live2D模型，也不建立第三种Visual Pack作为P0前提。四季宝loader验证角色根内相对路径与catalog闭集，并可建立本地运行时内容摘要索引，再投影为AssetRef；不要求现在修改OCLive标准角色schema。Gun Interaction Pack只引用稳定视觉/动作语义。若未来模型体积、独立授权或多视觉版本产生真实需求，可在AssetResolver后增加外置Visual Pack provider，而不改变ViewModel和角色事件契约。

P0实现范围只到PNG、短文本、HUD、维护页、完整快照重连和安全UiIntent；Live2D renderer、外置Visual Pack、差量协议、可靠动画队列和远程控制均不实现。扩展点必须有窄接口和测试，但不得提前制作未使用的provider、兼容矩阵或市场安装流程。

原因：完整快照以极低语义数据量换取重连自愈；loopback HTTP/WS复用Vue和未来独立renderer路径，同时通过零信任本地边界避免“只监听localhost就安全”的误区；复用OCLive已交付视觉设施避免角色资产分叉；传输与resolver抽象保留后续演进自由而不扩大P0。

## ADR-048：强类型感知三契约、最小三crate地基与独立伴随包导入

状态：Accepted（2026-08-27；用户确认`69A / 70B / 71A-R1`并补充项目正式命名与本机导入体验）

首个可实现感知契约采用`SensorObservation v0.2`、`PerceptionState v0.2`和`DeviceEvent v0.2`三份强类型DTO。SensorObservation只表示节点/适配器的低频观察；PerceptionState是带revision、freshness、confidence/source evidence和显式unknown的当前完整快照；DeviceEvent只表示感知状态机产生的有意义边沿。每个kind使用封闭payload、明确单位/范围和版本，禁止任意JSON穿透。`mode.changed`由Host拥有，连接、低电和系统故障进入状态/SystemCue，不伪装成角色事件。现有感知v0.1从未交付，只保留为历史草案；v0.2直接成为第一条兼容基线，不编写没有用户的v0.1 adapter。

当前单package启动骨架迁移为最小三crate workspace：`ailive-gun-spirit-contracts`只拥有DTO、版本和schema；`ailive-gun-spirit-perception-core`只拥有确定性融合/状态机并仅依赖contracts；`ailive-gun-spirit-host`拥有OCLive bridge、Input Scheduler/Output Arbiter接线、BLE/Linux/Web、导入器和composition root。host可以依赖contracts、perception-core与OCLive；前两者不得依赖OCLive或平台I/O。P0仍部署一个host进程，拆crate不等于微服务化。

命名分层固定为：仓库slug`oclive-四季宝器灵`；正式产品展示名`A.I.Live-ai枪娘器灵`；新crate/binary/service/schema使用ASCII前缀`ailive-gun-spirit`或`ailive.gun-spirit`。`四季宝器灵 / Skippy Spirit`、`oclive-skippy-spirit`和`skippy-host`只保留为灵感/迁移历史，不再生成新的公共技术标识。当前bootstrap在执行70B迁移时一次性改名，避免先做一半产生双重binary身份。

角色枪械反应正式采用独立“枪械交互伴随包”（Gun Interaction Companion Pack），格式ID为`ailive.gun-spirit.interaction/1`，受管安装目录为`content/gun-interaction-packs/<pack_id>/gun-interaction.json`。它与OCLive标准角色包分别导入、校验、版本化和存储，只通过`base_role { id, version_req }`关联；基础人格、记忆、通用Prompt和视觉模型仍以标准角色包为SSOT。多份伴随包命中同一角色时由本机绑定注册表显式选择活动pack/version，不按扫描顺序猜测，也不把binding写回角色包。

用户体验沿用OCLive本机导入传统，但不复用标准角色包loader混淆格式：维护页可选择电脑中的目录或压缩包；专用importer复制到临时区，限制大小/文件数并验证schema、identity/version、摘要、相对路径边界和基础角色约束，再原子安装到受管目录。导入源不直接执行、不作为运行时根、不被写回；失败不破坏已安装版本。P0伴随包只允许事件/模式反应、情绪偏置、主动概率、冷却和稳定视觉语义ID，不允许代码、shell、远程URL、任意路径、密钥、用户记忆或模型资产。加载失败采用fail-open：基础角色继续运行，枪械专属主动反应关闭并显示稳定维护原因。

原因：三契约让采样事实、持续认知和离散角色事件各自可测试；三crate提供真正的零依赖门禁而没有进程成本；独立伴随包既保持OCLive角色包纯净，又保留用户熟悉的导入与版本管理体验。新技术命名与正式产品对齐，同时把旧名限制在迁移历史中。

## ADR-049：Rust契约真源、双单调时间与显式未知事实

状态：Accepted（2026-08-27；用户接受建议`72A / 73B / 74A`）

`ailive-gun-spirit-contracts`中的Rust强类型是感知语义唯一真源。JSON Schema由相同类型确定性生成并提交仓库，CI重新生成后要求零差异；有效/无效fixtures负责表达生成器之外的跨字段语义不变量。BLE紧凑二进制、Web JSON和诊断输出只是显式codec/投影，不得成为并行业务契约。节点固件与Rust host通过共享golden vectors对拍正常帧、边界值、未知枚举、截断帧和版本拒绝。

时间模型采用节点与Host双单调时间，而不做分布式时钟同步。每条观察携带`node_uid + boot_id + sequence + node_monotonic_ms`，只在同一节点同一boot内完成去重、排序和回退检测；Host接收时补充`host_boot_id + received_monotonic_ms`，感知核心以Host单调时间、TTL和显式融合窗口判断跨节点新鲜度。不同节点的node monotonic值禁止比较，wall-clock只进入诊断。节点重启必须产生新boot_id并允许sequence归零；Host重启同样更换host_boot_id。

PerceptionState中的可失效事实使用判别联合`Fact<T> = Known{value, observed_at, fresh_until, confidence, source_refs} | Unknown{reason, since_revision}`。P0 UnknownReason闭集为`never_observed`、`node_offline`、`stale`、`uncalibrated`和`sensor_fault`。Known必须有value且不得携带unknown reason；Unknown必须有reason且不得把最后值继续作为当前事实。last-known只可进入隔离、明确标为historical的诊断摘要，融合、bridge、UI当前状态和角色上下文均不得消费它。

原因：Rust真源使核心枚举与状态机得到编译器保护，生成Schema保持跨语言可见；双单调时间解决BLE节点无共同纪元的问题而不引入同步协议；显式Unknown保留故障原因并从类型上阻止“断线等于松手”或“旧值继续生效”。

## ADR-050：证据强度定点化、有界来源引用与伴随包多版本回滚

状态：Accepted（2026-08-27；用户接受建议`75A / 76A / 77A`）

Known Fact使用`confidence_milli: 0..=1000`整数表示当前规则、阈值和校准版本下的证据强度/判定余量，而非统计概率。Unknown Fact不携带confidence。融合与状态转换可以按fact kind和calibration version使用版本化阈值；面向用户的UI只投影为稳定、不稳定、需要校准等语义，禁止把870显示成“87%概率”。整数表示避免MCU/Rust/JSON之间的浮点差异，同时不声称尚未经过统计标定的概率意义。

每个Fact最多保存4个有界`source_ref`，每项只包含`observation_id`、`node_uid`、`capability`及`relation: supports | contradicts`。完整SensorObservation、原始ADC/IMU和滤波中间值不嵌入PerceptionState；它们只进入有容量/时间上限的诊断与回放缓冲区，需要排错时通过observation id解析。DeviceEvent优先引用产生它的PerceptionState revision，不重复复制整条证据链。

枪械交互伴随包采用多版本并存和独立活动指针。受管布局细化为`content/gun-interaction-packs/<pack_id>/<version>/gun-interaction.json`，本机binding registry保存基础角色当前选择的`pack_id + version`。导入新版本先在临时区验证并原子安装，只有明确用户确认或受控升级成功后才切换活动指针；旧版本保留用于回滚。相同id/version/digest重复导入是幂等成功，相同id/version但digest不同必须拒绝为稳定`version_collision`，不能覆盖或暗改既有版本。卸载/垃圾回收是显式维护操作且不得删除活动版本。

原因：定点证据强度便于确定性跨平台实现又避免虚假概率；有界引用保留可解释性而不让快照携带原始流；伴随包体积很小，多版本存储成本远低于安全升级、A/B验证和快速回滚的价值。

## ADR-051：显式版本交集、语义revision与声明式模式覆盖

状态：Accepted（2026-08-27；用户确认建议`78A / 79A / 80A`）

BLE连接先以`NodeHello`报告明确的`supported_contract_versions`和各版本capabilities；Host从双方实现集合选择最高共同版本，不根据SemVer、未知字段或版本号大小猜测兼容。没有共同版本时，已信任节点仍可在维护页显示，但状态为`incompatible_protocol`且任何Observation不进入融合。协商后出现未知variant、非法字段或错误frame时先隔离该帧并记录稳定协议故障；有界窗口内重复违规达到HostProfile阈值后隔离节点。未知数据不得被包装成字符串送入感知核心、bridge或OCLive。

PerceptionState的revision只在Fact分类值/Unknown原因、节点健康或其它权威语义实际变化时递增；接受了一条数值不同但没有改变语义结果的Observation不产生新revision。Host可在新订阅、BLE/renderer重连、恢复或健康心跳时重新发送相同revision的最新完整快照，但这只是重发，不得生成DeviceEvent、角色回合或新状态日志。心跳周期由HostProfile和实测决定，不成为跨组件契约。DeviceEvent仍只由状态转换产生，并引用因果PerceptionState revision。

实现澄清（2026-08-27）：同值新Observation必须能延长TTL，否则1 Hz CurrentState快照无法自愈。故同revision允许刷新Fact的`observed_at/fresh_until/confidence/source_refs`证据包络，Replay会保存最新包络；分类值、Unknown原因、节点健康、host identity与revision才是边沿比较语义。包络刷新不得触发DeviceEvent或sensor turn。这是ADR-051“数值变化不等于语义revision”的直接实现，不新增角色或硬件语义。

枪械交互伴随包P0使用封闭声明式reaction map。每个受支持DeviceEvent id至多一条`default`规则，并可在`modes`下为已登记interaction mode提供字段覆盖；确定性解析顺序为`default → current mode override → Host安全/用户上限`，未提供覆盖就使用default。P0 schema只允许情绪偏置、主动回复概率、冷却和稳定视觉语义ID等白名单字段，不支持条件表达式、优先级规则链、JavaScript、脚本、任意Prompt或运行时代码。event/mode/visual id必须由对应注册表验证，不能运行时猜测。

原因：版本交集让兼容来自双方明确承诺；语义revision把高频观察和真正状态变化分开；default加模式覆盖足以表达把玩/场地差异，同时避免在P0引入难以验证的规则引擎。

## ADR-052：Host拥有主动门控、模式显式禁用、绑定钉住与四职责门禁

状态：Accepted（2026-08-27；用户确认`81A / 82A / 83A`，并要求新增功能跨越四条职责基线时必须重新做架构决策）

主动sensor turn的概率只由Host Input Scheduler执行。处理顺序固定为：DeviceEvent有效性/去重/TTL → default+当前mode反应解析 → `active_reply`状态 → Host安全/用户主动程度上限 → cooldown → `probability_milli`抽取 → 创建或跳过sensor turn，之后仍服从显式语音/文字优先及短合并窗口。测试通过RNG port注入固定序列；生产使用Host会话随机源并记录decision id、有效概率和通过/跳过结果。感知`confidence_milli`表示证据强度，调度`probability_milli`表示真实抽样概率，二者不得共用字段、阈值或所有者。

模式覆盖使用类型化`active_reply: disabled | probabilistic{probability_milli}`。disabled只禁止该事件独立开启sensor-origin PrimaryInput，不删除PerceptionState、DeviceEvent、System UI、诊断或中性DeviceContext，也不阻止事实在用户语音/文字回合中作为AuxContext。感知内核和OCLive都不能自行启用/禁用该策略；包只声明，Host最终裁决。

活动binding精确保存`pack_id + pack_version`。Host在每次加载及基础角色包升级后重新检查伴随包的`base_role.id + version_req`；兼容则保持，不兼容则标记`inactive_incompatible_role`，停用枪械专属反应并fail-open运行基础角色。维护页可列出已安装兼容候选并提供显式确认切换，但不得自动选择最新版本、静默改绑或忽略版本范围。

以下四条是必须守住的职责基线：枪械感知内核只产物理事实/状态转换；枪械交互伴随包只声明受限策略；Host/Input Scheduler只执行模式、上限、冷却、概率、合并、取消和回合投递；OCLive只结合角色包/记忆/上下文完成角色理解与表达。任何新增功能如果跨越这些职责，必须在实现前显式写出理由、替代方案、耦合成本、故障与回退影响并新增ADR，不能因实现便利顺手耦合。既有Output Arbiter唯一控制前端的边界不变。

原因：主动性是宿主调度政策，不是传感事实或模型自由决定；显式disabled避免用概率0隐藏语义；精确绑定避免角色更新后无声改变行为；跨层ADR门禁把当前清晰地基变成可执行的长期约束，而不是只存在于图中的建议。

## ADR-053：协议分级隔离、本机未签名包边界与独立Host状态库

状态：Accepted（2026-08-27；用户确认`84A / 85A / 86A`）

节点应用协议违规使用分级隔离而非首错永久封禁或无限容忍。P0 HostProfile初始值均为`ESTIMATED`：单条未知/非法应用帧丢弃并限速计数；滚动10秒内累计3条则断开并隔离30秒；连续3个连接周期都触发隔离后进入持久`protocol_fault`，需要维护页明确确认才能重新启用。超过长度上限、协商后仍使用错误版本等硬违规可以立即终止本次连接。只有已经进入应用解析层的协议违规计数；RSSI差、BLE链路重传、普通断连和未通过链路完整性的数据不计。阈值必须经fault injection与实机MEASURED数据覆盖，不写入跨组件contract。

P0本机导入的枪械交互伴随包不强制发布者数字签名。每个包仍必须提供规范化内容清单和SHA-256，importer验证schema、identity/version、摘要、路径边界、重复规范化路径、文件数、展开大小，并拒绝绝对路径、`..`越界、符号链接、硬链接和设备文件；用户明确确认后才安装。SHA-256只证明内容一致性，不证明作者身份，UI不得显示成“可信发布者”。该裁决建立在P0包禁止代码、脚本、远程URL、任意Prompt、密钥和模型资产的窄边界上。未来市场/第三方分发若需要Ed25519、发布者身份、信任库、换钥或吊销，必须独立ADR和schema升级。

已安装pack索引、精确`pack_id + pack_version` binding、禁用原因以及有界导入/切换审计保存在A.I.Live Gun Spirit Host独立SQLite状态库，逻辑名`ailive-gun-spirit-state.db`，实际路径由平台HostProfile决定。它不得与OCLive聊天、角色记忆或关系数据库混表。受管pack目录保持不可变；DB只保存identity/version/digest/受管相对路径、binding与审计。活动切换/回滚使用事务并启用foreign keys和schema migrations。维护页/诊断导出经过隐私过滤，不泄露原始导入绝对路径。SQLite journal/synchronous参数以及“文件原子安装+DB提交”之间的崩溃恢复状态机留待后续持久化批次冻结。

原因：分级隔离兼顾偶发错误与故障风暴；P0无代码本地配置无需提前背负发布者PKI，但必须诚实区分完整性和身份；独立SQLite提供事务、审计和回滚而不污染OCLive角色数据所有权。

## ADR-054：v0.2物理观察、三态载体事实与六个中性事件闭集

状态：Accepted（2026-08-27；用户确认`87A / 88A / 89A`）

SensorObservation v0.2 P0只允许五种低频物理观察：`GripContact{contact,strength_milli}`、`MotionClass{idle|moving}`、`OrientationEstimate{gravity_mg_x/y/z,quality_milli}`、`AuxControlContact{control_id=primary_trigger,contact}`和诊断`ShockObserved{severity_milli}`。OrientationEstimate是节点滤波后的低频重力方向，不包含目标/yaw语义；Host结合载体坐标与用户calibration派生pose。电量、固件、校准版本和故障码留在NodeStatus；unknown由Host根据TTL/断连/故障/校准有效性派生。正常运行禁止ADC原始流、完整IMU波形和任意厂商payload。

PerceptionState v0.2 P0包含`carrier_state: Fact<Standby|Held|Ready>`、`front_grip`、`rear_grip`、`motion`、`pose`、`primary_control`及有界`node_health`。旧`Active`从carrier state删除；屏幕睡眠/进程生命周期属于Host。只读辅助触点保持独立Fact，不把carrier状态推进成Active，也不证明发射器机构动作。shock不成为当前业务Fact，只留诊断/机械检查提示。

DeviceEvent v0.2 P0 kind闭集为`carrier.held_entered`、`carrier.held_exited`、`carrier.ready_entered`、`carrier.ready_exited`、`control.primary.engaged`和`control.primary.released`。每项引用因果PerceptionState revision；同一revision多个边沿由Input Scheduler合并。节点在线/低电、interaction mode、Host生命周期和shock走状态/SystemCue/诊断。不存在`device.triggered`、`weapon.fired`、弹道、目标或actuator事件；伴随包可将中性边沿解释成角色体验，但不能改写底层含义。

原因：节点输出物理观察能把用户校准和多节点融合集中在感知内核；三态carrier避免把触点观察误写成真实击发；六个中性边沿足以驱动拿起、准备、放下与辅助动作体验，同时保持安全边界和强类型闭集。

## ADR-055：可恢复包安装、实机SQLite耐久与分级诊断留存

状态：Accepted（2026-08-27；用户确认建议`90A / 91A / 92A`）

伴随包安装采用有事务日志的暂存恢复状态机。任意外部目录、压缩包或可移动介质先复制到受管pack根目录同一文件系统内的`.staging/<operation_id>`，再完成schema、规范化路径、manifest和digest验证。独立SQLite记录`install_operations`及阶段、目标受管相对路径和digest；验证成功后只在受管根内部使用原子rename发布不可变版本目录，再由下一笔事务标记`installed`。安装完成和活动binding切换是两次独立操作，不因导入成功自动启用。启动恢复遵循收敛规则：staging存在而正式目录不存在时按journal继续或清理；正式目录存在且DB仍为installing时重验digest后补记完成；DB称installed但文件缺失时标记`missing/disabled`；无DB记录的孤立目录进入隔离，不自动注册。不得假设跨文件系统rename具有原子性，也不得先让DB指向未完成文件。

Linux field HostProfile对`ailive-gun-spirit-state.db`采用`journal_mode=WAL`、`synchronous=FULL`、`foreign_keys=ON`和明确`busy_timeout`。checkpoint只在待机/维护等受控时机执行并设置WAL容量/时长上限；非正常启动执行恢复检查与`quick_check`，出现异常时保持基础角色fail-open并禁用受影响binding。运行中备份必须使用SQLite备份机制或一致性导出，不能只复制主`.db`并遗漏`-wal/-shm`。development profile可以显式选择WAL+NORMAL提高迭代速度，但必须可见标识；所有发布、断电和恢复测试使用field FULL配置，实测提交延迟、关机预算、写放大和WAL增长后再覆盖参数。

诊断按数据类别独立限额，不设一个会互相挤占的全局无限日志。P0初值均为`ESTIMATED`：安装/绑定/安全审计保留90天或1000条；协议/节点故障摘要保留30天或5000条；资源/renderer日志同时受时间和容量限制；原始传感窗口在field默认关闭，只能由用户进入维护模式后有界采集，初值10分钟或20 MiB并自动过期。达到任一时间/条数/容量上限即清理最旧数据，但活动安装事务和当前故障状态不作为普通日志淘汰。Host诊断不得复制OCLive对话、原始语音或角色记忆；导出必须由用户明确触发，并携带manifest、版本、时间范围和脱敏摘要。

原因：文件系统操作和SQLite事务不能组成天然的跨介质原子提交，最小journal状态机才能在任意断电点恢复到可解释状态；field设备具有可拆充电电池，低写入量下FULL耐久优先于微小吞吐收益；分级留存既保留实机排错证据，又不让高频故障挤掉安全审计或越过OCLive隐私边界。

## ADR-056：持久化白名单、两级低电断电与安静恢复门

状态：Accepted（2026-08-27；用户确认建议`93A / 94A / 95A`）

重启状态采用按所有者和生命周期定义的持久化白名单。Host持久化节点注册/信任、伴随包安装与精确binding、用户确认的校准profile、显示/交互配置及结构化生命周期记录；OCLive在自己的数据库中持久化角色身份、记忆/关系、会话连续摘要及预生成关机/恢复表达缓存。PerceptionState的Held/Ready、grip、motion、pose、primary control、ADC/IMU滤波窗、去抖计时器、BLE session、boot/sequence上下文、概率抽取和短期单调时钟cooldown全部是瞬态，重启后不得恢复为当前事实。新字段默认不落盘，只有明确owner、用途、迁移、清理和恢复语义后才能进入白名单。

产品field电源架构采用两级低电策略、Host Shutdown Coordinator和可控主电源切断。Low阶段只降背光并停止相机、Live2D、后台生成等可选重负载，感知、节点健康、实体键、最低System UI和数据库继续运行；Critical Reserve阶段拒绝新的安装/更新/校准/角色回合，使用正常电量时预生成的本地关机表达，限时提交OCLive连续状态与Host生命周期/关键事务，完成受控checkpoint，关闭背光并调用Linux poweroff。外部低功耗监督MCU、PMIC或锁存负载开关在收到完成信号后切断主5 V；软件超时或失联时按经实测的hold-up截止时间兜底切电，电芯保护板只承担过放/短路最后防线。节点独立低电不触发主机断电，只使对应Fact降级。具体阈值、迟滞、deadline和预留Wh必须通过电芯低温/老化、峰值负载和连续断电试验回填，不用一个未经测量的百分比代替。

阶段边界不变：桌面/首轮P0 bring-up可以继续使用成品5 V电源、手动安全关机和safe-to-cut指示验证完整软件次序，这相当于暂时由人完成最终切电；它不能被宣传为成品自动低电保护。进入field成品门前必须补齐可信电量/临界信号、可控断电和硬件兜底，不依赖普通充电宝或BMS突然掉电。

启动采用安静恢复门：`Booting → StorageRecovery → OCLiveContinuityRestore → NodeRearm → SensorBaseline → Ready|Degraded`。恢复期间Output Arbiter显示Host System UI；先收敛安装journal、检查SQLite和恢复各自所有者的持久状态，再让节点以新session/boot上下文连接。全部瞬时Fact先为Unknown；首次稳定新鲜观察只建立物理基线，不从Unknown产生held/ready/control DeviceEvent，也不补播关机前边沿。基线完成后出现的新物理转换才允许产生事件。缺失节点在有界等待后进入Degraded，相关Fact继续Unknown，基础OCLive和维护UI仍可用。

clean/unclean及关机reason可以由Host作为一次类型化LifecycleContext交给OCLive，并与同一生命周期记录的pre-shutdown/resume消费位去重；它不进入Gun Perception Kernel或DeviceEvent。OCLive可以在设备就绪后结合连续状态表达一次恢复，但不能把系统错误码、路径、堆栈或旧传感事件交给角色。等待窗口、必要capability集合和进入Degraded的时限属于HostProfile/实测，不成为跨组件常量。

原因：角色连续性是持久的，物理现实却必须重新观察；把二者分别恢复可以避免旧Ready/触点伪造新动作。Linux halt本身不会切断电池，产品必须把“保存完成”和“真正断电”纳入同一能源链路；安静恢复门则避免每次握着设备开机都被误认为刚刚拿起，同时允许节点缺失时可解释地降级。

## ADR-057：把既有体验决定拿给 wargame 爱好者简单复核

状态：Accepted（2026-08-27；用户补充并简化此前决策）

当前不开展泛化或正式用户研究。先从已经冻结的ADR中筛出会直接影响玩家体验的决定；等对应界面mock、质量假体或结构原型可以展示时，再找几名wargame爱好者简单复核。当前筛为八组：首版功能边界、外挂模块化、一体式三轴屏幕、前后无线节点、Held/Ready/Standby与用户校准、实体键/模式/屏幕层级、即时反馈与角色回应频率、续航/换电/内容导入。没有实物或界面可体验的项目不要求凭空回答。

参与者可以口述，由项目所有者代填，不规定样本量、量表、计时、A/B或画像分层。爱好者意见只补充用户体验，不直接决定电芯/充电/保护、BLE协议、传感阈值、OCLive/感知职责、持久化或火控边界。发热、松脱、夹手或遮挡必要操作时立即停止并工程复审；其它回答保留原话，重复问题优先复现，不因某一名资深玩家的意见立即冻结设计。

正式产品化后，再单独决定是否需要结构化任务、更多样本、A/B、量表、录像授权、画像分层和统计验收。当前零散回答不得宣传为市场验证。

原因：把已经做出的决定拿给真实玩家看，可以低成本发现开发者视角盲区；按决定逐项问比泛泛询问“产品怎么样”更可执行，过早建立完整研究体系则会增加负担且不能替代原型与实测。

## ADR-058：余弹采用循环短计数与磁性绝对范围双源融合

状态：Accepted Concept / Deferred to P1+（2026-08-28；用户确认方向，硬件参数等待实测）

余弹扩展保持为G5之后的独立只读能力，不进入P0，也不修改当前v0.2 `SensorObservation / PerceptionState / DeviceEvent`闭集。连续估算以可信的`mechanism cycle completed`为主计数源；磁性随动件和外部模拟霍尔阵列只在满足姿态与稳定条件时提供绝对范围锚点。Host同时保存长期诊断循环数和自最近可信锚点开始的短计数，新锚点用于收窄范围、重置短计数和发现两源冲突。循环只证明机械循环，不证明水弹实际通过；无逐弹通过证据时不得宣称精确颗数。

机械设计分成两个adapter：散装电动水弹弹匣候选采用半圆/D形低重心浮板、左右有余量的连续导向和靠外壁密封磁铁；所谓卡位只防脱/防旋转，不允许设置会悬住浮板的离散档点，并必须先解决顶部装填会埋住浮板的问题。弹簧供弹弹匣不另加浮板，而在原托弹板后方或非受力侧隐藏小型磁铁座。内部只保留无源磁性目标，外部弹匣套以非磁性机械基准固定霍尔阵列；任何部件失效、卡滞、脱落或节点断电都不能进入供弹机构或改变原弹匣工作。

循环源优先级为原火控明确只读输出、原循环传感器的隔离高阻旁路、与波箱经实测一比一的回膛件观察、独立循环传感器；电流、震动和声音只能作为低可信度回退。RADIAN 2.0公开资料尚不足以证明存在可用循环输出，未知火控信号不得直连MCU。实施时必须先建立新协议版本/能力协商、失败测试和独立测试切片；每个循环只更新HUD，不创建高频角色回合，OCLive只接收规范化档位/范围上下文。

融合遵循装匣会话内单调不增加、稳定观察才建锚、单源失效逐步降置信、两源冲突输出`unknown/source_conflict`和拔匣/明确装填后才允许上调。首个原型只验证一个智能弹匣；多弹匣不得依赖BLE RSSI猜身份。完整机械、接口、故障、测试和开放项以`MAGAZINE_AMMO_ESTIMATION.md`为SSOT，原始实测填入`test-worksheets/07-余弹融合与弹匣结构测试表.md`。

原因：循环计数提供连续和高分辨率变化但会因空击/供弹失败累计误差；霍尔范围能够重新观察绝对位置但会受姿态、摩擦和散装弹堆影响。短计数加绝对锚点把误差限制在相邻可信观测之间，同时保留失败开放和可解释降级，不需要让角色内核理解火控、ADC或弹匣机构。

## ADR-059：终端机外观、开发板板载主动散热与一体蜂窝后盖方向

状态：Accepted Direction（2026-08-30；用户确认纯视觉方向，并澄清风扇是开发板的板载散热风扇，背面是一体蜂窝后盖）

终端机外观采用微缩三防数据平板与相机半笼语言：3.5英寸横屏内缩于正面保护边框，哑光黑半笼、沉头紧固件、梯形装甲收边、战术灰和少量荧光黄构成主要视觉；背面使用蜂窝格栅/凹纹与四角泡棉形成贴枪轮廓。正式外观标识使用`A.I.LIVE // GUN SPIRIT`、`SPIRIT CORE`等当前名称，旧`SKIPPY`不进入新丝印；低压系统不使用`HIGH VOLTAGE`虚假警示，不存在的接口不得制作带功能文字的装饰盖。实体键继续按既有安全裁决保持`POWER + MAINT`两颗。

内部位置关系冻结为正面屏幕和保护层、中部独立可更换主板托盘与主板、直接安装在开发板SoC散热器上的小风扇、背面一体式可拆蜂窝后盖。风扇不固定在后盖上，后盖也不以PCB、元器件或风扇作为机械限位。后盖内侧按照主板、接头和散热组件包络设置流线型局部让位凹槽，使外壳贴近硬件，同时保留装配公差、绝缘层和气流净空。风扇料号、尺寸、厚度、PWM/反馈、进出风方向、常开/温控策略、具体散热片及凹槽最小净空保持OPEN，由板卡、renderer、完整外壳和贴枪工况实测冻结。电池不得阻断SoC散热组件的主风道，具体在下边、侧边或后盖非热区的布置由温升、质量、重心与4小时门裁决。

背面同时是潜在贴枪/贴墙面，蜂窝不能只在背面完全敞开时有效。四角泡棉必须配合刚性止挡保留中央空气间隙，侧边提供真实进/出风路径；泡棉不能成为唯一限位。蜂窝需防止手指、挂绳、泡棉和软包装触及扇叶。到货测试必须覆盖裸板、被动散热、风扇开启、贴枪收纳、蜂窝25%/50%遮挡、积尘与风扇停转；主动散热失效时Host降低可选renderer/后台负载并保留System UI与安全关机。

本ADR只冻结外观与层叠方向，不冻结最终CAD、IP等级、材料、总厚度、主板型号、风扇能力、后盖凹槽尺寸或电池BOM。详细视觉和工程占位以`TERMINAL_ENCLOSURE_VISUAL_V0_1.md`为入口，三轴、泡棉和力路径继续以`DISPLAY_ASSEMBLY.md`为SSOT，热数据填写`test-worksheets/02-主机与屏幕测试表.md`。

原因：明确三层位置关系可以让视觉、维护和散热从第一版CAD共享同一骨架；把贴枪遮挡和风扇故障提前纳入，则避免蜂窝沦为装饰或在收纳时形成被堵死的单点风道，同时不在实物数据到来前过早锁定高风险热参数。

## ADR-060：主机切换为Orange Pi Zero 3W 6GB/A733，本地3B进入可选实测

状态：Accepted with Measured Gates（2026-08-31；用户确认更换主板，本地3B质量与速度继续实测裁决）

当前模块化bring-up主板由Orange Pi Zero 2W 2GB切换为Orange Pi Zero 3W 6GB / Allwinner A733。该板约65×32 mm，采用2×Cortex-A76 + 6×Cortex-A55和LPDDR5，能够在屏幕后继续保持Zero级平面包络，并为OCLive、Web/PNG renderer、未来Live2D、相机和CPU量化3B提供明显大于旧2GB路线的内存/计算余量。ADR-022中关于Zero 2W的采购优先级、mini-HDMI链路和2GB资源门被本ADR覆盖；双区域BLE、三轴同舱、零跨轴电气线和Lyra有界回退仍保持。

视频链改为`Zero 3W USB-C DisplayPort Alt Mode → 主动DP转HDMI → 现有HDMI屏`。主动转接器的芯片、EDID、800×480时序、冷/热启动、功耗、温升、带接头包络和Linux BSP兼容性成为必测项。Zero 3W使用直接安装在开发板SoC上的散热器/小风扇，一体蜂窝后盖只保护、让位和导流。原5V/3A级电源只能作为起测点，不能在A733、风扇、主动视频转接、屏幕和推理峰值实测前继续视为充分条件。

6GB允许把3B `Q4_K_M`/相近4bit GGUF作为本地角色表达候选。首轮使用ARM64 llama.cpp CPU路径和独立可失败服务；上下文、量化、核心亲和、prompt缓存、TTFT、decode、RSS、功耗和温度均不冻结。公开同板证据只直接证明0.5B CPU结果，并显示两个A76核心可能优于全部八核；A733 NPU当前对较大语言模型的导出/质量仍不成熟，所以3 TOPS和GPU/NPU不得写入P0承诺。模型失败必须回到网络模型或审核台词池，不影响感知、即时UI、实体键、数据库与关机。

用户给出的语音方向是10–40字的高情绪角色短句。它们进入`LOCAL_3B_INFERENCE.md`质量语料，但P0仍不实现TTS/扬声器。未来语音采用“时机敏感的审核/预生成音频立即播放 + 可选3B动态补充文本再交TTS”的分层；不能让“喵！、自己人、急救”等即时喊声串行等待3B和TTS。受伤、队友、接近等事实必须来自明确上游事件/用户输入，当前FSR/IMU不能自行推断。

本板未来可以承载P1只读相机、拐角视野、录像或HUD实验，但仓库既有边界不变：不实现/宣传自动瞄准、弹道计算、目标识别或火控。质量和速度按`LOCAL_3B_INFERENCE.md`与`ORANGE_PI_BRINGUP_WORKSHEET.md`回填；在MEASURED数据前只冻结主板采购方向，不宣称本地3B、NPU、四小时续航或最终外壳已经达标。

原因：新板以相近Zero级尺寸提供6GB内存和更强异构CPU，使本地表达与后续视觉不再因2GB硬门提前被排除；同时，主动视频转接、新SoC BSP、主动散热和更高峰值负载引入新的集成风险。把主板定向与模型体验分开裁决，可以立即停止旧板采购，又不会把“内存装得下”等同于“角色质量、响应速度和续航已经可用”。

## ADR-061：到板后比较3B及以下三个尺寸档，训练按问题类型逐级进入

状态：Accepted Test Direction（2026-08-31；用户确认到板后进行对话测试，并比较3B以下小模型）

本地角色表达不在到货前冻结单一模型。实机矩阵覆盖约3B、1.5–1.7B和0.5–0.6B三个代表尺寸档；“都试”表示每档至少一个有中文角色能力、许可证和部署链可接受的候选，不要求穷举所有公开checkpoint。每档先使用原始量化模型和关闭扩展思考的短输出基线，再加入相同的有界角色上下文，使用同一场景ID、允许事实、角色版本、目标长度和评分表比较TTFT、decode、RSS、功耗、温度、事实一致性、角色感、中文自然度与重复率。

RAG只负责检索稳定角色核心、当前模式/关系、已确认事实、少量相关表达示例和近期记忆，不用于弥补基础理解能力，也不允许把未确认现实写成事实。若问题主要是角色口吻，则在PC GPU或云端做LoRA/SFT并把合并量化产物部署到Zero 3W；若问题主要是时延，则优先较小尺寸档；若问题是事实错误，则先修契约/检索。0.5–0.6B全参数微调只有在LoRA已有稳定对照、数据集经过人工复核且另有通用能力保留测试时才评审，不成为默认补救路径。

最终允许按任务分流而非只保留一个模型：即时事件继续走审核语音池；快速、窄语义短句可走较小模型；需要关系/记忆组合的短对话可走较大本地模型或网络模型。任意本地模型仍是可失败RoleCue provider，不进入感知、System UI、实体键、持久化与关机硬依赖。完整矩阵和原始证据分别以`LOCAL_3B_INFERENCE.md`和`ORANGE_PI_BRINGUP_WORKSHEET.md`为准。

原因：模型规模、角色风格、检索质量和板端时延是四个不同变量。先做可比较的尺寸档基线，能避免把RAG错误误诊成模型能力不足，也避免在数据尚未稳定时用全参数训练提前制造成本和灾难性遗忘风险；允许按任务分流则保留未来自由度。

## ADR-062：建立硬件开发基线、项目边界与逻辑分层文档体系

状态：Accepted（2026-09-04；用户确认硬件到货，并要求先学习硬件工程、重设文档体系、归并四散文档和确认四季宝边界）

项目新增`PROJECT_BASELINE.md`作为从桌面半闭环进入硬件开发的共同起点，新增`PROJECT_BOUNDARIES.md`明确本仓、OCLive、角色内容、P0/P1和禁止范围，新增`DOCUMENTATION_SYSTEM.md`定义权威顺序、文档角色、状态词表、变更联动与外部归并规则；`docs/README.md`成为唯一导航入口。硬件到货状态只记为`REPORTED_RECEIVED`，在丝印、批次、完好、接口、电压和连接器方向逐件核验前不得升级为`MEASURED`或开始组合上电。

本轮不批量移动既有文档。当前工作树存在大量未形成干净基线提交的代码与文档变化，立即移动会混淆技术修改和路径重构并制造断链；先通过逻辑分类消除“哪个文件权威”的歧义，待形成可追踪提交后再用单独的纯重构提交物理迁移目录。桌面个人文档通过`history/EXTERNAL_DOCUMENT_AUDIT_2026-09-04.md`逐项审计：相同内容指向仓库canonical，旧导出标记SUPERSEDED，不复制成第二套SSOT；明确独立且涉及自动瞄准的思维实验排除在本项目与backlog之外。

硬件bring-up顺序固定为身份/外观→断电电阻/极性→主板单独→屏幕单独→视频→触摸→Host/UI→单节点→双节点→节点电池→主机移动电源→机械假体→上枪静态→场地。每一步只增加一个变量，异常热、气味、限流、反供、反复brownout、极性不明、电芯异常或改变原载体机构动作时立即停止。学习入口为`HARDWARE_ENGINEERING_GUIDE.md`，但具体接线仍必须服从datasheet、领域SSOT和实测工作表。

原因：硬件阶段的主要风险已经从“缺少想法”转为“文档重复、状态混淆和同时接入过多变量”。先建立权威、边界和可重复bring-up基线，能让项目所有者边学边做而不把旧副本、模型推断或一次成功上电误写成工程事实。

## ADR-063：先做「PC 前置 + 枪侧瘦客户端」桌面形态，独立主机舱降为后续 field 形态

状态：CANDIDATE（2026-09-23；项目所有者提出「放到电脑前把玩、枪侧做小、用 ESP32 级别 MCU、算力交给 PC」的新思路，尚未裁决）

把「算力放在哪」与「产品形态」解耦，定义两个共享同一契约与感知内核的宿主形态：

- **P0-A　桌面瘦客户端（建议先做）**：枪侧只保留传感器 + 最小 MCU（ESP32 级别，可选微型 SPI 屏与本地按键）；**PC 作为唯一 Host**，运行本地 OCLive、角色包、记忆/状态、Output Arbiter、本地大模型与 TTS/ASR；显示与声音由 PC 承担（PC 显示器 + 音箱/耳机）。
- **P0-B　独立主机舱（保留但后置）**：现有 Zero 3W 6GB + 3.5 英寸 HDMI 屏 + 主电池 + 三轴舱形态按已冻结基线继续；其验证门（4 h 续航、移动舱 ≤350 g、厚度 ≤40 mm、热与降频、本地 3B 及以下矩阵）**不因 P0-A 取消**，只是不再阻塞近期进度。

两种形态共享：v0.2 契约（`SensorObservation`/`PerceptionState`/`DeviceEvent`）、`perception-core` 的确定性边沿、Output Arbiter 的「Host 拥有屏幕内容」原则、节点注册/信任/校准语义与工具链。差异只在物理适配器与组合根，即既有的 `development` 与 `field` 两套 HostProfile 划分；因此这不是架构分叉，而是执行顺序与形态的重新排序。

枪侧 MCU 候选（**本 ADR 不冻结**，由台架实测裁决）：ESP32 / ESP32-S3 内置 Wi-Fi，可直接以 UDP/WebSocket 回传，省去 BLE 配对与主机端 dongle，S3 还能驱动微型 SPI IPS 屏；若最终仍需要下场独立形态，则保留 XIAO nRF52840 的低功耗 BLE 路径。裁决数据至少包括：FSR 分压的 ADC 线性与噪声、Wi-Fi 发射峰值电流、两节点并发下的丢包与 p95 延迟。

P0-A 直接解除的既有阻塞：随附 Mini HDMI 线的 DDC 故障（`GS-HW-002`）与 3.5 英寸屏的 EDID/时序不确定性**不再位于关键路径**——桌面形态可以不使用 HDMI，改用 PC 显示器或 MCU 驱动的 SPI 屏。

边界不变：产品仍是娱乐、把玩与角色陪伴；不实现或宣传智能火控、弹道计算、自动瞄准、武器控制或军警用途；P0 不接载体内部火控、扳机或供弹机构。「设备必须能本地启动、断网可用」这条红线在 P0-A 中的含义是 **PC 本地**（全部闭环在 PC 完成，不依赖云端）；在 P0-B 中仍指独立板端本地。

原因：当前硬件形态的主要成本来自「把算力与能源塞进枪上」——4 h 电池、移动舱质量与三轴刚度、散热与显示接口；而近期真正要回答的问题是「感知 → 状态机 → 角色 → 反馈」是否成立，不是板端能跑多大模型。把 Host 放到 PC 可立即获得更大模型、TTS/ASR、更低成本枪侧电子与更快迭代，同时把独立形态保留为带明确实测门的后续目标。两者共用契约与内核，避免出现第二套真源。

裁决前的待办（**未裁决前不做**）：

1. 台架实验：ESP32 与 XIAO 各做 FSR ADC 线性/噪声对比；ESP32 Wi-Fi 回传的丢包与 p95；两节点并发。
2. 形态裁决：P0-A 是否成为 P0 主线——涉及 `PROJECT_BASELINE.md`、`ROADMAP.md`、`HARDWARE_IMPLEMENTATION_PLAN.md`、`test-worksheets/` 与采购清单的同步更新。
3. 体验复核：按 `UX_RESEARCH.md` 的简短问题，把「束在电脑前的把玩感」拿给真实使用者（含项目所有者本人）确认，而不是由个人偏好直接改写边界。
4. P0-A 数据回来前，**不追加 P0-B 硬件采购**。

### ADR-063 补充：距离是决定形态的第一变量（2026-09-23）

场景分层与可用链路：

| 使用场景 | 距离量级 | 首选链路 | 说明 |
|---|---|---|---|
| 桌面把玩 | 0–2 m | 有线 USB 串口 / BLE | 零射频风险，最适合先验证协议与状态机 |
| 同一房间 | 2–10 m | BLE 或 Wi-Fi 2.4G（经路由） | 两者都够用；取舍看功耗与主机端便利性 |
| 隔 1–2 道墙 | 10–25 m | **Wi-Fi 2.4G 经路由（PC 走有线）** | 路由器天线远好于 PC 端蓝牙 dongle |
| 全屋游走 | 25 m 以上 | Wi-Fi + 可选中继节点 | 该量级 BLE 不可靠 |
| 无 PC 的场地 | — | P0-B 独立主机舱 | 不由本形态覆盖 |

三条容易被低估的事实：

1. **PC 端常是弱环**：台式机多数没有蓝牙，USB dongle 天线质量参差；"设备 → 路由器 → 有线 PC"这一跳通常比"设备 → PC 蓝牙"更远也更稳。
2. **枪体遮挡与天线位置**：金属或碳纤护木、贴胸持握都会造成明显衰减；节点天线必须留在非金属外露面，必要时改用外置天线（IPEX）。这与"传感器默认不走机匣内部线"出于同一逻辑。
3. **低速率可以换距离**：本形态只需约 1 Hz 状态快照与边沿事件，可用 BLE 5 Coded PHY（Long Range，125 kbps）显著换取距离，代价是链路两端都要支持。

**距离是梯度，不是悬崖**：链路变差时复用既有降级语义——相关事实按 TTL 进入 `unknown`、界面进入可见 Degraded、角色退回固定/审核台词池，而不是整体失效。近 PC 时使用 PC 的完整算力（更大模型、TTS/ASR、可选娱乐视觉）；链路退化时保留感知与最小反馈。这样距离只改变**表达能力**，不改变感知与安全语义。

**距离扫频实验（并入前述待办 1）**：

- 变量：距离 1 / 3 / 5 / 10 / 15 / 25 m × 三种条件（同房间无遮挡、隔 1 道墙、隔 2 道墙）× 四种链路（BLE、BLE Long Range、Wi-Fi 经路由、ESP-NOW + PC 端 dongle）。
- 方法：每格持续 5 分钟、以 10 Hz 带序号上报；记录收包率、序号缺口、延迟 p50/p95/p99 与 RSSI；节点"握持/静置"各测一轮以量化人体遮挡。
- 通过标准（沿用既有门）：1,000 条事件无丢失、事件到本地画面 p95 ≤100 ms；新增"目标把玩距离内投递率 ≥99.5%"。
- 记录：按纪律写入 `test-worksheets/runs/<日期>-<主题>.md`，并回填 `WIRELESS_SENSOR_NETWORK.md` 与 `POWER_BUDGET.md`。

## ADR-064：小屏 HDMI 兼容性判定——换屏，并保留 DP→HDMI 与 ESP32 两条备选

状态：ACCEPTED（2026-09-23；项目所有者据"屏在 PC 上正常、在板端不出图"判定换屏，并要求增加 ESP32 备选）

**判定**：当前的 3.5 英寸 480×800 HDMI 屏（EDID 产品名 `HDMI480x800HH`，克隆 EDID，DTD 图像尺寸字段为垃圾值）判定为**屏侧 HDMI 兼容性问题**，不作为 P0 的最终显示件。

已确立的证据链：

- 板端链路参数全部正常：HPD 稳定、驱动内部成功解析 EDID（`drm get edid support modes: 5`）、按屏原生时序设置 `480*800`、`hdmi drv has been enable!`、CRTC 与平面在扫描输出、`/dev/fb0` 与 fbcon 就绪；
- 同一块屏在 **PC** 上可正常显示桌面；
- 驱动日志 `sunxi hdmi select vic 0 use hdmi14 vsif` 表明 480×800 属**非 CEA 模式**，走 HDMI 1.4 厂商专用信息帧；
- 因此故障不在板卡 HDMI 驱动、不在绿联线缆（两者已由 EDID/HPD/模式设置与 PC 交叉使用侧面验证），而在屏的 HDMI 接收端对该信号组合的兼容性。

**本 ADR 的三项决定**：

1. **换屏**。新屏的选型硬门增加一条：**必须能作为通用 HDMI 接收端工作**（即在普通 PC/显卡上作为显示器正常工作，且 EDID 可被通用 HDMI 源正常解析）。优先候选为具备完整标准 EDID 的小尺寸 HDMI 屏，或改用不做 HDMI 兼容协商的 **MIPI-DSI / SPI** 面板。
2. **保留并优先验证基线既定的 `USB-C DP Alt → 主动式 DP→HDMI` 路线**。`DISPLAY_SELECTION.md` 原本就把它写成 P0 的必要部件，而本轮实测走的是 **Mini HDMI 直连**——这是路径偏离。DisplayPort 是分组协议，不使用 HDMI 的 VIC/AVI/VSIF 机制，主动转接件自带 HDMI 发送端通常会输出标准 CEA 信号，很可能直接绕开本兼容问题。**在更换屏幕之前先验证这条 30–80 元的路径**。
3. **ESP32(-S3) 列为枪侧节点首选候选**，与 ADR-063 的桌面形态配套：内置 Wi-Fi 可直接回传、可直驱微型 SPI 屏、成本低。它同时是"不做 HDMI 兼容协商"的显示备选——SPI 屏由 MCU 直接驱动，不存在 EDID/VIC/信息帧问题。XIAO nRF52840 继续作为低功耗 BLE 与 P0-B 独立形态的保留路径。**节点 MCU 仍不在此冻结**，由台架实测（FSR ADC 线性/噪声、Wi-Fi 峰值电流、丢包与 p95）裁决。

**两条本轮实测得到的驱动限制**（写入技术债与运行记录，避免重复踩）：

- 在本厂商驱动上执行 `modetest -s <conn>@<crtc>:<mode>` 会导致 SSH 会话断开并使板子崩溃重启（实测两次）→ 运行期换模式不可用，只能改内核命令行并在开机时生效。
- 本内核的 `drm_kms_helper` **不提供 `edid_firmware` 参数**，EDID 覆盖功能未编入 → 无法用伪造 EDID 强制厂商时序；`video=` 也不会覆盖 EDID 首选模式。

**证据与限制**：显示器交叉测试（板子 → PC 显示器、同一根绿联线）的结论目前为**项目所有者报告**，尚未由本会话独立复核；它不影响"换屏"决定的成立（屏在 PC 端可用、板端不可用这一对照本身已足够），但补做该测试可把责任划分做得更严密。运行记录见 `test-worksheets/runs/2026-09-23-zero3w-display-link-retest.md` 与 `test-worksheets/runs/2026-09-23-zero3w-panel-no-image.md`。

原因：把"板卡输出正常"与"屏能作为通用接收端工作"分成两个独立门，可以避免用更换主板的代价去解决一个屏侧兼容问题；同时保留 DP 与 SPI 两条不依赖 HDMI 兼容协商的显示路径，让显示不再成为单点阻塞。
