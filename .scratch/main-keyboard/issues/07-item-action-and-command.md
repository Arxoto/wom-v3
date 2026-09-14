# Item Action 与触发命令

Type: grilling
Status: resolved

## Question

动作表从哪来？「当前动作」归谁、何时重置？命令长什么样、失败怎么办？

## Answer

- 动作表在 **Rust 侧按 `ItemType` 写死**，经元数据下发到前端；不塞进 `ItemDisplay`（按 `CONTEXT.md` 的定义，它只带类型、名称、描述）。
- 每条动作带一个 **ASCII 国际化键**做显示标签（如 `action.copy`），由前端字典解析成中文；枚举名是「东西的名字」，标签键是「文案的名字」，两者分开，中文一个字都不进 Rust。
- **本 effort 不切动作**：`←` / `→` 完全放行（没法判断用户想移动光标还是换动作——不产生意图、不 `preventDefault()`），每个条目只跑它的默认动作（动作表第一个）。切换动作留给将来的动作菜单那条线（见 map 的 Out of scope）。
- 命令是一条通用的 `run_item_action({ item_index, action })`，与「传递 Item 或索引以及对应动作」一致；越界索引或未知动作当无操作并记 warn，不 panic，也不做版本校验（沿用仓库「开发期一律破坏性处理」的态度）。
- 具体的类型 → 动作映射与元数据形状待 [Item Action 映射表与元数据](10-item-action-table-and-metadata.md)。

## Comments

- 触发动作要有防重复触发的保护：`index_main.tsx` 挂着 `React.StrictMode`，一次按键意图可能被算两次。
- 落地为一条结构约束，不需要 nonce 之类的去重：副作用（`run_item_action`、退场、检索、预请求）只在事件回调里执行，不在 reducer 或 state updater 里做。见 [前端接缝的形状](11-frontend-module-shape.md)。
- 2026-09-14 取消「`←` / `→` 切换动作」与「当前动作下标」这个状态，理由见 Answer 里那条。`run_item_action` 的 `item_index` 在 JS 侧按 tauri 的默认约定写成 `itemIndex`。
