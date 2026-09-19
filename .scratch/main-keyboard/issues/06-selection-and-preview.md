# Selection 与 Preview 的行为

Type: grilling
Status: resolved

## Question

Selection 怎么移动？新结果回来落在哪？鼠标 hover 算不算选中？Preview 与 Selection 是什么关系？

## Answer

- 新结果回来 `Selection` 回到第一条；↑/↓ 到边界 clamp，不环绕。
- 鼠标 hover 本 effort 只保留现有 CSS 效果，不写入 `Selection`。
- `Preview` 跟随 `Selection`（结果变了就换条），结果为空时自动关闭。
- **结果为空**：列表区显示一行占位文案——输入为空时是「输入关键字开始搜索」，有输入但没匹配时是「没有匹配的条目」，Tail 保持列表模式那套提示；依赖条目的按键（Enter）无操作。
- `Preview` 打开时：↑/↓ 照常移动 `Selection`（`Preview` 跟着换条；不滚预览内容，预览内容的滚动属于 Note 渲染那条线）；`←` / `→` 放行（见 [Item Action 与触发命令](07-item-action-and-command.md)），Enter 行为与未打开时一致。
- `Preview` 打开时 input 不接受输入：落在 `readOnly` 上，焦点不动（主动 `blur()` 会打断可能正在进行的合成），所以不存在「预览打开时打字」这一情形。关闭 `Preview` 时不做额外动作，输入自然恢复。
- ESC 在 `Preview` 打开时只关 `Preview`，再按一次才退场（见 [退场与窗口模式](08-dismiss-and-window-mode.md)）。

## Comments

- 2026-09-19（依代码订正）：空结果那一条现在只指**结论确实没匹配**——列表区此时说一行「没有匹配的条目」；输入为空（还没有结论）时列表区留白，提示在 `Head` 的 ghost 里（`empty = conclusion === null`，见 [spec.md](../spec.md) §2 / §3）。下面 2026-09-19 那条评论里「输入为空时列表区显示『输入关键字开始搜索』」随之作废。
- 2026-09-19：占位文案分成两种——输入为空（不检索、结果已置空）时「输入关键字开始搜索」，有输入没匹配时仍是「没有匹配的条目」。
- ~~原文「↑/↓ 不动 `Selection`」~~ 2026-09-18 改为「预览打开时 ↑/↓ 照常移动 `Selection`」：预览跟随 `Selection`，锁住上下键只会让人先在列表模式选好再开预览，多一次来回。
- 「↑/↓ 也不做别的」与「input 不接受输入」是 2026-09-14 讨论补的：原文只写了「↑/↓ 不动 Selection」，没交代预览打开时输入这条路还通不通。
- 「空结果」那条与「input 不接受输入」是 2026-09-14 补的：前者原文只定了 Preview 自动关闭，后者原文只写了「↑/↓ 不动 Selection」，没交代预览打开时输入这条路还通不通。
