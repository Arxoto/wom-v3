use tauri::{AppHandle, Manager, Result, WebviewUrl, WebviewWindow};
use tauri_plugin_log::log::{debug, warn};

use crate::{configs, constants, window_effect};

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
        create_main_window(app, true, true)?;
    }
    Ok(())
}

pub fn show_main_window(app: &AppHandle) -> Result<()> {
    if let Some(w) = app.get_webview_window(constants::LABEL_MAIN) {
        w.unminimize()?;
        w.show()?;
        w.set_focus()?;
    } else {
        create_main_window(app, true, true)?;
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

pub fn create_main_window(app: &AppHandle, shown: bool, focused: bool) -> Result<WebviewWindow> {
    let conf = configs::get_data();
    let effect = conf.effect();
    let (width, height) = conf.window_size();
    debug!("create window {:}", "index.html");

    let the_builder = WebviewWindow::builder(
        app,
        constants::LABEL_MAIN,
        WebviewUrl::App("index.html".into()),
    )
    .title("wom")
    .transparent(effect.transparent()) // 窗口透明
    .decorations(effect.window_frame()) // 原生框架
    // 尺寸固定：面板里 head / tail / item 的高度与数量都按配置算好，
    // 前端布局（尤其是列表区的高度与逐行下翻）依赖这个尺寸，拖拽改大小会把布局搞乱
    .resizable(false)
    .maximizable(false) // 最大化同样会改尺寸，一并关掉
    .inner_size(width, height)
    .fullscreen(false) // 全屏
    .center() // 居中
    .always_on_top(conf.always_on_top) // 置顶
    .visible(shown) // 初始可见
    .focused(focused); // 获取焦点

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

/// 按最新配置重建主窗口，保留可见性与位置
///
/// 窗口标签要等旧窗口真正销毁后才能复用，所以重建放在 `Destroyed` 回调里；
/// 该回调在主线程执行，正好满足窗口效果 `apply` 的主线程要求。
/// 主窗口不存在时什么都不做——它下次被创建时本来就会读到最新配置。
pub fn recreate_main_window(app: &AppHandle) -> Result<()> {
    rebuild_main_window(app, true)
}

/// 重置主窗口：按最新配置重建，位置回到配置里的居中位置，不沿用旧位置
///
/// 同一个菜单项也可能用于把跑到屏幕外的窗口救回来，所以这里不抄旧位置。
pub fn reset_main_window(app: &AppHandle) -> Result<()> {
    rebuild_main_window(app, false)
}

/// 销毁并重建主窗口；`keep_position` 为 true 时把旧窗口的位置抄给新窗口
fn rebuild_main_window(app: &AppHandle, keep_position: bool) -> Result<()> {
    let Some(window) = app.get_webview_window(constants::LABEL_MAIN) else {
        return Ok(());
    };

    // 取不到可见性就按隐藏处理：宁可少弹一次，也不要凭空出现在屏幕上
    let shown = window.is_visible().unwrap_or(false);
    let position = if keep_position {
        window.outer_position().ok()
    } else {
        None
    };
    let app_handle = app.clone();

    window.on_window_event(move |event| {
        if !matches!(event, tauri::WindowEvent::Destroyed) {
            return;
        }

        // 重建不抢焦点：保存配置或重置时，主窗口都不该跳到前台
        match create_main_window(&app_handle, shown, false) {
            Ok(window) => {
                if let Some(position) = position {
                    let _ = window.set_position(position);
                }
            }
            Err(err) => warn!("rebuild main window failed: {}", err),
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
