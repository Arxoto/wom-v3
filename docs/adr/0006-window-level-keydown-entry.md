# 主窗口的按键收在一个 window 级入口后面

主窗口的按键只在 `window` 捕获阶段的单个 `keydown` 监听里接收，不挂在 input 上。按键 → 意图 → 状态分三层：`resolve_key(key, mod, ctx) → Intent | null` 出意图，`reduce` 落状态，接线（防抖、请求令牌、预请求、invoke）只留在 `useMainInteraction` 一处；副作用只在事件回调里执行，不进 reducer / state updater。

落地：`src/main/interaction/`（`keys.ts` / `reducer.ts` / `useMainInteraction.ts`）。退场、失焦隐藏、触发动作后是否隐藏、显示时聚焦并全选，由 Rust 侧按 `Config` 与窗口事件决定，前端只表达意图。
