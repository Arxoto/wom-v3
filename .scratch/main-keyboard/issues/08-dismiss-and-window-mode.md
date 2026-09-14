# 退场与窗口模式

Type: grilling
Status: resolved

## Question

ESC 与「触发动作后」要不要退场？由谁判定？退场后输入内容与焦点怎么办？

## Answer

- **不再有「销毁窗口」**：退场一律是隐藏，不存在关掉再重建。
- `main_window_mode` 现在管两件事：`Always` = 失焦不隐藏 + 触发动作后不隐藏；`HideAndShow` = 失焦隐藏（已有实现）+ 触发动作后隐藏（新增）。
- **ESC 一律隐藏**，与配置无关：它是显式意图，配置描述的是失焦、触发动作这两条被动隐藏规则。
- **判定落在 Rust**：新增一条退场命令 `dismiss_main_window`，前端只表达「我要退场」，命令本身无条件隐藏（它服务的就是 `ESC`）。失焦隐藏已经在 Rust 的窗口事件里，触发动作后的隐藏落在 `run_item_action` 里，不设第二个判定点。
- **触发动作后**：前端先关掉 `Preview`（`Selection` 与输入内容都不动），再跑动作；`Always` 下窗口留着、`HideAndShow` 下隐藏，这个判断由 Rust 在动作里做。
- **窗口每次显示**：先强制关掉 `Preview`（唤出等于重新开始），再让 input 聚焦并全选内容——既能看到上次的结果，又能直接重打。触发点见 [显示时的聚焦与全选](12-focus-and-select-all-trigger.md)。

## Comments

- 「先强制关掉 `Preview`」是 2026-09-14 补的：不强制关的话，「每次显示都聚焦 input」与 [Selection 与 Preview 的行为](06-selection-and-preview.md) 的「预览打开时 input 不接受输入」互相矛盾。
- 「触发动作后」那条的判定点改到动作里：原文写的是「再走同一条退场命令」，但 `ESC` 那条命令要无条件隐藏、触发动作后又要跟配置走，同一条无参命令分不出这两种来由。现在 `dismiss_main_window` 无条件隐藏（只服务 `ESC`），触发动作后的隐藏由 `run_item_action` 按 `main_window_mode` 决定。
