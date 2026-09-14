# IME 合成期间的事件语义

Type: research
Status: open

## Question

在 Windows WebView2（Chromium）配合 React 19 受控 input 时，怎么可靠判断「正在合成」（拼音打字阶段）？

要确认：合成期间的完整事件序列（compositionstart / compositionupdate / beforeinput+input(isComposing) / keydown / compositionend）与 React 19 onChange 的关系；可靠的判据（`KeyboardEvent.isComposing`、composition 配对状态、`keyCode === 229` 的历史坑）；用 Enter 提交候选、ESC 取消组合那一次按键的行为；受控 input 在合成期间 setState 的已知问题；WebView2 与 Chrome 的差异。

## Answer

待研究 agent 回报。产出落在 `.scratch/main-keyboard/research/ime-composition.md`，结论接进 [打字到检索的时序](05-typing-search-and-ime.md)。

## Comments

- 问题已收窄成两条：
  1. `compositionend` 与「提交之后那次 input 事件」的先后顺序（Windows 走 WebView2 / Chromium，macOS 走 WKWebView / WebKit，两端都要看）：若 input 在后，配对布尔量的方案能自然发出最终文本的检索；若在前，会漏掉最后一次检索。
  2. 用 `compositionstart` / `compositionend` 的配对布尔量去拦按键是否足够安全，还是必须同时看 `KeyboardEvent.isComposing`（以及 `keyCode === 229` 这个老判据在 WebView2 上是否还需要）。
