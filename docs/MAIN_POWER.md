# 一体式显示主机舱电池与充放电路线

**SSOT 范围**：本文负责主机电源拓扑、测量与升级门；当前功耗、Wh、质量和包络占位以`HARDWARE_IMPLEMENTATION_PLAN.md`为准。
**最后更新**：2026-08-31
**状态**：Current

## 1. 电源域

```text
complete protected 5 V power source
  → protected distribution
      ├── Orange Pi Zero 3W 6GB / A733 + board-mounted cooling
      ├── HDMI screen + USB touch
      └── P1 USB hub/camera (later)
```

主机电源与前/后BLE节点的1S LiPo完全独立。P0不拆成品电源、不改电芯、不用PD诱骗请求9/12V。旧路线的固定5V/3A级只保留为首个台架检查点，不能假定足以同时覆盖A733、板载风扇、主动DP转HDMI、屏幕和本地3B峰值；先使用可限流且有余量的台架电源测出真实最低电压与峰值，再冻结成品电源。屏幕和主板由分配点分别供电，禁止屏幕反向代供主板。

## 2. 为什么 P0 先用完整成品电源

第一轮目标是获得真实平均/峰值功耗、四小时能量、热与三轴手感数据。完整受保护电源能把自制锂电 pack、充电 power-path、NTC 和低压切断风险推迟到有数据之后。代价是低电遥测、自动休眠和边充边用行为可能不可控；不合格时更换整件，不拆壳修改。

P0 充电时正常关闭 Linux 和负载，不承诺 pass-through。V1 是否定制 1S 升压、2S 降压、监督 MCU 或电池匣，只由 P0 数据触发。

## 3. 功耗与续航测量

输入侧至少记录：关机静耗、启动峰值、屏亮四档、背光关、Wi-Fi idle/传输、两个BLE节点、SQLite/OCLive idle、3B模型未加载/常驻空闲/短句生成/连续压力、一次可选网络回合和P1 UVC。每档记录平均W、峰值W、5V最低电压、温度、CPU throttling和持续时间。

估算只使用：

```text
required_Wh = measured_average_W × target_hours / usable_efficiency
```

效率、可用深度和低温余量必须写清。P0 目标是满足四小时的最小/轻量方案，不提前为八小时堆电池。

## 4. 安全关机

P0 人工闭环：

```text
recessed shutdown request
  → Host stops new work
  → OCLive + Host checkpoint
  → Linux shutdown
  → independent safe-to-cut indication
  → user cuts main 5 V
```

普通充电宝不给 Linux 可信电量时，不虚构自动欠压保存。V1 监督电源若加入，仍复用同一 Shutdown Coordinator：Low 卸载可选负载；Critical Reserve 停新写/新角色回合，限时 checkpoint；监督器收到 ACK 或硬超时后切主 rail。

实体键和最低 System UI 不受 renderer/OCLive 故障阻塞。突然断电、日志写满和数据库恢复在副本卡上做 fault injection，不直接拿唯一数据卡试错。

## 5. 移动舱与回退

P0 电源与屏、板一起移动，换取无跨轴供电线和完整外观。移动质量按屏、板、电源、散热、外壳、接头、线缆与关节从动件总和计算，并用硬件 SSOT 的三档假体测试。

若超过质量退出门：先减电源容量、外壳和非必要接口/改紧凑板屏；仍失败才把电池移到固定导轨底座。后者会新增一根跨三轴 5 V 柔性线，必须重新做弯曲、应变释放、限位、压降、噪声和断线降级评审，不能只改 CAD。

## 6. 背光策略

P0 保持 Linux、BLE 和触摸运行，只管理`active → dim → backlight_off`。黑色画面不等于关背光，不用反复切整个 HDMI 板电源省电。时间窗和唤醒延迟是 HostProfile/实测参数；只有背光策略仍无法达成四小时，才评审环境光、CPU governor/外设电源域或更深低功耗，不先使用可能破坏 BLE/HDMI 恢复的 suspend。

## 7. 冻结门

完成`test-worksheets/04-主机电源与关机测试表.md`和`ORANGE_PI_BRINGUP_WORKSHEET.md`后，才冻结电源料号/能量/布局。至少满足：20 次冷启动、峰值不 brownout、四小时完整负载、背光控制、充电关机流程、低电/硬切行为已知、温升可接受、关机恢复与数据库检查通过。
