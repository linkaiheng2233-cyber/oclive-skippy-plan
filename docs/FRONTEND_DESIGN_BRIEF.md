# 800×480 前端设计准备包

**用途**：让项目所有者可以先画界面、做交互原型和资产占位，不必等待 BLE、实体屏幕或最终 Linux renderer。本文只定义当前 P0 的信息架构和交付物，不冻结美术风格、WebView 引擎或 Live2D。

## 1. 先按什么画布准备

- 目标画布：`800 × 480`，横向。候选 3.5 英寸屏通常以 `480 × 800` 标注，安装后旋转为横屏使用。
- 建立一份 `800 × 480` 原始画板，不按电脑窗口比例另画一套。
- 四周先留 `24 px` 设计安全区；这是桌面 mock 的 **PROVISIONAL** 值，样屏到货后按真实边框、海绵遮挡和可视角重新测量。
- 不把重要文字、低电提示和实体键说明贴在边缘。屏幕旋转或贴枪保护时，边缘最容易被海绵、壳体和视角吃掉。
- 若使用触摸，主要触控目标建议先画到 `80 × 80 px` 左右。800×480 的 3.5 英寸面板约为 10.5 px/mm，普通桌面端常见的 44–48 px 按钮在这里物理上偏小，戴手套更难用。

## 2. 一张屏幕只有三层

```text
SystemOverlay  安全、关机、配对、校准、必须确认的故障（最高）
       ↑
RoleStage      角色立绘、短回复、动作/表情（正常主体）
       ↑
StatusBar      主机电量、前后节点、网络、模式（持续但克制）
```

`OutputArbiter`是唯一决定主画面层的人。页面只能渲染最终`ScreenViewModel`，不能直接读取 BLE、修改感知状态，或让 OCLive 绕过系统故障层。

### RoleStage

- 正常画面的视觉主体，给角色资产最大的面积。
- 消费 `primary_layer.role.visual_state_id` 和 `text`。
- P0先使用PNG占位；`visual_state_id`找不到资产时回退默认立绘，不能显示破图或空白页。
- 文本以一到两行短回复为主；长文本允许展开到次级页，不遮住关键状态。

### StatusBar

- 始终能看到，但不与角色争夺注意力。
- 当前可直接使用：`carrier_state`、`front_grip`、`rear_grip`、`motion`、`pose`、`primary_control`、`nodes[]`。
- `Unknown`必须显示为未知/待校准/离线，不得沿用旧值假装当前正常。
- 主机电量、无线、交互模式尚未进入当前`ScreenViewModel`，先画占位，不得在代码中伪造数据。

### SystemOverlay

- 消费 `primary_layer.system.code/message`。
- 节点离线、校准、维护、低电和关机流程使用全屏或大面积覆盖。
- 必须在PNG/WebGL/角色渲染失败时仍能显示；它是最低可用界面，不依赖角色资产。

## 3. 第一轮只画九个状态

| 状态画板 | 主层 | 画面要回答的问题 | 当前数据状态 |
|---|---|---|---|
| `01_booting` | System | 系统正在启动到哪一步？ | Host生命周期字段待实现，先占位 |
| `02_standby` | Role/Status | 设备待机、节点是否正常？ | 已有carrier与nodes |
| `03_held` | Role | 已拿起，但尚未进入Ready | 已有 |
| `04_ready` | Role | 已进入用户校准的举持状态 | 已有 |
| `05_control` | Role | 只读primary control按下/释放 | 已有；不得画成开火/弹药 |
| `06_node_offline` | System | 哪个节点离线、还能做什么？ | 已有 |
| `07_uncalibrated` | System | 哪项需用户校准、怎样继续？ | Fact reason已有；流程待实现 |
| `08_low_power` | System | 当前低电阶段与剩余动作 | 电源字段待实现，先占位 |
| `09_shutdown` | System | 正在保存/可安全断电/已关机 | 生命周期待实现，先占位 |

这九张足够先验证信息层级。设置商城、复杂角色管理、相机视野、语音、Live2D编辑器都不进入第一轮。

## 4. 推荐先做的三个视觉方向

### A. 角色舞台 + 相机状态框（推荐起点）

角色占主体，四周使用相机监视器式细边框展示电量、节点和模式。优点是既保留“器灵”体验，又能与枪身/摄影配件语言融合；缺点是要控制状态框密度，否则容易像游戏HUD。

### B. 纯角色侧屏

接近相机翻转屏或掌机角色页，状态只在角落出现。优点是干净、角色感强；缺点是节点异常和校准状态容易被忽略，必须依赖强SystemOverlay兜底。

### C. 战术HUD

状态、姿态和节点信息优先，角色缩小为头像或侧栏。优点是信息清楚；缺点是容易把项目做成普通仪表盘，削弱“枪娘器灵”核心体验，也会诱导加入不可靠的弹药/击发信息。

第一轮可以只为`standby / held / ready`各画A、B、C三张低保真稿，用同一组内容比较，而不是立刻画九套精稿。

## 5. 设计文件怎样分层

建议在 Figma、MasterGo 或其它工具中建立以下页面：

```text
00 Foundations   800×480画布、安全区、颜色、字号、间距
01 Components    状态栏、节点徽标、短回复、系统弹层、触控按钮
02 States        上述九个权威状态画板
03 Flows         启动、拿起→Ready→放回、离线、校准、关机
04 Asset Map     visual_state_id → PNG占位映射
05 Prototype     可点击演示，只做流程，不复制另一套组件
```

组件至少准备这些变体：

- Fact：`known / unknown-never / unknown-offline / unknown-stale / unknown-uncalibrated / unknown-fault`。
- Node：`online / low-battery / offline / incompatible`；当前代码只完成部分状态，未实现项标注占位。
- Role reply：`none / one-line / two-line / timeout / stale-dropped`。
- SystemOverlay：`info / warning / critical / confirmation / progress`。
- Touch action：`normal / pressed / disabled / locked`；危险操作必须二次确认或实体键确认。

## 6. 从现有代码拿数据

桌面状态源不是手写假接口，而是当前模拟器：

```powershell
cargo run -p ailive-gun-spirit-host --bin ailive-gun-spirit-sim
```

每行JSON中的`screen_view_model`就是前端当前应消费的最小快照。前端mock应保存这些完整快照并逐张回放，不自行拼接“可能的后端字段”。

现有关键映射：

| ViewModel | 前端用途 |
|---|---|
| `perception_revision` | 丢弃迟到角色画面、调试版本 |
| `primary_layer.kind=status` | 显示常规RoleStage/StatusBar |
| `primary_layer.kind=role` | 显示角色文本与`visual_state_id` |
| `primary_layer.kind=system` | SystemOverlay抢占 |
| `carrier_state` | Standby/Held/Ready主状态 |
| `front_grip/rear_grip` | 校准/维护详情，不建议常驻大字显示 |
| `motion/pose` | 调试与校准；正常角色页可弱化 |
| `primary_control` | 只读交互反馈，不解释为击发 |
| `nodes[]` | 前后节点健康指示 |

## 7. 第一轮评审看什么

先评审体验，不评审美术完成度：

1. 一眼能否分清Standby、Held、Ready。
2. 角色回复出现时，节点离线和低电是否仍不会被遮住。
3. 戴手套时是否只有少量、大目标触控；常用动作能否由实体键完成。
4. 枪身左右、上方或斜置屏幕时，核心信息是否仍位于安全区。
5. `Unknown`是否被诚实显示，而不是做成“看起来正常”。
6. SystemOverlay出现后，角色动画是否立即让位。

## 8. 暂不冻结

- 字体、主色、材质、角色占屏比例和边框装饰。
- WebView/Chromium/DRM renderer选择。
- 30/60 fps、动画时长和Live2D模型预算；先用PNG与实机资源数据决定。
- 主机电量、网络、模式和Lifecycle的最终ViewModel字段。
- 触摸是否成为主要操作方式。P0仍以实体键与少量维护触控为安全起点。

完成第一轮低保真稿后，需要项目所有者二次决定：选A/B/C哪种主体语言、角色默认占屏比例、状态栏常驻信息，以及正常操作是否允许触摸。
