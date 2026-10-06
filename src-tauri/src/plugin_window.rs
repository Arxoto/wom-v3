//! 插件自己那个 HTML 窗口：窗口的创建与导航
//!
//! 前端插件条目的动作（`plugin_proxy_html`）与 JS 插件的 `open_window` 都落到这里，
//! 于是"一个包一个窗口、再开一次是换页 + 聚焦"只有这一份实现。
//! 窗口直接装插件自己包里的页面：asset URL 借用主窗口的 `convert_file_src` 换算（与前端
//! `convertFileSrc` 同一条路），Rust 侧不自己编 URL。
//! 页面外框由 `initialization_script` 注入。
//! 窗口用系统原生边框——这一轮的定位是"能打开、能看见"，自绘外框是后面的事。

use std::path::Path;

use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindow};

use crate::constants;

/// 没给尺寸时的缺省大小
const DEFAULT_WIDTH: f64 = 800.0;
const DEFAULT_HEIGHT: f64 = 600.0;

const PLUGIN_FRAME_SCRIPT: &str = r#"
class WomFrame extends HTMLElement {
    constructor() {
        super();
        this.attachShadow({ mode: "open" }).innerHTML = "<slot></slot>";
    }
}

customElements.define("wom-frame", WomFrame);
"#;

fn plugin_page_url(app: &AppHandle, path: &Path) -> Result<Url, String> {
    let main = app
        .get_webview_window(constants::LABEL_MAIN)
        .ok_or_else(|| format!("main window not found: {}", constants::LABEL_MAIN))?;
    let url = main
        .convert_file_src(path, None)
        .map_err(|err| err.to_string())?;

    Url::parse(&url).map_err(|err| err.to_string())
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

    let url = plugin_page_url(app, path)?;

    let label = format!("{}{plugin_id}", constants::PLUGIN_WINDOW_LABEL_PREFIX);

    // 已经开着就换页 + 聚焦：这个包的窗口只有这一个
    if let Some(window) = app.get_webview_window(&label) {
        window.navigate(url).map_err(|err| err.to_string())?;
        window.set_title(title).map_err(|err| err.to_string())?;
        window.show().map_err(|err| err.to_string())?;
        window.set_focus().map_err(|err| err.to_string())?;
        return Ok(());
    }

    let (width, height) = if width > 0.0 && height > 0.0 {
        (width, height)
    } else {
        (DEFAULT_WIDTH, DEFAULT_HEIGHT)
    };

    WebviewWindow::builder(app, label, WebviewUrl::CustomProtocol(url))
        .title(title)
        .inner_size(width, height)
        .min_inner_size(320.0, 240.0)
        .center()
        .initialization_script(PLUGIN_FRAME_SCRIPT)
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
