//! JS 插件打开的 HTML 窗口（命令 [`crate::commands::plugin::plugin_open_html_window`]）
//!
//! 窗口装的是插件自己包里的一个 HTML 文件，地址走 asset protocol：与插件代码的装载走同一条
//! 路（见 `docs/adr/0011`），宿主不读文件内容。
//! 相对路径是插件给的**不可信输入**，只能落在它自己的包目录里（见
//! [`crate::plugin_package::resolve_in_package`]）。
//!
//! 一个包一个窗口：标签由 `PluginId` 拼出，再次打开同一个包就导航过去并聚焦，不会一直堆窗口。
//! 窗口的创建与导航本身在 [`crate::plugin_window`]，这里只做"包 → 绝对路径"这一步。

use tauri::Manager;

use super::packages::PackageHost;

/// 打开（或聚焦）一个插件包的 HTML 窗口
///
/// `relative` 是包内相对路径；`title` 为空时用清单里的包名；`width` / `height` 非正数时用缺省值。
pub fn open_html_window(
    app: &tauri::AppHandle,
    plugin_id: &str,
    relative: &str,
    title: &str,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let host = app.state::<PackageHost>();
    let path = host.resolve_file(plugin_id, relative)?;
    let title = if title.is_empty() {
        host.name_of(plugin_id)
    } else {
        title.to_string()
    };

    crate::plugin_window::open(app, plugin_id, &path, &title, width, height)
}
