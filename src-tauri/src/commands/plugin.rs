use tauri::{Manager, State};

use crate::{
    configs,
    plugin_framework::{self, ActionId, ActionOutcome, ItemHandle, PluginRegistry},
    window_utils,
};

// 命令只做转发，实现留在 plugin_framework 的 PluginRegistry 里——那一层不知道 Tauri，
// 也就不该在这里出现半条检索或动作逻辑。

/// 使用关键字在插件注册表上检索（实现见 [`PluginRegistry::search`]）
///
/// 空串是合法输入：后端按「空关键字匹配所有」给出全部条目。
#[tauri::command]
pub async fn plugin_search(
    plugin_registry: State<'_, PluginRegistry>,
    k: &str,
) -> Result<plugin_framework::ItemSearchPage, String> {
    Ok(plugin_registry.search(k))
}

/// 对插件检索结果翻页（实现见 [`PluginRegistry::page`]）
///
/// `token` 由后端下发、前端原样回传，对不上就报错。
#[tauri::command]
pub async fn plugin_search_page(
    plugin_registry: State<'_, PluginRegistry>,
    index: usize,
    token: u32,
) -> Result<plugin_framework::ItemSearchPage, String> {
    plugin_registry.page(index, token)
}

/// 按下标取出条目并跑它的一个 Plugin Action（实现见 [`PluginRegistry::run_action`]）
///
/// 只有动作**真的执行了**，才轮到「按 `main_window_mode`
/// 决定要不要隐藏窗口」这一问（见 spec §3 / §4.3）。认不出的插件、落不到的行、没挂在该条目
/// 类型上的动作都当无操作并记 warn——不 panic、不隐藏，前端也不提示失败。
///
/// 寻址用 [`ItemHandle`] 而不是下标：主列表与 `Plugin Search Page` 两层列表因此共用这一条命令
/// （结果行不在框架持有的那一集里，下标对它不成立，见 spec §3.4）。
#[tauri::command]
pub fn plugin_run_item_action(
    app: tauri::AppHandle,
    plugin_registry: State<'_, PluginRegistry>,
    handle: ItemHandle,
    action: String,
) -> Result<(), String> {
    if plugin_registry.run_action(&handle, &ActionId(action)) != ActionOutcome::Done {
        return Ok(());
    }

    if configs::get_data().hide_main_after_action() {
        window_utils::hide_main_window(&app).map_err(|err| err.to_string())?;
    }

    Ok(())
}

/// 触发一次 `Plugin Search` 并拿到 `Plugin Search Page` 的数据
///
/// 这个命令是**异步**的：它把请求发给 webview，等插件把结果行报回来（见
/// [`crate::plugin_host::open_plugin_search`]）。修不成插件的结果页是空页，不是错误。
#[tauri::command]
pub async fn plugin_open_plugin_search(
    app: tauri::AppHandle,
    plugin_id: String,
    keyword: String,
) -> Result<plugin_framework::ItemSearchPage, String> {
    crate::plugin_host::open_plugin_search(&app, &plugin_id, &keyword).await
}

/// 插件执行完搜索后把结果行交回来（实现见 [`crate::plugin_host::report_search_results`]）
///
/// 这是 [`plugin_open_plugin_search`] 等的那一头：没有在飞的请求就说明这份结果过期了。
#[tauri::command]
pub fn plugin_report_search_results(
    app: tauri::AppHandle,
    plugin_id: String,
    items: Vec<crate::plugin_proxy_js::SearchRow>,
) -> Result<(), String> {
    crate::plugin_host::report_search_results(&app, &plugin_id, items)
}

/// 打开（或聚焦）插件自己的一个 HTML 窗口
///
/// `path` 是**包内相对路径**，插件只能寻址自己包目录里的文件；窗口这一轮用系统原生边框
/// （实现见 [`crate::plugin_host::open_html_window`]）。同一个包只有一个窗口：再打开一次是
/// 换页 + 聚焦。
#[tauri::command]
pub fn plugin_open_html_window(
    app: tauri::AppHandle,
    plugin_id: String,
    path: String,
    title: String,
    width: f64,
    height: f64,
) -> Result<(), String> {
    crate::plugin_host::open_html_window(&app, &plugin_id, &path, &title, width, height)
}

/// 取当前插件窗口要显示的页面绝对路径（承载页 `index_iframe.html` 用）
#[tauri::command]
pub fn plugin_window_page(window: tauri::WebviewWindow) -> Result<String, String> {
    window
        .state::<crate::plugin_window::PluginWindowPages>()
        .get(window.label())
        .ok_or_else(|| format!("no plugin page for window: {}", window.label()))
}
