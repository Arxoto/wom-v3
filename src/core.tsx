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

/**
 * 列表条目的渲染结构（对应 Rust 侧 search::ItemDisplay）
 *
 * 只带渲染需要的类型、名称与描述，外加 `item_index`（Item Index）：条目在整集里的
 * 下标，前端跑 Item Action 时用它寻址。列表里的行号是 List Position，两者不是一回事。
 * 关键字与动作不在其中。
 */
export interface ItemDisplay {
    the_type: ItemType,
    name: string,
    desc: string,
    item_index: number,
}

/**
 * Item 的类型（对应 Rust 侧 base::ItemType）
 */
export type ItemType = "snip" | "sys" | "note" | "cmd" | "web" | "scan";

/**
 * Item Action 的标识（对应 Rust 侧 action::ItemActionId）
 */
export type ItemActionId = "copy" | "open_url" | "open_path" | "reveal" | "open_note";

/**
 * 一个 Item Action 的元数据（对应 Rust 侧 action::ItemAction）
 *
 * `label_key` 按 ItemType + 动作分套，中文文案见 src/main/interaction/action_labels.ts。
 */
export interface ItemAction {
    id: ItemActionId,
    label_key: string,
}

/**
 * 每种 ItemType 支持的动作（对应 Rust 侧 action::table）
 *
 * 顺序即优先级，第一个是该类型的默认动作。
 */
export type ItemTypeActions = Record<ItemType, ItemAction[]>;

/** 还没拉到动作表时的空表：拉到之前没有条目显示动作图标 */
export const EMPTY_ITEM_TYPE_ACTIONS: ItemTypeActions = {
    snip: [],
    sys: [],
    note: [],
    cmd: [],
    web: [],
    scan: [],
};

/**
 * 检索结果的一页（对应 Rust 侧 search::ItemSearchPage）
 *
 * `index` 是这一页在结果集里的起始位置（不是页码），
 * 三个分组边界用来在列表里画匹配模式的分割线；
 * `token` 是后端下发的结果令牌，翻页时原样回传，前端不自己造。
 */
export interface ItemSearchPage {
    token: number,
    total: number,
    index: number,
    item_list: ItemDisplay[],
    index_eq: number,
    index_starts_with: number,
    index_contains: number,
    index_match: number,
}

/**
 * 扫描根路径变量（对应 Rust 侧 persistence::scan_base::ScanBase）
 *
 * 值就是设置文件里写的变量名，各平台落到哪个目录由 tauri 决定。
 * 手写的设置文件可以用这张表以外的变量，配置页的下拉只提供这些。
 */
export type ScanBase =
    | "$HOME" | "$DESKTOP" | "$DOWNLOAD" | "$DOCUMENT" | "$PICTURE" | "$AUDIO" | "$VIDEO"
    | "$CONFIG" | "$DATA" | "$LOCALDATA" | "$RESOURCE"
    | "$APPCONFIG" | "$APPDATA" | "$APPLOCALDATA";

/**
 * 扫描根路径的下拉选项（对应 Rust 侧 ScanBaseOption）
 */
export interface ScanBaseOption {
    value: ScanBase,
    label: string,
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

// #region built-in plugin
//
// 内建条目（builtin_plugins）那一套：与下面的插件注册表那一套并行存在，是有意的（见 ADR 0008）。
// 界面这一轮已经切到插件那一套，这里的封装与类型原样留着——它们对应的命令仍在 Rust 侧注册着。

export const get_scan_base_options = async () => {
    return (await invoke_backend('fetch_scan_base_options')) as ScanBaseOption[];
}

/**
 * 使用关键字检索
 *
 * 空串是合法输入：后端按「空关键字匹配所有」给出全部条目。
 */
export const search = async (k: string) => {
    return (await invoke_backend('search', { k })) as ItemSearchPage;
}

/**
 * 取检索结果的一页
 *
 * - `index` 是这一页在结果集里的起始位置（不是页码）：预请求传已加载条数。
 * - `token` 是从后端拿到的、屏上那份结论的令牌：原样回传，不自己造；与后端缓存对不上就会报错。
 */
export const search_page = async (index: number, token: number) => {
    return (await invoke_backend('search_page', { index, token })) as ItemSearchPage;
}

/**
 * 每种 ItemType 支持的动作
 *
 * 前端只在挂载时拉一次：配置重载会重建窗口，不需要热更新。
 */
export const get_item_type_actions = async () => {
    return (await invoke_backend('fetch_item_type_actions')) as ItemTypeActions;
}

/**
 * 跑一个 Item Action
 *
 * 按下标与动作 id 派发；越界索引、认不出的动作与还没实现的动作在 Rust 侧当无操作
 * 并记日志。跑完之后要不要隐藏主窗口由 Rust 按 `main_window_mode` 决定，前端不参与。
 */
export const run_item_action = async (item_index: number, action: ItemActionId) => {
    await invoke_backend('run_item_action', { itemIndex: item_index, action });
}

// #endregion

// #region plugin framework
//
// 插件体系那一套（对应 Rust 侧 plugin_framework 与 plugin_impl_launcher）。
// 界面读的是这一套：类型名由插件定义，框架与前端都不解释它，前端只按它查图标与文案。

/**
 * 插件条目的渲染结构（对应 Rust 侧 plugin_framework::PluginItemDisplay）
 *
 * 与内建条目同一个形状，多一个 `action_ids`：条目自带的动作 id 列表，顺序即优先级、
 * 第一个是默认动作（见 spec §1.5）。`the_type` 是插件定义的类型名，是**不透明字符串**。
 */
export interface PluginItemDisplay {
    the_type: string,
    name: string,
    desc: string,
    item_index: number,
    action_ids: string[],
}

/**
 * 动作表里的一条动作（对应 Rust 侧 plugin_framework::PluginActionView）
 *
 * `label_key` 由插件给出，中文文案见 src/main/interaction/action_labels.ts。
 */
export interface PluginActionView {
    id: string,
    label_key: string,
}

/**
 * 插件的动作表（对应 Rust 侧 plugin_framework::ActionTableView）
 *
 * 类型名 → 该类型的动作，顺序即优先级、第一个是默认动作；类型名是不透明字符串。
 */
export type PluginActionTable = Record<string, PluginActionView[]>;

/** 还没拉到动作表时的空表：拉到之前没有条目显示动作图标 */
export const EMPTY_PLUGIN_ACTION_TABLE: PluginActionTable = {};

/**
 * 插件检索结果的一页（对应 Rust 侧 plugin_framework::ItemSearchPage）
 *
 * 字段与内建那一页一致，见上面 [`ItemSearchPage`] 的说明。
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

/**
 * 取插件检索结果的一页
 *
 * `index` 与 `token` 的语义与 [`search_page`] 完全一致：令牌从后端拿，原样回传。
 */
export const plugin_search_page = async (index: number, token: number) => {
    return (await invoke_backend('plugin_search_page', { index, token })) as PluginItemSearchPage;
}

/**
 * 插件注册的动作表
 *
 * 前端只在挂载时拉一次：动作表相对插件与类型是固定的（Q22），配置重载会重建窗口。
 */
export const get_plugin_actions = async () => {
    return (await invoke_backend('fetch_plugin_actions')) as PluginActionTable;
}

/**
 * 跑一个 Plugin Action
 *
 * 按下标与动作 id 派发；语义与 [`run_item_action`] 一致，隐藏窗口同样由 Rust 决定。
 */
export const plugin_run_item_action = async (item_index: number, action: string) => {
    await invoke_backend('plugin_run_item_action', { itemIndex: item_index, action });
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
