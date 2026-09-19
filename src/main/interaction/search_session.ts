import type { ItemSearchPage } from "../../core";

/** 打字到检索的尾防抖：停手这么久才发请求 */
const SEARCH_DEBOUNCE_MS = 50;

/** 请求令牌的模：远大于同时在飞的请求数，环形递增就够 */
const TOKEN_MOD = 256;

/**
 * 预请求的余量：指针落到已加载列表的后 10 位就续下一页。写死 10，不跟随 main_item_n
 *
 * `PREFETCH_MARGIN` 同时是后端的时间预算：
 * 请求在指针站上倒数第 10 行时发出，往下再走 n 次就会使高亮行下移，具体见 `Body` 里的计算。
 * 按住 ↓ 时 `SELECT_REPEAT_MS` 限流到最快 100ms 一次（见 useMainInteraction），
 * 所以后端得在 `(10 - n) * 100ms` 内返回，否则高亮行提前下移，等预请求的结果返回会有跳变。
 */
const PREFETCH_MARGIN = 10;

/**
 * 从输入里切出检索关键字：第一个空格之前的内容
 *
 * 空格之后的部分不参与检索（留给条目参数），所以以空格开头时关键字就是空串——空串是合法
 * 关键字，后端按它给出全部条目。
 */
const search_key = (value: string) => value.split(" ")[0];

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
    search: (key: string) => Promise<ItemSearchPage>;
    /** 取结果集里第 `page_start` 条起的一页，参数是起始位置不是页码 */
    search_page: (page_start: number) => Promise<ItemSearchPage>;
    /** 新检索的第一页回来了；`undefined` 表示这一轮没有进行搜索（输入为空） */
    on_page_loaded: (page: ItemSearchPage | undefined) => void;
    /** 预请求续上的一页回来了 */
    on_page_appended: (page: ItemSearchPage) => void;
}

/**
 * 检索会话的对外形状
 *
 * 方法按「发生了什么」命名，不吐结果：两类页各自经注入的回调派发。判据要用的状态由调用方
 * 传入，会话不认识 React state；「该不该真发、什么时候发」全在会话里，接线层只报事件。
 */
export interface SearchSession {
    /**
     * 输入变了（DOM 的 input 事件，或合成结束时补的那次）：排一次 `SEARCH_DEBOUNCE_MS` 的
     * 尾防抖，到点交给内部的 `send` 决定搜不搜、搜什么（搜不搜与关键字都在那里判断）；
     * 合成期间调用整个丢弃（中间态不是最终文本，见 [`begin_composition`]）
     */
    input_changed(value: string): void;
    /** 合成会话开始：丢掉已经排上的防抖，此后 [`input_changed`] 一律丢弃 */
    begin_composition(): void;
    /** 合成会话结束：解锁，并把读到的当前输入 `value` 交给 [`input_changed`] 补一次 */
    end_composition(value: string): void;
    /** 丢掉还没到点的防抖；接线层卸载时调用 */
    dispose(): void;
    /** 判据成立且这一页不在飞时，续下一页 */
    prefetch(ctx: PrefetchContext): void;
}

/**
 * 建一次检索会话
 *
 * 非 React 模块：防抖定时器、请求令牌、在飞的预请求、合成锁、「这次输入该不该搜」（空输入
 * 不搜）、关键字怎么从输入里切出来（第一个空格之前，见 [`search_key`]）都在这里记账，
 * 后两件事只发生在 `send` 一处——防抖到点把原始输入交给它，搜不搜、搜什么由它定；
 * 接线层只把事件转过来（输入变了 / 合成开始或结束 / 卸载）。这样做的原因是这些不变量
 * 互相咬合——令牌一前进就要清掉在飞那一笔、响应又要拿令牌判过期、合成一开始已排上的防抖
 * 要作废——散进 hook 的各个闭包就会变成一份看不见的共享状态（见 spec §2 / §3）。
 * 依赖全部注入，所以这个文件不碰 React、不碰 invoke。
 *
 * 「关键字没变不重发」是这里的判据：`compositionend` 补的那次与提交之后那次 input
 * 谁先谁后，都收敛到同一个结果。
 */
export const create_search_session = (deps: SearchSessionDeps): SearchSession => {
    /** 上一次真正发出去的关键字；`null` 表示还没发过，空串是合法关键字 */
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
    /**
     * 合成会话开着没有：合成期间输入框照常更新，只是每次变更都不排检索。
     *
     * 和防抖 / 去重同族——回答的都是「这一刻的文本能不能搜」，所以判据放在这里，
     * 而不是让写入者（DOM 事件）与读出者（输入路径）各拿一半。
     */
    let composing = false;

    /** 丢掉还没到点的那一次防抖 */
    const clear_timer = () => {
        window.clearTimeout(debounce_timer);
        debounce_timer = undefined;
    };

    /**
     * 真正把这次输入落到检索上
     *
     * 不在对外形状里：接线层只有 [`input_changed`] 一个入口，搜不搜、搜什么、什么时候发
     * 由这里决定。空输入是「这一轮不搜」——结果回到空态（`on_page_loaded(undefined)`），
     * 同时在飞的那一笔作废、去重记录复位。
     */
    const send = (value: string) => {
        const current_token = (latest_token + 1) % TOKEN_MOD;
        latest_token = current_token;

        // 新检索作废在飞的预请求：它回来时令牌对不上，整包丢弃
        in_flight_page = undefined;

        // 空输入不搜：把输入清空后列表整份清掉，等有内容再检索。
        // 判据是原始输入而不是关键字——以空格开头时关键字是空串，那次照常搜
        if (value === "") {
            // 去重记录复位：清空后重打同一个关键字要能重新搜（列表此时是空的）
            last_sent = null;
            deps.on_page_loaded(undefined);
            return;
        }

        const key = search_key(value);
        // 关键字没变不重发：后端另有 input_key 缓存兜底，这里省掉一次往返
        if (key === last_sent) return;
        last_sent = key;

        deps.search(key)
            .then(page => {
                // 过期响应整包丢弃，只比相等不比大小
                if (latest_token !== current_token) return;
                deps.on_page_loaded(page);
            })
            // 检索失败不打断输入，列表保持上一次的样子
            .catch(() => { });
    };

    const input_changed = (value: string) => {
        // 合成中间态不是最终文本：这一次不排，等 `end_composition` 自己补
        if (composing) return;
        // 排上的那一次已经作废，不管这次要不要搜
        clear_timer();
        debounce_timer = window.setTimeout(() => send(value), SEARCH_DEBOUNCE_MS);
    };

    const begin_composition = () => {
        composing = true;
        // 已经排上的那一次带的是合成中间态：作废
        clear_timer();
    };

    const end_composition = (value: string) => {
        composing = false;
        // 补这一次也走防抖：连续提交候选（每次都是一份新输入）只有停手后那一次真的检索，
        // 直发会让后端整集重扫多次
        input_changed(value);
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

    return { input_changed, begin_composition, end_composition, dispose: clear_timer, prefetch };
};
