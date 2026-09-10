use tauri::{plugin::TauriPlugin, Runtime};
use tauri_plugin_log::log::{debug, info};

mod constants;

mod shortcuts;

mod configs;

mod window_utils;

#[cfg(desktop)]
mod global_shortcut;

mod inner_plugins;

mod app_stat {
    use std::sync::atomic::{AtomicBool, Ordering};

    static RUNNING_STAT: AtomicBool = AtomicBool::new(true);

    pub(super) fn is_running() -> bool {
        RUNNING_STAT.load(Ordering::SeqCst)
    }

    pub(super) fn stop_running() {
        RUNNING_STAT.store(false, Ordering::SeqCst);
    }
}

mod tray {
    use tauri::{
        menu::{Menu, MenuItem, PredefinedMenuItem},
        tray::TrayIconBuilder,
        App, Result,
    };
    use tauri_plugin_log::log::{debug, info, warn};

    use crate::{app_stat, configs, global_shortcut, inner_plugins, window_utils};

    pub(super) fn create_tray(app: &App) -> Result<()> {
        let show_main_desc = "Show Main Window";
        let reset_main_desc = "Reset Main Window";
        let open_config_desc = "Open Config Window";
        let register_desc = "register Global-Shortcut";
        let unregister_desc = "unregister Global-Shortcut";
        let re_plugin_desc = "Reload Inner-Plugin Settings";
        let reload_desc = "Reload Global Config";
        let quit_desc = "Quit";

        let _tray = TrayIconBuilder::new()
            .icon(app.default_window_icon().unwrap().clone())
            .show_menu_on_left_click(true)
            .menu(&Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "show_main", show_main_desc, true, None::<&str>)?,
                    &MenuItem::with_id(app, "reset_main", reset_main_desc, true, None::<&str>)?,
                    &MenuItem::with_id(app, "open_config", open_config_desc, true, None::<&str>)?,
                    &PredefinedMenuItem::separator(app)?,
                    &MenuItem::with_id(app, "register", register_desc, true, None::<&str>)?,
                    &MenuItem::with_id(app, "unregister", unregister_desc, true, None::<&str>)?,
                    &PredefinedMenuItem::separator(app)?,
                    &MenuItem::with_id(app, "re_plugin", re_plugin_desc, true, None::<&str>)?,
                    &PredefinedMenuItem::separator(app)?,
                    &MenuItem::with_id(app, "reload", reload_desc, true, None::<&str>)?,
                    &MenuItem::with_id(app, "quit", quit_desc, true, None::<&str>)?,
                ],
            )?)
            .on_menu_event(|app, event| match event.id.as_ref() {
                "show_main" => {
                    let _r = window_utils::show_main_window(app);
                    #[cfg(debug_assertions)]
                    debug!("show_main_window {:?}", _r);
                }
                "reset_main" => {
                    let _r = window_utils::destory_main_window(app);
                    #[cfg(debug_assertions)]
                    debug!("destory_main_window {:?}", _r);
                }
                "open_config" => {
                    let _ = window_utils::show_config_window(app);
                }
                "register" => {
                    info!("try register_global_shortcut");
                    let r = global_shortcut::register_global_shortcut(app);
                    if let Err(e) = r {
                        warn!("Failed to register_global_shortcut: {}", e);
                        #[cfg(debug_assertions)]
                        debug!("Registration error details: {:?}", e);
                    }
                }
                "unregister" => {
                    info!("try unregister_global_shortcut");
                    let r = global_shortcut::unregister_global_shortcut(app);
                    if let Err(e) = r {
                        warn!("Failed to unregister_global_shortcut: {}", e);
                        #[cfg(debug_assertions)]
                        debug!("Unregistration error details: {:?}", e);
                    }
                }
                "re_plugin" => {
                    inner_plugins::reload_setting(app);
                }
                "reload" => {
                    info!("try reload config data");
                    configs::reload_data(app);
                }
                "quit" => {
                    debug!("try exit");
                    app_stat::stop_running();
                    app.exit(0);
                }
                _ => {}
            })
            .build(app)?;
        Ok(())
    }
}

/// 自定义日志打印和日志回滚
fn log_setting_init<R: Runtime>() -> TauriPlugin<R> {
    if cfg!(debug_assertions) {
        tauri_plugin_log::Builder::new()
            .targets([
                tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
            ])
            .level(tauri_plugin_log::log::LevelFilter::Debug)
            .level_for("tao", tauri_plugin_log::log::LevelFilter::Info) // 去除不必要的事件循环通知
            .build()
    } else {
        tauri_plugin_log::Builder::new()
            .targets([tauri_plugin_log::Target::new(
                tauri_plugin_log::TargetKind::LogDir { file_name: None },
            )])
            .level(tauri_plugin_log::log::LevelFilter::Warn)
            .max_file_size(1024 * 1024)
            .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(3))
            .build()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // 日志
        .plugin(log_setting_init())
        // 系统通知
        .plugin(tauri_plugin_notification::init())
        // 默认打开方式
        .plugin(tauri_plugin_opener::init())
        // 全局快捷键：系统级注册能力
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(global_shortcut::handler_global_shortcut)
                .build(),
        )
        // 全局快捷键：WOM 逻辑（托管状态 + 就绪时按配置注册）
        .plugin(global_shortcut::wom_global_shortcut_init())
        // 自定义插件
        .plugin(inner_plugins::init())
        .invoke_handler(tauri::generate_handler![configs::fetch_layout_config])
        .setup(|app| {
            configs::load_data(app.handle());
            let conf = configs::get_data();

            tray::create_tray(app)?;
            window_utils::create_main_window(app.handle(), conf.show_main_auto)?;

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                if app_stat::is_running() {
                    debug!("will not close");
                    api.prevent_exit(); // 阻止所有窗口关闭时退出应用
                } else {
                    info!("will close");
                }
            }
        });
}
