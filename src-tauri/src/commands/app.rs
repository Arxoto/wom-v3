use crate::{configs, window_effect, window_utils};

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
