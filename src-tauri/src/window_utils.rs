use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter, Manager, Result, WebviewUrl, WebviewWindow};
use tauri_plugin_log::log::{debug, warn};
use tauri_plugin_opener::OpenerExt;

use crate::{configs, constants, window_effect};

static PENDING_SHOW: AtomicBool = AtomicBool::new(false);
static PENDING_FOCUS: AtomicBool = AtomicBool::new(false);
static REBUILDING: AtomicBool = AtomicBool::new(false);

pub fn show_hide_main_window(app: &AppHandle) -> Result<()> {
    if let Some(w) = app.get_webview_window(constants::LABEL_MAIN) {
        if w.is_visible()? {
            return w.hide();
        }
    }
    request_show_main_window(app, true)
}

pub fn request_show_main_window(app: &AppHandle, focus: bool) -> Result<()> {
    if app.get_webview_window(constants::LABEL_MAIN).is_none() {
        create_main_window(app, true, focus)?;
        return Ok(());
    }

    PENDING_SHOW.store(true, Ordering::SeqCst);
    PENDING_FOCUS.store(focus, Ordering::SeqCst);
    emit_main_will_show(app)
}

pub fn show_main_window_now(app: &AppHandle) -> Result<()> {
    if !PENDING_SHOW.swap(false, Ordering::SeqCst) {
        return Ok(());
    }
    let focused = PENDING_FOCUS.swap(false, Ordering::SeqCst);

    let Some(w) = app.get_webview_window(constants::LABEL_MAIN) else {
        return Ok(());
    };

    w.unminimize()?;
    w.show()?;
    if focused {
        w.set_focus()?;
    }
    emit_main_shown(app)
}

fn emit_main_will_show(app: &AppHandle) -> Result<()> {
    app.emit_to(constants::LABEL_MAIN, constants::EVENT_MAIN_WILL_SHOW, ())
}

/// 通知前端「主窗口刚被显示」
///
/// 前端收到后关掉 Preview、聚焦并全选输入框（见 spec §4.5）。
/// 想要显示时先发的是 will-show（见 [`emit_main_will_show`]）：先让前端把入场动效起好，
/// 前端回过话来窗口才真的亮，这条在那之后才发。
fn emit_main_shown(app: &AppHandle) -> Result<()> {
    app.emit_to(constants::LABEL_MAIN, constants::EVENT_MAIN_SHOWN, ())
}

/// 无条件隐藏主窗口
///
/// ESC 那条显式意图：不看 `main_window_mode`，窗口不存在时什么都不做。
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

pub fn create_main_window(app: &AppHandle, shown: bool, focused: bool) -> Result<WebviewWindow> {
    let conf = configs::get_data();
    let effect = conf.effect();
    let (width, height) = conf.window_size();
    debug!("create window {:}", "index.html");

    let on_navigation_app = app.clone();
    let the_builder = WebviewWindow::builder(
        app,
        constants::LABEL_MAIN,
        WebviewUrl::App("index.html".into()),
    )
    .title("wom")
    .transparent(true) // 窗口透明
    .decorations(false) // 面板自己画外框
    // 尺寸固定：面板里 head / tail / item 的高度与数量都按配置算好，
    // 前端布局（尤其是列表区的高度与逐行下翻）依赖这个尺寸，拖拽改大小会把布局搞乱
    .resizable(false)
    .maximizable(false) // 最大化同样会改尺寸，一并关掉
    .inner_size(width, height)
    .fullscreen(false) // 全屏
    .center() // 居中
    .always_on_top(conf.always_on_top) // 置顶
    .visible(false) // 等前端把页面摆好、回过话来才显示（见 show_main_window_now）
    .focused(false)
    .background_color(tauri::window::Color(0, 0, 0, 0))
    .on_navigation(move |url| allow_page(&on_navigation_app, url));

    // Platform 跨平台特性
    let the_builder = the_builder
        .shadow(true) // 系统原生阴影
        .skip_taskbar(true); // 在任务栏隐藏图标

    // 类原生应用
    // 禁用右键菜单 ContextMenu 在前端实现
    // 禁用快捷键：Windows 侧关掉网页加速键（见 apply_native_webview_settings），其余平台由前端兜底
    // 禁用文本选择 css 实现
    let the_builder = the_builder
        .devtools(cfg!(debug_assertions)) // 禁用开发工具
        .zoom_hotkeys_enabled(false); // 禁用页面缩放

    PENDING_SHOW.store(shown, Ordering::SeqCst);
    PENDING_FOCUS.store(focused, Ordering::SeqCst);

    let w = the_builder.build()?;
    // 毛玻璃效果，见 https://github.com/tauri-apps/window-vibrancy
    window_effect::apply(&w, effect);
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    apply_native_webview_settings(&w);
    #[cfg(target_os = "macos")]
    w.set_visible_on_all_workspaces(true)?;

    let w_handle = w.clone();
    w.on_window_event(move |event| match event {
        tauri::WindowEvent::CloseRequested { api, .. } => {
            if REBUILDING.swap(false, Ordering::SeqCst) {
                return;
            }
            api.prevent_close();
            let _ = w_handle.hide();
        }
        tauri::WindowEvent::Focused(focused) => {
            let conf = configs::get_data();
            if conf.hide_main_unfocused() && !*focused {
                let _ = w_handle.hide();
            }
        }
        _ => {}
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

    REBUILDING.store(true, Ordering::SeqCst);
    window.close()
}

pub fn create_config_window(app: &AppHandle) -> Result<()> {
    let on_navigation_app = app.clone();
    let w = WebviewWindow::builder(
        app,
        constants::LABEL_CONFIG,
        WebviewUrl::App("index_config.html".into()),
    )
    .title("wom Config")
    .on_navigation(move |url| allow_page(&on_navigation_app, url))
    .build()?;
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    apply_native_webview_settings(&w);
    Ok(())
}

fn allow_page(app: &AppHandle, url: &tauri::Url) -> bool {
    if url.scheme() == "tauri" || url.host_str() == Some("tauri.localhost") {
        return true;
    }
    if cfg!(dev) && url.host_str() == Some("localhost") {
        return true;
    }

    if matches!(url.scheme(), "http" | "https") {
        if let Err(err) = app.opener().open_url(url.as_str(), None::<&str>) {
            warn!("open url {} failed: {}", url, err);
        }
    }
    false
}

#[cfg(target_os = "windows")]
fn apply_native_webview_settings(window: &WebviewWindow) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        ICoreWebView2Settings3, ICoreWebView2Settings4, ICoreWebView2Settings5,
        ICoreWebView2Settings6,
    };
    use windows_core::Interface;

    let result = window.with_webview(|webview| unsafe {
        let Ok(core) = webview.controller().CoreWebView2() else {
            return;
        };
        let Ok(settings) = core.Settings() else {
            return;
        };

        let _ = settings.SetAreDefaultContextMenusEnabled(false);
        let _ = settings.SetIsStatusBarEnabled(false);
        let _ = settings.SetIsZoomControlEnabled(false);

        if let Ok(settings) = settings.cast::<ICoreWebView2Settings3>() {
            let _ = settings.SetAreBrowserAcceleratorKeysEnabled(false);
        }
        if let Ok(settings) = settings.cast::<ICoreWebView2Settings4>() {
            let _ = settings.SetIsPasswordAutosaveEnabled(false);
            let _ = settings.SetIsGeneralAutofillEnabled(false);
        }
        if let Ok(settings) = settings.cast::<ICoreWebView2Settings5>() {
            let _ = settings.SetIsPinchZoomEnabled(false);
        }
        if let Ok(settings) = settings.cast::<ICoreWebView2Settings6>() {
            let _ = settings.SetIsSwipeNavigationEnabled(false);
        }
    });

    if let Err(err) = result {
        warn!("apply native webview settings failed: {}", err);
    }
}

#[cfg(target_os = "macos")]
fn apply_native_webview_settings(window: &WebviewWindow) {
    use objc2_web_kit::WKWebView;

    let result = window.with_webview(|webview| unsafe {
        let view = webview.inner().cast::<WKWebView>();
        if view.is_null() {
            return;
        }

        let view: &WKWebView = &*view;
        view.setAllowsMagnification(false);
        view.setAllowsBackForwardNavigationGestures(false);
        view.setAllowsLinkPreview(false);
    });

    if let Err(err) = result {
        warn!("apply native webview settings failed: {}", err);
    }
}
