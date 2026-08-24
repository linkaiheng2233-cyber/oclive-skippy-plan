# 屏幕后置主机电池与充放电路线

## 1. 与无线节点电池分层

本文件描述导轨主机的主电池：它位于屏幕后方，向 Linux 主板、屏幕、固定 IMU 和 BLE Central 供电，并支持 USB-C 充电。Grip/Shoulder Node 仍是物理独立的无线节点，继续使用 `POWER_BUDGET.md` 中的 CR2032/CR1632 路线；两类电池不能混为一个功耗模型。

## 2. 冻结电源拓扑

```text
USB-C input
  → input protection / current limit
  → 1S charger with true power path + NTC
       ├→ protected rechargeable 1S Li-ion/Li-polymer pack
       └→ system rail → DC/DC → Linux board / LCD / fixed sensor root
                         └→ fuel measurement / orderly shutdown
```

- 电芯为带保护与温度采样的 1S 可充锂电包，具体尺寸/容量在功耗测量后选择。
- 必须有真正的 power-path/load-sharing：插拔充电器时主机不重启，系统负载优先，余量用于充电。
- 充电管理需要输入限流、NTC 温度监控、过压/过流/短路与安全定时；不能只把廉价充电小板接到运行中的 Linux 板。
- 主板需要可读电量/电压状态，并在临界电量时完成屏幕提示、日志落盘和正常关机。
- v0.1 不做无线/磁吸充电，不做热插拔双电池。

TI BQ25628E/BQ2407x 级器件用于说明所需能力：单节锂电、系统 power path、温度监控和系统边运行边充电。最终芯片根据所选板卡电压、峰值电流、USB-C 输入与散热重新评审，不在机械方案阶段锁料号。

## 3. 屏幕后置电池的真实约束

相机侧翻屏只能给出正面尺寸，不能证明 Linux 主机电池也能做得同样薄。电池所需能量按下式估算：

屏幕功耗基线由 `DISPLAY_SELECTION.md` 先行确定。当前 2.8 英寸候选的 LCD 逻辑 + 满亮背光典型量级约 0.52 W；这只是裸屏负载，不包含 SPI 传输、Linux 板、Wi-Fi/BLE、DC/DC 和充电损耗。必须对 25/50/75/100% PWM 档位实测，不能用标称峰值直接推算整机续航。

```text
E_pack_Wh = P_system_avg_W × runtime_h / (conversion_efficiency × usable_fraction)
```

以下只用于展示量级，假设转换效率 85%、可用容量 80%、单节标称电压 3.7 V：

| 主机平均功耗 | 4 小时所需包能量/等效容量 | 8 小时所需包能量/等效容量 |
|--------------|---------------------------|---------------------------|
| 1.5 W | 约 8.8 Wh / 2,400 mAh | 约 17.6 Wh / 4,800 mAh |
| 2.5 W | 约 14.7 Wh / 4,000 mAh | 约 29.4 Wh / 7,900 mAh |
| 3.5 W | 约 20.6 Wh / 5,600 mAh | 约 41.2 Wh / 11,100 mAh |

因此“主云端、开发板小”仍不等于低功耗。屏幕背后能否同时满足薄、轻和 8 小时，取决于 Linux 板、屏幕背光、Wi-Fi 占空比和待机策略。容量和厚度必须在 USB 功耗实测后冻结。

## 4. 功耗状态

| 状态 | 主机行为 | 目标 |
|------|----------|------|
| Active | 屏幕正常亮度、Wi-Fi 可用、处理事件/OCLive 回合 | 记录峰值与平均功耗 |
| Interactive Idle | 屏幕降亮、Wi-Fi 保持、BLE 连接 | 数秒内恢复完整亮度 |
| Standby | 屏幕关闭，应用状态机与 IMU/BLE 唤醒路径保持 | 首轮不假定 Linux suspend 可用 |
| Critical Battery | 屏幕提示后落盘并正常关机 | 不让文件系统因硬断电损坏 |
| Charge + Run | 外部电源同时带系统和充电 | 不重启、不过热、不反复充停 |

屏幕背光是直接可控负载：无握持且 IMU 静止时应先降亮/熄屏，而不是立即挂起整台 Linux。只有所选板卡在实机上通过 suspend/resume、IMU GPIO 唤醒和 Wi-Fi 恢复测试后，才增加系统级休眠。

## 5. 屏幕舱堆叠

从屏幕正面到后壳的建议顺序：

```text
replaceable clear protector
→ recessed LCD + perimeter gasket
→ rigid LCD carrier / heat spread path
→ main PCB and antenna keep-out zones
→ insulated protected battery pocket
→ puncture-resistant rear cover
```

- 电池不能被 LCD、主板、泡棉或后盖持续压缩，预留制造公差和合理膨胀空间。
- 电池与铰链螺钉之间设置刚性隔板和防穿刺距离。
- 主板高热区、充电芯片与电芯不直接叠在同一热点；亮屏充电是温升验收的最坏工况。
- 天线不得夹在电池与金属背板/导轨之间。
- USB-C 口、开关键和维护螺钉不能处在三轴夹点或收纳接触面。

## 6. 原型顺序

1. 先让候选样屏通过 `DISPLAY_SELECTION.md` 的 D1 光学、刷新和保护片测试。
2. 使用 USB 功耗仪/电源分析仪测量候选 Linux 板 + 实际显示背板：开机、屏灭、25/50/75/100% 亮度、Wi-Fi idle、一次云端回合和 BLE 扫描/连接。
3. 用可编程电源模拟 1S 电池与 DC/DC，验证最低电压、峰值、电压跌落和正常关机。
4. 按实测平均功耗分别计算 4 小时与 8 小时电池包，制作等质量/等厚度假体。
5. 把电池假体装入三轴屏幕舱，测重心、所需转轴扭矩和导轨夹具负载。
6. 选择带保护/NTC 的实际电芯与 power-path 方案，完成充放电、边充边用和热测试。
7. 最后冻结屏幕舱厚度；不得反过来先定“很薄”再牺牲电芯安全余量。

## 7. 主机电池验收门

- 外部电源插拔、空电池启动和边充边用不导致 Linux 异常重启。
- 充电、亮屏、Wi-Fi 与高 CPU 同时发生时，电池/主板温度在电芯与器件规格范围内。
- 低电提示后能完成日志落盘和正常关机。
- 4 小时为首个场地 Alpha 最低目标；8 小时是设计目标，必须用完整工作负载验证。
- 500 次三轴循环后电池线、NTC、USB-C 与固定 IMU 链路无磨损/间歇断路；1,000 次为 K4 前验收。
- 收纳贴合时泡棉与机械止挡承力，不能把外力传给电芯。

## 8. 官方参照

- [TI BQ25628E 单节锂电 power-path 充电管理](https://www.ti.com/product/BQ25628E)
- [TI BQ2407x 单节锂电 power-path 数据表](https://www.ti.com/lit/ds/symlink/bq24074.pdf)
