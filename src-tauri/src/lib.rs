use tauri::{plugin::TauriPlugin, Manager, Runtime};
use tauri_plugin_log::log::{debug, info};

mod app_stat;

mod constants;

mod shortcuts;

mod window_effect;

mod configs;

mod window_utils;

#[cfg(desktop)]
mod global_shortcut;

mod commands;

// 插件体系：框架层（framework）、第一个插件（launcher）与宿主侧接线（host）。
mod plugin_framework;

mod plugin_impl_launcher;

// 插件包：`Plugin Package` 的格式与扫描，加它在框架里的两种代理。
mod plugin_package;

mod plugin_proxy_package;

mod plugin_proxy_js;

mod plugin_proxy_html;

// 插件窗口：一个包在磁盘上的一个页面 → 一个窗口
mod plugin_window;

mod plugin_host;

mod tray;

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
        // 系统剪贴板
        .plugin(tauri_plugin_clipboard_manager::init())
        // 默认打开方式
        .plugin(tauri_plugin_opener::init())
        // 全局快捷键
        .plugin(global_shortcut::global_shortcut_handler_init())
        .plugin(global_shortcut::global_shortcut_startup_register())
        // 注册命令
        .invoke_handler(tauri::generate_handler![
            commands::app::fetch_config,
            commands::app::fetch_effect_info,
            commands::app::should_show_main_auto,
            commands::app::save_config,
            commands::app::register_global_shortcut,
            commands::app::rebuild_main_window,
            commands::app::dismiss_main_window,
            commands::app::show_main_window,
            // 插件体系那一套
            commands::plugin::plugin_search,
            commands::plugin::plugin_search_page,
            commands::plugin::plugin_run_item_action,
            commands::plugin::plugin_open_plugin_search,
            commands::plugin::plugin_report_search_results,
            commands::plugin::plugin_open_html_window,
        ])
        .setup(|app| {
            // 隐藏 Dock 图标， App 级配置，即使是打开配置窗口也不会出现在 Dock 栏
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // 主窗口重建状态：第一次建窗之前托管
            window_utils::init_state(app.handle());

            // 插件注册表：组装（注册 launcher 并读它的 manifest）也在建窗口前完成，
            // 检索命令以 `State<PluginRegistry>` 取用
            // 插件包宿主的运行期状态要先于注册表托管：组装注册表时要往里写扫描结果
            app.manage(plugin_host::PackageHost::new());
            app.manage(plugin_host::create_registry(app.handle())?);

            configs::load_data(app.handle());

            tray::create_tray(app)?;
            window_utils::create_main_window(app.handle())?;

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
