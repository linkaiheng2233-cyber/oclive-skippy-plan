# P0-A 板端运行平台设计（板子的"操作系统"）

**状态**：`CANDIDATE`（2026-11-14）。**已实现部分标 ✅，未实现标 ⏳。**
**范围**：ESP32-S3 一体板上的**整个运行时**——启动序列、任务模型、时钟、内存与存储、配置面、诊断、故障恢复、更新、以及 C++ ↔ Rust 的 ABI 契约。
**不负责**：交互与表情映射（`FRONTEND_DESIGN_BRIEF.md` §9）· 帧协议字段（`PROTOCOL_V1_DRAFT.md`）· 消息语义分层（`P0A_SOFTWARE_DESIGN.md`）· 无线模块（`MODULAR_SENSOR_NETWORK.md`）。

> 这里说的"操作系统"**不是自研 RTOS**：底座是 **FreeRTOS + Arduino-ESP32 3.0.2 + LVGL 8.3.11**，本文定义的是**我们在这层底座上的运行规范**。

---

## 1. 启动序列（`BOOT`）

```
① ROM 一级引导
② 二级引导 → 读 otadata → 选择 app0（出厂）或 app1（我们的）
③ Arduino 运行时初始化（堆、PSRAM、USB-CDC）
④ setup() 开始
   ├─ Serial.begin + MEM boot 诊断                      ✅
   ├─ 载入 ffat 资产（GSA7 blob → PSRAM）               ✅
   ├─ gs_kernel_init()（Rust 感知内核）                  ✅
   ├─ 显示初始化（QSPI + LVGL + 触摸）                   ✅
   ├─ Wi-Fi：读 NVS 凭据并连接（无凭据则静默跳过）        ✅
   └─ 打印 READY / MEM display_ready                     ✅
⑤ loop()：命令解析 → 传感器/合成观测 → 内核 tick → 帧协议发布 → 触摸上报 → Wi-Fi pump
```

**失败行为（fail-open，逐项）**：

| 步骤失败 | 行为 | 不允许 |
|---|---|---|
| 资产加载失败 | 打印 `ERR ASSETS_*`，界面降级但仍可通信 | 卡死在加载 |
| 内核初始化失败 | 打印 `KERNEL init=-N`，帧协议回退到占位 STATE | 阻塞显示与通信 |
| 显示初始化失败 | 打印 `ERR DISPLAY_INIT`，仍保留串口/网络 | 复位循环 |
| Wi-Fi 无凭据/连不上 | 静默跳过，串口台架照常可用 | 阻塞 loop |

---

## 2. 任务模型（FreeRTOS）

**当前（✅ 已实现）**：全部逻辑跑在 Arduino 默认 `loopTask`（优先级 1、栈约 8 KB），loop 内做分时：

| 阶段 | 周期/预算 | 说明 |
|---|---|---|
| `read_commands()` | 每轮 ≤128 B（帧模式 ≤256 B） | 串口 ASCII 或帧协议输入，**有界预算防止饿死触摸** |
| 合成观测（`KSYNTH`） | 200 ms | **台架专用**，IMU 自动采样开启后应关闭（代码会告警） |
| IMU 采样服务（`imu_service`） | 100 Hz | 读三轴 → 扣零偏 → 运动判定 → 送内核；连续 5 次失败自动降级为 `fault` |
| `wire::pump()` | 1 s | 帧协议 STATE 发布 |
| `wifi_link::pump()` | 每轮 | 连接维护、客户端收发、发现广播 |
| 触摸事件上报 | 每轮非阻塞 | 队列 `touch_events`（16 项） |
| `delay(5)` | — | 让出 CPU |

**为什么现在够用**：单线程顺序执行，没有锁与优先级反转风险；实测栈余量 6.2 KB、内部堆 197 KB。

**⏳ 目标结构（IMU 到货后）**：拆出独立任务，避免 100 Hz 采样被 UI 阻塞。

| 任务 | 优先级 | 栈（估） | 周期 | 职责 | 禁止 |
|---|---|---|---|---|---|
| `sensor_task` | 5 | 3 KB | 100 Hz | I²C 读 IMU → 滤波 → `ingest_orientation/motion` | 碰显示锁、做网络 |
| `reducer_task` | 4 | 4 KB | 观测驱动 + 1 Hz | `gs_kernel_tick`、事件生成 | 直接渲染 |
| `comms_task` | 3 | 6 KB | 事件驱动 | 串口/帧协议收发、Wi-Fi 维护 | 阻塞等待超过 100 ms |
| `ui_task`（Arduino loop） | 1 | 8 KB | 30 fps | LVGL 渲染、命令解析、触摸上报 | 直接读传感器 |

**共享与同步**：

- **显示锁**（`bsp_display_lock`）**只允许 ui_task 获取**；sensor/reducer/comms 一律不得持锁（否则 100 Hz 会被渲染阻塞）。
- **有界队列**：`sensor → reducer`、`reducer → ui` 均用有界队列；**满则丢最旧并计数**，绝不阻塞采样。
- **单点写入**：只有 `reducer_task` 持有 `PerceptionReducer`；其他任务通过 ABI 队列投递观测。

---

## 3. 时钟与时间

| 用途 | 时钟 | 约束 |
|---|---|---|
| 采样时间戳、TTL、事件时刻 | `esp_timer_get_time()` / `millis()`（单调） | **绝不使用墙钟**做融合 |
| 日志打标 | 允许墙钟（若将来接 SNTP） | 不得参与判定 |
| 跨模块（未来无线模块） | 需授时偏移估计 | 见 `P0A_SOFTWARE_DESIGN.md` §4.5.4 |

**单调性要求**：`millis()` 溢出（约 49.7 天）后必须仍保持单调比较的语义；实现上用无符号差值（`now - last`）而不是大小比较。

---

## 4. 内存与存储

**实测预算**（2026-11-14，标准构建 `-BlobAssets -WithKernel -WithWifi`）：

| 项 | 数值 | 来源 |
|---|---|---|
| 固件体积 | 1,221,129 B（app1 的 58%） | `MEASURED` |
| 内部堆（boot） | ≈270 KB | `MEASURED` |
| 内部堆（display_ready + Wi-Fi） | **197,236 B** | `MEASURED` |
| 预计 +BLE 初始化后 | ≈129 KB | `ESTIMATED`（BLE 实测 ≈68 KB） |
| 全局变量 | 100,496 B（30%） | `MEASURED` |
| PSRAM 剩余（加载立绘后） | 7,131,284 B | `MEASURED` |
| 栈余量（loopTask） | 6.2 KB | `MEASURED` |

**分配纪律**：

- 大缓冲（>256 B）**不得放栈**；STATE 编码缓冲放在静态区（已修正过一次：512 B 静态）。
- LVGL 帧缓冲在 **PSRAM**（实测 307,200 B）。
- 立绘在 **PSRAM**（946,400 B），来自 ffat 资产区。

**存储分区用途**：

| 分区 | 用途 | 格式 |
|---|---|---|
| `nvs` | Wi-Fi 凭据、未来校准参数 | 键值（命名空间 `gunspirit`） |
| `ffat` | 立绘资产 | **裸资产区**：`GSA1` 头 + 24 B/条索引 + 原始 RGB565，**不挂 FAT** |
| `coredump` | 崩溃转储 | ESP-IDF 格式（⏳ 未启用读取） |

**NVS 键空间（约定）**：

| 键 | 类型 | 说明 |
|---|---|---|
| `ssid` / `pass` | string | Wi-Fi 凭据（明文，已知风险） |
| `imu_cal.*` | blob | IMU 零偏/轴向标定（⏳ 待定） |

---

## 5. 配置面

| 层 | 谁改 | 例子 | 纪律 |
|---|---|---|---|
| **构建期** | 开发者 | `-BlobAssets/-WithKernel/-WithWifi`、LVGL 配置 | 开关矩阵见 `P0A_SOFTWARE_DESIGN.md` §6.6 |
| **运行时·串口** | 现场 | `WIFI <ssid> <pass>`、`KGRIP`、`WIRE on/off` | 凭据**只进 NVS**，不回显、不进仓库 |
| **运行时·帧协议** | 电脑 | `EXPR`、未来的配置下行 | 需在契约中定义后才实现 |
| **资产** | 打包工具 | `assets.bin` | 带 magic/版本/索引，加载时校验 |

**红线**：任何凭据、内网地址、MAC 都不得写入仓库文件；这些只存在于 NVS 或仓库外工件。

---

## 6. 诊断与可观测性

**串口 ASCII 命令（全部 ✅ 已实现）**：

| 命令 | 用途 |
|---|---|
| `PING` / `GET STATE` / `STATE <id>` / `STEP next|prev` | 基本联调 |
| `MEM` | 内部堆/PSRAM/最大块/DMA/栈水位 |
| `IMU` / `IMU RD <hex>` / `IMU WR <hex> <hex>` | IMU 生数据与寄存器探针 |
| `I2C SCAN` | 扫描 I2C 总线（0x08–0x77），接线后先确认器件地址 |
| `IMU AUTO on\|off` | 开启/关闭 100 Hz 采样服务（自动送入感知内核） |
| `IMU CAL start\|end\|clear\|status` | 静止标定：采集 200 样本求加速度计零偏，存 NVS 并在采样时扣除 |
| `IMU STATUS` | 状态机（uninitialized/missing/ready/sampling/fault）、自动开关、零偏 |
| `KGRIP <front|rear> <engaged|released>` | 合成握持观测（台架） |
| `KSYNTH on|off` | 持续合成观测（台架） |
| `KTICK` | 手动推进内核时间（触发 TTL） |
| `KERNEL` | ABI 版本 + 自检 revision |
| `WIRE on\|off` | 切换帧协议 / ASCII |
| `TOUCHDBG on|off` | 每 200 ms 打印 LVGL 输入设备原始坐标（排查触摸映射） |
| `SETTINGS on\|off` | 打开/关闭屏幕设置层（无需触摸） |
| `WIFI <ssid> <pass>` / `WIFI status` / `WIFI scan` / `WIFI forget` | Wi-Fi 管理 |

**帧协议可观测**：`STATE`（每秒）、`EVENT`（边沿）、`PING/PONG`（往返/断线判定）。

**⏳ 待补**：帧错误计数、重连次数、事件丢弃计数的**对外暴露**（当前只在台架侧统计）。

---

## 7. 故障与恢复

| 场景 | 期望行为 | 状态 |
|---|---|---|
| `panic!`（Rust） | 停机等看门狗复位 | ✅ 已实现（loop 停住） |
| C++ 异常/断言 | Arduino 默认重启 | ✅ |
| 显示任务卡死 | 需引入任务看门狗 | ⏳ |
| 内核 tick 卡死 | reducer 自身超时保护；外部由看门狗兜 | ⏳ |
| 重启后诊断 | 读 `coredump` 并在 `SystemOverlay` 显示"上次异常" | ⏳ |
| 帧解析错误 | 丢弃单帧并计数，会话继续 | ✅ |
| 网络断开 | 5 s 重试；失联停服；恢复后重建会话 | ✅ |

**原则**：任何感知/网络故障只能让功能**降级**，不得让设备**不可用**。

---

## 8. 更新与版本

| 项 | 现状 |
|---|---|
| 固件更新 | **USB 烧 `app1`**（`scripts/flash-p0a.ps1` 一键） |
| 出厂恢复 | `app0` 永久保留；`--factory` 切回 |
| OTA | ❌ 不做（需两个自有 app 槽，会牺牲恢复能力） |
| 资产更新 | `--assets-force` 重写 ffat 资产区 |
| 版本标识 | **⏳ 缺失**：固件内没有版本字符串 |

**⏳ 建议补齐**：固件内嵌 `P0A_FIRMWARE_VERSION`（构建时注入 git 短哈希 + 构建日期），通过 `KERNEL`/`VERSION` 命令与帧协议 `HELLO` 暴露，便于"这台设备跑的是哪版"这类问题。

---

## 9. C++ ↔ Rust ABI 契约

**版本**：`GS_KERNEL_ABI_VERSION = 2`（静态常量 + `gs_kernel_abi_version()`）。

| 函数 | 语义 | 失败码 |
|---|---|---|
| `gs_kernel_init()` | 建立 reducer（单节点全能力 profile） | `-1` 构造失败 |
| `gs_kernel_ready()` | 是否已初始化 | 返回 0/1 |
| `gs_kernel_ingest_grip/reation/motion/control` | 摄入各类观测 | `-1` 未初始化、`-2` 观测构造失败、`-3` 内核拒绝、`-10/-11` 参数非法 |
| `gs_kernel_tick(millis)` | 推进时间（TTL/状态重算） | 同上 |
| `gs_kernel_encode_state(buf, cap)` | 用 **protocol crate 的同一份编码器**输出 STATE | `-1/-2/-3/-4` |
| `gs_kernel_state_revision()` | 当前 revision | — |

**线程模型**：ABI 面向**单线程调用**（当前 loopTask）。引入多任务后必须加互斥或改为消息队列。

**兼容规则**：

| 变更 | 处理 |
|---|---|
| 新增函数 | ABI 次版本 +1；旧固件不受影响 |
| 修改签名/语义 | ABI 主版本 +1；固件侧编译期校验，不匹配直接拒绝启动内核 |

---

## 10. 构建与验收纪律

| 项 | 规则 |
|---|---|
| 标准构建 | `-BlobAssets -WithKernel -WithWifi`（`scripts/flash-p0a.ps1`） |
| 门禁 | `./scripts/verify.ps1`（fmt/边界/clippy/测试/schema/no_std 对拍/audit） |
| 真机验收 | 每次烧板后至少验证：`PING`、`GET STATE`、`MEM`、`KERNEL`、帧协议 `hello=1` |
| 禁止 | 把凭据写进仓库；把大缓冲放栈；在感知路径持显示锁 |

---

## 11. 待决

1. **任务拆分时机**：IMU 到货后立刻拆 `sensor_task`，还是先在 loop 里做 100 Hz 采样？
2. **看门狗策略**：只挂 UI，还是也监控 reducer/sensor 心跳？
3. **版本标识方案**：构建期注入 git 哈希（需要 build.ps1 支持）是否值得？
4. **coredump 启用**：读出来做什么（串口打印？Overlay？）




