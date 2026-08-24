# 四季宝器灵仓库协作说明

本仓库是 **OCLive 四季宝器灵（Skippy Spirit）** 的独立实体宿主工程。目标是先把 OCLive 角色接入语义设备事件与小屏输出；当前阶段是 S0/S1，先完成传感器 → 状态机 → OCLive → 屏幕的桌面闭环，再接 Linux ARM64 硬件。

## 仓库边界

- OCLive 通用内核、角色包格式和跨宿主回合契约属于兄弟仓 `../oclivenewnew`。
- 本仓负责 DeviceEvent、器灵状态机、输出仲裁、模拟器、ARM Linux 宿主和硬件驱动。
- 修改兄弟仓前必须读取其 `AGENTS.md`，并把通用改动留在兄弟仓；不要复制一份 OCLive 内核到本仓维护。
- AN94、MP5 等正式角色资产以 OCLive 角色包 SSOT/市场为准。本仓 `roles/default` 仅是生成器提供的无头烟测夹具。
- 产品定位是娱乐、把玩和角色陪伴。不得实现或宣传智能火控、弹道计算、自动瞄准、武器控制或军警用途。

## 架构红线

- 原始 IMU/GPIO/压力采样必须先在驱动层滤波、去抖和聚合；只有低频语义 DeviceEvent 可以进入器灵状态机。
- 不得用 `[系统事件]` 文本前缀冒充类型化来源。sensor 回合必须具有明确 origin，并默认跳过用户情绪、记忆、关系与人格副作用。
- 即时屏幕反馈不等待 LLM。LLM 只生成低频角色表达，失败时不得阻塞设备状态机。
- v0.1 不实现 ASR、TTS、扬声器、音频缓存或触觉输出；先完成传感器、屏幕和主机软件闭环。
- v0.1 也不包含相机、AI 视觉、心率、精确弹药计数、Live2D 和多 BLE 节点。

## 工作顺序

1. 先改 `schemas/` 的协议与兼容策略。
2. 再改器灵状态机和屏幕状态仲裁。
3. 先接 Windows mock/simulator，再接 Linux 传感器与屏幕适配器。
4. 最后补齐单测、事件回放和实机证据。

路线 SSOT：`docs/ROADMAP.md`。关键决策：`docs/DECISIONS.md`。架构边界：`docs/ARCHITECTURE.md`。

## 基础验证

```powershell
cargo fmt --check
cargo check
cargo test
```

涉及 JSON Schema 时，至少确认文件能被严格 JSON 解析，并为有效/无效样例补测试。中文文档使用 UTF-8，不写 BOM，不用 ASCII 管道覆写。
