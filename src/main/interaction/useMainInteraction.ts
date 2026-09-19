import { Dispatch, RefObject, useCallback, useEffect, useReducer, useRef, useState } from "react";

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
import { MAIN_STATE_INIT, MainAction, reduce_main } from "./reducer";
import { create_search_session } from "./search_session";

/** 按住 ↑/↓ 连发的限流间隔：首次按键立即响应，之后的连发最快 100ms 一次 */
const SELECT_REPEAT_MS = 100;

/** 还没读到配置时的兜底条数，真值来自 Config::main_item_n */
const FALLBACK_ITEM_N = 10;

/** 移动 Selection 的意图：连发限流只对它们生效 */
const is_select_intent = (intent: Intent) =>
    intent === "select_prev" || intent === "select_next";

/**
 * 主窗口显示时的复位
 * 
 * 输入与合成事件挂在同一个 input 上，所以 ref 由这里持有再传进去
 *
 * 触发点有两个：
 * - 每次显示窗口都会走一次（见 window_utils::emit_main_shown）；
 * - 启动即显示、窗口刚建出来这两种情况下事件早于前端就绪，所以挂载时也做一次。
 */
const useMainWindowFocus = (
    input_ref: RefObject<HTMLInputElement | null>,
    dispatch: Dispatch<MainAction>,
) => {
    /**
     * 每次显示都要做两件事：回到列表模式（关 `Preview`），输入框聚焦并全选。
     *
     * 全选让「直接重打」与「看到上次的结果」同时成立：输入内容与已加载的结果都保留。
     * 不主动 blur 输入框，免得打断可能正在进行的合成。
     */
    const reset = useCallback(() => {
        dispatch({ kind: "main_shown" });

        const input = input_ref.current;
        if (!input) return;
        input.focus();
    }, [input_ref, dispatch]);

    useEffect(() => {
        reset();

        let unlisten: (() => void) | undefined;
        let cancelled = false;

        void on_main_shown(reset).then(stop => {
            // 注册还没回来就卸载了，就地退订
            if (cancelled) stop();
            else unlisten = stop;
        });

        return () => {
            cancelled = true;
            unlisten?.();
        };
    }, [reset]);
};

/**
 * 主窗口交互的接线层
 *
 * 每次按键与每次输入都从这里进：状态变化交给 [`reduce_main`]，检索的时序（防抖、令牌、
 * 在飞页、合成锁）交给 [`create_search_session`]，窗口显示时的复位与聚焦交给
 * [`useMainWindowFocus`]，这里只剩事件注册、invoke 与「什么时候做」。
 */
export const useMainInteraction = () => {
    /** 主状态 */
    const [state, dispatch] = useReducer(reduce_main, MAIN_STATE_INIT);
    /** 主界面显示几行 item */
    const [item_n, set_item_n] = useState(FALLBACK_ITEM_N);
    /** 动作表：挂载时拉一次，拉到之前是空表（没有条目显示动作图标） */
    const [type_actions, set_type_actions] = useState<ItemTypeActions>(EMPTY_ITEM_TYPE_ACTIONS);

    const input_ref = useRef<HTMLInputElement>(null);
    /** 上一次移动 Selection 的时刻；初值 -Infinity 让首次按键立即响应 */
    const last_select_at = useRef(Number.NEGATIVE_INFINITY);

    // 检索会话不随每次渲染重建：时序状态都在它里面，重建就等于丢掉在飞的请求
    const [session] = useState(() => create_search_session({
        search: (key: string, token: number) => {
            search(key).then(page => {
                session.search_reply(token, page);
            })
        },
        search_page: (page_start: number, token: number) => {
            search_page(page_start).then(page => {
                session.prefetch_reply(token, page);
            })
        },
        on_conclusion: page => dispatch({ kind: "settled", page }),
        on_page_appended: page => dispatch({ kind: "page_appended", page }),
    }));

    /**
     * 静默续下一页
     *
     * 判据用的都是当前状态，由这里现算后交给检索会话；在飞的记账与令牌在会话里
     * （见 search_session.ts / spec §3 的「预请求」）。
     */
    const prefetch_next_page = useCallback(() => {
        session.prefetch({
            // 已加载条数就是下一页的起始下标：列表按顺序追加，中间没有空洞
            loaded_count: state.conclusion?.item_list.length ?? 0,
            selection: state.selection,
            total: state.conclusion?.total ?? 0,
        });
    }, [session, state.conclusion, state.selection]);

    const on_input_change = useCallback((value: string) => {
        dispatch({ kind: "typing", value });
        // 阶段怎么走（合成中间态不排检索、空输入当场落定、关键字怎么切出）全在会话里，
        // 这里只报「输入变了」（见 search_session.ts）
        session.input_changed(value);
    }, [session]);

    /**
     * 跑当前条目的默认动作
     *
     * 结果为空、或条目没有可执行动作时，整个流程都不发生：不关 Preview，也不派发。
     * 有动作时先关掉 Preview（Selection 与输入内容都不动），再把它交给 Rust——
     * 要不要隐藏窗口由 Rust 按 `main_window_mode` 决定（见 spec §3 / §4.3）。
     */
    const run_current_action = useCallback(() => {
        const item = state.conclusion?.item_list[state.selection];
        const action = item ? default_action(type_actions, item.the_type) : null;
        if (!item || !action) return;

        dispatch({ kind: "intent", intent: "run_action" });
        void run_item_action(item.item_index, action.id);
    }, [state.conclusion, state.selection, type_actions]);

    // 输入与合成（IME）事件都挂在真实 input 上：Head 只留受控的 value 与一个占位处理器，
    // 语义全在这一层（按键路径见下面那个 window 级入口）
    useEffect(() => {
        const input = input_ref.current;
        if (!input) return;

        const on_input = () => on_input_change(input.value);
        const on_composition_start = () => session.begin_composition();
        const on_composition_end = () => session.end_composition(input.value);
        // 输入框获得焦点就全选
        const on_focus = () => input.select();

        input.addEventListener("input", on_input);
        input.addEventListener("compositionstart", on_composition_start);
        input.addEventListener("compositionend", on_composition_end);
        input.addEventListener("focus", on_focus);
        return () => {
            input.removeEventListener("input", on_input);
            input.removeEventListener("compositionstart", on_composition_start);
            input.removeEventListener("compositionend", on_composition_end);
            input.removeEventListener("focus", on_focus);
        };
    }, [session, on_input_change]);

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

    // 获取配置
    useEffect(() => {
        // 可见条数就是配置里的 main_item_n，窗口高度也按它算好
        void get_config().then(config => set_item_n(config.main_item_n));
        // 动作表只在挂载时拉一次：配置重载会重建窗口，不需要热更新
        void get_item_type_actions().then(set_type_actions);
    }, []);

    // 卸载时丢掉还没到点的防抖
    useEffect(() => () => session.dispose(), [session]);

    useMainWindowFocus(input_ref, dispatch);

    return { state, item_n, type_actions, input_ref };
}
