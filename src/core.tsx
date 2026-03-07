import { invoke } from '@tauri-apps/api/core';

export const set_page_main = () => {
    set_near_native();
    set_page_layout();
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

interface LayoutConfig {
    head_h: number,
    tail_h: number,
    item_h: number,
    item_n: number,
}

export const get_layout_config = async () => {
    return (await invoke('fetch_layout_config')) as LayoutConfig;
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
 * 根据配置分配布局；
 * 尽量父子元素高宽值确定性能最佳，其次子元素填满父元素，最次子元素撑起父元素；
 * 绝对布局在固定布局的性能上略胜弹性布局（忽略不计），但是重新布局的场景弹性布局远胜固定布局；
 */
const set_page_layout = async () => {
    const conf = await get_layout_config();
    set_layout_px('--head-h', conf.head_h);
    set_layout_px('--tail-h', conf.tail_h);
    set_layout_px('--item-h', conf.item_h);
}
