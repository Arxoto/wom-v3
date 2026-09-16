import { useCallback, useEffect, useReducer, useRef, useState } from "react";

import {
    EMPTY_ITEM_TYPE_ACTIONS,
    dismiss_main_window,
    get_config,
    get_item_type_actions,
    on_main_shown,
    run_item_action,
    search,
    search_page,
    type ItemTypeActions,
} from "../../core";
import { default_action } from "./action_labels";
import { resolve_key, type Intent } from "./keys";
import { MAIN_STATE_INIT, reduce_main } from "./reducer";

/** 打字到检索的尾防抖：停手这么久才发请求 */
const SEARCH_DEBOUNCE_MS = 50;

/** 按住 ↑/↓ 连发的限流间隔：首次按键立即响应，之后的连发最快 100ms 一次 */
const SELECT_REPEAT_MS = 100;

/** 预请求的余量：指针落到已加载列表的后 10 位就续下一页。写死 10，不跟随 main_item_n */
const PREFETCH_MARGIN = 10;

/** 请求令牌的模：远大于同时在飞的请求数，环形递增就够 */
const TOKEN_MOD = 256;

/** 还没读到配置时的兜底条数，真值来自 Config::main_item_n */
const FALLBACK_ITEM_N = 10;

/** 移动 Selection 的意图：连发限流只对它们生效 */
const is_select_intent = (intent: Intent) =>
    intent === "select_prev" || intent === "select_next";

/**
 * 主窗口交互的接线层
 *
 * 唯一碰副作用的地方：防抖、请求令牌、invoke。每次按键与每次输入都从这里进，
 * 状态变化交给 [`reduce_main`]，所以这里只管「什么时候做」。
 */
export const useMainInteraction = () => {
    const [state, dispatch] = useReducer(reduce_main, MAIN_STATE_INIT);
    const [item_n, set_item_n] = useState(FALLBACK_ITEM_N);
    /** 动作表：挂载时拉一次，拉到之前是空表（没有条目显示动作图标） */
    const [type_actions, set_type_actions] = useState<ItemTypeActions>(EMPTY_ITEM_TYPE_ACTIONS);

    const input_ref = useRef<HTMLInputElement>(null);
    const debounce_timer = useRef<number | undefined>(undefined);
    /** 上一次移动 Selection 的时刻；初值 -Infinity 让首次按键立即响应 */
    const last_select_at = useRef(Number.NEGATIVE_INFINITY);
    /** 合成会话开着没有：合成期间输入框照常更新，只是不检索 */
    const composing = useRef(false);
    /** 上一次真正发出去的文本；`null` 表示还没发过，空串是合法输入 */
    const last_sent = useRef<string | null>(null);
    /** 最新的请求令牌；响应拿自己的令牌跟它比，不等就是过期响应 */
    const latest_token = useRef(0);
    /** 在飞的预请求：页的起始下标 → 它属于哪一次检索的令牌；新检索到来时整表作废 */
    const in_flight_pages = useRef(new Map<number, number>());

    const send_search = useCallback((k: string) => {
        // 文本没变不重发：后端另有 input_key 缓存兜底，这里省掉一次往返
        if (k === last_sent.current) return;
        last_sent.current = k;

        // 新检索作废在飞的预请求：它们回来时令牌对不上，整包丢弃
        in_flight_pages.current.clear();

        const token = (latest_token.current + 1) % TOKEN_MOD;
        latest_token.current = token;

        search(k)
            .then(page => {
                // 过期响应整包丢弃，只比相等不比大小
                if (latest_token.current !== token) return;
                dispatch({ kind: "page_loaded", page });
            })
            // 检索失败不打断输入，列表保持上一次的样子
            .catch(() => { });
    }, []);

    /**
     * 静默续下一页
     *
     * 判据是「这一步 ↓ 之后指针落在已加载列表的后 `PREFETCH_MARGIN` 位」且「还有没加载到的
     * 结果」——取的是区间而不是某一行，所以失败之后不用额外重试：指针还在区间里，下一次 ↓
     * 会再发一次（见 spec §3 的「预请求」）。
     */
    const prefetch_next_page = useCallback(() => {
        const index = state.item_list.length;
        if (index >= state.total) return;
        // 后 10 位是「已加载条数 - 10」往右。这里用 selection + 1 当这一步 ↓ 之后的指针：
        // 只有撞到底时两者才不一样，而那时候本来就没有更多结果
        if (state.selection + 1 < index - PREFETCH_MARGIN) return;
        // 同一页已经在飞就不重复发
        if (in_flight_pages.current.has(index)) return;

        const token = latest_token.current;
        in_flight_pages.current.set(index, token);

        // 收摊时只删自己那一笔：这一页可能已经被新检索清掉、或被另一次请求顶替
        const settle = () => {
            if (in_flight_pages.current.get(index) === token) in_flight_pages.current.delete(index);
        };

        search_page(index)
            .then(page => {
                settle();
                // 过期响应整包丢弃：新检索已经来了，这一页不再属于当前结果集
                if (latest_token.current !== token) return;
                dispatch({ kind: "page_appended", page });
            })
            // 失败静默：列表保持原样，指针还在区间里时下一次 ↓ 会再试
            .catch(() => settle());
    }, [state.item_list.length, state.selection, state.total]);

    const on_input_change = useCallback((value: string) => {
        dispatch({ kind: "typing", value });

        // 合成中间态不是最终文本：这一次不发，等 compositionend 自己补
        if (composing.current) return;

        window.clearTimeout(debounce_timer.current);
        debounce_timer.current = window.setTimeout(() => send_search(value), SEARCH_DEBOUNCE_MS);
    }, [send_search]);

    /**
     * 跑当前条目的默认动作
     *
     * 结果为空、或条目没有可执行动作时，整个流程都不发生：不关 Preview，也不派发。
     * 有动作时先关掉 Preview（Selection 与输入内容都不动），再把它交给 Rust——
     * 要不要隐藏窗口由 Rust 按 `main_window_mode` 决定（见 spec §3 / §4.3）。
     */
    const run_current_action = useCallback(() => {
        const item = state.item_list[state.selection];
        const action = item ? default_action(type_actions, item.the_type) : null;
        if (!item || !action) return;

        dispatch({ kind: "intent", intent: "run_action" });
        void run_item_action(item.item_index, action.id);
    }, [state.item_list, state.selection, type_actions]);

    /**
     * 主窗口显示时要做的两件事：回到列表模式，输入框聚焦并全选
     *
     * 全选让「直接重打」与「看到上次的结果」同时成立：输入内容与已加载的结果都保留。
     * 不主动 blur 输入框，免得打断可能正在进行的合成。
     */
    const on_main_shown_reset = useCallback(() => {
        dispatch({ kind: "main_shown" });

        const input = input_ref.current;
        if (!input) return;
        input.focus();
        input.select();
    }, []);

    // 每次显示窗口都会走一次（见 window_utils::emit_main_shown）；
    // 启动即显示、窗口刚建出来这两种情况下事件早于前端就绪，所以挂载时也做一次。
    useEffect(() => {
        on_main_shown_reset();

        let unlisten: (() => void) | undefined;
        let cancelled = false;

        void on_main_shown(on_main_shown_reset).then(stop => {
            // 注册还没回来就卸载了，就地退订
            if (cancelled) stop();
            else unlisten = stop;
        });

        return () => {
            cancelled = true;
            unlisten?.();
        };
    }, [on_main_shown_reset]);

    // 合成事件挂在真实 input 上（按键路径见下面那个 window 级入口）
    useEffect(() => {
        const input = input_ref.current;
        if (!input) return;

        const on_composition_start = () => {
            composing.current = true;
            window.clearTimeout(debounce_timer.current);
        };
        const on_composition_end = () => {
            composing.current = false;
            // 读输入框当前值补一次；与提交之后那次 input 谁先谁后，
            // 都靠「文本没变不重发」收敛到同一个结果
            send_search(input.value);
        };

        input.addEventListener("compositionstart", on_composition_start);
        input.addEventListener("compositionend", on_composition_end);
        return () => {
            input.removeEventListener("compositionstart", on_composition_start);
            input.removeEventListener("compositionend", on_composition_end);
        };
    }, [send_search]);

    // 按键只有一个入口：window 捕获阶段的 keydown。挂在真实 input 上会漏掉
    // 「预览打开」「鼠标点过 body 之后焦点不在 input」这些情形（见 ADR-0006）。
    //
    // 已知风险：合成（IME 候选框打开）期间按键会不会到达这里还没实测（见 spec §6），
    // 真到了页面，几下 ↑/↓ 会移动 Selection、Enter 会跑动作、ESC 会关预览或隐藏窗口，
    // 所以这里暂时不做合成判断。
    useEffect(() => {
        const on_key_down = (event: KeyboardEvent) => {
            const intent = resolve_key(
                event.key,
                { shift: event.shiftKey },
                { preview_open: state.preview_open },
            );
            if (intent === null) return;

            // 只有移动选中的键保留浏览器默认行为（单行输入框里光标顶到首尾），
            // Enter / Shift+Enter / ESC 一律拦下来（见 spec §1）
            if (!is_select_intent(intent)) event.preventDefault();

            // 退场交给 Rust：它无条件隐藏主窗口，不看 main_window_mode
            if (intent === "dismiss") {
                void dismiss_main_window();
                return;
            }

            // 跑动作也交给 Rust，只是先由接线层确认这一次真的有事可做
            if (intent === "run_action") {
                run_current_action();
                return;
            }

            // 按住连发的限流：首次按键立即响应，其余最快 100ms 一次。
            // 只有移动 Selection 的意图限流，其他按键各管各的。
            if (is_select_intent(intent)) {
                const now = performance.now();
                if (now - last_select_at.current < SELECT_REPEAT_MS) return;
                last_select_at.current = now;

                // 朝后走才看一眼要不要续下一页；预请求是纯追加，不挡这次移动
                if (intent === "select_next") prefetch_next_page();
            }

            dispatch({ kind: "intent", intent });
        };

        window.addEventListener("keydown", on_key_down, true);
        return () => window.removeEventListener("keydown", on_key_down, true);
    }, [state.preview_open, run_current_action, prefetch_next_page]);

    useEffect(() => {
        // 首屏：没有输入时后端按空关键字给出全部条目
        send_search("");
        // 可见条数就是配置里的 main_item_n，窗口高度也按它算好
        void get_config().then(config => set_item_n(config.main_item_n));
        // 动作表只在挂载时拉一次：配置重载会重建窗口，不需要热更新
        void get_item_type_actions().then(set_type_actions);
    }, [send_search]);

    // 卸载时丢掉还没到点的防抖
    useEffect(() => () => window.clearTimeout(debounce_timer.current), []);

    return { state, item_n, type_actions, input_ref, on_input_change };
}
