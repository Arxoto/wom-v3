//! 前端调用的命令
//!
//! 实现的归属：能留在各自模块里的就只在这里转发（如检索命令，见 [`crate::plugin_framework`]）；
//! 配置相关的命令要同时指挥 configs / window_utils / global_shortcut，实现直接落在这里——
//! 放进 `configs` 会让它反向依赖 `window_utils`，而依赖只能单向流动。
//!
//! 应用只有一张命令注册表：`lib.rs` 的 `invoke_handler`。
//!
//! 检索与动作走插件注册表那一套（`plugin_search`、`plugin_run_item_action` …）。
//! 这一套**没有**取动作表的命令：界面用的动作顺序、图标与文案都在前端的插件注册表里
//! （见 `docs/adr/0010`）；条目自带的动作 id 列表随检索结果一起下来。

use tauri::State;

use crate::{
    configs,
    plugin_framework::{self, ActionId, ActionOutcome, ItemHandle, PluginRegistry},
    window_effect, window_utils,
};

#[cfg(desktop)]
use crate::global_shortcut;

#[tauri::command]
pub async fn fetch_config() -> configs::Config {
    (*configs::get_data()).clone()
}

#[tauri::command]
pub async fn fetch_effect_info() -> configs::EffectInfo {
    let config = configs::get_data();
    let effective = config.effect();

    configs::EffectInfo {
        configured: config.window_effect,
        effective,
        available: window_effect::available(),
        alpha: window_effect::alpha(effective),
    }
}

/// 主窗口创建完成时是否应该自动显示
#[tauri::command]
pub fn should_show_main_auto() -> bool {
    configs::get_data().show_main_auto()
}

/// 保存整份配置：落盘、重新加载运行时配置
///
/// 返回是否需要重新注册全局快捷键（见 [`register_global_shortcut`]）；
/// 主窗口不跟着变，要重建窗口由前端调用 [`rebuild_main_window`]。
#[tauri::command]
pub fn save_config(app: tauri::AppHandle, config: serde_json::Value) -> Result<bool, String> {
    let config = configs::parse_full_config(config)?;
    config.validate()?;

    // 文件与运行时始终是同一份内容
    config.save(&app).map_err(|err| err.to_string())?;
    let _ = configs::reload_data(&app);

    #[cfg(desktop)]
    let needs_registration = global_shortcut::needs_registration(&app);
    #[cfg(not(desktop))]
    let needs_registration = false;

    Ok(needs_registration)
}

/// 重新注册全局快捷键（内部会与当前注册值比对）
#[tauri::command]
pub fn register_global_shortcut(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(desktop)]
    global_shortcut::register_global_shortcut(&app).map_err(|err| err.to_string())?;

    #[cfg(not(desktop))]
    let _ = app;

    Ok(())
}

/// 重建主窗口
///
/// 同步命令：窗口效果只能在主线程应用（见 [`crate::window_effect::apply`] ）。
#[tauri::command]
pub fn rebuild_main_window(app: tauri::AppHandle) -> Result<(), String> {
    window_utils::rebuild_main_window(&app).map_err(|err| err.to_string())
}

/// 无条件隐藏主窗口（实现见 [`window_utils::hide_main_window`]）
///
/// 摁下 ESC 始终隐藏，与 `main_window_mode` 无关。
#[tauri::command]
pub fn dismiss_main_window(app: tauri::AppHandle) -> Result<(), String> {
    window_utils::hide_main_window(&app).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    window_utils::show_main_window_now(&app).map_err(|err| err.to_string())
}

// #region 插件注册表
//
// 命令只做转发，实现留在 plugin_framework 的 PluginRegistry 里——那一层不知道 Tauri，
// 也就不该在这里出现半条检索或动作逻辑。

/// 使用关键字在插件注册表上检索（实现见 [`PluginRegistry::search`]）
///
/// 空串是合法输入：后端按「空关键字匹配所有」给出全部条目。
#[tauri::command]
pub async fn plugin_search(
    plugin_registry: State<'_, PluginRegistry>,
    k: &str,
) -> Result<plugin_framework::ItemSearchPage, ()> {
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

/// 列出已发现的 `Plugin Package`（实现见 [`crate::plugin_host::list_packages`]）
///
/// `id` / `name` / 入口绝对路径与"入口在不在"，用来核对扫描与重扫的结果。
#[tauri::command]
pub fn plugin_list_packages(app: tauri::AppHandle) -> Vec<crate::plugin_host::PackageInfo> {
    crate::plugin_host::list_packages(&app)
}

/// 重扫 `Plugin Folder`（实现见 [`crate::plugin_host::reload_packages`]）
///
/// 打包之后往安装目录里加一个包，走这一条就能被扫到，不必重新编译（Q3）。
#[tauri::command]
pub fn plugin_reload_packages(app: tauri::AppHandle) -> Vec<crate::plugin_host::PackageInfo> {
    crate::plugin_host::reload_packages(&app)
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

// #endregion
