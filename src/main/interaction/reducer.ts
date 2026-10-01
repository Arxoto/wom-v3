import type { PluginItemDisplay, PluginItemSearchPage } from "../../core";
import type { Intent } from "./keys";

/** 一份落定的结论：查询有了结果 */
export interface Conclusion {
    /** 已加载的结果，翻页时往后追加 */
    item_list: PluginItemDisplay[],
    /** 结果总数，用来判断还有没有下一页 */
    total: number,
}

/**
 * Plugin Search Page：与 [`Conclusion`] 并列的另一份列表（spec §4.2）
 *
 * 它不是主列表的一份延续：结果行是每次查询现算的（不注册进框架、不翻页），
 * 主列表的 conclusion 在这一页打开期间原样留着，返回时不需要重新检索。
 */
export interface PluginSearchState {
    /** 这一页是哪个插件的搜索：返回主列表后要回到那一行，靠它认人 */
    plugin_id: string,
    item_list: PluginItemDisplay[],
    selection: number,
    total: number,
    /** 与主列表的 `action_indices` 同一套记账方式（按 Item Index） */
    action_indices: Record<number, number>,
}

/** 主窗口的交互状态（渲染相关） */
export interface MainState {
    /** 输入框的值（受控） */
    input: string,
    /** 当前展示的结论；`null` = 还没有结论（没输入过，或输入为空，或答复还在路上） */
    conclusion: Conclusion | null,
    /** List Position：Selection 落在已加载列表的第几行，不是 Item Index */
    selection: number,
    /** 每个条目记住的当前动作（按 Item Index 记）；没记过的条目走默认动作 */
    action_indices: Record<number, number>,
    /** Preview 是否打开 */
    preview_open: boolean,
    /** 插件搜索页；`null` = 没打开（这时一切照旧走主列表） */
    plugin_search: PluginSearchState | null,
}

export const MAIN_STATE_INIT: MainState = {
    input: "",
    conclusion: null,
    selection: 0,
    action_indices: {},
    preview_open: false,
    plugin_search: null,
}

export const current_item = (item_list: PluginItemDisplay[] | null | undefined, selection: number) =>
    item_list?.[selection];

/** 没有结论时的空表：常量而不是每次新建，允许 `memo(Body)` 的 bailout */
const NO_ITEMS: PluginItemDisplay[] = [];

/** 当前生效的那份列表：插件搜索页打开时就是它，否则是主列表 */
export const active_items = (state: MainState): PluginItemDisplay[] =>
    state.plugin_search?.item_list ?? state.conclusion?.item_list ?? NO_ITEMS;

/** 当前生效的选中位置 */
export const active_selection = (state: MainState): number =>
    state.plugin_search?.selection ?? state.selection;

/** 当前生效的动作下标记账：两层列表各记各的，互不干扰 */
const active_action_indices = (state: MainState): Record<number, number> =>
    state.plugin_search?.action_indices ?? state.action_indices;

/** 条目记住的动作下标；没记过就是 0，也就是动作表第一个 */
export const action_index_of = (state: MainState, item_index: number): number =>
    active_action_indices(state)[item_index] ?? 0;

/**
 * 状态变化的来源
 */
export type MainAction =
    | { kind: "typing", value: string }
    /** 结论落定：`page` 为 `null` 表示这一轮没有查询（输入为空） */
    | { kind: "settled", page: PluginItemSearchPage | null }
    | { kind: "page_appended", page: PluginItemSearchPage }
    | { kind: "intent", intent: Intent }
    /** 记下某个条目切到了哪个动作：往哪一边还挪得动由接线层按动作表算 */
    | { kind: "action_selected", item_index: number, action_index: number }
    /** 插件搜索页拿到了结果：整份换上去 */
    | { kind: "plugin_search_settled", plugin_id: string, page: PluginItemSearchPage }
    /** 从插件搜索页回到主列表 */
    | { kind: "plugin_search_closed" }
    | { kind: "close_preview" };

/**
 * 意图怎么改状态
 *
 * 切换动作要先知道当前条目有几个动作，而那是动作表里的事：接线层把它折算成
 * `action_selected` 派发。进出插件搜索页要发请求，也由接线层先处理掉。
 * 剩下的这几个意图到这里只是覆盖全 `Intent`。
 */
const apply_intent = (state: MainState, intent: Intent): MainState => {
    switch (intent) {
        case "select_prev":
            return step_selection(state, -1);
        case "select_next":
            return step_selection(state, 1);
        case "action_prev":
        case "action_next":
        case "open_search":
        case "close_search":
            return state;
        case "toggle_preview":
            // 空结果没有可预览的条目；插件搜索页里不开预览（它的延伸内容就是这一页）
            return {
                ...state,
                preview_open: state.plugin_search === null
                    && active_items(state).length > 0
                    && !state.preview_open,
            };
        case "run_action":
            // 触发动作只关掉 Preview：动作本身由 Rust 执行，窗口要不要隐藏也由 Rust 决定。
            return state.preview_open ? { ...state, preview_open: false } : state;
        case "dismiss":
            return state;
    }
}

/** 移动一行，边界 clamp 不环绕；空结果没有可指的条目，不动 */
const step_selection = (state: MainState, delta: number): MainState => {
    const loaded = active_items(state).length;
    if (loaded === 0) return state;

    const selection = Math.min(Math.max(active_selection(state) + delta, 0), loaded - 1);

    return state.plugin_search === null
        ? { ...state, selection }
        : { ...state, plugin_search: { ...state.plugin_search, selection } };
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
        case "action_selected":
            // 改哪一层由当前是不是开着搜索页决定，接线层不必知道
            return state.plugin_search === null
                ? {
                    ...state,
                    action_indices: { ...state.action_indices, [action.item_index]: action.action_index },
                }
                : {
                    ...state,
                    plugin_search: {
                        ...state.plugin_search,
                        action_indices: {
                            ...state.plugin_search.action_indices,
                            [action.item_index]: action.action_index,
                        },
                    },
                };
        case "plugin_search_settled":
            // 选中位置每次都从头开始：结果页的选中位置不保留（Q24）
            return {
                ...state,
                // 进搜索页就把预览收掉：两者的位置都是右侧那一块，留着只会打架
                preview_open: false,
                plugin_search: {
                    plugin_id: action.plugin_id,
                    item_list: action.page.item_list,
                    selection: 0,
                    total: action.page.total,
                    action_indices: {},
                },
            };
        case "plugin_search_closed":
            // 主列表的 conclusion 一直留着，所以返回不需要重新检索（spec §4.3）
            return state.plugin_search === null ? state : { ...state, plugin_search: null };
        case "close_preview":
            return state.preview_open ? { ...state, preview_open: false } : state;
    }
}
