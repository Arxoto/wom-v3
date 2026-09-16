# 03: Preview 的开关与退场

**What to build:** `Shift+Enter` 开关 `Preview`；`ESC` 在预览打开时只关预览，列表模式下隐藏主窗口。Tail 的两套提示跟着真实状态走，不再是写死的常量。

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] `Shift+Enter` 切换 `Preview`；`Preview` 打开时再按一次关掉
- [ ] `ESC` 在预览打开时关预览、**不**隐藏窗口；再按一次才隐藏
- [ ] 新增无条件隐藏主窗口的命令，与 `main_window_mode` 无关（`ESC` 是显式意图，不吃配置）
- [ ] `Preview` 打开时输入框只读，`↑` / `↓` 与 `←` / `→` 都不产生意图；输入内容保留
- [ ] 结果为空时 `Preview` 自动关闭
- [ ] `Enter` / `Shift+Enter` / `ESC` 三个键阻止默认行为；其余键一律放行
- [ ] `Preview` 跟随 `Selection`，不再借用别的状态显示

依据：[spec.md](../main-keyboard/spec.md) §1 的按键表、§2 的空结果、§4.4。
