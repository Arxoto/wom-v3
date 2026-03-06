/**
 * 类原生应用设置
 */
function near_native_setting() {
    // 禁用右键菜单
    window.addEventListener('contextmenu', (e) => {
        e.preventDefault();
    });
}

export { near_native_setting };