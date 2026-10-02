import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { attachConsole, debug, info, warn } from '@tauri-apps/plugin-log';

if (import.meta.env.DEV) {
    void attachConsole();
}

const EDIT_SHORTCUT_KEYS = new Set(["KeyA", "KeyC", "KeyV", "KeyX", "KeyY", "KeyZ"]);

const is_browser_shortcut = (e: KeyboardEvent) => {
    if (/^F\d{1,2}$/.test(e.key)) return true;
    if (e.altKey) return true;
    if (e.metaKey) return true;
    if (e.ctrlKey) {
        if (e.shiftKey) return true;
        return !EDIT_SHORTCUT_KEYS.has(e.code);
    }
    return false;
};

const set_near_native = () => {
    // 禁用 Alt 菜单栏与浏览器快捷键，只放行编辑类组合
    const on_key_down = (e: KeyboardEvent) => {
        if (e.isComposing) return;
        if (is_browser_shortcut(e)) e.preventDefault();
    };
    window.addEventListener("keydown", on_key_down);

    // 禁用右键菜单
    const on_ctx_menu = (e: PointerEvent) => e.preventDefault();
    window.addEventListener("contextmenu", on_ctx_menu);

    const on_mouse_down = (e: MouseEvent) => {
        if (e.button === 1 || e.button === 3 || e.button === 4) e.preventDefault();
    };
    window.addEventListener("mousedown", on_mouse_down);
    window.addEventListener("auxclick", on_mouse_down);

    const on_drag = (e: Event) => e.preventDefault();
    window.addEventListener("dragstart", on_drag);
    window.addEventListener("dragover", on_drag);
    window.addEventListener("drop", on_drag);

    return () => {
        window.removeEventListener("keydown", on_key_down);
        window.removeEventListener("contextmenu", on_ctx_menu);
        window.removeEventListener("mousedown", on_mouse_down);
        window.removeEventListener("auxclick", on_mouse_down);
        window.removeEventListener("dragstart", on_drag);
        window.removeEventListener("dragover", on_drag);
        window.removeEventListener("drop", on_drag);
    }
}

/**
 * 窗口效果（对应 Rust 侧 window_effect::WindowEffect）
 *
 * 这个类型是手写的，Rust 侧改了字段/变体名不会有编译期报错，
 * 由 src-tauri/tests/fixtures/config.golden.json 的金样本测试兜底。
 */
export type WindowEffect = "Solid" | "Mica" | "Acrylic" | "Vibrancy";

/**
 * 主窗口模式（对应 Rust 侧 configs::MainWindowMode）
 */
export type MainWindowMode = "Always" | "HideAndShow";

/**
 * 配置（对应 Rust 侧 configs::Config，也就是 config.json 的形状）
 *
 * 整份读写：拿到什么形状就回传什么形状，后端会校验后再落盘。
 */
export interface Config {
    main_window_mode: MainWindowMode,
    window_effect: WindowEffect | null,
    always_on_top: boolean,
    main_width: number,
    main_head_h: number,
    main_tail_h: number,
    main_item_h: number,
    main_item_n: number,
    hot_key_alt: boolean,
    hot_key_ctrl: boolean,
    hot_key_meta: boolean,
    hot_key_shift: boolean,
    hot_key_char: string,
}

/**
 * 窗口效果的解析结果与平台能力（对应 Rust 侧 configs::EffectInfo）
 */
export interface EffectInfo {
    configured: WindowEffect | null,
    effective: WindowEffect,
    available: WindowEffect[],
    alpha: number,
}

const invoke_backend = async (cmd: string, args?: Record<string, unknown>): Promise<unknown> => {
    const start = performance.now();
    void info(`invoke ${cmd} start`);
    try {
        const value = await invoke(cmd, args);
        void debug(`invoke ${cmd} done in ${Math.round(performance.now() - start)}ms`);
        return value;
    } catch (err) {
        void warn(`invoke ${cmd} failed in ${Math.round(performance.now() - start)}ms`);
        throw err;
    }
}

// #region: global

export const get_config = async () => {
    return (await invoke_backend('fetch_config')) as Config;
}

/**
 * 保存配置
 * @param config 保存的配置
 * @returns 是否需要 register_global_shortcut
 */
export const save_config = async (config: Config) => {
    return (await invoke_backend('save_config', { config })) as boolean;
}

export const get_effect_info = async () => {
    return (await invoke_backend('fetch_effect_info')) as EffectInfo;
}

const should_show_main_auto = async () => {
    return (await invoke_backend('should_show_main_auto')) as boolean;
}

export const register_global_shortcut = async () => {
    await invoke_backend('register_global_shortcut');
}

// #endregion

// #region plugin framework
//
// 插件体系那一套（对应 Rust 侧 plugin_framework 与 plugin_impl_launcher）。
// 界面读的是这一套：类型名与动作 id 由插件定义，Rust 侧与这里都不解释它们。
// 「类型名画哪张图标、动作 id 画哪张图标、label_key 是什么中文」归前端插件注册表，
// 见 src/plugins/registry.tsx。

/**
 * 插件条目的渲染结构（对应 Rust 侧 plugin_framework::PluginItemDisplay）
 *
 * 检索结果里的每条条目都是这个形状，`action_ids` 是条目自带的动作 id 列表：顺序即优先级、
 * 第一个是默认动作（见 spec §1.5）。`the_type` 是插件定义的类型名，是**不透明字符串**。
 *
 * `handle` 是条目身份（对应 Rust 侧 plugin_framework::ItemHandle）：
 * `item_index` 只给主列表翻页与缓存记账，**动作派发一律用 handle**——
 * 插件搜索结果行不在主列表那一集里，下标对它不成立（见 spec §3.4）。
 */
export interface PluginItemDisplay {
    the_type: string,
    name: string,
    desc: string,
    /**
     * 条目自带的图标：**绝对路径**，空串表示没有
     *
     * 与"类型图标"不是一回事：类型图标一张画给同类型的每一行（前端注册表按类型名查），
     * 这一张是条目自己的图片（`Plugin Package` 清单里写的那张），由前端转成 asset URL。
     */
    icon: string,
    item_index: number,
    action_ids: string[],
    handle: ItemHandle,
}

/**
 * 条目地址（对应 Rust 侧 plugin_framework::ItemAddress）
 *
 * 注册条目是它在该插件块内的注册序号，插件搜索结果行则是它在最近一次搜索结果里的下标
 * （结果行不注册进框架，见 spec §3.4）。
 */
export type ItemAddress =
    | { kind: "registered", index: number }
    | { kind: "row", index: number }

/**
 * 条目身份（对应 Rust 侧 plugin_framework::ItemHandle）
 *
 * `plugin_id` + 一个 `ItemAddress`；两者都由后端生成，前端只原样回传。
 */
export interface ItemHandle {
    plugin_id: string,
    address: ItemAddress,
}

/**
 * 一条 Plugin Search Result（对应 Rust 侧 plugin_impl_js::SearchRow）
 *
 * 插件自己的搜索函数交出来的纯数据：`the_type` 是它声明的结果行类型名，
 * `action_ids` 是这一行自己的动作列表（顺序即优先级，可以为空）。
 */
export interface PluginSearchRow {
    the_type: string,
    name: string,
    desc: string,
    action_ids: string[],
}

/**
 * 插件检索结果的一页（对应 Rust 侧 plugin_framework::ItemSearchPage）
 *
 * `index` 是这一页在结果集里的起始位置（不是页码），
 * 三个分组边界用来在列表里画匹配模式的分割线；
 * `token` 是后端下发的结果令牌，翻页时原样回传，前端不自己造。
 */
export interface PluginItemSearchPage {
    token: number,
    total: number,
    index: number,
    item_list: PluginItemDisplay[],
    index_eq: number,
    index_starts_with: number,
    index_contains: number,
    index_match: number,
}

/** 使用关键字在插件注册表上检索；空串是合法输入，后端按「空关键字匹配所有」给出全部条目 */
export const plugin_search = async (k: string) => {
    return (await invoke_backend('plugin_search', { k })) as PluginItemSearchPage;
}

/** 取插件检索结果的一页；`index` 是页起始下标，`token` 从后端拿、原样回传 */
export const plugin_search_page = async (index: number, token: number) => {
    return (await invoke_backend('plugin_search_page', { index, token })) as PluginItemSearchPage;
}

/**
 * 跑一个 Plugin Action
 *
 * 按条目身份与动作 id 派发；认不出的插件、落不到的行与还没实现的动作在 Rust 侧当无操作
 * 并记日志。跑完之后要不要隐藏主窗口由 Rust 按 `main_window_mode` 决定，前端不参与。
 */
export const plugin_run_item_action = async (handle: ItemHandle, action: string) => {
    await invoke_backend('plugin_run_item_action', { handle, action });
}

/**
 * 触发一次 Plugin Search，拿回 `Plugin Search Page` 的数据
 *
 * Rust 会把请求转给 webview（装载插件、跑插件自己的搜索），等结果行回来再投影成这一页。
 * 插件装载失败或搜索抛错时是一份空页，不是错误（见 spec §4.4）。
 */
export const plugin_open_plugin_search = async (plugin_id: string, keyword: string) => {
    return (await invoke_backend('plugin_open_plugin_search', { pluginId: plugin_id, keyword })) as PluginItemSearchPage;
}

/**
 * 把插件搜索的结果行交回 Rust（`plugin_impl_js` 里那次搜索的回程）
 *
 * 只有插件自己报过的那一份行能派发动作：Rust 按同样的下标把它们记下来。
 */
export const plugin_report_search_results = async (plugin_id: string, items: PluginSearchRow[]) => {
    await invoke_backend('plugin_report_search_results', { pluginId: plugin_id, items });
}

/**
 * 打开（或聚焦）插件自己的一个 HTML 窗口
 *
 * `path` 是**包内相对路径**，插件只能寻址自己包目录里的文件；`title` 为空时用清单里的包名，
 * `width` / `height` 非正数时用后端缺省值。同一个包只有一个窗口：再打开一次是换页 + 聚焦。
 */
export const plugin_open_html_window = async (plugin_id: string, path: string, title: string, width: number, height: number) => {
    await invoke_backend('plugin_open_html_window', { pluginId: plugin_id, path, title, width, height });
}

// #endregion

// #region main_window

/**
 * 实际体验无需 fade-in anim
 */
export const show_main_window = async () => {
    await invoke_backend('show_main_window');
}

export const dismiss_main_window = async () => {
    await invoke_backend('dismiss_main_window');
}

export const rebuild_main_window = async () => {
    await invoke_backend('rebuild_main_window');
}

const EVENT_MAIN_SHOWN = "main_shown";

/** 后端显示主界面后，再次通知前端 */
export const on_main_shown = async (handler: () => void) => {
    return await listen(EVENT_MAIN_SHOWN, handler);
}

// #endregion

let main_window_ready = false;

export const should_show_main_on_ready = async () => {
    if (main_window_ready) return false;
    main_window_ready = true;
    return await should_show_main_auto();
}

/**
 * 设置一个 css 变量，值的单位是 px
 * @param k css 变量名称
 * @param v 值，单位 px
 */
const set_layout_px = (k: string, v: number) => {
    document.documentElement.style.setProperty(k, v + 'px');
}

/**
 * 按最新配置设置页面：布局尺寸 + 面板底色透明度
 *
 * 布局值来自可编辑配置，面板底色的透明度来自窗口效果的解析结果（见 window_effect::alpha）。
 * 颜色本身留在 css 的调色板里，这里只给透明度，由 index_main.css 的 body::before 用 opacity 消费。
 */
const set_page_config_data = async () => {
    const config = await get_config();
    set_layout_px('--head-h', config.main_head_h);
    set_layout_px('--tail-h', config.main_tail_h);
    set_layout_px('--item-h', config.main_item_h);

    const effect_info = await get_effect_info();
    document.documentElement.style.setProperty('--color-bg-alpha', String(effect_info.alpha));
}

/** 主页面初始化 */
export const setup_page_main = () => {
    void set_page_config_data();
    return set_near_native();
}

/** 配置页面初始化 */
export const setup_page_config = () => {
    return set_near_native();
}
