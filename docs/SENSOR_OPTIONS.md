# 传感器候选方案与实测路线

**SSOT 范围**：本文负责传感器作用、候选与升级门；当前节点拓扑、电路起点和参数以`HARDWARE_IMPLEMENTATION_PLAN.md`为准。
**最后更新**：2026-08-27
**状态**：Current

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
