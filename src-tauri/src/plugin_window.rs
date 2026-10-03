//! 插件自己那个 HTML 窗口：窗口的创建、导航与承载页
//!
//! 前端插件条目的动作（`plugin_proxy_html`）与 JS 插件的 `open_window` 都落到这里，
//! 于是"一个包一个窗口、再开一次是换页 + 聚焦"只有这一份实现。
//! 窗口装的是应用自己的承载页 `index_iframe.html`，插件页面在它的 iframe 里打开：
//! 宿主只给页面**绝对路径**，asset URL 由承载页用 `convertFileSrc` 拼（与条目图标同一条路，
//! 见 `plugin_proxy_package`），Rust 侧不再自己编 URL。
//! 窗口用系统原生边框——这一轮的定位是"能打开、能看见"，自绘外框是后面的事。

use std::{
    collections::HashMap,
    path::Path,
    sync::Mutex,
};

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow};

use crate::constants;

/// 没给尺寸时的缺省大小
const DEFAULT_WIDTH: f64 = 800.0;
const DEFAULT_HEIGHT: f64 = 600.0;

const IFRAME_PAGE: &str = "index_iframe.html";

/// 每个插件窗口当前要显示的页面：窗口标签 → 页面绝对路径
pub struct PluginWindowPages {
    pages: Mutex<HashMap<String, String>>,
}

impl PluginWindowPages {
    pub fn new() -> Self {
        Self {
            pages: Mutex::new(HashMap::new()),
        }
    }

    fn set(&self, label: &str, page: String) {
        self.pages
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .insert(label.to_string(), page);
    }

    pub fn get(&self, label: &str) -> Option<String> {
        self.pages
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .get(label)
            .cloned()
    }
}

impl Default for PluginWindowPages {
    fn default() -> Self {
        Self::new()
    }
}

/// 打开（或聚焦）一个包在绝对路径上的一个页面
///
/// `title` 已由调用方定好；`width` / `height` 非正数时用缺省值。
pub fn open(
    app: &AppHandle,
    plugin_id: &str,
    path: &Path,
    title: &str,
    width: f64,
    height: f64,
) -> Result<(), String> {
    if !path.is_file() {
        return Err(format!("plugin html not found: {}", path.display()));
    }

    let label = format!("{}{plugin_id}", constants::PLUGIN_WINDOW_LABEL_PREFIX);

    app.state::<PluginWindowPages>()
        .set(&label, path.to_string_lossy().into_owned());

    // 已经开着就换页 + 聚焦：这个包的窗口只有这一个
    if let Some(window) = app.get_webview_window(&label) {
        window.set_title(title).map_err(|err| err.to_string())?;
        let url = window.url().map_err(|err| err.to_string())?;
        window.navigate(url).map_err(|err| err.to_string())?;
        window.show().map_err(|err| err.to_string())?;
        window.set_focus().map_err(|err| err.to_string())?;
        return Ok(());
    }

    let (width, height) = if width > 0.0 && height > 0.0 {
        (width, height)
    } else {
        (DEFAULT_WIDTH, DEFAULT_HEIGHT)
    };

    WebviewWindow::builder(app, label, WebviewUrl::App(IFRAME_PAGE.into()))
        .title(title)
        .inner_size(width, height)
        .min_inner_size(320.0, 240.0)
        .center()
        .on_navigation(allow_plugin_window_page)
        .build()
        .map_err(|err| err.to_string())?;

    Ok(())
}

/// 插件窗口的顶层只认应用自己的页面与 asset protocol 的资源
fn allow_plugin_window_page(url: &tauri::Url) -> bool {
    url.scheme() == "tauri"
        || url
            .host_str()
            .is_some_and(|host| host == "localhost" || host.ends_with(".localhost"))
}
