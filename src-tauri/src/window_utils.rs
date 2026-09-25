use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Result, WebviewUrl, WebviewWindow};
use tauri_plugin_log::log::{debug, warn};

use crate::{configs, constants, window_effect};

static REBUILD: Mutex<Rebuild> = Mutex::new(Rebuild::Idle);

/// 待办的主窗口重建
///
/// 窗口销毁是异步的，需要有个状态去保存
enum Rebuild {
    /// 没有待办的重建
    Idle,
    /// 重建后回到配置里的居中位置
    Centered,
    /// 重建后沿用旧窗口的位置
    KeepPosition(PhysicalPosition<i32>),
}

/// 登记一次重建
fn plan_rebuild(plan: Rebuild) {
    let mut state = REBUILD.lock().unwrap_or_else(|err| err.into_inner());
    *state = plan;
}

/// 取走待办的重建，没有待办时回 [`Rebuild::Idle`]
fn take_rebuild() -> Rebuild {
    let mut state = REBUILD.lock().unwrap_or_else(|err| err.into_inner());
    std::mem::replace(&mut *state, Rebuild::Idle)
}

fn rebuilding() -> bool {
    let state = REBUILD.lock().unwrap_or_else(|err| err.into_inner());
    match *state {
        Rebuild::Idle => false,
        Rebuild::Centered | Rebuild::KeepPosition(_) => true,
    }
}

pub fn toggle_main_window(app: &AppHandle) -> Result<()> {
    if let Some(w) = app.get_webview_window(constants::LABEL_MAIN) {
        if w.is_visible()? {
            return w.hide();
        }
    }
    request_show_main_window(app)
}

/// 通知前端准备显示窗口
pub fn request_show_main_window(app: &AppHandle) -> Result<()> {
    if app.get_webview_window(constants::LABEL_MAIN).is_none() {
        create_main_window(app)?;
    }

    show_main_window_now(app)
}

/// 显示主界面，并通知前端
pub fn show_main_window_now(app: &AppHandle) -> Result<()> {
    let Some(w) = app.get_webview_window(constants::LABEL_MAIN) else {
        return Ok(());
    };

    w.unminimize()?;
    w.show()?;
    w.set_focus()?;

    app.emit_to(constants::LABEL_MAIN, constants::EVENT_MAIN_SHOWN, ())
}

pub fn hide_main_window(app: &AppHandle) -> Result<()> {
    if let Some(w) = app.get_webview_window(constants::LABEL_MAIN) {
        w.hide()?;
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

pub fn create_main_window(app: &AppHandle) -> Result<WebviewWindow> {
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
    .center() // 居中
    .always_on_top(conf.always_on_top) // 置顶
    .transparent(true) // 窗口透明
    .decorations(false) // 前端绘制外框
    .background_color(tauri::window::Color(0, 0, 0, 0))
    // 尺寸固定：面板里 head / tail / item 的高度与数量都按配置算好，
    // 前端布局（尤其是列表区的高度与逐行下翻）依赖这个尺寸，拖拽改大小会把布局搞乱
    .resizable(false)
    .fullscreen(false)
    .maximizable(false)
    .minimizable(false)
    .inner_size(width, height)
    // 窗口创建时始终隐藏，前端准备好后进入统一的显示流程
    .visible(false)
    .focused(false)
    .on_navigation(allow_page);

    // Platform 平台不兼容的特性
    let the_builder = the_builder
        .shadow(true) // 系统原生阴影
        .skip_taskbar(true); // 任务栏隐藏图标

    // 类原生应用
    // 禁用右键菜单 ContextMenu 在前端实现
    // 禁用快捷键：前端实现（后端 rust 是 unsafe 代码）
    // 禁用文本选择 css 实现
    let the_builder = the_builder
        .devtools(cfg!(debug_assertions)) // 禁用开发工具
        .zoom_hotkeys_enabled(false); // 禁用页面缩放

    let w = the_builder.build()?;
    // 毛玻璃效果，见 https://github.com/tauri-apps/window-vibrancy
    window_effect::apply(&w, effect);

    let app_handle = app.clone();
    let w_handle = w.clone();
    w.on_window_event(move |event| match event {
        tauri::WindowEvent::CloseRequested { api, .. } => {
            if rebuilding() {
                // 重建时放行
                return;
            }
            api.prevent_close();
            let _ = w_handle.hide();
        }
        tauri::WindowEvent::Destroyed => {
            let position = match take_rebuild() {
                Rebuild::Idle => return,
                Rebuild::Centered => None,
                Rebuild::KeepPosition(position) => Some(position),
            };

            match create_main_window(&app_handle) {
                Ok(window) => {
                    if let Some(position) = position {
                        let _ = window.set_position(position);
                    }
                }
                Err(err) => warn!("rebuild main window failed: {}", err),
            }
        }
        tauri::WindowEvent::Focused(focused) => {
            if !focused {
                let conf = configs::get_data();
                if conf.hide_main_unfocused() {
                    let _ = w_handle.hide();
                }
            }
        }
        _ => {}
    });
    Ok(w)
}

/// 保留位置去重建主窗口
pub fn recreate_main_window(app: &AppHandle) -> Result<()> {
    rebuild_main_window(app, true)
}

/// 重置主窗口
pub fn reset_main_window(app: &AppHandle) -> Result<()> {
    rebuild_main_window(app, false)
}

/// 销毁并重建主窗口
fn rebuild_main_window(app: &AppHandle, keep_position: bool) -> Result<()> {
    let Some(window) = app.get_webview_window(constants::LABEL_MAIN) else {
        create_main_window(app)?;
        return Ok(());
    };

    // 已经有一次重建在跑：它读到的是最新配置，这次直接跳过
    if rebuilding() {
        return Ok(());
    }

    // todo 优化，能否让 position 在创建时生效，而不是单独维护一个重建状态

    // 位置只有这一刻问得出来，取不到就退回居中
    plan_rebuild(if keep_position {
        match window.outer_position() {
            Ok(position) => Rebuild::KeepPosition(position),
            Err(_) => Rebuild::Centered,
        }
    } else {
        Rebuild::Centered
    });

    if let Err(err) = window.close() {
        take_rebuild();
        return Err(err);
    }
    Ok(())
}

pub fn create_config_window(app: &AppHandle) -> Result<()> {
    let _ = WebviewWindow::builder(
        app,
        constants::LABEL_CONFIG,
        WebviewUrl::App("index_config.html".into()),
    )
    .title("wom Config")
    .on_navigation(allow_page)
    .build()?;
    Ok(())
}

fn allow_page(url: &tauri::Url) -> bool {
    if url.scheme() == "tauri" {
        return true;
    }
    if url.host_str() == Some("tauri.localhost") {
        return true;
    }
    if cfg!(dev) && url.host_str() == Some("localhost") {
        return true;
    }

    false
}
