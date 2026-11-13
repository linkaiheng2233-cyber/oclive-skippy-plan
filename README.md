# A.I.Live-ai枪娘器灵 · 四季宝计划

**探索角色走向实体设备的 P0 原型：设备提供类型化事件，Host 组合角色表达与小屏反馈。**

“四季宝器灵 / Skippy Spirit”是早期工程代号；正式产品名沿用本仓的 A.I.Live-ai枪娘器灵。当前工作围绕玩具／模型载体的娱乐、把玩与角色陪伴展开。

A P0 physical character-Host project for toy/model props. Software fixtures and plans are not a finished or field-validated device.

**2026-11-09 当前硬件选择（ADR-068）**：P0-A 先用 **晶彩 JC3248W535 一体板**跑通基础闭环，控制起步成本，屏幕不超过 3.5 英寸。LILYGO T4-S3 作为按需的小型 AMOLED 备选；分体板屏暂缓。采购规格、报价来源与未实测项见 [硬件 SSOT §0.1](docs/HARDWARE_IMPLEMENTATION_PLAN.md)。上方旧进度摘要及下方 Linux 三轴主机舱描述不是当前首版采购指令，分别按 [当前基线](docs/PROJECT_BASELINE.md) 和 P0-B 历史/后置方案阅读。

**2026-11-13 到货更新**：晶彩带壳一体屏已到货，下一步用电脑 USB-C 数据线检查出厂画面、触摸和 Windows 设备识别。验收见 [硬件到货清单](docs/test-worksheets/01-硬件到货清单.md)。

**P0-A 部署更新（ADR-069）**：ESP32-S3 运行感知内核与本地显示，电脑运行 OCLive。首轮先接固定在导轨底座上的 IMU，板侧走附件内短线，电脑正常联机走局域网 Wi-Fi。实施与实测见 [硬件计划 §0.1.2](docs/HARDWARE_IMPLEMENTATION_PLAN.md#012-单板传感器与电脑无线连接)。

首台原型载体为 **悍迪 191**；森柏龙 RADIAN MODEL 1 为历史参考。实施顺序是桌面套件 → 套件外壳/导轨适配 → 上枪验证，不在软件和电子闭环完成前修改或连接载体内部机构。

## 当前状态

公开源码包含感知契约、状态与融合逻辑、模拟／回放夹具和 Host 接线研究。软硬件仍在验证中；本仓没有成品设备或已经上传的安装包，桌面软件夹具通过不等于实体设备、续航、性能或模型质量已经验收。

本项目不提供智能火控、弹道计算、自动瞄准、武器控制或军警用途。

## 从哪里读

| 目标 | 入口 |
|---|---|
| 了解公开源码的基线与下一步 | [项目基线](docs/PROJECT_BASELINE.md) · [路线图](docs/ROADMAP.md) |
| 找专题文档 | [文档首页](docs/README.md) |
| 接手现有工作 | [交接入口](docs/handoff/README.md) |
| 了解角色运行时与内容格式 | [A.I.Live 主仓库](https://github.com/linkaiheng2233-cyber/oclivenewnew) |

具体设计、待测条件与已验证结果在对应文档维护。本页不复制硬件参数表，也不把计划中的实现写成已经完成的能力。

## 开发与验证

按 [AGENTS.md](AGENTS.md)读取本仓边界，再针对受影响软件包验证；交付前的仓库检查入口是：

```powershell
.\scripts\verify.ps1
```

本仓是设备感知与角色 Host 的组合工程；OCLive 通用运行时和角色包契约仍由主仓维护。

源码构建尚有依赖复现限制：Host 使用同级 OCLive 主仓的路径依赖，现有锁文件不能与任意主仓版本混用；当前相邻主仓下的 `--locked` 验证仍要求更新锁文件。本仓尚未冻结一组可独立复现的依赖与 CI，处理入口见[技术债 `GS-CI-001`](docs/TECHNICAL_DEBT.md)。此限制不因文档整理而解除。

## 反馈与许可

欢迎提交[体验建议与问题](https://github.com/linkaiheng2233-cyber/oclive-skippy-plan/issues)，请区分软件回放观察与真实设备测试结果。软件许可见 [LICENSE](LICENSE)，第三方素材和硬件资料分别遵循其来源声明。


<!-- Preserve previously published section bookmarks. -->
<a id="oclive-四季宝器灵"></a>
<a id="项目分层"></a>
<a id="v01-闭环"></a>
<a id="当前骨架"></a>
<a id="远端与权威"></a>
<a id="启动无头宿主"></a>
<a id="无硬件生命周期回放"></a>
<a id="路线入口"></a>
<a id="本地完整验证"></a>
<a id="许可证"></a>

旧版章节书签保留在此。[整理前的首页](https://github.com/linkaiheng2233-cyber/oclive-skippy-plan/blob/da543736dc8ea9c5d7f194e718392a66011eb201/README.md)可用于查阅历史说明；现行入口以本页与所链文档为准。
