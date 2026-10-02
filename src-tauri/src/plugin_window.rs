//! 插件自己那个 HTML 窗口：窗口的创建、导航与 asset URL 的拼装
//!
//! 前端插件条目的动作（`plugin_proxy_html`）与 JS 插件的 `open_window` 都落到这里，
//! 于是"一个包一个窗口、再开一次是换页 + 聚焦"只有这一份实现。
//! 地址走 asset protocol：与插件代码的装载走同一条路（见 `docs/adr/0011`），宿主不读文件内容。
//! 窗口用系统原生边框——这一轮的定位是"能打开、能看见"，自绘外框是后面的事。
//!
//! 一个包一个窗口：标签由 `PluginId` 拼出，再次打开同一个包就导航过去并聚焦，不会一直堆窗口。

use std::path::Path;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow};

use crate::constants;

/// 没给尺寸时的缺省大小
const DEFAULT_WIDTH: f64 = 800.0;
const DEFAULT_HEIGHT: f64 = 600.0;

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

    let url = asset_url(path)?;
    let label = format!("{}{plugin_id}", constants::PLUGIN_WINDOW_LABEL_PREFIX);

    // 已经开着就换页 + 聚焦：这个包的窗口只有这一个
    if let Some(window) = app.get_webview_window(&label) {
        window.set_title(title).map_err(|err| err.to_string())?;
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

    WebviewWindow::builder(app, label, WebviewUrl::External(url))
        .title(title)
        .inner_size(width, height)
        .min_inner_size(320.0, 240.0)
        .center()
        .on_navigation(allow_asset_page)
        .build()
        .map_err(|err| err.to_string())?;

    Ok(())
}

/// 插件窗口只认 asset protocol 的页面：插件自己的 HTML 与它引用的同目录资源
fn allow_asset_page(url: &tauri::Url) -> bool {
    url.scheme() == "asset" || url.host_str() == Some("asset.localhost")
}

/// 把一个绝对路径转成 asset URL
///
/// 与前端 `convertFileSrc` 同一条路：Windows / Android 走 `http://asset.localhost/`，
/// 其余平台走 `asset://localhost/`；路径按 `encodeURIComponent` 的规则转义。
fn asset_url(path: &Path) -> Result<tauri::Url, String> {
    let encoded = percent_encode(&path.to_string_lossy());
    let raw = if cfg!(any(target_os = "windows", target_os = "android")) {
        format!("http://asset.localhost/{encoded}")
    } else {
        format!("asset://localhost/{encoded}")
    };

    tauri::Url::parse(&raw).map_err(|err| format!("plugin html url is invalid: {err}"))
}

/// `encodeURIComponent` 的等价实现：只留 unreserved 字符，其余按字节转义
fn percent_encode(text: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    let mut encoded = String::with_capacity(text.len());

    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric()
            || matches!(
                byte,
                b'-' | b'_' | b'.' | b'!' | b'~' | b'*' | b'\'' | b'(' | b')'
            )
        {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push(HEX[(byte >> 4) as usize] as char);
            encoded.push(HEX[(byte & 0x0f) as usize] as char);
        }
    }

    encoded
}
