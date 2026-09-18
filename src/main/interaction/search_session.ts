import type { ItemSearchPage } from "../../core";

/** 打字到检索的尾防抖：停手这么久才发请求 */
const SEARCH_DEBOUNCE_MS = 50;

/** 请求令牌的模：远大于同时在飞的请求数，环形递增就够 */
const TOKEN_MOD = 256;

/** 预请求的余量：指针落到已加载列表的后 10 位就续下一页。写死 10，不跟随 main_item_n */
const PREFETCH_MARGIN = 10;

/** 预请求判据要用的状态：已加载条数、当前 `Selection`、结果总数 */
export interface PrefetchContext {
    /** 已加载条数；下一页的起始下标就是它（列表按顺序追加，中间没有空洞） */
    loaded_count: number;
    selection: number;
    total: number;
}

/** 检索会话要的外部能力：两个 invoke 与两个结果派发，全部注入 */
export interface SearchSessionDeps {
    /** 按关键字检索第一页 */
    search: (k: string) => Promise<ItemSearchPage>;
    /** 取结果集里第 `page_start` 条起的一页，参数是起始位置不是页码 */
    search_page: (page_start: number) => Promise<ItemSearchPage>;
    /** 新检索的第一页回来了 */
    on_page_loaded: (page: ItemSearchPage) => void;
    /** 预请求续上的一页回来了 */
    on_page_appended: (page: ItemSearchPage) => void;
}

/**
 * 检索会话的对外形状
 *
 * 方法是「做什么」，不吐结果：两类页各自经注入的回调派发。判据要用的状态由调用方传入，
 * 会话不认识 React state。
 */
export interface SearchSession {
    /** 立即检索关键字 `k`：文本没变不发，新检索作废在飞的预请求 */
    send(k: string): void;
    /** 尾防抖：停手 `SEARCH_DEBOUNCE_MS` 之后才走 [`send`] */
    schedule(k: string): void;
    /** 丢掉还没到点的防抖（合成开始、卸载） */
    cancel(): void;
    /** 判据成立且这一页不在飞时，续下一页 */
    prefetch(ctx: PrefetchContext): void;
}

/**
 * 建一次检索会话
 *
 * 非 React 模块：防抖定时器、请求令牌、在飞的预请求都在这里记账，接线层只回答
 * 「什么时候调哪个方法」。这样做的原因是这些不变量互相咬合——令牌一前进就要清掉在飞那一笔、
 * 响应又要拿令牌判过期——散进 hook 的各个闭包就会变成一份看不见的共享状态
 * （见 spec §2 / §3）。依赖全部注入，所以这个文件不碰 React、不碰 invoke。
 *
 * 「文本没变不重发」是这里的判据：`compositionend` 补的那次与提交之后那次 input
 * 谁先谁后，都收敛到同一个结果。
 */
export const create_search_session = (deps: SearchSessionDeps): SearchSession => {
    /** 上一次真正发出去的文本；`null` 表示还没发过，空串是合法输入 */
    let last_sent: string | null = null;
    /** 最新的请求令牌；响应拿自己的令牌跟它比，不等就是过期响应 */
    let latest_token = 0;
    /**
     * 在飞的那一页；没有在飞的就是 `undefined`
     *
     * 最多只有一页在飞，所以不需要按页下标记账：
     * 1. 请求的起始下标恒等于已加载条数——追加是列表唯一的增长方式，中间没有空洞；
     * 2. 已加载条数只有两种走法：预请求落账（先销账、后派发），或新检索换掉整份结果
     *    （换的那一步已经清掉在飞这一笔）；
     * 3. `Selection` 被 clamp 在已加载范围内，光标撞不出第二个起始下标。
     * 三条合起来：下一个起始下标只在上一页落账之后才存在，所以同一时间只可能有一笔。
     * 真要让两个起始下标同时在飞（例如允许 `Selection` 越过已加载区、边滚边取），
     * 这里得改回按页下标记账。
     *
     * `token` 不是装饰：新检索会清掉这一笔，旧响应回来时不能把新检索的账销掉。
     */
    let in_flight_page: { page_start: number, token: number } | undefined = undefined;
    /** 还没到点的防抖定时器 */
    let debounce_timer: number | undefined = undefined;

    const cancel = () => {
        window.clearTimeout(debounce_timer);
        debounce_timer = undefined;
    };

    const send = (k: string) => {
        // 文本没变不重发：后端另有 input_key 缓存兜底，这里省掉一次往返
        if (k === last_sent) return;
        last_sent = k;

        // 新检索作废在飞的预请求：它回来时令牌对不上，整包丢弃
        in_flight_page = undefined;

        const current_token = (latest_token + 1) % TOKEN_MOD;
        latest_token = current_token;

        deps.search(k)
            .then(page => {
                // 过期响应整包丢弃，只比相等不比大小
                if (latest_token !== current_token) return;
                deps.on_page_loaded(page);
            })
            // 检索失败不打断输入，列表保持上一次的样子
            .catch(() => { });
    };

    const schedule = (k: string) => {
        cancel();
        debounce_timer = window.setTimeout(() => send(k), SEARCH_DEBOUNCE_MS);
    };

    /**
     * 静默续下一页
     *
     * 判据是「这一步 ↓ 之后指针落在已加载列表的后 `PREFETCH_MARGIN` 位」且「还有没加载到的
     * 结果」——取的是区间而不是某一行，所以失败之后不用额外重试：指针还在区间里，下一次 ↓
     * 会再发一次（见 spec §3 的「预请求」）。
     */
    const prefetch = (ctx: PrefetchContext) => {
        // 下一页的起始下标恒等于已加载条数（见 in_flight_page 的说明）
        const page_start = ctx.loaded_count;
        if (page_start >= ctx.total) return;
        // 后 10 位是「已加载条数 - 10」往右。 直接用 selection 判断即可，差别不大
        if (ctx.selection < page_start - PREFETCH_MARGIN) return;
        // 这一页已经在飞就不重复发；最多一页在飞，一个槽就够
        if (in_flight_page?.page_start === page_start) return;

        const token = latest_token;
        in_flight_page = { page_start, token };

        // 销账只认自己那一笔（起始下标 + 令牌）：新检索可能已经把这笔清掉，
        // 旧响应回来时不能动新检索的账
        const release = () => {
            if (in_flight_page === undefined) return;
            if (in_flight_page.page_start !== page_start || in_flight_page.token !== token) return;
            in_flight_page = undefined;
        };

        deps.search_page(page_start)
            .then(page => {
                release();
                // 过期响应整包丢弃：新检索已经来了，这一页不再属于当前结果集
                if (latest_token !== token) return;
                deps.on_page_appended(page);
            })
            // 失败静默：列表保持原样，指针还在区间里时下一次 ↓ 会再试
            .catch(() => release());
    };

    return { send, schedule, cancel, prefetch };
};
