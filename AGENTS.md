# 四季宝器灵仓库协作说明

本仓库是 **A.I.Live-ai枪娘器灵** 的独立实体宿主工程；“四季宝器灵 / Skippy Spirit”仅为历史代号。目标是先把 OCLive 角色接入语义设备事件与小屏输出；当前阶段是 S0/S1，产品原型仍为 P0，首个正式感知契约基线为 v0.2。先完成传感器 → 状态机 → OCLive → 屏幕的桌面闭环，再接 Linux ARMv7/ARM64 硬件。

## 仓库边界

- OCLive 通用内核、角色包格式和跨宿主回合契约属于兄弟仓 `../oclivenewnew`。
- 本仓负责 DeviceEvent、器灵状态机、输出仲裁、模拟器、ARM Linux 宿主和硬件驱动。
- 修改兄弟仓前必须读取其 `AGENTS.md`，并把通用改动留在兄弟仓；不要复制一份 OCLive 内核到本仓维护。
- AN94、MP5 等正式角色资产以 OCLive 角色包 SSOT/市场为准。本仓 `roles/default` 仅是生成器提供的无头烟测夹具。
- 首台原型载体固定为森柏龙 RADIAN MODEL 1；先完成独立桌面套件，再设计外壳/导轨适配，最后上枪验证。
- 机械目标是通用电子核心 + 可更换底座；首发只承诺兼容标准皮卡汀尼导轨，非标准安装通过独立转接件处理。
- 主机采用三轴一体式显示主机舱：屏幕、Linux 小板、显示接口和可充电主机电池共同位于移动舱内，视频链路只在壳内短距离连接。当前模块化 bring-up 基线是 Orange Pi Zero 3W 6GB（Allwinner A733）+ 3.5inch 480×800 HDMI IPS 电容屏（横向 800×480）；板端 USB-C DisplayPort Alt Mode 到 HDMI 的主动转接、厂商 Linux BSP、板载风扇、功耗和本地 3B 推理均须实测。Luckfox Lyra Zero W + Waveshare 2.8inch DSI LCD 仅保留为重量/厚度触发后的紧凑回退。导轨夹具刚性锁定，yaw/pitch/roll 转轴提供恒扭矩阻尼；载体参考 IMU 位于固定的前下导轨 BLE 节点，不得跟随主机舱转动。当前硬件 SSOT 见 `docs/HARDWARE_IMPLEMENTATION_PLAN.md`。
- 产品定位是娱乐、把玩和角色陪伴。不得实现或宣传智能火控、弹道计算、自动瞄准、武器控制或军警用途。

## 架构红线

- 原始 IMU/GPIO/压力采样必须先在节点/驱动层滤波、去抖和聚合；无线节点只发送低频 `SensorObservation`，由导轨主机融合成 `DeviceEvent` 后才能进入器灵状态机。
- 不得用 `[系统事件]` 文本前缀冒充类型化来源。sensor 回合必须具有明确 origin，并默认跳过用户情绪、记忆、关系与人格副作用。
- 即时屏幕反馈不等待 LLM。LLM 只生成低频角色表达，失败时不得阻塞设备状态机。
- P0 不实现 ASR、TTS、扬声器、音频缓存或触觉输出；先完成传感器、屏幕和主机软件闭环。
- Zero 3W 的 6GB 内存允许把 CPU 量化 3B 及以下模型作为实测候选。到板后按约3B、1.5–1.7B和0.5–0.6B三个代表尺寸档比较base与有界角色上下文；LoRA和全参数训练按ADR-061逐级进入。任一模型都只是可选角色表达后端；即时画面、感知、安全生命周期和关机不能等待或依赖模型，NPU/GPU加速不得在驱动与质量验证前写成基线能力。
- P0 不包含相机、AI 视觉、心率、精确弹药计数和 Live2D；实现前下导轨与后握把两个区域 BLE 节点。相机是 P1 独立扩展，不阻塞 P0 传感器—本地 OCLive—屏幕闭环。
- P0 不接 RADIAN 内部火控、扳机或供弹机构；实体尺寸与导轨适配以实物测量为 SSOT，不以网图估算。
- 余弹融合是 G5 之后的 P1+ 独立只读扩展，不修改 P0 v0.2 闭集。它以机械循环短计数提供连续估算、以磁性随动件和外部霍尔阵列提供粗范围锚点；循环不得称为实际过弹，故障不得影响供弹。实施前先读`docs/MAGAZINE_AMMO_ESTIMATION.md`并建立新契约版本/能力协商。
- 传感器默认不走机匣/波箱内部线。P0 使用前下导轨节点与后握把节点；每个区域内部允许藏在自制握把套/护木/节点外壳内的可维护短线，节点之间通过 BLE，不依赖磁性放置底座。
- XIAO 原型节点使用受保护可充 1S LiPo。容量、包络、续航和充电温升只在`docs/HARDWARE_IMPLEMENTATION_PLAN.md`维护；CR2032/CR1632 只保留给未来专用低功耗节点，绝不能接入 XIAO 可能执行充电的电池路径。
- 不得混淆三个电源域：一体式主机舱内的可充电主电池包供 Linux 主机/屏幕/后续 USB 相机；前、后 BLE 节点各有独立 1S LiPo，不从主机拉供电线。
- OCLive Host、角色包、记忆/状态、DeviceEvent、屏幕即时反馈、网络降级和安全生命周期必须本地运行。外部 LLM API 只能作为可选表达能力，不得成为设备闭环或启动的前提。
- 原型阶段把已经冻结且会影响玩家体验的决定翻译成简短问题，拿已有假体/界面/原型向 wargame 爱好者复核；不做泛化产品研究，也不得让个人偏好直接覆盖电气安全、协议、感知/OCLive职责和火控边界。问题与ADR映射见`docs/UX_RESEARCH.md`。

## 工作顺序

1. 先列能力链和职责边界；协议变更先改 contracts Rust 类型/语义校验并同时补测试与 fixtures。
2. 由 Rust 类型生成 `schemas/`，运行零漂移门；禁止直接手改 v0.2 生成 Schema。
3. 再改器灵状态机、Host bridge、Input Scheduler 和屏幕状态仲裁，逐段补回放/降级测试。
4. 先接 Windows mock/simulator，再接 Linux 传感器与屏幕适配器；实机结论回填测试工作表。

统一文档入口：`docs/README.md`。当前开发共同起点：`docs/PROJECT_BASELINE.md`。项目/跨仓边界：`docs/PROJECT_BOUNDARIES.md`。路线 SSOT：`docs/ROADMAP.md`。关键决策：`docs/DECISIONS.md`。架构边界：`docs/ARCHITECTURE.md`。开发与验证纪律：`docs/DEVELOPMENT_DISCIPLINE.md`。活跃风险：`docs/TECHNICAL_DEBT.md`。面向人类与后续 AI 的交接入口：`docs/handoff/README.md`。文档冲突、状态和归档按`docs/DOCUMENTATION_SYSTEM.md`处理。

## 基础验证

```powershell
./scripts/verify.ps1
```

开发期可以先跑受影响包的窄测；准备交付时运行完整脚本。禁止把`cargo fmt --all`用于本仓门禁，因为路径依赖会越界检查兄弟 OCLive。涉及 JSON Schema 时，必须由 Rust 真源生成并为有效/无效样例补测试。中文文档使用 UTF-8，不写 BOM，不用 ASCII 管道覆写。
