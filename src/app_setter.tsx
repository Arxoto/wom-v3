const set_page_main = () => {
    set_near_native();
    set_page_layout();
}

const set_page_config = () => {
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

const set_layout_px = (k: string, v: number) => {
    document.documentElement.style.setProperty(k, v + 'px');
}

/**
 * 根据配置分配布局；
 * 尽量父子元素高宽值确定性能最佳，其次子元素填满父元素，最次子元素撑起父元素；
 * 绝对布局在固定布局的性能上略胜弹性布局（忽略不计），但是重新布局的场景弹性布局远胜固定布局；
 */
const set_page_layout = () => {
    let head_h = 55;
    let tail_h = 55;
    let item_h = 40;
    set_layout_px('--head-h', head_h);
    set_layout_px('--tail-h', tail_h);
    set_layout_px('--item-h', item_h);
}

export { set_page_main, set_page_config };