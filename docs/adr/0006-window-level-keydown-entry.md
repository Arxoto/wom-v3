# 主窗口的按键收在一个 window 级入口后面

主窗口的按键只在 `window` 捕获阶段的单个 `keydown` 监听里接收，不挂在 input 上。按键 → 意图 → 状态分三层：`resolve_key(key, mod, ctx) → Intent | null` 出意图，`reduce` 落状态，接线（事件注册、invoke、聚焦）只留在 `useMainInteraction` 一处；副作用只在事件回调里执行，不进 reducer / state updater。

落地：`src/main/interaction/`（`keys.ts` / `reducer.ts` / `search_session.ts` / `useMainInteraction.ts`），检索的时序（防抖、请求令牌、预请求记账、合成锁）收在 `search_session.ts`。退场、失焦隐藏、触发动作后是否隐藏，由 Rust 侧按 `Config` 与窗口事件决定，前端只表达意图；显示这条路反过来——「什么时候能露面」只有前端知道，所以主窗口一律隐藏着创建，后端只发一条 `main_will_show`，前端收到后回话让窗口真的亮，显示完成再由后端发 `main_shown` 通知前端聚焦（见 `window_utils`）。创建后要不要立刻显示同样由前端起头：挂载时问一次 `should_show_main_auto`（配置仍归 Rust），要显示就走同一条流程。
