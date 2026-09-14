# Selection 与 Preview 的行为

Type: grilling
Status: resolved

## Question

Selection 怎么移动？新结果回来落在哪？鼠标 hover 算不算选中？Preview 与 Selection 是什么关系？

## Answer

- 新结果回来 `Selection` 回到第一条；↑/↓ 到边界 clamp，不环绕。
- 鼠标 hover 本 effort 只保留现有 CSS 效果，不写入 `Selection`。
- `Preview` 跟随 `Selection`（结果变了就换条），结果为空时自动关闭。
- `Preview` 打开时：↑/↓ 不动 `Selection`，←/→ 与 Enter 行为与未打开时一致。
- ESC 在 `Preview` 打开时只关 `Preview`，再按一次才退场（见 [退场与窗口模式](08-dismiss-and-window-mode.md)）。
