import type { ItemDisplay, ItemSearchPage } from "../../core";

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
 * 只有三类：打字、新结果回来、按键意图。意图在 02 号票据接入。
 */
export type MainAction =
    | { kind: "typing", value: string }
    | { kind: "page_loaded", page: ItemSearchPage };

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
            // 新结果回到第一条
            return {
                ...state,
                item_list: action.page.item_list,
                total: action.page.total,
                selection: 0,
            };
    }
}
