import type { ItemSearchPage } from "../../core";

/** 打字到检索的尾防抖：停手这么久才发请求 */
const SEARCH_DEBOUNCE_MS = 50;

/**
 * 预请求的余量：指针落到已加载列表的后 10 位就续下一页。写死 10，不跟随 main_item_n
 *
 * `PREFETCH_MARGIN` 同时是后端的时间预算：
 * - 假设窗口设定余量为 n （具体见 `Body` 里的计算），即选中行在窗口的倒数第 n 行时向下时，优先尝试移动整个窗口，窗口到底后才移动选中行；
 * - 请求会在选中行上到达最后 10 行时发出，而连续按住下时 `SELECT_REPEAT_MS` 限流到最快 100ms 一次（见 `useMainInteraction` ）；
 * - 所以后端需要在 `(10 - n) * 100ms` 内返回，否则高亮行提前下移，等预请求的结果返回会有跳变。
 */
const PREFETCH_MARGIN = 10;

/**
 * 从输入里切出检索关键字：第一个空格前的内容视为关键字，之后视为参数
 *
 * 以空格开头时关键字就是空串，后端按它给出全部条目。
 */
const search_key = (value: string) => value.split(" ")[0];

/** 预请求判据要用的状态 */
export interface PrefetchContext {
    /** 已加载条数；下一页的起始下标就是它（列表按顺序追加，中间没有空洞） */
    loaded_count: number;
    /** 当前选中项的索引 */
    selection: number;
    /** 结果总数；用来判断还有没有下一页 */
    total: number;
}

/** 检索会话要的外部能力注入 */
export interface SearchSessionDeps {
    /** 按关键字检索第一页 */
    search: (key: string) => Promise<ItemSearchPage>;
    /** 请求下一页：起始位置 + 当前令牌 */
    search_page: (page_start: number, token: number) => Promise<ItemSearchPage>;
    /** 检索结果落定：`null` 表示没有查询（输入为空） */
    on_conclusion: (page: ItemSearchPage | null) => void;
    /** 预请求续上的一页回来了 */
    on_page_appended: (page: ItemSearchPage) => void;
}

/**
 * 检索会话的对外形状
 *
 * 方法按「事件」和「动作」命名。
 */
export interface SearchSession {
    /** 输入改变，内部控制防抖和检索请求 */
    input_changed(value: string): void;
    /** 合成会话开始：丢掉防抖请求；并且锁住输入检索 */
    begin_composition(): void;
    /** 合成会话结束：解锁，并立即进行检索请求 */
    end_composition(value: string): void;

    /** 预请求下一页 */
    prefetch(ctx: PrefetchContext): void;

    /** 丢掉还没到点的防抖；接线层卸载时调用 */
    dispose(): void;
}

const create_search_state = () => {
    /** 上一次请求的关键字；`null` 表示上一次没有请求 */
    let last_sent: string | null = null;
    /** 当前结论的令牌，由后端随检索结果下发；`null` = 没搜索 */
    let settled_token: number | null = null;
    /**
     * 预请求记账：结论页的起始索引 → 请求结果是否到达
     *
     * 索引不在映射里 = 这一页没请求过；`false` = 请求已发出、结果还没到；`true` = 结果已到。
     */
    const prefetch_arrived = new Map<number, boolean>();

    return {
        /** 
         * 关键字是「过期响应」的判据，答复到达时必须与当前关键字相同。
         * 
         * @param key 传入想要检索的关键字，空串是合法关键字；`null` 表示上一次没有请求
         */
        is_same_key: (key: string | null) => key === last_sent,
        /**
         * 记下这一次发出的关键字，并清掉预请求记账
         *
         * @param key 传入新的关键字，空串是合法关键字；`null` 表示这一次没有请求
         */
        mark_sent: (key: string | null) => {
            last_sent = key;
            prefetch_arrived.clear();
        },

        /** 当前结论的令牌（`null` = 没搜索） */
        get_settled_token: () => settled_token,
        /** 结论换了一份（或清空）时记下它的令牌 */
        set_settled_token: (token: number | null) => settled_token = token,
        /** 这一页还是不是屏上那份结论的；令牌由后端生成，前端只回传，不自己造 */
        is_current_token: (token: number) => token === settled_token,

        /** 这一页请求过了没有：在飞与已到达都算，判据只有「索引在不在映射里」 */
        is_prefetch_requested: (page_start: number) => prefetch_arrived.has(page_start),
        /** 记下这一页的请求已发出，结果还没到 */
        mark_prefetch_sent: (page_start: number) => { prefetch_arrived.set(page_start, false); },
        /** 记下这一页的结果已到达 */
        mark_prefetch_arrived: (page_start: number) => { prefetch_arrived.set(page_start, true); },
        /** 撤销这一页的记账，让它能被重新请求 */
        release_prefetch: (page_start: number) => { prefetch_arrived.delete(page_start); },

    }
}

/**
 * 建一次检索会话
 *
 * 通过令牌保证一致性，修改数据等副作用通过外部注入。
 */
export const create_search_session = (deps: SearchSessionDeps): SearchSession => {
    const search_state = create_search_state();

    /** 发送检索 */
    const send = (key: string) => {
        // 关键字没变，不重复发送
        if (search_state.is_same_key(key)) return;

        search_state.mark_sent(key);

        deps.search(key).then(page => {
            // 输入修改，丢弃
            if (!search_state.is_same_key(key)) return;
            search_state.set_settled_token(page.token);
            deps.on_conclusion(page);
        });
    };

    /** 防抖检索句柄 */
    let debounce_handle: number | undefined = undefined;

    /** 取消防抖 */
    const debounce_cancel = () => {
        window.clearTimeout(debounce_handle);
        debounce_handle = undefined;
    }

    /** 防抖检索 */
    const debounce_search = (key: string) => {
        window.clearTimeout(debounce_handle);
        debounce_handle = window.setTimeout(() => {
            debounce_handle = undefined;
            send(key);
        }, SEARCH_DEBOUNCE_MS);
    }

    /** 合成期间不请求检索 */
    let composing = false;

    const input_changed = (value: string) => {
        // 合成中间态，不做任何事
        if (composing) return;

        // 空输入，不检索
        if (value === "") {
            // 上一次也是不检索，不变
            if (search_state.is_same_key(null)) return;
            
            debounce_cancel();
            search_state.mark_sent(null);
            search_state.set_settled_token(null);
            deps.on_conclusion(null);
            return;
        }

        const key = search_key(value);
        // 输入变了但是关键字没变，可能正在输入参数，不做任何事
        if (search_state.is_same_key(key)) return;

        debounce_search(key);
    };

    const begin_composition = () => {
        composing = true;
        debounce_cancel();
    };

    const end_composition = (value: string) => {
        composing = false;
        // 也触发一次，不同平台事件触发顺序不一样
        input_changed(value);
    };

    /** 静默续下一页 */
    const prefetch = (ctx: PrefetchContext) => {
        // 下一页的起始下标
        const page_start = ctx.loaded_count;
        // 结果已经全部加载（或还没有结论，`total` 是 0）：没有下一页可续
        if (page_start >= ctx.total) return;
        // 替屏上那份结论续页：它的令牌要跟着请求一起走，Rust 拿它比对缓存
        const settled_token = search_state.get_settled_token();
        if (settled_token === null) return;
        // 在余量内才进行获取
        if (ctx.selection < page_start - PREFETCH_MARGIN) return;
        // 这一页请求过（在飞或已到），不重复发
        if (search_state.is_prefetch_requested(page_start)) return;

        search_state.mark_prefetch_sent(page_start);
        deps.search_page(page_start, settled_token).then(
            page => {
                // 已过时，丢弃
                if (!search_state.is_current_token(settled_token)) return;
                search_state.mark_prefetch_arrived(page_start);
                deps.on_page_appended(page);
            },
            // 失败捕获，允许手动重试
            err => {
                console.error("search_page failed:", err);
                if (search_state.is_current_token(settled_token)) search_state.release_prefetch(page_start);
            },
        );
    };

    return { input_changed, begin_composition, end_composition, prefetch, dispose: debounce_cancel };
};
