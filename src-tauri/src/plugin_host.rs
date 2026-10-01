//! 插件框架的宿主侧接线：把 Tauri 的能力包成插件能用的窄接口，并把注册表交给应用托管
//!
//! 框架层不依赖 Tauri（Q3），插件也不依赖；两者之间只有 [`PluginContext`] 这一个横切面，
//! 所以它的实现只能落在宿主这一侧，也就是这里。依赖方向是单向的：
//! 本模块 → `plugin_framework` / `plugin_impl_launcher`，反过来这两个模块都不认识它。
//!
//! 组装（[`create_registry`]）在应用 `setup` 里、建窗口之前完成，与内建条目的
//! `builtin_plugins::load_stat` 并列；托盘菜单的插件重载（[`reload_launcher`]）与命令层的
//! `State<PluginRegistry>` 都从这里出发。

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use tauri::{path::BaseDirectory, AppHandle, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_log::log::{info, warn};
use tauri_plugin_opener::OpenerExt;

use crate::{
    plugin_framework::{Plugin, PluginContext, PluginError, PluginRegistry},
    plugin_impl_launcher::LauncherPlugin,
};

/// 宿主给插件的上下文：一个 [`AppHandle`] 加一层窄接口
///
/// 插件的每一个动作都要经过它，所以这里也是"插件不许碰 Tauri"这条线的落点：
/// 换掉剪贴板或文件管理器的实现，只需要改这一个文件。
struct HostPluginContext {
    app: AppHandle,
}

impl PluginContext for HostPluginContext {
    fn app_data_dir(&self) -> Result<PathBuf, String> {
        self.app.path().app_data_dir().map_err(|err| err.to_string())
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

/// 组装注册表并注册全部插件；由应用 `setup` 调用，返回值交给 `manage` 托管
///
/// 这里**只**会因为应用数据目录取不到而失败（框架在最外层兜一道）。某个插件自己的 `init`
/// 出错不算失败：框架记 warn 并跳过那个插件，其余插件照常注册，启动不受影响（Q14）。
pub fn create_registry(app: &AppHandle) -> Result<PluginRegistry, PluginError> {
    let cx: Arc<dyn PluginContext> = Arc::new(HostPluginContext { app: app.clone() });
    let mut registry = PluginRegistry::new(cx)?;

    registry.register_plugin(Box::new(LauncherPlugin::new()));

    Ok(registry)
}

/// 重新 `init` launcher 插件：manifest 改过之后走这一条，不必重启应用
///
/// 与内建设置的重载（`builtin_plugins::reload_setting`）一起挂在托盘菜单上：
/// 两套体系并行期间，那一个菜单项要把两边都重新读一遍。
pub fn reload_launcher(app: &AppHandle) {
    let plugin_id = LauncherPlugin::new().id();

    app.state::<PluginRegistry>().reload_plugin(&plugin_id);
}
