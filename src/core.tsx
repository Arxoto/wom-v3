import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { attachConsole, debug, info, warn } from '@tauri-apps/plugin-log';

if (import.meta.env.DEV) {
    void attachConsole();
}

const show_page_main = async () => {
    await set_page_config_data();
    play_panel_show();
    await show_main_window();
}

export const set_page_main = () => {
    void show_page_main();
    return set_near_native();
}

export const set_page_config = () => {
    return set_near_native();
}

const EDIT_SHORTCUT_KEYS = new Set(["a", "c", "v", "x", "y", "z"]);

const is_browser_shortcut = (e: KeyboardEvent) => {
    if (/^F\d{1,2}$/.test(e.key)) return true;
    if (e.altKey) return true;
    if (!e.ctrlKey && !e.metaKey) return false;
    return !EDIT_SHORTCUT_KEYS.has(e.key.toLowerCase());
};

const set_near_native = () => {
    // 禁用右键菜单
    const on_ctx_menu = (e: PointerEvent) => e.preventDefault();
    window.addEventListener("contextmenu", on_ctx_menu);

    // 禁用 Alt 菜单栏与浏览器快捷键，只放行编辑类组合
    const on_key_down = (e: KeyboardEvent) => {
        if (e.isComposing) return;
        if (is_browser_shortcut(e)) e.preventDefault();
    };
    window.addEventListener("keydown", on_key_down);

    const on_mouse_down = (e: MouseEvent) => {
        if (e.button === 1) e.preventDefault();
    };
    window.addEventListener("mousedown", on_mouse_down);

    const on_drag = (e: Event) => e.preventDefault();
    window.addEventListener("dragstart", on_drag);
    window.addEventListener("dragover", on_drag);
    window.addEventListener("drop", on_drag);

    return () => {
        window.removeEventListener("contextmenu", on_ctx_menu);
        window.removeEventListener("keydown", on_key_down);
        window.removeEventListener("mousedown", on_mouse_down);
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
export type ItemType = "snip" | "sys" | "note" | "cmd" | "web" | "file" | "scan";

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
    file: [],
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

export const get_config = async () => {
    return (await invoke_backend('fetch_config')) as Config;
}

export const set_config = async (config: Config) => {
    await invoke_backend('set_config', { config });
}

export const get_effect_info = async () => {
    return (await invoke_backend('fetch_effect_info')) as EffectInfo;
}

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

/**
 * 无条件隐藏主窗口
 *
 * 它就是 ESC 那条显式意图，与 main_window_mode 无关——配置只管失焦与触发动作
 * 这两条被动隐藏规则。
 */
export const dismiss_main_window = async () => {
    await invoke_backend('dismiss_main_window');
}

export const show_main_window = async () => {
    await invoke_backend('show_main_window');
}

/** 主窗口被显示时后端发来的事件名（对应 Rust 侧 constants::EVENT_MAIN_SHOWN） */
const EVENT_MAIN_SHOWN = "main_shown";

const EVENT_MAIN_WILL_SHOW = "main_will_show";

/**
 * 监听主窗口被显示（每次显示都会发）
 *
 * 返回取消监听的函数，调用方负责在卸载时取消。
 */
export const on_main_shown = async (handler: () => void) => {
    return await listen(EVENT_MAIN_SHOWN, handler);
}

export const on_main_will_show = async (handler: () => void) => {
    return await listen(EVENT_MAIN_WILL_SHOW, handler);
}

const PANEL_SHOW_FRAMES: Keyframe[] = [
    { opacity: 0, transform: "scale(0.98)" },
    { opacity: 1, transform: "scale(1)" },
];

export const play_panel_show = () => {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    document.body.animate(PANEL_SHOW_FRAMES, {
        duration: 140,
        easing: "ease-out",
        fill: "backwards",
    });
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
