import type { ItemSearchPage } from "../../core";

/** 打字到检索的尾防抖：停手这么久才发请求 */
const SEARCH_DEBOUNCE_MS = 50;

/** 请求令牌的模：远大于同时在飞的请求数，环形递增就够 */
const TOKEN_MOD = 256;

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
    total: number;
}

/** 检索会话要的外部能力注入 */
export interface SearchSessionDeps {
    /** 按关键字检索第一页 */
    search: (key: string) => Promise<ItemSearchPage>;
    /** 请求下一页，参数是起始位置 */
    search_page: (page_start: number) => Promise<ItemSearchPage>;
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
    /** 检索请求令牌；响应拿自己的令牌跟它比，不等就是过期响应；翻页请求也同样保证令牌一致 */
    let search_token = 0;
    /** 预请求页是否在飞 */
    let prefetch_in_flight = false;

    return {
        /** @param key 传入想要检索的关键字，空串是合法关键字；`null` 表示上一次没有请求 */
        is_same_key: (key: string | null) => key === last_sent,
        is_current_round: (current_token: number) => current_token === search_token,
        get_current_token: () => search_token,
        is_prefetch_in_flight: () => prefetch_in_flight,
        set_prefetch: (in_flight: boolean) => prefetch_in_flight = in_flight,
        /**
         * 关键字改变，步进令牌至下一轮
         * 
         * @param key 传入新的关键字，空串是合法关键字；`null` 表示上一次没有请求
         */
        next_round: (key: string | null) => {
            last_sent = key;
            search_token = (search_token + 1) % TOKEN_MOD;
            prefetch_in_flight = false;
        },
    }
}

/**
 * 建一次检索会话
 *
 * 内部通过令牌规避并发请求问题，修改数据等副作用通过外部注入。
 */
export const create_search_session = (deps: SearchSessionDeps): SearchSession => {
    const search_state = create_search_state();

    /** 发送检索 */
    const send = (key: string) => {
        search_state.next_round(key);

        const token = search_state.get_current_token();
        deps.search(key).then(page => {
            if (search_state.is_current_round(token)) {
                deps.on_conclusion(page);
            }
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
            search_state.next_round(null);
            deps.on_conclusion(null);
            return;
        }

        // 重复 key 不改变任何状态
        const key = search_key(value);
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
        // 下一页的起始下标恒等于已加载条数（见 prefetch_in_flight 的说明）
        const page_start = ctx.loaded_count;
        // 在余量内才进行获取
        if (ctx.selection < page_start - PREFETCH_MARGIN) return;
        // 已经在飞，不重复发
        if (search_state.is_prefetch_in_flight()) return;

        search_state.set_prefetch(true);
        const token = search_state.get_current_token();
        deps.search_page(page_start).then(page => {
            if (search_state.is_current_round(token)) {
                search_state.set_prefetch(false);
                deps.on_page_appended(page);
            }
        });
    };

    return { input_changed, begin_composition, end_composition, prefetch, dispose: debounce_cancel };
};
