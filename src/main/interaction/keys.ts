/** 按键意图 */
export type Intent =
    | "select_prev"
    | "select_next"
    | "action_prev"
    | "action_next"
    | "run_action"
    | "toggle_preview"
    | "open_search"
    | "close_search"
    | "dismiss";

/** 移动 Selection 的意图：连发限流只对它们生效 */
export const is_select_intent = (intent: Intent) =>
    intent === "select_prev" || intent === "select_next";

/** 左右切换当前 Item Action 的意图：要不要拦下按键取决于这一侧还有没有动作 */
export const is_action_intent = (intent: Intent) =>
    intent === "action_prev" || intent === "action_next";

/** 进出插件搜索页的意图：它们要发请求，所以由接线层处理，不进 reducer */
export const is_search_intent = (intent: Intent) =>
    intent === "open_search" || intent === "close_search";

/** 按下时按着的修饰键 */
export interface KeyMod {
    shift: boolean,
}

/** 判定意图时用得上的状态 */
export interface KeyContext {
    /** Preview 打开时 ESC 只关它（见 spec §1 的按键表） */
    preview_open: boolean,
    /** 插件搜索页打开时：ESC 与退格都从这里返回主列表（spec §4.3） */
    search_open: boolean,
    /** 当前条目是不是插件条目：Enter 与 ⇧+Enter 都进插件搜索页（spec §4.1） */
    plugin_item: boolean,
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
            // 搜索页里 Enter 就跑这一行的动作；插件条目的两条路（Enter 与 ⇧+Enter）
            // 都通向它的 Plugin Search Page——插件条目的延伸内容就是那一页（spec §4.1）
            if (ctx.search_open) return "run_action";
            if (ctx.plugin_item) return "open_search";
            return mod.shift ? "toggle_preview" : "run_action";
        case "Backspace":
            // 退格只在搜索页里是一个意图，别的时候归输入框自己
            return ctx.search_open ? "close_search" : null;
        case "Escape":
            if (ctx.search_open) return "close_search";
            // 页面不同，行为不同
            return ctx.preview_open ? "toggle_preview" : "dismiss";
        default:
            return null;
    }
}
