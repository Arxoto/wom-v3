function global_setting() {
    // 禁用右键菜单
    window.addEventListener('contextmenu', (e) => {
        e.preventDefault();
    });
}
