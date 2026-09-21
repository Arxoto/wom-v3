# 主窗口的按键收在一个 window 级入口后面

主窗口的按键只在 `window` 捕获阶段的单个 `keydown` 监听里接收，不挂在 input 上。按键 → 意图 → 状态分三层：`resolve_key(key, mod, ctx) → Intent | null` 出意图，`reduce` 落状态，接线（事件注册、invoke、聚焦）只留在 `useMainInteraction` 一处；副作用只在事件回调里执行，不进 reducer / state updater。

落地：`src/main/interaction/`（`keys.ts` / `reducer.ts` / `search_session.ts` / `useMainInteraction.ts`），检索的时序（防抖、请求令牌、预请求记账、合成锁）收在 `search_session.ts`。退场、失焦隐藏、触发动作后是否隐藏，由 Rust 侧按 `Config` 与窗口事件决定，前端只表达意图；显示这条路反过来——要不要显示仍由 Rust 判（配置在它手里），但「什么时候能露面」只有前端知道，所以后端只发一条 `main_will_show`，由前端起入场动效、回过话，窗口才真的亮（见 `window_utils`）。
