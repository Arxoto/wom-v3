use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    App, Result,
};
use tauri_plugin_log::log::{debug, info, warn};

use crate::{app_stat, configs, global_shortcut, plugin_host, window_utils};

pub fn create_tray(app: &App) -> Result<()> {
    let show_main_desc = "Show Main Window";
    let reset_main_desc = "Reset Main Window";
    let open_config_desc = "Open Config Window";
    let register_desc = "register Global-Shortcut";
    let unregister_desc = "unregister Global-Shortcut";
    let re_plugin_desc = "Reload Plugin Settings";
    // issue 04 的临时验证入口：打包之后往 `Plugin Folder` 里加一个包，
    // 走这一条确认能被扫到。转正或删除由接口定稿时决定（spec §五 / Q25）。
    let re_packages_desc = "Reload Plugin Packages";
    let reload_desc = "Reload Global Config";
    let quit_desc = "Quit";

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .show_menu_on_left_click(true)
        .menu(&Menu::with_items(
            app,
            &[
                // 尽量保证整齐不换行
                &MenuItem::with_id(app, "show_main", show_main_desc, true, None::<&str>)?,
                &MenuItem::with_id(app, "reset_main", reset_main_desc, true, None::<&str>)?,
                &MenuItem::with_id(app, "open_config", open_config_desc, true, None::<&str>)?,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(app, "register", register_desc, true, None::<&str>)?,
                &MenuItem::with_id(app, "unregister", unregister_desc, true, None::<&str>)?,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(app, "re_plugin", re_plugin_desc, true, None::<&str>)?,
                &MenuItem::with_id(app, "re_packages", re_packages_desc, true, None::<&str>)?,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(app, "reload", reload_desc, true, None::<&str>)?,
                &MenuItem::with_id(app, "quit", quit_desc, true, None::<&str>)?,
            ],
        )?)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show_main" => {
                let _r = window_utils::request_show_main_window(app);
                #[cfg(debug_assertions)]
                debug!("request_show_main_window {:?}", _r);
            }
            "reset_main" => {
                let _r = window_utils::reset_main_window(app);
                #[cfg(debug_assertions)]
                debug!("reset_main_window {:?}", _r);
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
                info!("try reload plugin setting");
                plugin_host::reload_launcher(app);
                rebuild_main(app);
            }
            // issue 04 的临时验证入口，见上面的菜单项说明
            "re_packages" => {
                info!("try reload plugin packages");
                let count = plugin_host::reload_packages(app);
                info!("plugin packages reloaded: {count}");
                rebuild_main(app);
            }
            "reload" => {
                info!("try reload config data");
                let _ = configs::reload_data(app);
                rebuild_main(app);
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

/// 显式触发，无论如何都重建窗口，否则可能认为没有触发
fn rebuild_main(app: &tauri::AppHandle) {
    if let Err(err) = window_utils::rebuild_main_window(app) {
        warn!("rebuild main window failed: {}", err);
    }
}
