use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    App, AppHandle, Manager, Result, WebviewUrl, WebviewWindow,
};
use tauri_plugin_log::log::debug;

mod configs;
mod constants;

fn destory_main_window(app: &AppHandle) -> Result<()> {
    if let Some(w) = app.get_webview_window(constants::LABEL_MAIN) {
        w.close()?;
    }
    Ok(())
}

fn show_main_window(app: &AppHandle) -> Result<()> {
    if let Some(w) = app.get_webview_window(constants::LABEL_MAIN) {
        w.unminimize()?;
        w.show()?;
        w.set_focus()?;
    } else {
        create_main_window(app)?;
    }
    Ok(())
}

fn show_config_window(app: &AppHandle) -> Result<()> {
    if let Some(w) = app.get_webview_window(constants::LABEL_CONFIG) {
        w.unminimize()?;
        w.show()?;
        w.set_focus()?;
    } else {
        create_config_window(app)?;
    }
    Ok(())
}

fn create_main_window(app: &AppHandle) -> Result<()> {
    let conf = {
        let config = app.state::<configs::ConfigOuter>();
        let conf = config.0.read().expect("get config reader failed");
        conf.clone()
    };
    let url_name = if conf.custom_shadow {
        "index_main_frame.html"
    } else {
        "index_main.html"
    };
    debug!("create window {:}", url_name);

    let _ = WebviewWindow::builder(app, constants::LABEL_MAIN, WebviewUrl::App(url_name.into()))
        .title("wom")
        .decorations(conf.window_frame) // 原生框架
        .transparent(!conf.window_frame) // 透明
        .fullscreen(false) // 全屏
        .resizable(conf.window_frame) // 大小可变
        .inner_size(conf.main_width, conf.main_height)
        .center() // 居中
        .always_on_top(conf.always_on_top) // 置顶
        .visible(true) // 可见
        .shadow(!conf.custom_shadow)  // 尝试解决不能拖拽问题
        .build()?;
    Ok(())
}

fn create_config_window(app: &AppHandle) -> Result<()> {
    let _ = WebviewWindow::builder(
        app,
        constants::LABEL_CONFIG,
        WebviewUrl::App("index_config.html".into()),
    )
    .title("wom Config")
    .build()?;
    Ok(())
}

fn create_tray(app: &App) -> Result<()> {
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
                let _r = show_main_window(app);
                #[cfg(debug_assertions)]
                debug!("show_main_window {:?}", _r);
            }
            "reset_main" => {
                let _r = destory_main_window(app);
                #[cfg(debug_assertions)]
                debug!("destory_main_window {:?}", _r);
            }
            "open_config" => {
                let _ = show_config_window(app);
            }
            "register" => {
                todo!()
            }
            "unregister" => {
                todo!()
            }
            "reload" => {
                let conf: configs::ConfigInner = configs::ConfigSettings::load(app)
                    .expect("load config failed")
                    .into();
                {
                    let config = app.state::<configs::ConfigOuter>();
                    let mut conf_ref = config.0.write().expect("get config writer failed");
                    *conf_ref = conf;
                }
            }
            "quit" => {
                app.exit(0);
                debug!("try exit");
            }
            _ => {}
        })
        .build(app)?;
    Ok(())
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
            let config_settings: configs::ConfigSettings =
                configs::ConfigSettings::load(app.handle())?;
            let conf_inner: configs::ConfigInner = config_settings.into();
            let should_open_main = conf_inner.show_main_auto;
            let conf: configs::ConfigOuter = conf_inner.into();
            app.manage(conf);

            create_tray(app)?;

            if should_open_main {
                create_main_window(app.handle())?;
            }

            Ok(())
        })
        .on_window_event(|w, event| match event {
            tauri::WindowEvent::Focused(focused) => {
                if !focused && w.label() == constants::LABEL_MAIN {
                    let (should_hide, should_close) = {
                        let config = w.state::<configs::ConfigOuter>();
                        let conf = config.0.read().expect("get config reader failed");
                        (conf.hide_when_lost_focused, conf.close_when_lost_focused)
                    };

                    if should_hide {
                        debug!("will hide by lost_focused");
                        let _ = w.hide();
                    } else if should_close {
                        debug!("will close by lost_focused");
                        let _ = w.close();
                    }
                }
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                // 阻止所有窗口关闭时退出应用
                api.prevent_exit();
            }
            _ => {}
        });
}
