import type { ItemDisplay, ItemSearchPage } from "../../core";
import type { Intent } from "./keys";

/**
 * 主窗口的交互状态
 *
 * 渲染用得上的都在这里；请求时序（令牌、在飞的请求、防抖定时器）留在检索会话里，只有
 * 「结果还在路上」这一个渲染要用的结果镜像进来。
 */
export interface MainState {
    /** 输入框的值（受控） */
    input: string,
    /** 已加载的结果，翻页时往后追加 */
    item_list: ItemDisplay[],
    /** 结果总数，用来判断还有没有下一页 */
    total: number,
    /** List Position：Selection 落在已加载列表的第几行，不是 Item Index */
    selection: number,
    /** Preview 是否打开 */
    preview_open: boolean,
    /** 结果还在路上：第一页发出去还没落定 */
    searching: boolean,
}

export const MAIN_STATE_INIT: MainState = {
    input: "",
    item_list: [],
    total: 0,
    selection: 0,
    preview_open: false,
    searching: false,
}

/**
 * 状态变化的来源
 */
export type MainAction =
    | { kind: "typing", value: string }
    | { kind: "page_loaded", page: ItemSearchPage | undefined }
    | { kind: "page_appended", page: ItemSearchPage }
    | { kind: "intent", intent: Intent }
    | { kind: "main_shown" }
    | { kind: "search_pending", pending: boolean };

/**
 * 意图怎么改状态
 */
const apply_intent = (state: MainState, intent: Intent): MainState => {
    switch (intent) {
        case "select_prev":
            return { ...state, selection: step_selection(state, -1) };
        case "select_next":
            return { ...state, selection: step_selection(state, 1) };
        case "toggle_preview":
            // 空结果没有可预览的条目
            return { ...state, preview_open: state.item_list.length > 0 && !state.preview_open };
        case "run_action":
            // 触发动作只关掉 Preview：动作本身由 Rust 执行，窗口要不要隐藏也由 Rust 决定。
            return state.preview_open ? { ...state, preview_open: false } : state;
        case "dismiss":
            return state;
    }
}

/** 移动一行，边界 clamp 不环绕；空结果没有可指的条目，不动 */
const step_selection = (state: MainState, delta: number): number => {
    if (state.item_list.length === 0) return state.selection;
    return Math.min(Math.max(state.selection + delta, 0), state.item_list.length - 1);
}

/**
 * 唯一改状态的地方
 *
 * 纯函数，不碰副作用：`index_main.tsx` 挂着 StrictMode，
 * dev 下 reducer 会被调用两次，副作用放这里就是一次输入打两枪。
 */
export const reduce_main = (state: MainState, action: MainAction): MainState => {
    switch (action.kind) {
        case "typing":
            return { ...state, input: action.value };
        case "page_loaded":
            // 没进行搜索（输入为空）：结果整份清空，列表区换成提示输入
            if (action.page === undefined) return {
                ...state,
                item_list: [],
                total: 0,
                selection: 0,
                preview_open: false,
                searching: false,
            };
            // 新结果回到第一条；关闭 Preview ；搜索结果落地
            return {
                ...state,
                item_list: action.page.item_list,
                total: action.page.total,
                selection: 0,
                preview_open: false,
                searching: false,
            };
        case "page_appended":
            // 预请求是纯追加，不影响展示
            return { ...state, item_list: [...state.item_list, ...action.page.item_list] };
        case "intent":
            return apply_intent(state, action.intent);
        case "main_shown":
            // 唤出等于重新开始：强制关掉 Preview；输入内容与已加载的结果都留着
            return state.preview_open ? { ...state, preview_open: false } : state;
        case "search_pending":
            return { ...state, searching: action.pending };
    }
}
