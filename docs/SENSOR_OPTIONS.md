# 传感器候选方案与实测路线

**SSOT 范围**：本文负责传感器作用、候选与升级门；当前节点拓扑、电路起点和参数以`HARDWARE_IMPLEMENTATION_PLAN.md`为准。
**最后更新**：2026-11-13
**状态**：Current

## P0-A：单板传感器设计候选

`CANDIDATE`，待台架与实物验证。首件按 **IMU → 后握 FSR → 按需增加前握 FSR** 的顺序。屏上板承担采样和感知，传感器共享该板的 3.3 V/GND。传感器线收在附件内，附件与电脑走 Wi-Fi。 [硬件接口](HARDWARE_IMPLEMENTATION_PLAN.md#012-单板传感器与电脑无线连接) · [候选布局](MECHANICAL_INTERFACE.md#p0-a悍迪-191-原型布置候选)

### IMU 与装配校准

1. 首选带原理图、支持 3.3 V I²C 的 ICM-42688-P 小板，QMI8658 为候选。用 P3 或 P4 的 3.3 V/GND/GPIO17/GPIO18。从同一条四芯线引出，初试可配置 GPIO17 为 SDA、GPIO18 为 SCL。确认模块上拉到 3.3 V，以及实际插座针序与线束两端的通断。芯片封装尺寸和含插头的小板尺寸分别记录。 [TDK 器件资料](https://www.invensense.tdk.com/en-us/products/6-axis/icm-42688-p)
2. 原厂触摸驱动使用 `I2C_NUM_0` 和 GPIO4/8。IMU 初试用另一硬件控制器，以 100 kHz 启动，验证触摸与 IMU 可同时工作。扫描外接总线、记录地址，再读取 `WHO_AM_I` 寄存器 `0x75`，ICM-42688-P 预期为 `0x47`。原始六轴可从 100 Hz 起做诊断，同时记录实际间隔与超时，再按实测定滤波参数。 [乐鑫 I²C 控制器说明](https://docs.espressif.com/projects/esp-idf/en/stable/esp32s3/api-reference/peripherals/i2c.html) · [TDK 寄存器手册](https://product.tdk.com/system/files/dam/doc/product/sensor/mortion-inertial/imu/data_sheet/ds-000347-icm-42688-p-v1.6.pdf)
3. 固定在导轨底座的小硬座后，标记装配坐标 +X 向前、+Y 向左、+Z 向上。用各面静置及缓慢转动验证轴符号。记录装配版本、静止偏移和零偏，再显式采样放低与举起两姿态。底座或 IMU 朝向改变时重验校准；移动造成重力估计质量不足、总线掉线或观测过期时，对应事实进入 `unknown`。 [状态机源码](../crates/ailive-gun-spirit-perception-core/src/reducer.rs)
4. IMU 实机验证运动、静止与重力姿态。第二步加上后握后，可由明确的后握接触判 `Held`，与校准后的举起姿态合成 `Ready`。完整 `Standby` 需要前后握都明确松开且静止持续达到门限。首轮缺少的握持事实保持 `unknown`，用桌面或板端回放覆盖其余状态。 [状态机源码](../crates/ailive-gun-spirit-perception-core/src/reducer.rs)

### 握持片、走线与断线判定

- 后握先用可拆薄套内的一片 FSR。用纸胶带找掌心接触点，以局部平整刚性背托保持膜片平整，受力层按「手 → 薄套外皮 → 小弹性压垫 → FSR → 背托」。先试 402 类圆片（有效区约 Ø12.7 mm）或等价同类薄膜，有明显手位盲区再比较窄条。检查空载预压和弯曲造成的基线漂移，尾片接插件按实际料号选。 [Interlink 集成指南](https://www.interlinkelectronics.com/downloads/integration-guides/fsr-400-series-integration-guide.pdf)
- 压片在握把，分压、滤波与 ADC 转接板放在板屏壳内。回来的两芯线受套件盖线槽保护，模拟线沿途远离电源转换区域，握套可单独拔插。P2 第 2 脚为 GPIO6（ADC1 候选），第 3 脚为 GPIO7（留给前握）。P2 的 8 芯线只有信号，供电和地从另一只 P3/P4 引来。 [原理图接口核对](HARDWARE_IMPLEMENTATION_PLAN.md#011-原厂手册核对2026-11-09)
- 若要把拔线和松手分开，首轮用末端电阻方案试验。把 `220 kΩ` 与 FSR 并在传感器端、放在可拔线束之后；板端串 `10 kΩ`，ADC 节点用 `47 kΩ` 下拉，可评估在 ADC 边加 `100 nF` 去噪。以 3.3 V 计算，拔线近 0 V、完好松开约 0.56 V（FSR 空载电阻很高时）、重压趋近 2.72 V。初测用万用表和校准过的 ADC 核对窗口，再按实测设断线窗口、握持/释放双阈值及去抖。该接法主要区分开路，线束两芯短接可能与强握落在同一窗口。 [乐鑫 ADC 校准与去噪](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/peripherals/adc/adc_calibration.html)

```text
3.3 V → 10 kΩ → 两芯线去程 →（FSR ∥ 220 kΩ）→ 两芯线回程 → ADC 节点
ADC 节点 → GPIO6
ADC 节点 → 47 kΩ → GND
```

先量四组曲线，分别为裸手、手套、放置轻碰和拔插。位置稳定后沿用既有抽样门，做左右手裸手和手套各 25 次，再做 50 次负样本。以正样本总成功率 ≥95%、各组 ≥90%、负样本误判 ≤1/50、释放不卡住及拔线进入 `unknown` 为样件验收起点。若某组明显漏报，先换压片位置和垫片，再决定是否加第二接触点。前握片只有在需要独立前手证据、双手识别或完整 `Standby` 时加入。 [测试表 §B/C](test-worksheets/03-无线传感节点测试表.md)

若后握到屏壳之间没有可维护的遮护线槽，当前样件维持板屏加 IMU，后握保持 `unknown`。需要后握交互时再比较把 FSR、BLE 和小电池集中在可拆后握底盒的方案。底盒能省去跨区线束，会增加第二块电池、充电和掉线管理，需按原有多节点方案单独评审。 [机械布局](MECHANICAL_INTERFACE.md#p0-a悍迪-191-原型布置候选)

以下双区域 BLE 方案用于 P0-B。

## 1. 当前 P0 组合

```text
front rail node
  ├── XIAO nRF52840 Sense
  ├── fixed carrier-reference IMU
  ├── movable front FSR pad (regional short wire)
  └── protected rechargeable 1S LiPo

rear grip node
  ├── XIAO nRF52840-class BLE MCU
  ├── single rear-back FSR layer
  ├── independent read-only primary-control contact
  └── protected rechargeable 1S LiPo
```

两个区域之间只用 BLE。前部 FSR 可跟随前握位置，电子盒/IMU 留在前下导轨后段；后握 FSR 和辅助触点只用握把内部/外套内的可维护短线进入底盒。不新增顶部节点，不拆机匣，不连接电机、MOSFET、供弹或火控。

## 2. 为什么分成两个节点

| 需求 | 前节点 | 后节点 |
|------|--------|--------|
| 从前部拿取/前握 | FSR | 无 |
| 后握持枪意图 | 无 | FSR |
| 载体姿态/运动 | 固定 IMU | 无 |
| 只读辅助控制触点 | 无 | 独立输入 |
| 线缆 | 同一区域短线 | 握把套/底盒短线 |

一个节点会迫使前后传感器之间出现长外露线或把 IMU 放在随屏幕转动的位置；每颗传感器各一节点又会增加电池、配对和维护。两个区域节点是当前最小可验证平衡。

## 3. 传感器角色

| 传感器 | 原始量 | 节点输出 | Host 派生 | 不得声称 |
|--------|--------|----------|-----------|----------|
| 前/后 FSR | ADC 分压 | 接触、归一化强度、校准/故障状态 | front/rear grip Fact、Held/Ready 证据 | 精确牛顿值、用户身份 |
| 前节点 IMU | 加速度/角速度 | motion class、orientation estimate、shock diagnostic | pose、载体运动、置信度 | 自动瞄准、弹道、真实击发 |
| 辅助触点 | GPIO 电平 | primary control engaged/released | 中性控制边沿 | 机构已完成动作、射击计数 |
| 电池 ADC | 电压 | battery mV、粗粒度状态 | 节点健康/关机协作 | 未标定的精确百分比 |

节点先滤波、去抖、迟滞和校准，再发送低频`SensorObservation`。OCLive 不读取高频 ADC/IMU 原始流。

## 4. FSR 结构与升级门

后握 V0 使用单条纵向 FSR：薄外套 → 力扩散片 → FSR → 原握把刚性背面。先完成分组 100 次握持/释放正样本和 50 次负样本；只有出现固定手型/手套/左右手盲区且机械与校准调整仍失败，才升级双侧 FSR。

前握 FSR 与电子盒分离，具体膜片、扩散片、线长和护片形态按实际前握位置调整。FSR 尾片的连接方式服从料号资料，不默认直接焊聚合物尾片。

简单分压、电阻、采样率和去抖只作为台架起点；具体值、EOL 是否安装和故障窗口均在硬件 SSOT 与工作表中冻结。

## 5. IMU 与校准

- IMU 固定在前下导轨节点，建立载体坐标，不随三轴屏幕旋转。
- canonical frame 为右手系：+X 向前、+Y 向左、+Z 向上；装配变换绑定 assembly revision。
- 用户通过显式向导选择放低/举起校准点并预览结果；不开机静默自学，不把当前姿态自动当零点。
- Shock 只进诊断；没有独立机构证据时不得翻译成 firing/weapon 事件。

## 6. 后续候选

| 候选 | 何时评审 | 默认状态 |
|------|----------|----------|
| 双侧后握 FSR | 单条通过机械/校准优化后仍有系统盲区 | 关闭 |
| 第三个 BLE 节点 | 双节点无法覆盖一个已定义高价值交互，且收益大于电池/配对成本 | 关闭 |
| 压电/高频冲击 | 只作为诊断或娱乐效果，需大量误判数据 | P1+ |
| 拉栓/供弹等只读机械传感 | 专用载体 adapter，且不改变原机构 | P1+独立立项 |
| 相机 | P0 全枪联动通过后，UVC 独立扩展 | P1 |

精确验收样本与记录入口见`test-worksheets/03-无线传感节点测试表.md`和`test-worksheets/05-机械与场地测试表.md`。
