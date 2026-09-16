# 06: Enter 跑动作

**What to build:** 选中条目按 `Enter`，执行它默认的 `Item Action`。本轮只打通 `copy`：片段的内容真的进系统剪贴板。触发前先关掉 `Preview`，要不要隐藏窗口由 Rust 按 `main_window_mode` 决定。

**Blocked by:** 02, 05

**Status:** resolved

- [x] `ItemDisplay` 带上 `Item Index`；前端寻址用它，不用列表行号
- [x] 新增派发动作的命令：按下标 + 动作 id 执行，随后按 `main_window_mode` 决定是否隐藏（`HideAndShow` 隐藏）
- [x] 越界索引或未知动作当无操作并记 warn：不 panic、不隐藏、不做版本校验
- [x] 新增剪贴板插件依赖并注册，`copy` 真的写进系统剪贴板
- [x] 触发动作前先关掉 `Preview`，`Selection` 与输入内容都不动
- [x] 条目没有可执行动作、或结果为空时，整个流程不发生：不关预览、不隐藏窗口
- [x] `Enter` 阻止默认行为；`Shift+Enter` 仍是开关预览

依据：[spec.md](../main-keyboard/spec.md) §1 的 `run_action`、§3 的「显示 / 退场」、§4.1、§4.3。
