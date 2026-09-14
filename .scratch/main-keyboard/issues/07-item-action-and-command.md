# Item Action 与触发命令

Type: grilling
Status: resolved

## Question

动作表从哪来？「当前动作」归谁、何时重置？命令长什么样、失败怎么办？

## Answer

- 动作表在 **Rust 侧按 `ItemType` 写死**，经元数据下发到前端；不塞进 `ItemDisplay`（按 `CONTEXT.md` 的定义，它只带类型、名称、描述）。
- 每条动作带一个 **ASCII 国际化键**做显示标签（如 `action.copy`），由前端字典解析成中文；枚举名是「东西的名字」，标签键是「文案的名字」，两者分开，中文一个字都不进 Rust。
- 前端持有「当前动作下标」；切换 `Selection` 时重置为该 Item 的第一个动作（默认动作）；←/→ 到边界 clamp，不环绕，也不记住每个 Item 上次的动作。
- 命令是一条通用的 `run_item_action({ item_index, action })`，与「传递 Item 或索引以及对应动作」一致；越界索引或未知动作当无操作并记 warn，不 panic，也不做版本校验（沿用仓库「开发期一律破坏性处理」的态度）。
- 具体的类型 → 动作映射与元数据形状待 [Item Action 映射表与元数据](10-item-action-table-and-metadata.md)。

## Comments

- 触发动作要有防重复触发的保护：`index_main.tsx` 挂着 `React.StrictMode`，一次按键意图可能被算两次。
