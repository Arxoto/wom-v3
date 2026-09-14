# 显示时的聚焦与全选

Type: grilling
Status: resolved

## Question

「窗口每次显示 → input 聚焦并全选内容」由谁触发、怎么触发？Rust 在 show 的时候发一条事件给前端，还是前端自己监听窗口的 focus 变化？隐藏时输入内容保留，重新显示时怎么保证不闪、不丢光标状态？

## Answer

走 Rust 侧事件（评论里的选项 ②）：显示的代码只有 `window_utils`（`show_main_window` / `show_hide_main_window` 的显示分支，以及 `create_main_window` 的初次显示），在那里向主窗口发一条事件（暂名 `main_shown`）；前端 `useMainInteraction` 监听它并 `input_ref.current?.select()`，挂载时也做一次（覆盖「启动即显示」时事件早于前端就绪的情况）。

理由：隐藏期间 WebView 的 DOM focus 语义不可靠，前端自己听 `focus` 会出「有时全选有时不选」；显示这件事只有 Rust 知道，而窗口收 / 发的判定本来就全在 Rust（见 [退场与窗口模式](08-dismiss-and-window-mode.md)）。

## Comments

- 两条候选路，本票要定其一：① 前端自己在挂载时与窗口获焦时聚焦并全选（`inputRef.current.select()`），不需要 Rust 参与；② Rust 在显示窗口时发一条事件，由前端响应。
