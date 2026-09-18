import type { ItemDisplay, ItemSearchPage } from "../../core";
import type { Intent } from "./keys";

/**
 * 主窗口的交互状态
 *
 * 渲染用得上的都在这里；请求时序（令牌、在飞的请求、防抖定时器）留在 hook 内部，不占状态。
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
}

export const MAIN_STATE_INIT: MainState = {
    input: "",
    item_list: [],
    total: 0,
    selection: 0,
    preview_open: false,
}

/**
 * 状态变化的来源
 *
 * 只有五类：打字、新结果回来（`undefined` 表示这一轮没进行搜索）、下一页回来、按键意图、
 * 主窗口被显示。
 */
export type MainAction =
    | { kind: "typing", value: string }
    | { kind: "page_loaded", page: ItemSearchPage | undefined }
    | { kind: "page_appended", page: ItemSearchPage }
    | { kind: "intent", intent: Intent }
    | { kind: "main_shown" };

/**
 * 意图怎么改状态
 *
 * 移动到哪、预览开不开都指着某个条目，所以空结果（列表长度为 0）一律退化成无操作
 * ——判据是列表长度，不是 `selection` 的取值。
 * 跑动作只关掉 Preview：动作本身由 Rust 执行，窗口要不要隐藏也由 Rust 决定
 * （见 spec §3 / §4.3 / §4.4）。
 */
const apply_intent = (state: MainState, intent: Intent): MainState => {
    switch (intent) {
        case "select_prev":
            return { ...state, selection: step_selection(state, -1) };
        case "select_next":
            return { ...state, selection: step_selection(state, 1) };
        case "toggle_preview":
            // 空结果没有可预览的条目：开了也只会让 Tail 的提示与画面各说各话
            return { ...state, preview_open: state.item_list.length > 0 && !state.preview_open };
        case "run_action":
            // 触发动作前先关掉 Preview；Selection 与输入内容都不动。
            // 条目没有可执行动作时整个流程都不发生——那道判断在接线层，走不到这里
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
            };
            // 新结果回到第一条；空结果没有可预览的条目，Preview 自动关闭
            return {
                ...state,
                item_list: action.page.item_list,
                total: action.page.total,
                selection: 0,
                preview_open: action.page.item_list.length > 0 && state.preview_open,
            };
        case "page_appended":
            // 预请求是纯追加：Selection 与滚动位置（Selection 的派生值）都不因为
            // 这一页的成败动一下，只剩列表变长
            return { ...state, item_list: [...state.item_list, ...action.page.item_list] };
        case "intent":
            return apply_intent(state, action.intent);
        case "main_shown":
            // 唤出等于重新开始：强制关掉 Preview；输入内容与已加载的结果都留着
            return state.preview_open ? { ...state, preview_open: false } : state;
    }
}
