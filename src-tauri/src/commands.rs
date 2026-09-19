//! 前端调用的命令
//!
//! 实现的归属：能留在各自模块里的就只在这里转发（如检索命令，见 [`stat`]）；
//! 配置相关的命令要同时指挥 configs / window_utils / global_shortcut，实现直接落在这里——
//! 放进 `configs` 会让它反向依赖 `window_utils`，而依赖只能单向流动。
//!
//! 应用只有一张命令注册表：`lib.rs` 的 `invoke_handler`。

use tauri::State;
use tauri_plugin_log::log::warn;

use crate::{
    builtin_plugins::{
        action::{self, ItemActionTable},
        persistence::scan_base::{ScanBase, ScanBaseOption},
        search::ItemSearchPage,
        stat::{self, BuiltinStat},
    },
    configs, window_effect, window_utils,
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

/// 保存整份配置，并让它立刻生效
///
/// 同步命令：窗口效果只能在主线程应用（见 [`crate::window_effect::apply`] ）。
#[tauri::command]
pub fn set_config(app: tauri::AppHandle, config: serde_json::Value) -> Result<(), String> {
    let config = configs::parse_full_config(config)?;
    config.validate()?;

    // 文件与运行时始终是同一份内容
    config.save(&app).map_err(|err| err.to_string())?;

    // 配置真的变了就把主窗口删掉重新建立：新窗口启动时自然读到新配置
    if configs::reload_data(&app) {
        if let Err(err) = window_utils::recreate_main_window(&app) {
            warn!("recreate main window failed: {}", err);
        }
    }

    // 快捷键可能变了，重新注册（内部会与当前注册值比对）
    #[cfg(desktop)]
    global_shortcut::register_global_shortcut(&app).map_err(|err| err.to_string())?;

    Ok(())
}

/// 使用关键字进行检索（实现见 [`stat::search`]）
#[tauri::command]
pub async fn search(builtin_stat: State<'_, BuiltinStat>, k: &str) -> Result<ItemSearchPage, ()> {
    Ok(stat::search(&builtin_stat, k))
}

/// 无条件隐藏主窗口（实现见 [`window_utils::hide_main_window`]）
///
/// 它就是 ESC 那条显式意图，与 `main_window_mode` 无关。
#[tauri::command]
pub fn dismiss_main_window(app: tauri::AppHandle) -> Result<(), String> {
    window_utils::hide_main_window(&app).map_err(|err| err.to_string())
}

/// 对检索结果进行翻页（实现见 [`stat::search_page`]）
///
/// `k` 是前端正在展示的那份结论所属的关键字（见 spec §3 的「预请求」）。
#[tauri::command]
pub async fn search_page(
    builtin_stat: State<'_, BuiltinStat>,
    index: usize,
    k: &str,
) -> Result<ItemSearchPage, String> {
    stat::search_page(&builtin_stat, index, k)
}
