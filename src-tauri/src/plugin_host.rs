//! 插件框架的宿主侧接线：把 Tauri 的能力包成插件能用的窄接口，并把注册表交给应用托管
//!
//! 框架层不依赖 Tauri（Q3），插件也不依赖；两者之间只有 [`PluginContext`] 这一个横切面，
//! 所以它的实现只能落在宿主这一侧。依赖方向是单向的：
//! 本模块 → `plugin_framework` / `plugin_impl_launcher` / `plugin_package` / `plugin_proxy_js`，
//! 反过来这些模块都不认识它。
//!
//! 组装（[`create_registry`]）在应用 `setup` 里、建窗口之前完成；托盘菜单的插件重载
//! （[`reload_launcher`]）与命令层的 `State<PluginRegistry>` 都从这里出发。
//!
//! 分工：
//!
//! - 本文件：注册表组装与重载；
//! - [`context`]：`PluginContext` 的 Tauri 实现；
//! - [`js`]：JS 插件的宿主侧——扫描同步、运行期账本与搜索回程；
//! - [`window`]：插件自己那个 HTML 窗口。

mod context;
mod js;
mod window;

pub use js::{
    list_packages, open_plugin_search, reload_packages, report_search_results, JsHost, PackageInfo,
};
pub use window::open_html_window;

use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::{
    plugin_framework::{Plugin, PluginContext, PluginError, PluginRegistry},
    plugin_impl_launcher::LauncherPlugin,
};

/// 组装注册表并注册全部插件；由应用 `setup` 调用，返回值交给 `manage` 托管
///
/// 这里**只**会因为应用数据目录取不到而失败（框架在最外层兜一道）。某个插件自己的 `init`
/// 出错不算失败：框架记 warn 并跳过那个插件，其余插件照常注册，启动不受影响（Q14）。
///
/// `Plugin Package` 的扫描也在这一步同步完成：条目与动作表全部来自清单，
/// 与插件的 JS 能不能跑起来无关（spec §1.3 末）。
pub fn create_registry(app: &AppHandle) -> Result<PluginRegistry, PluginError> {
    let cx: Arc<dyn PluginContext> = Arc::new(context::HostPluginContext::new(app.clone()));
    let mut registry = PluginRegistry::new(cx)?;

    registry.register_plugin(Box::new(LauncherPlugin::new()));

    js::sync_js_packages(app, &registry);

    Ok(registry)
}

/// 重新 `init` launcher 插件：manifest 改过之后走这一条，不必重启应用
pub fn reload_launcher(app: &AppHandle) {
    let plugin_id = LauncherPlugin::new().id();

    app.state::<PluginRegistry>().reload_plugin(&plugin_id);
}
