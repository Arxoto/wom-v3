# Item Action 映射表与元数据

Type: grilling
Status: resolved

## Question

`ItemType` → `Item Action` 的具体映射表是什么？四个候选动作——复制进剪贴板、用默认浏览器打开、用默认方式打开、在资源管理器中选中该文件——分别挂到哪些 `ItemType` 上，各自第几个是默认动作？

## Answer

**元数据的形状（2026-09-14 已定）**：

- 命令叫 `fetch_item_type_actions`，无入参，返回 `ItemType`（Rust 里那个同名枚举，序列化成 `snip` / `sys` / `note` / `cmd` / `web` / `file` / `scan` 这组字符串）→ 该类型支持的动作列表。
- `ItemDisplay` 已经带着 `the_type`（见 `search::ItemDisplay`），前端因此不需要第二套映射就能查到动作表；`core.tsx` 的镜像把 `the_type: string` 收成 `ItemType` 联合类型。
- 每条动作带 ASCII 的 `id` 与 `label_key`。**`label_key` 按 `ItemType` + 动作分套**（`action.file.copy` = 「复制完整路径」、`action.web.copy` = 「复制链接」）：同一个 `copy` 在不同类型上文案不同，这正是「根据 ItemType + Action 描述」的来源。
- 不按平台剔除动作：桌面三平台上四类动作都有官方路径（见 [`research/item-action-apis.md`](../research/item-action-apis.md)），真遇到某个平台缺 API 时再说。
- 字典里查不到 `label_key` 时显示键本身（不崩、也一眼看得出漏翻译）。

**映射表（2026-09-14 定，顺序即优先级，第一个是默认动作）**：

| `ItemType` | 动作 | 依据 |
| --- | --- | --- |
| `Snippets` | `copy` | 注释「片段 仅允许复制」 |
| `System` | 无（空列表） | 不挂动作 |
| `Note` | `open_note` | 占位，不具体实现 |
| `Cmd` | `copy` | 注释「可复制并自动打开终端、后台执行」，执行不在四类动作里 |
| `Web` | `open_url`, `copy` | 注释「支持使用默认浏览器打开、复制连接」 |
| `File` | `open_path`, `reveal`, `copy` | 注释「支持默认方式打开、在文件夹中选中、复制完整路径」 |
| `Scan` | `open_path`, `reveal`, `copy` | 与 File 同类行为 |

动作 id 全集：`copy` / `open_url` / `open_path` / `reveal` / `open_note`。真实执行不在这批决策范围内，只保留一条最小动作用于验证链路（见 [Destination 与范围](03-destination-and-scope.md)）；用哪一条待确认，倾向 `copy`（见 [spec.md](../spec.md) §4.3）。
