# A.I.Live-ai枪娘器灵 · 四季宝计划

**探索角色走向实体设备的 P0 原型：设备提供类型化事件，Host 组合角色表达与小屏反馈。**

“四季宝器灵 / Skippy Spirit”是早期工程代号；正式产品名沿用本仓的 A.I.Live-ai枪娘器灵。当前工作围绕玩具／模型载体的娱乐、把玩与角色陪伴展开。

A P0 physical character-Host project for toy/model props. Software fixtures and plans are not a finished or field-validated device.

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
