use tauri::{AppHandle, Manager, Result, WebviewUrl, WebviewWindow};
use tauri_plugin_log::log::debug;

use crate::{configs, constants};

pub fn destory_main_window(app: &AppHandle) -> Result<()> {
    if let Some(w) = app.get_webview_window(constants::LABEL_MAIN) {
        w.close()?;
    }
    Ok(())
}

pub fn show_hide_main_window(app: &AppHandle) -> Result<()> {
    if let Some(w) = app.get_webview_window(constants::LABEL_MAIN) {
        if w.is_visible()? {
            w.hide()?
        } else {
            w.unminimize()?;
            w.show()?;
            w.set_focus()?;
        }
    } else {
        create_main_window(app, true)?;
    }
    Ok(())
}

pub fn show_main_window(app: &AppHandle) -> Result<()> {
    if let Some(w) = app.get_webview_window(constants::LABEL_MAIN) {
        w.unminimize()?;
        w.show()?;
        w.set_focus()?;
    } else {
        create_main_window(app, true)?;
    }
    Ok(())
}

pub fn show_config_window(app: &AppHandle) -> Result<()> {
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
pub fn create_main_window(app: &AppHandle, shown: bool) -> Result<()> {
    let conf = configs::get_data();
    let url_name = if conf.custom_shadow {
        "index_frame.html"
    } else {
        "index.html"
    };
    debug!("create window {:}", url_name);

    let the_builder =
        WebviewWindow::builder(app, constants::LABEL_MAIN, WebviewUrl::App(url_name.into()))
            .title("wom")
            .transparent(!conf.window_frame) // 窗口透明
            .decorations(conf.window_frame) // 原生框架
            .resizable(conf.window_frame) // 大小可变
            .inner_size(conf.main_width, conf.main_height)
            .fullscreen(false) // 全屏
            .center() // 居中
            .always_on_top(conf.always_on_top) // 置顶
            .visible(shown) // 初始可见
            .focused(shown); // 获取焦点

    // Platform 跨平台特性
    let the_builder = the_builder
        .shadow(!conf.custom_shadow) // 系统原生阴影
        .skip_taskbar(!conf.window_frame); // 在任务栏隐藏图标

    // 类原生应用
    // 禁用右键菜单 ContextMenu 在前端实现
    // 禁用快捷键（没有优雅实现，放开限制）
    // 禁用文本选择 css 实现
    let the_builder = the_builder
        .devtools(cfg!(debug_assertions)) // 禁用开发工具
        .zoom_hotkeys_enabled(false); // 禁用页面缩放

    let w = the_builder.build()?;
    let w_handle = w.clone();
    w.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(focused) = event {
            let conf = configs::get_data();
            if conf.hide_main_unfocused && !focused {
                let _ = w_handle.hide();
            }
        }
    });
    Ok(())
}

pub fn create_config_window(app: &AppHandle) -> Result<()> {
    let _ = WebviewWindow::builder(
        app,
        constants::LABEL_CONFIG,
        WebviewUrl::App("index_config.html".into()),
    )
    .title("wom Config")
    .build()?;
    Ok(())
}
