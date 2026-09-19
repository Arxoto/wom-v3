# 显示时的聚焦与全选

Type: grilling
Status: resolved

## Question

「窗口每次显示 → input 聚焦并全选内容」由谁触发、怎么触发？Rust 在 show 的时候发一条事件给前端，还是前端自己监听窗口的 focus 变化？隐藏时输入内容保留，重新显示时怎么保证不闪、不丢光标状态？

## Answer

走 Rust 侧事件（评论里的选项 ②）：显示的代码只有 `window_utils`（`show_main_window` / `show_hide_main_window` 的显示分支，以及 `create_main_window` 的初次显示），在那里向主窗口发一条事件（暂名 `main_shown`）；前端 `useMainInteraction` 监听它并 `input_ref.current?.select()`，挂载时也做一次（覆盖「启动即显示」时事件早于前端就绪的情况）。

理由：隐藏期间 WebView 的 DOM focus 语义不可靠，前端自己听 `focus` 会出「有时全选有时不选」；显示这件事只有 Rust 知道，而窗口收 / 发的判定本来就全在 Rust（见 [退场与窗口模式](08-dismiss-and-window-mode.md)）。

## Comments

- 2026-09-19：补一条窗口**开着**时的路径——输入框自己再获得焦点（点一下、Tab 回来）也全选。挂在 `useMainInteraction` 的 input 事件那一层：`focus` 事件里 `select()`，外加「焦点不在输入框上」时把 `mousedown` 的默认动作拦掉自己聚焦——按下的默认动作排在 focus 之后，会把刚做的全选覆盖成落点光标；已经聚焦时照常放行，点第二下就是放光标。窗口显示那条路照旧走 Rust 的 `main_shown`：本票否掉前端听 `focus`，否的是**隐藏期间**的语义（不可靠），这里管的是可见窗口里的真实焦点变化，两者不冲突。
- 两条候选路，本票要定其一：① 前端自己在挂载时与窗口获焦时聚焦并全选（`inputRef.current.select()`），不需要 Rust 参与；② Rust 在显示窗口时发一条事件，由前端响应。
