/**
 * 按键到意图的映射
 *
 * 纯函数：[`resolve_key`] 只回答「这一次按键是什么意图」，不碰 DOM、不改状态、不做副作用。
 * 改状态的意图交给 [`reduce_main`](./reducer.ts)，退场这类要对外发号施令的留给
 * `useMainInteraction`，三者分开才各自可验（见 docs/adr/0006-window-level-keydown-entry.md）。
 *
 * 放行的口径：返回 `null` 就是不认这个键——不产生意图，也不阻止默认行为。
 */

/** 按键意图 */
export type Intent =
    | "select_prev"
    | "select_next"
    | "run_action"
    | "toggle_preview"
    | "dismiss";

/** 移动 Selection 的意图：连发限流只对它们生效 */
export const is_select_intent = (intent: Intent) =>
    intent === "select_prev" || intent === "select_next";

/** 按下时按着的修饰键 */
export interface KeyMod {
    shift: boolean,
}

/** 判定意图时用得上的状态 */
export interface KeyContext {
    /** Preview 打开时 ESC 只关它（见 spec §1 的按键表） */
    preview_open: boolean,
}

/**
 * 一次 keydown 的意图；未识别与本层放行的键返回 `null`
 *
 * `↑` / `↓` 在两种模式下都移动 `Selection`：`Preview` 跟随 `Selection`，预览开着时也要能
 * 直接上下换条目。它们产生意图但不阻止默认行为：单行输入框里光标被顶到首尾是接受的副作用。
 * `←` / `→` 与其余未识别的键一律放行，光标照常移动——本 effort 没有切换 Item Action 的键位。
 * `ESC` 在预览打开时只关预览，所以那时它映射成 `toggle_preview`，而不是 `dismiss`。
 */
export const resolve_key = (key: string, mod: KeyMod, ctx: KeyContext): Intent | null => {
    switch (key) {
        case "ArrowUp":
            return "select_prev";
        case "ArrowDown":
            return "select_next";
        case "Enter":
            // 预览打开时 Enter 与列表模式一致，都是跑动作
            return mod.shift ? "toggle_preview" : "run_action";
        case "Escape":
            // 预览打开时只关预览、不隐藏窗口；回到列表模式才是退场
            return ctx.preview_open ? "toggle_preview" : "dismiss";
        default:
            return null;
    }
}
