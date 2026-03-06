use tauri_plugin_log::log::debug;

mod configs;
mod constants;

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

mod window {
    use tauri::{AppHandle, Manager, Result, WebviewUrl, WebviewWindow};
    use tauri_plugin_log::log::debug;

    use crate::configs;
    use crate::constants;

    pub(super) fn destory_main_window(app: &AppHandle) -> Result<()> {
        if let Some(w) = app.get_webview_window(constants::LABEL_MAIN) {
            w.close()?;
        }
        Ok(())
    }

    pub(super) fn show_main_window(app: &AppHandle) -> Result<()> {
        if let Some(w) = app.get_webview_window(constants::LABEL_MAIN) {
            w.unminimize()?;
            w.show()?;
            w.set_focus()?;
        } else {
            create_main_window(app, true)?;
        }
        Ok(())
    }

    pub(super) fn show_config_window(app: &AppHandle) -> Result<()> {
        if let Some(w) = app.get_webview_window(constants::LABEL_CONFIG) {
            w.unminimize()?;
            w.show()?;
            w.set_focus()?;
        } else {
            create_config_window(app)?;
        }
        Ok(())
    }

    // todo use window-vibrancy 毛玻璃效果（然后 css 中手动加上 1px 的边框） https://github.com/tauri-apps/window-vibrancy
    pub(super) fn create_main_window(app: &AppHandle, shown: bool) -> Result<()> {
        let conf = configs::get_data();
        let url_name = if conf.custom_shadow {
            "index_main_frame.html"
        } else {
            "index_main.html"
        };
        debug!("create window {:}", url_name);

        let the_builder = WebviewWindow::builder(app, constants::LABEL_MAIN, WebviewUrl::App(url_name.into()))
        .title("wom")
        .transparent(!conf.window_frame) // 窗口透明
        .decorations(conf.window_frame) // 原生框架
        .resizable(conf.window_frame) // 大小可变
        .inner_size(conf.main_width, conf.main_height)
        .fullscreen(false) // 全屏
        .center() // 居中
        .always_on_top(conf.always_on_top) // 置顶
        .visible(shown) // 初始可见
        .focused(shown) // 获取焦点
        ;
        // Platform 跨平台特性
        let the_builder = the_builder
        .shadow(!conf.custom_shadow) // 系统原生阴影
        .skip_taskbar(!conf.window_frame) // 在任务栏隐藏图标
        ;
        // 类原生应用
        let the_builder = the_builder
        .devtools(cfg!(debug_assertions)) // 禁用开发工具
        .zoom_hotkeys_enabled(false) // 禁用页面缩放
        // 禁用右键菜单 ContextMenu 在前端实现
        // 禁用快捷键（没有优雅实现，放开限制）
        // 禁用文本选择 css 实现
        ;
        let w = the_builder.build()?;
        let w_handle = w.clone();
        w.on_window_event(move |event| match event {
            tauri::WindowEvent::Focused(focused) => {
                let conf = configs::get_data();
                if conf.hide_main_unfocused && !focused {
                    let _ = w_handle.hide();
                }
            }
            _ => {}
        });
        Ok(())
    }

    pub(super) fn create_config_window(app: &AppHandle) -> Result<()> {
        let _ = WebviewWindow::builder(
            app,
            constants::LABEL_CONFIG,
            WebviewUrl::App("index_config.html".into()),
        )
        .title("wom Config")
        .build()?;
        Ok(())
    }
}

mod tray {
    use tauri::{
        menu::{Menu, MenuItem, PredefinedMenuItem},
        tray::TrayIconBuilder,
        App, Result,
    };
    use tauri_plugin_log::log::debug;

    use crate::{app_stat, configs, window};

    pub(super) fn create_tray(app: &App) -> Result<()> {
        let show_main_desc = "Show Main Window";
        let reset_main_desc = "Reset Main Window";
        let open_config_desc = "Open Config Window";
        let register_desc = "register Global-Shortcut";
        let unregister_desc = "unregister Global-Shortcut";
        let reload_desc = "Reload";
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
                    &MenuItem::with_id(app, "reload", reload_desc, true, None::<&str>)?,
                    &MenuItem::with_id(app, "quit", quit_desc, true, None::<&str>)?,
                ],
            )?)
            .on_menu_event(|app, event| match event.id.as_ref() {
                "show_main" => {
                    let _r = window::show_main_window(app);
                    #[cfg(debug_assertions)]
                    debug!("show_main_window {:?}", _r);
                }
                "reset_main" => {
                    let _r = window::destory_main_window(app);
                    #[cfg(debug_assertions)]
                    debug!("destory_main_window {:?}", _r);
                }
                "open_config" => {
                    let _ = window::show_config_window(app);
                }
                "register" => {
                    todo!()
                }
                "unregister" => {
                    todo!()
                }
                "reload" => {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(if cfg!(debug_assertions) {
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                ])
                .level(tauri_plugin_log::log::LevelFilter::Debug)
                .build()
        } else {
            tauri_plugin_log::Builder::new()
                .targets([tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir { file_name: None },
                )])
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build()
        })
        .setup(|app| {
            configs::load_data(app.handle());
            let conf = configs::get_data();

            tray::create_tray(app)?;
            window::create_main_window(app.handle(), conf.show_main_auto)?;

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                if app_stat::is_running() {
                    debug!("will not close");
                    api.prevent_exit(); // 阻止所有窗口关闭时退出应用
                } else {
                    debug!("will close");
                }
            }
            _ => {}
        });
}
