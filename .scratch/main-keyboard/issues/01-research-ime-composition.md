# IME 合成期间的事件语义

Type: research
Status: resolved

## Question

在 Windows WebView2（Chromium）配合 React 19 受控 input 时，怎么可靠判断「正在合成」（拼音打字阶段）？

要确认：合成期间的完整事件序列（compositionstart / compositionupdate / beforeinput+input(isComposing) / keydown / compositionend）与 React 19 onChange 的关系；可靠的判据（`KeyboardEvent.isComposing`、composition 配对状态、`keyCode === 229` 的历史坑）；用 Enter 提交候选、ESC 取消组合那一次按键的行为；受控 input 在合成期间 setState 的已知问题；WebView2 与 Chrome 的差异。

## Answer

**已定（2026-09-14）**：

- **input 路径**：`compositionstart` / `compositionend` 维护一个 lock 布尔量，判断「这次变更是不是合成中间态」。
- `compositionend` 自己也触发一次检索（读 input 的当前值），靠 [打字到检索的时序](05-typing-search-and-ime.md) 的「文本与上次发送相同就不重发」去重。这样 `compositionend` 与提交后那次 input 谁先谁后都得到同一结果，票里第 1 条不必再赌。
- 不加「显示时清 lock」的兜底：lock 卡住只可能发生在「合成被放弃、`compositionend` 没来」这种罕见情形，而且下一次走 IME 的输入会把状态自然翻回来。注意纯英文输入与粘贴不发任何 composition 事件，所以那种情况下不会自愈——2026-09-14 判断为可接受。

**keydown 路径：按「没问题」的假设先实现（2026-09-14 定）**：

- 不做任何合成判断：不读 `KeyboardEvent.isComposing`，`Enter` 也不读；lock 不参与 keydown。
- 实现时在 keydown 入口留一条注释，说明「候选框打开期间这些按键一旦到达页面就会直接产生意图」这个已知风险（`Enter` 会跑动作、`↑` / `↓` 会移动 `Selection`、`ESC` 会关预览或隐藏窗口）。
- 实测推迟到功能验收：那时确认候选框打开时 `Enter` / `↑` / `↓` / `←` / `→` / `ESC` 到底会不会到达页面、带什么 `isComposing` / `keyCode`（WebKit 的尾随 229 一并看）。收不到就不用判断；收到且能可靠判出合成，再按最小改动补判据（`isComposing`，必要时补 `keyCode === 229`）。

## Comments

- 问题已收窄成两条：
  1. `compositionend` 与「提交之后那次 input 事件」的先后顺序（Windows 走 WebView2 / Chromium，macOS 走 WKWebView / WebKit，两端都要看）：若 input 在后，配对布尔量的方案能自然发出最终文本的检索；若在前，会漏掉最后一次检索。
  2. 用 `compositionstart` / `compositionend` 的配对布尔量去拦按键是否足够安全，还是必须同时看 `KeyboardEvent.isComposing`（以及 `keyCode === 229` 这个老判据在 WebView2 上是否还需要）。

- **两个判据方案的差别（2026-09-14 与用户讨论）**：

  **方案 A：`compositionstart` / `compositionend` 的配对布尔量**

  - 描述的是「合成会话开着没有」，不是「这一次按键」。
  - 与引擎无关，两个引擎都发这两个事件，实现最简单。
  - 致命点是失同步：合成中途失焦 / 隐藏窗口 / 输入法自己取消，`compositionend` 可能不来，标志永远停在 `true`，此后所有被拦截的键全部失灵。本应用失焦即隐藏（`HideAndShow`），这是常态场景而不是边角。
  - 合成结束后的尾随事件它看不见（那时标志已经翻回 `false`）。
  - 它也说不出「这次 `input` 是合成的中间态还是最终态」，只能赌 `compositionend` 与 `input` 的先后——也就是上面第 1 条。

  **方案 B：读单次事件上的 `KeyboardEvent.isComposing`（必要时兼看 `keyCode === 229`）**

  - 描述的是「这次按键是否落在合成会话内」，无状态，不会失同步。
  - 判据本身有引擎差异：Chromium 上提交候选那一下的 `keydown` 在 `compositionend` 之前，`isComposing = true`（正是我们要的：Enter 不触发动作）；WebKit 历史上有「`compositionend` 之后补一次 `keyCode === 229` 的 `keydown`」的行为，那次 `isComposing` 可能已经是 `false`，单看它就当成真按键了——`keyCode === 229` 这个老判据就是为这个存在的。
  - 有些输入法把 Enter / ESC 直接吞掉，页面收不到 `keydown`。这种情况两个方案都收不到，不是差异点，但验证时要确认（收不到就不必为它设计）。

- **要验证的五个事实**（前三条决定判据怎么写）：
  1. 提交候选那一下的 `keydown`：页面收得到吗？`isComposing` 与 `keyCode` 是多少？相对 `compositionend` 在前还是在后？
  2. `compositionend` 与「提交后那次 input」谁先谁后（决定最后一次文本会不会触发检索）。
  3. 合成中途失焦 / 隐藏后，`compositionend` 会不会来？重新显示后还能不能正常打字？
  4. ESC 取消组合时页面收到什么。
  5. 合成结束后有没有尾随的 `keydown`（WebKit 的 229 就是它）。

- **倾向**：B 做主判据，A 只作为辅助信息（判断这次 `input` 是不是合成产物，给检索时序用），并补一条「窗口显示时强制清掉合成标志」的兜底。`keyCode === 229` 先不写，等 3 与 5 的验证结果——出现了才加。

- **用户方案与我的更正（2026-09-14）**：用户提的是「lock 只在 input 里判断，keydown 不管它」。这个说法成立——失同步不会让按键失灵，因为按键根本不受 lock 控制；我之前那句「所有被拦截的键失灵」只适用于 lock 参与 keydown 判断的写法。要更正的一处后果是：lock 卡在 `true` 时 input 全被当成合成中间态，**检索会停摆**（文本照常变、结果不更新），直到下一次完整的 `compositionstart` / `compositionend` 才恢复。用户判断这个场景罕见、不值得为它加代码，采纳。keydown 那条路仍需要一个判据，用 `e.isComposing`；如果验证发现某个引擎的 keydown 不带这个标记，再升级成 `lock || e.isComposing`。

- **验证办法（本机、不改代码）**：`pnpm tauri dev` 打开主窗口 → devtools Console 粘贴下面的监听，然后用拼音输入法依次走：输入拼音（不选词）→ 空格选词 → Enter 提交候选 → 重来一次用 ESC 取消组合 → 合成到一半点别的窗口让主窗口隐藏，再唤出。

  ```js
  const t0 = performance.now();
  const p = (...a) => console.log(((performance.now() - t0) / 1000).toFixed(3), ...a);
  for (const [type, extra] of [
    ['compositionstart',  e => e.data],
    ['compositionupdate', e => e.data],
    ['compositionend',    e => e.data],
    ['beforeinput',       e => e.inputType],
    ['input',             e => JSON.stringify(e.target.value)],
  ]) {
    window.addEventListener(type, e => p(type, extra(e), 'isComposing=' + e.isComposing), true);
  }
  window.addEventListener('keydown', e => p('keydown', e.key, 'code=' + e.code, 'keyCode=' + e.keyCode, 'isComposing=' + e.isComposing), true);
  window.addEventListener('keyup',   e => p('keyup',   e.key, 'code=' + e.code, 'keyCode=' + e.keyCode, 'isComposing=' + e.isComposing), true);
  ```
