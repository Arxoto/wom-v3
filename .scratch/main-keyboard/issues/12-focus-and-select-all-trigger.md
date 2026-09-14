# 显示时的聚焦与全选

Type: grilling
Status: open

## Question

「窗口每次显示 → input 聚焦并全选内容」由谁触发、怎么触发？Rust 在 show 的时候发一条事件给前端，还是前端自己监听窗口的 focus 变化？隐藏时输入内容保留，重新显示时怎么保证不闪、不丢光标状态？

## Answer

待解决。

## Comments

- 两条候选路，本票要定其一：① 前端自己在挂载时与窗口获焦时聚焦并全选（`inputRef.current.select()`），不需要 Rust 参与；② Rust 在显示窗口时发一条事件，由前端响应。
