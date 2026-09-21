import { Dispatch, RefObject, useCallback, useEffect, useEffectEvent, useReducer, useRef, useState } from "react";

import { debug, info } from "@tauri-apps/plugin-log";

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
import { actions_of, current_action } from "./action_labels";
import { is_action_intent, is_select_intent, resolve_key, type Intent } from "./keys";
import { MAIN_STATE_INIT, MainAction, action_index_of, current_item, reduce_main } from "./reducer";
import { create_search_session } from "./search_session";
import { SELECT_REPEAT_MS } from "./timing";
import { resolve_wheel } from "./wheel";

/** 还没读到配置时的兜底条数，真值来自 Config::main_item_n */
const FALLBACK_ITEM_N = 10;

/**
 * 主窗口显示时的复位（关预览页面、输入框聚焦并全选）
 * 
 * 输入与合成事件挂在同一个 input 上，所以 ref 由这里持有再传进去
 *
 * 触发点有两个：
 * - 每次显示窗口都会走一次（见 `window_utils::emit_main_shown`）；
 * - 启动即显示、窗口刚建出来这两种情况下事件早于前端就绪，所以挂载时也做一次。
 */
const useMainWindowFocus = (
    input_ref: RefObject<HTMLInputElement | null>,
    dispatch: Dispatch<MainAction>,
) => {
    const reset = useCallback(() => {
        dispatch({ kind: "close_preview" });

        const input = input_ref.current;
        if (!input) return;
        void info("focus the main input");
        input.focus();
    }, [input_ref, dispatch]);

    useEffect(() => {
        void debug("main window reset on mount");
        reset();

        let unlisten: (() => void) | undefined;
        let cancelled = false;

        void on_main_shown(() => {
            void debug("main window reset on main_shown");
            reset();
        }).then(stop => {
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
 * 每次按键与每次输入都从这里进：
 * - 状态变化交给 [`reduce_main`]
 * - 检索的时序交给 [`create_search_session`]
 * - 窗口显示时的复位与聚焦交给 [`useMainWindowFocus`]
 * - 本身只负责事件注册、invoke 与触发时机
 */
export const useMainInteraction = () => {
    // 主状态
    const [state, dispatch] = useReducer(reduce_main, MAIN_STATE_INIT);
    // 主界面显示几行 item
    const [item_n, set_item_n] = useState(FALLBACK_ITEM_N);
    // 动作表：挂载时拉一次，拉到之前是空表（没有条目显示动作图标）
    const [type_actions, set_type_actions] = useState<ItemTypeActions>(EMPTY_ITEM_TYPE_ACTIONS);

    const input_ref = useRef<HTMLInputElement>(null);
    // 上一次连续切换的时刻；初值 -Infinity 让首次切换立即响应
    const last_step_at = useRef(Number.NEGATIVE_INFINITY);

    // 检索会话不随每次渲染重建：时序状态都在它里面，重建就等于丢掉在飞的请求
    const [session] = useState(() => create_search_session({
        search: search,
        search_page: search_page,
        on_conclusion: page => dispatch({ kind: "settled", page }),
        on_page_appended: page => dispatch({ kind: "page_appended", page }),
    }));

    /**
     * 限流逻辑，只限制长摁和滚轮，手动连摁不限制
     */
    const pace_step = (continuous: boolean): boolean => {
        const now = performance.now();
        if (continuous && now - last_step_at.current < SELECT_REPEAT_MS) return false;
        last_step_at.current = now;
        return true;
    };

    /** 计算当前条目的目标动作 */
    const action_step_target = (delta: number): { item_index: number, action_index: number } | null => {
        const item = current_item(state.conclusion?.item_list, state.selection);
        if (!item) return null;

        const action_index = action_index_of(state, item.item_index) + delta;
        if (action_index < 0 || action_index >= actions_of(type_actions, item.the_type).length) return null;

        return { item_index: item.item_index, action_index };
    };

    /** 切换当前条目的动作 */
    const switch_action = (delta: number, continuous: boolean) => {
        if (!pace_step(continuous)) return;

        const target = action_step_target(delta);
        if (target === null) return;

        dispatch({ kind: "action_selected", item_index: target.item_index, action_index: target.action_index });
    };

    /** 静默续下一页 */
    const prefetch_next_page = () => {
        session.prefetch({
            // 已加载条数就是下一页的起始下标：列表按顺序追加，中间没有空洞
            loaded_count: state.conclusion?.item_list.length ?? 0,
            selection: state.selection,
            total: state.conclusion?.total ?? 0,
        });
    };

    /** 移动选中条目 */
    const move_selection = (intent: Intent, continuous: boolean) => {
        if (!pace_step(continuous)) return;

        // 朝后走才看一眼要不要续下一页；预请求是纯追加，不挡这次移动
        if (intent === "select_next") prefetch_next_page();

        dispatch({ kind: "intent", intent });
    };

    const run_current_action = () => {
        const item = current_item(state.conclusion?.item_list, state.selection);
        if (!item) return;
        const action = current_action(type_actions, item.the_type, action_index_of(state, item.item_index));
        if (!action) return;

        void info(`run action item_index=${item.item_index} action=${action.id}`);
        dispatch({ kind: "intent", intent: "run_action" });
        void run_item_action(item.item_index, action.id);
    };

    /**
     * 按键触发行为
     *
     * 合成（ IME 候选框打开）期间的按键用 `event.isComposing` 挡掉：
     * - 候选框里的 ↑/↓/←/→、提交候选的 Enter、取消候选的 ESC 都归输入法。
     * - 残余风险是提交候选那一下的事件顺序与 isComposing 取值（待实测）。
     */
    const on_key_down = useEffectEvent((event: KeyboardEvent) => {
        // 部分平台 isComposing 行为【在较新版本中】仍未完全符合预期
        // 因此仍然加上 keyCode 判断（即使他已经被废弃）
        if (event.isComposing || event.keyCode === 229) return;

        const intent = resolve_key(
            event.key,
            { shift: event.shiftKey },
            { preview_open: state.preview_open },
        );
        if (intent === null) return;

        if (!is_select_intent(intent) && !is_action_intent(intent)) event.preventDefault();

        if (intent === "dismiss") {
            void dismiss_main_window();
            return;
        }

        if (intent === "run_action") {
            run_current_action();
            return;
        }

        if (is_select_intent(intent)) {
            move_selection(intent, event.repeat);
            return;
        }

        if (is_action_intent(intent)) {
            const delta = intent === "action_next" ? 1 : -1;
            switch_action(delta, event.repeat);
            return;
        }

        dispatch({ kind: "intent", intent });
    });

    useEffect(() => {
        window.addEventListener("keydown", on_key_down, true);
        return () => window.removeEventListener("keydown", on_key_down, true);
    }, []);

    /**
     * 滚轮触发行为
     */
    const on_wheel = useEffectEvent((event: WheelEvent) => {
        const intent = resolve_wheel(event, event.shiftKey);
        if (intent === null) return;

        if (is_action_intent(intent)) {
            const delta = intent === "action_next" ? 1 : -1;
            switch_action(delta, true);
            return;
        }

        move_selection(intent, true);
    });

    useEffect(() => {
        // 要 preventDefault 就不能挂被动监听
        window.addEventListener("wheel", on_wheel, { capture: true, passive: false });
        return () => window.removeEventListener("wheel", on_wheel, { capture: true });
    }, []);

    /** 输入改变，具体逻辑交给 search_session.ts 代理 */
    const on_input_change = useEffectEvent((value: string, is_composing: boolean) => {
        dispatch({ kind: "typing", value });
        session.input_changed(value, is_composing);
    });

    useEffect(() => {
        const input = input_ref.current;
        if (!input) return;

        const on_input = (event: InputEvent) => on_input_change(input.value, event.isComposing);
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
    }, [session]);

    // 获取配置
    useEffect(() => {
        void get_config().then(config => set_item_n(config.main_item_n));
        void get_item_type_actions().then(set_type_actions);
    }, []);

    // 卸载时丢掉还没到点的防抖
    useEffect(() => () => session.dispose(), [session]);

    useMainWindowFocus(input_ref, dispatch);

    return { state, item_n, type_actions, input_ref };
}
