# RADIAN MODEL 1 原型套件路线

**SSOT 范围**：本文负责 kit-first 装配顺序和阶段门；硬件参数以`HARDWARE_IMPLEMENTATION_PLAN.md`为准，软件阶段以`ROADMAP.md`为准。
**最后更新**：2026-08-31
**状态**：Current

## 1. 边界

- 首台载体是森柏龙 RADIAN MODEL 1，但软件、板屏、节点和电源先在桌面独立跑通。
- P0 是外挂、只读、可逆套件；不接内部火控、扳机机构、电机、MOSFET 或供弹。
- 通用电子核心通过可更换底座适配标准皮卡汀尼导轨；RADIAN 专用件只处理机械包络。
- P0 不做相机、语音、触觉或 Live2D；P1 相机不阻塞全枪联动。

## 2. 当前最小套件

| 部件 | 当前 P0 角色 |
|------|---------------|
| Orange Pi Zero 3W 6GB / A733 | 本地OCLive、Host、SQLite、BLE Central、Renderer；CPU量化3B为可选实测后端 |
| USB-C DP Alt Mode主动转HDMI件 | 把Zero 3W视频输出接入现有HDMI屏；必须计入体积、功耗、EDID和冷启动门 |
| 3.5inch 480×800 HDMI IPS touch 候选 | 横向 800×480 角色页、状态条、System Overlay |
| 完整受保护 5 V 电源 | 屏/板一体移动舱；四小时轻量路线 |
| 前下导轨 XIAO nRF52840 Sense 节点 | 前 FSR、固定 IMU、受保护 1S LiPo |
| 后握节点 | 单条后背 FSR、只读辅助触点、受保护 1S LiPo |
| 三轴机构假体 | 刚性导轨夹具、yaw/pitch/roll、泡棉保护和质量/重心验证 |

具体容量、包络、阈值和假体档位不在本文复制，统一查硬件 SSOT。

## 3. 实施顺序

### K0：软件 mock

用`ailive-gun-spirit-sim`完成标准生命周期、Unknown、Host reboot、重复/乱序与非法契约；用 800×480 mock Renderer 验证即时 UI 不等待 OCLive。

### K1：板屏桌面

Orange Pi使用稳定桌面电源，经USB-C DP主动转HDMI点亮屏幕并接USB touch，运行本地Host/OCLive和mock ViewModel；记录20次冷启动、资源、温度、功耗、首帧与两小时稳定性。此时不接电池/外壳，先确认现有5V/3A级电源是否能覆盖A733与屏幕峰值，不能假定沿用旧板结论。

### K2：节点 USB 台架

前后 XIAO 先用 USB/台架电源：单传感器 → 节点滤波 → BLE codec → Host replay。完成 FSR/IMU/辅助触点校准、断线/重启/低电 fault injection 和 golden vectors。

### K3：节点电池

加入受保护 1S LiPo、开关和电池匣假体，完成八小时、低温、重连峰值、充电温升、10 次受控低电关机和换电维护。

### K4：桌面纵向闭环

两个真实节点 → Host融合 → DeviceEvent → 即时ViewModel → 可选OCLive RoleCue → Output Arbiter；断网/模型失败不影响本地拿起—放回反馈。基础闭环通过后再挂接CPU量化3B服务，使用短角色台词语料测首字、解码、质量和功耗，不反向阻塞K4。

### K5：主机电源与外壳假体

使用完整成品 5 V 电源测四小时负载和人工 safe-to-cut；把屏、板、电源、散热、接头和外壳全部计入三档质量/厚度假体，先在人手/桌面验证三轴、保护、天线和热。

### K6：RADIAN 断电适配

测导轨槽、瞄具/操作/检修净空和重心；先装断电假体，检查握持、瞄准、拆卸、勾挂、收纳和原机构不受影响。只冻结可逆机械底座。

### K7：静态通电与场地 Alpha

按断电 → 通电静态 → 场地的顺序推进。至少三次场地记录误触、漏触、断连、重启、滑移、下垂、碰撞、续航和体验者反馈。

## 4. 每阶段退出条件

- 上一阶段失败时不把问题带到下一阶段“边装边猜”。
- 板屏/节点/电池/机构任何一个未通过，都能退回 simulator 或桌面电源继续开发。
- 机械失败先减质量/重排/换底座；电气失败先限流分域；协议失败使相关 Fact unknown，不允许猜测事件。
- 所有`MEASURED`结果绑定硬件/固件/软件/装配 revision。

## 5. 记录入口

- 到货与单项测试：`test-worksheets/README.md`
- Orange Pi：`ORANGE_PI_BRINGUP_WORKSHEET.md`
- 软件与阶段验收：`ROADMAP.md`
- 五链路与最终参数：`HARDWARE_IMPLEMENTATION_PLAN.md`
- 新裁决：`DECISIONS.md`
