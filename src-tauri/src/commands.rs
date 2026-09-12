//! 前端调用的命令
//!
//! 放在这里而不是 `configs`：保存配置后要重建窗口，而依赖只能单向流动
//! （commands → configs / window_utils / global_shortcut），configs 不能反过来依赖 window_utils。

use tauri_plugin_log::log::warn;

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
