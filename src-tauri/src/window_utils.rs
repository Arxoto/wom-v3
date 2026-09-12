use tauri::{AppHandle, LogicalSize, Manager, Result, WebviewUrl, WebviewWindow};
use tauri_plugin_log::log::{debug, warn};

use crate::{configs, constants, window_effect};

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

pub fn create_main_window(app: &AppHandle, shown: bool) -> Result<WebviewWindow> {
    let conf = configs::get_data();
    let effect = conf.effect();
    debug!("create window {:}", "index.html");

    let the_builder = WebviewWindow::builder(
        app,
        constants::LABEL_MAIN,
        WebviewUrl::App("index.html".into()),
    )
    .title("wom")
    .transparent(effect.transparent()) // 窗口透明
    .decorations(effect.window_frame()) // 原生框架
    .resizable(effect.window_frame()) // 大小可变
    .inner_size(conf.window_width(), conf.window_height())
    .fullscreen(false) // 全屏
    .center() // 居中
    .always_on_top(conf.always_on_top) // 置顶
    .visible(shown) // 初始可见
    .focused(shown); // 获取焦点

    // Platform 跨平台特性
    let the_builder = the_builder
        .shadow(true) // 系统原生阴影
        .skip_taskbar(!effect.window_frame()); // 在任务栏隐藏图标

    // 类原生应用
    // 禁用右键菜单 ContextMenu 在前端实现
    // 禁用快捷键（没有优雅实现，放开限制）
    // 禁用文本选择 css 实现
    let the_builder = the_builder
        .devtools(cfg!(debug_assertions)) // 禁用开发工具
        .zoom_hotkeys_enabled(false); // 禁用页面缩放

    let w = the_builder.build()?;
    // 毛玻璃效果，见 https://github.com/tauri-apps/window-vibrancy
    window_effect::apply(&w, effect);

    let w_handle = w.clone();
    w.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(focused) = event {
            let conf = configs::get_data();
            if conf.hide_main_unfocused() && !focused {
                let _ = w_handle.hide();
            }
        }
    });
    Ok(w)
}

/// 主窗口存在时，把尺寸同步到最新的布局配置
pub fn resize_main_window(app: &AppHandle, size: (f64, f64)) {
    let Some(window) = app.get_webview_window(constants::LABEL_MAIN) else {
        return;
    };

    if let Err(err) = window.set_size(LogicalSize::new(size.0, size.1)) {
        warn!("resize main window failed: {}", err);
    }
}

/// 按最新配置重建主窗口，保留可见性与位置
///
/// 窗口标签要等旧窗口真正销毁后才能复用，所以重建放在 `Destroyed` 回调里；
/// 该回调在主线程执行，正好满足窗口效果 `apply` 的主线程要求。
pub fn recreate_main_window(app: &AppHandle) -> Result<()> {
    let Some(window) = app.get_webview_window(constants::LABEL_MAIN) else {
        // 主窗口不存在（例如被托盘销毁过）：在主线程上补建
        let app_handle = app.clone();
        app.run_on_main_thread(move || {
            if let Err(err) = create_main_window(&app_handle, true) {
                warn!("create main window failed: {}", err);
            }
        })?;
        return Ok(());
    };

    let shown = window.is_visible().unwrap_or(true);
    let position = window.outer_position().ok();
    let app_handle = app.clone();

    window.on_window_event(move |event| {
        if !matches!(event, tauri::WindowEvent::Destroyed) {
            return;
        }

        match create_main_window(&app_handle, shown) {
            Ok(window) => {
                if let Some(position) = position {
                    let _ = window.set_position(position);
                }
            }
            Err(err) => warn!("recreate main window failed: {}", err),
        }
    });

    window.close()
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
