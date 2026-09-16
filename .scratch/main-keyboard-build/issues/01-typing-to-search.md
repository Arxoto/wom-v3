# 01: 打字到检索

**What to build:** 主窗口的输入框变成真的——输入内容后停止 50ms，列表显示真实检索结果，替掉现在的假数据。同时落下本 effort 的前端接缝（纯 reducer + 接线 hook），后面几张票据都在它上面加东西。

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] 输入框受控：值来自交互状态，输入立刻反映在界面上
- [ ] 50ms 尾防抖；文本与上次发出的相同就不重发
- [ ] 过期响应整包丢弃：先发的请求后回来，不能覆盖新结果
- [ ] 拼音合成期间只更新输入框、不检索；`compositionend` 自己补一次检索，与随后那次变更去重后只得到一个结果
- [ ] 列表渲染真实结果，可见条数按 `Config` 的 `main_item_n`
- [ ] 结果为空时列表区显示一行占位文案
- [ ] 接线层不把副作用放进 reducer 或 state updater（`index_main.tsx` 挂着 StrictMode，dev 下会调用两次）

依据：[spec.md](../main-keyboard/spec.md) §1 的输入路径、§2 的接缝与状态、§3 的「打字 → 检索」，以及 [IME 合成期间的事件语义](../main-keyboard/issues/01-research-ime-composition.md) 的结论。
