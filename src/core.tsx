import { invoke } from '@tauri-apps/api/core';

export const set_page_main = () => {
    set_near_native();
    set_page_config_data();
}

export const set_page_config = () => {
    set_near_native();
}

/**
 * 类原生应用设置
 */
const set_near_native = () => {
    // 禁用右键菜单
    window.addEventListener('contextmenu', (e) => {
        e.preventDefault();
    });
}

/**
 * 窗口效果（对应 Rust 侧 window_effect::WindowEffect）
 *
 * 这个类型是手写的，Rust 侧改了字段/变体名不会有编译期报错，
 * 由 src-tauri/tests/fixtures/config.golden.json 的金样本测试兜底。
 */
export type WindowEffect = "Solid" | "Framed" | "Mica" | "Acrylic" | "Vibrancy";

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
 * 只带渲染需要的类型、名称与描述；关键字与动作不在其中。
 */
export interface ItemDisplay {
    the_type: string,
    name: string,
    desc: string,
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

export const get_config = async () => {
    return (await invoke('fetch_config')) as Config;
}

export const set_config = async (config: Config) => {
    await invoke('set_config', { config });
}

export const get_effect_info = async () => {
    return (await invoke('fetch_effect_info')) as EffectInfo;
}

export const get_scan_base_options = async () => {
    return (await invoke('fetch_scan_base_options')) as ScanBaseOption[];
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
