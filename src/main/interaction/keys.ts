/** 按键意图 */
export type Intent =
    | "select_prev"
    | "select_next"
    | "action_prev"
    | "action_next"
    | "run_action"
    | "toggle_preview"
    | "dismiss";

/** 移动 Selection 的意图：连发限流只对它们生效 */
export const is_select_intent = (intent: Intent) =>
    intent === "select_prev" || intent === "select_next";

/** 左右切换当前 Item Action 的意图：要不要拦下按键取决于这一侧还有没有动作 */
export const is_action_intent = (intent: Intent) =>
    intent === "action_prev" || intent === "action_next";

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
 * 将操作输入识别为意图；未识别的返回 `null`
 */
export const resolve_key = (key: string, mod: KeyMod, ctx: KeyContext): Intent | null => {
    switch (key) {
        case "ArrowUp":
            return "select_prev";
        case "ArrowDown":
            return "select_next";
        case "ArrowLeft":
            return "action_prev";
        case "ArrowRight":
            return "action_next";
        case "Enter":
            return mod.shift ? "toggle_preview" : "run_action";
        case "Escape":
            // 页面不同，行为不同
            return ctx.preview_open ? "toggle_preview" : "dismiss";
        default:
            return null;
    }
}
