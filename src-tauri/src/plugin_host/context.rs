//! `PluginContext` 的 Tauri 实现：宿主给插件的那一层窄接口
//!
//! 插件的每一个动作都要经过它，所以这里也是"插件不许碰 Tauri"这条线的落点：
//! 换掉剪贴板或文件管理器的实现，只需要改这一个文件。

use std::path::{Path, PathBuf};

use tauri::{path::BaseDirectory, AppHandle, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_log::log::{info, warn};
use tauri_plugin_opener::OpenerExt;

use crate::plugin_framework::PluginContext;

/// 宿主给插件的上下文：一个 [`AppHandle`] 加一层窄接口
pub(super) struct HostPluginContext {
    app: AppHandle,
}

impl HostPluginContext {
    pub(super) fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl PluginContext for HostPluginContext {
    fn app_data_dir(&self) -> Result<PathBuf, String> {
        self.app
            .path()
            .app_data_dir()
            .map_err(|err| err.to_string())
    }

    fn resolve_base(&self, base: &str) -> Option<PathBuf> {
        // 变量名与各平台的具体目录都由 tauri 决定，宿主这边只做一次翻译：
        // 插件不自己维护变量名映射表，免得与平台差异脱节（spec §2.5）
        let base_directory = BaseDirectory::from_variable(base)?;

        // 空路径 join 上去就是基目录本身，扫描的相对路径由插件自己拼
        self.app.path().resolve("", base_directory).ok()
    }

    fn write_clipboard(&self, text: &str) -> Result<(), String> {
        self.app
            .clipboard()
            .write_text(text.to_string())
            .map_err(|err| err.to_string())
    }

    fn open_url(&self, url: &str) -> Result<(), String> {
        self.app
            .opener()
            .open_url(url, None::<&str>)
            .map_err(|err| err.to_string())
    }

    fn open_path(&self, path: &Path) -> Result<(), String> {
        self.app
            .opener()
            .open_path(path.to_string_lossy().into_owned(), None::<&str>)
            .map_err(|err| err.to_string())
    }

    fn reveal(&self, path: &Path) -> Result<(), String> {
        self.app
            .opener()
            .reveal_item_in_dir(path)
            .map_err(|err| err.to_string())
    }

    fn log_info(&self, msg: &str) {
        info!("{msg}");
    }

    fn log_warn(&self, msg: &str) {
        warn!("{msg}");
    }
}
