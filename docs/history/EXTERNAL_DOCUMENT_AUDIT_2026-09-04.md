# 外部四季宝相关文档归并审计（2026-09-04）

**状态**：HISTORICAL AUDIT  
**扫描位置**：`C:\Users\13603\Desktop\个人文档`  
**规则**：不删除桌面文件；仓库已有相同或更新SSOT时不制造第二份可编辑副本。哈希用于确认本次扫描对象，不代表内容真实性。

| 外部文件 | 字节 | SHA-256 | 处理结果 | 仓库入口 |
|---|---:|---|---|---|
| `四季宝-人类阅读版.txt` | 66229 | `0E805062789039C73BC0EF2A198EB9A74DB11799DD2D7CEA0A8872B41CEF11F5` | `SUPERSEDED`：仓库版更新到Zero 3W/模型矩阵 | `../handoff/四季宝-人类阅读版.txt` |
| `四季宝-智能战术配件系统-ai阅读版.txt` | 252232 | `176760E8D4799EA78522ADC7FDC819D9F20BA5ADE0C78B58AEB16DC5EE8F5D5E` | `SUPERSEDED`：仓库版包含后续决策 | `../handoff/四季宝-智能战术配件系统-ai阅读版.txt` |
| `A.I.Live-ai枪娘器灵-采购清单-完整版.md` | 5516 | `8FCEFB52EB3FFF49F7F6B5DC6B96C257FA15B596D766C32CC3CEDFEB011F9572` | `SUPERSEDED`：仓库清单已改为Zero 3W并增加实测门 | `../test-worksheets/采购核对清单.md` |
| `A.I.Live-ai枪娘器灵-技术债与审查台账.md` | 5574 | `FB96C239F872C831BF69CFC0E4D56EBDF007910194D862E21814A6AB09302F62` | `SUPERSEDED`：仓库台账更新 | `../TECHNICAL_DEBT.md` |
| `A.I.Live-ai枪娘器灵-既有决策体验复核清单.md` | 4825 | `E6614960A5A197AA0BB138CB23973593CB86FDF787470E221FF9376A3DB6E0F2` | 与仓库SSOT逐字节一致，不再复制 | `../UX_RESEARCH.md` |
| `A.I.Live-ai枪娘器灵-开发纪律.md` | 10056 | `508E13757B3FCA734778AF14B735FA1DB9A73BE066EFDAB46D4F28ACE6BAE77C` | 与仓库SSOT逐字节一致，不再复制 | `../DEVELOPMENT_DISCIPLINE.md` |
| `A.I.Live-ai枪娘器灵-余弹融合扩展设计.md` | 13384 | `935F6B5E4EECC1B2DB6B260148FC72A5BE5DCF859F78E623186D4E81F0171BE7` | 与仓库SSOT逐字节一致，不再复制 | `../MAGAZINE_AMMO_ESTIMATION.md` |
| `水弹智能瞄准-结构思维实验记录.md` | 44032 | `FF746C74C4AF26FB03C3893B94C031575B4A46953FA889FBD6F6A9D6A1A5D9A5` | `EXCLUDED`：原文声明独立于本仓，且自动瞄准跨越项目禁止边界 | 无；不进入backlog |

个人文档目录中的`OCLive-*`、开发故事、用户/开发者简介、聊天导出、路演材料和其它项目文件没有导入：它们属于OCLive、个人背景或其它项目，不是四季宝工程SSOT。若未来某段内容确实形成四季宝独有需求，应提炼成ADR/领域文档，不能整份聊天或个人画像复制进工程仓。

本次“归并”含义是：所有仍有效的四季宝工程主题已经有仓库canonical入口，外部旧副本有明确去向；不是把重复文件机械复制进仓库。桌面副本以后只可作为导出/填写件，修改完成后必须回填仓库，不得双向各自演化。
