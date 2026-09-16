import { useCallback, useEffect, useReducer, useRef, useState } from "react";

import { get_config, search } from "../../core";
import { MAIN_STATE_INIT, reduce_main } from "./reducer";

/** 打字到检索的尾防抖：停手这么久才发请求 */
const SEARCH_DEBOUNCE_MS = 50;

/** 请求令牌的模：远大于同时在飞的请求数，环形递增就够 */
const TOKEN_MOD = 256;

/** 还没读到配置时的兜底条数，真值来自 Config::main_item_n */
const FALLBACK_ITEM_N = 10;

/**
 * 主窗口交互的接线层
 *
 * 唯一碰副作用的地方：防抖、请求令牌、invoke。每次按键与每次输入都从这里进，
 * 状态变化交给 [`reduce_main`]，所以这里只管「什么时候做」。
 */
export const useMainInteraction = () => {
    const [state, dispatch] = useReducer(reduce_main, MAIN_STATE_INIT);
    const [item_n, set_item_n] = useState(FALLBACK_ITEM_N);

    const input_ref = useRef<HTMLInputElement>(null);
    const debounce_timer = useRef<number | undefined>(undefined);
    /** 合成会话开着没有：合成期间输入框照常更新，只是不检索 */
    const composing = useRef(false);
    /** 上一次真正发出去的文本；`null` 表示还没发过，空串是合法输入 */
    const last_sent = useRef<string | null>(null);
    /** 最新的请求令牌；响应拿自己的令牌跟它比，不等就是过期响应 */
    const latest_token = useRef(0);

    const send_search = useCallback((k: string) => {
        // 文本没变不重发：后端另有 input_key 缓存兜底，这里省掉一次往返
        if (k === last_sent.current) return;
        last_sent.current = k;

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

    const on_input_change = useCallback((value: string) => {
        dispatch({ kind: "typing", value });

        // 合成中间态不是最终文本：这一次不发，等 compositionend 自己补
        if (composing.current) return;

        window.clearTimeout(debounce_timer.current);
        debounce_timer.current = window.setTimeout(() => send_search(value), SEARCH_DEBOUNCE_MS);
    }, [send_search]);

    // 合成事件挂在真实 input 上（按键路径见 02 号票据）
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

    useEffect(() => {
        // 首屏：没有输入时后端按空关键字给出全部条目
        send_search("");
        // 可见条数就是配置里的 main_item_n，窗口高度也按它算好
        void get_config().then(config => set_item_n(config.main_item_n));
    }, [send_search]);

    // 卸载时丢掉还没到点的防抖
    useEffect(() => () => window.clearTimeout(debounce_timer.current), []);

    return { state, item_n, input_ref, on_input_change };
}
