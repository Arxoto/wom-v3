import type { ItemDisplay, ItemSearchPage } from "../../core";
import type { Intent } from "./keys";

/** 一份落定的结论：查询有了结果 */
export interface Conclusion {
    /** 已加载的结果，翻页时往后追加 */
    item_list: ItemDisplay[],
    /** 结果总数，用来判断还有没有下一页 */
    total: number,
}

/** 主窗口的交互状态（渲染相关） */
export interface MainState {
    /** 输入框的值（受控） */
    input: string,
    /** 当前展示的结论；`null` = 还没有结论（没输入过，或输入为空，或答复还在路上） */
    conclusion: Conclusion | null,
    /** List Position：Selection 落在已加载列表的第几行，不是 Item Index */
    selection: number,
    /** Preview 是否打开 */
    preview_open: boolean,
}

export const MAIN_STATE_INIT: MainState = {
    input: "",
    conclusion: null,
    selection: 0,
    preview_open: false,
}

export const current_item = (item_list: ItemDisplay[] | null | undefined, selection: number) =>
    item_list?.[selection];

/**
 * 状态变化的来源
 */
export type MainAction =
    | { kind: "typing", value: string }
    /** 结论落定：`page` 为 `null` 表示这一轮没有查询（输入为空） */
    | { kind: "settled", page: ItemSearchPage | null }
    | { kind: "page_appended", page: ItemSearchPage }
    | { kind: "intent", intent: Intent }
    | { kind: "close_preview" };

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
            return {
                ...state,
                preview_open: (state.conclusion?.item_list.length ?? 0) > 0 && !state.preview_open,
            };
        case "run_action":
            // 触发动作只关掉 Preview：动作本身由 Rust 执行，窗口要不要隐藏也由 Rust 决定。
            return state.preview_open ? { ...state, preview_open: false } : state;
        case "dismiss":
            return state;
    }
}

/** 移动一行，边界 clamp 不环绕；空结果没有可指的条目，不动 */
const step_selection = (state: MainState, delta: number): number => {
    const loaded = state.conclusion?.item_list.length ?? 0;
    if (loaded === 0) return state.selection;
    return Math.min(Math.max(state.selection + delta, 0), loaded - 1);
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
        case "settled":
            // 结论整份换掉：`null` 是「这一轮没有查询」，别的就是这一次查询的答案
            return {
                ...state,
                conclusion: action.page === null ? null : {
                    item_list: action.page.item_list,
                    total: action.page.total,
                },
                selection: 0,
                preview_open: false,
            };
        case "page_appended":
            // 预请求是纯追加：结论还是那一份，只在后面接一页
            if (state.conclusion === null) return state;
            return {
                ...state,
                conclusion: {
                    ...state.conclusion,
                    item_list: [...state.conclusion.item_list, ...action.page.item_list],
                },
            };
        case "intent":
            return apply_intent(state, action.intent);
        case "close_preview":
            return state.preview_open ? { ...state, preview_open: false } : state;
    }
}
