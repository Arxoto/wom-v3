//! 前端调用的命令
//!
//! 实现的归属：能留在各自模块里的就只在这里转发（如检索命令，见 [`stat`]）；
//! 配置相关的命令要同时指挥 configs / window_utils / global_shortcut，实现直接落在这里——
//! 放进 `configs` 会让它反向依赖 `window_utils`，而依赖只能单向流动。
//!
//! 应用只有一张命令注册表：`lib.rs` 的 `invoke_handler`。
//!
//! 这一层现在有两套检索与动作：内建条目那一套（[`stat`] / [`action`]，`search`、
//! `run_item_action` …）与插件注册表那一套（`plugin_search`、`plugin_run_item_action` …）。
//! 两套并行是**有意**的（见 `docs/adr/0008`）：前端这一轮走插件那一套，内建那一套原样留着。
//! 参数与返回值的形状两边一致，前端换的只是命令名。

use tauri::State;

use crate::{
    builtin_plugins::{
        action::{self, ItemActionTable},
        persistence::scan_base::{ScanBase, ScanBaseOption},
        search::ItemSearchPage,
        stat::{self, BuiltinStat},
    },
    configs,
    plugin_framework::{self, ActionId, ActionOutcome, PluginRegistry},
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

/// 扫描根路径的可选项，配置页用来渲染下拉
///
/// 顺带给出每个变量在这台机器上解析出来的目录，前端可以直接显示
#[tauri::command]
pub async fn fetch_scan_base_options() -> Vec<ScanBaseOption> {
    ScanBase::options()
}

/// 每种 ItemType 支持的动作（实现见 [`action::table`]）
///
/// 顺序即优先级，第一个是默认动作；前端只在挂载时拉一次。
#[tauri::command]
pub async fn fetch_item_type_actions() -> ItemActionTable {
    action::table()
}

/// 按下标取出条目并跑它的一个 Item Action（实现见 [`stat::run_item_action`]）
///
/// 动作真的跑完，才轮到「按 `main_window_mode` 决定要不要隐藏窗口」这一问：
/// `HideAndShow` 隐藏，`Always` 留着（见 spec §3 / §4.3）。越界索引、认不出的动作
/// 与还没实现的动作都当无操作并记 warn——不 panic、不隐藏，前端不提示失败。
#[tauri::command]
pub fn run_item_action(
    app: tauri::AppHandle,
    builtin_stat: State<'_, BuiltinStat>,
    item_index: usize,
    action: String,
) -> Result<(), String> {
    if !stat::run_item_action(&app, &builtin_stat, item_index, &action) {
        return Ok(());
    }

    if configs::get_data().hide_main_after_action() {
        window_utils::hide_main_window(&app).map_err(|err| err.to_string())?;
    }

    Ok(())
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

/// 使用关键字进行检索（实现见 [`stat::search`]）
#[tauri::command]
pub async fn search(builtin_stat: State<'_, BuiltinStat>, k: &str) -> Result<ItemSearchPage, ()> {
    Ok(stat::search(&builtin_stat, k))
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

/// 对检索结果进行翻页（实现见 [`stat::search_page`]）
///
/// `token` 是后端在检索结果里下发的令牌：前端原样回传，不自己造（见 spec §3 的「预请求」）。
#[tauri::command]
pub async fn search_page(
    builtin_stat: State<'_, BuiltinStat>,
    index: usize,
    token: u32,
) -> Result<ItemSearchPage, String> {
    stat::search_page(&builtin_stat, index, token)
}

// #region 插件注册表（插件体系那一套）
//
// 与上面内建那一套并行：命令名带 `plugin_` 前缀，形状与语义一一对应（见本文件头的说明）。
// 命令只做转发，实现留在 plugin_framework 的 PluginRegistry 里——那一层不知道 Tauri，
// 也就不该在这里出现半条检索或动作逻辑。

/// 使用关键字在插件注册表上检索（实现见 [`PluginRegistry::search`]）
///
/// 空串与内建那条链一样是合法输入：后端按「空关键字匹配所有」给出全部条目。
#[tauri::command]
pub async fn plugin_search(
    plugin_registry: State<'_, PluginRegistry>,
    k: &str,
) -> Result<plugin_framework::ItemSearchPage, ()> {
    Ok(plugin_registry.search(k))
}

/// 对插件检索结果翻页（实现见 [`PluginRegistry::page`]）
///
/// `token` 的语义与 [`search_page`] 完全一致：后端下发、前端原样回传，对不上就报错。
#[tauri::command]
pub async fn plugin_search_page(
    plugin_registry: State<'_, PluginRegistry>,
    index: usize,
    token: u32,
) -> Result<plugin_framework::ItemSearchPage, String> {
    plugin_registry.page(index, token)
}

/// 插件注册的动作表：类型名 → 该类型的动作（实现见 [`PluginRegistry::action_table`]）
///
/// 与 [`fetch_item_type_actions`] 同一个形状，只是类型名的值由插件定义、只有插件自己解释；
/// 顺序即优先级，第一个是默认动作。前端同样只在挂载时拉一次。
#[tauri::command]
pub fn fetch_plugin_actions(
    plugin_registry: State<'_, PluginRegistry>,
) -> plugin_framework::ActionTableView {
    plugin_registry.action_table()
}

/// 按下标取出条目并跑它的一个 Plugin Action（实现见 [`PluginRegistry::run_action`]）
///
/// 与 [`run_item_action`] 同一套语义：只有动作**真的执行了**，才轮到「按 `main_window_mode`
/// 决定要不要隐藏窗口」这一问（见 spec §3 / §4.3）。越界索引、认不出的动作、没挂在该条目类型上的
/// 动作都当无操作并记 warn——不 panic、不隐藏，前端也不提示失败。
#[tauri::command]
pub fn plugin_run_item_action(
    app: tauri::AppHandle,
    plugin_registry: State<'_, PluginRegistry>,
    item_index: usize,
    action: String,
) -> Result<(), String> {
    if plugin_registry.run_action(item_index, &ActionId(action)) != ActionOutcome::Done {
        return Ok(());
    }

    if configs::get_data().hide_main_after_action() {
        window_utils::hide_main_window(&app).map_err(|err| err.to_string())?;
    }

    Ok(())
}

// #endregion
