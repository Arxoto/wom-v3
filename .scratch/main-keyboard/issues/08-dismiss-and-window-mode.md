# 退场与窗口模式

Type: grilling
Status: resolved

## Question

ESC 与「触发动作后」要不要退场？由谁判定？退场后输入内容与焦点怎么办？

## Answer

- **不再有「销毁窗口」**：退场一律是隐藏，不存在关掉再重建。
- `main_window_mode` 现在管两件事：`Always` = 失焦不隐藏 + 触发动作后不隐藏；`HideAndShow` = 失焦隐藏（已有实现）+ 触发动作后隐藏（新增）。
- **ESC 一律隐藏**，与配置无关：它是显式意图，配置描述的是失焦、触发动作这两条被动隐藏规则。
- **判定落在 Rust**：新增一条退场命令（暂名 `dismiss_main_window`），前端只表达「我要退场」，是否真隐藏由 Rust 读 `Config` 决定。失焦隐藏已经在 Rust 的窗口事件里，同一套规则不设第二个判定点。
- **触发动作后**：前端先关掉 `Preview`（`Selection` 与输入内容都不动），再走同一条退场命令。`Always` 下窗口留着，`HideAndShow` 下隐藏。
- **窗口每次显示**：input 聚焦并全选内容——既能看到上次的结果，又能直接重打。触发点见 [显示时的聚焦与全选](12-focus-and-select-all-trigger.md)。
