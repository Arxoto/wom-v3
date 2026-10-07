//! 前端插件（清单 `type` 是 `html` 的包）在框架里的代理
//!
//! 与 JS 插件的代理（[`crate::plugin_proxy_js`]）共用 [`PackageProxy`]：同样持有包目录、
//! 在 `init` 里重读清单、只注册一条宿主定类型的条目。差别只在条目类型与动作：
//! 触发它的唯一动作直接打开清单里那个页面，不走 webview 的搜索与动作回程。
//!
//! 打开页面复用 [`crate::plugin_window`] 那条路："一个包一个窗口"与 JS 插件的
//! `open_window` 同一条；打开这个动作本身不需要额外放行，插件窗口自己那份 capability 见
//! `docs/adr/0015`。

use tauri::AppHandle;

use crate::{
    plugin_framework::{
        ActionId, ActionOutcome, ItemHandle, ItemRegistrar, Plugin, PluginAction, PluginContext,
        PluginError, PluginId, PluginItem,
    },
    plugin_package::{self, PluginPackage},
    plugin_proxy_package::PackageProxy,
};

pub const HTML_PLUGIN_ITEM_TYPE: &str = "html_plugin";

pub const ACTION_OPEN_HTML: &str = "open_html";

pub const OPEN_HTML_LABEL_KEY: &str = "action.html_host.html_plugin.open_html";

pub struct HtmlPlugin {
    app: AppHandle,
    /// 目录、清单与"清单 → 条目"的换算：与 JS 插件共用（见 [`PackageProxy`]）
    proxy: PackageProxy,
}

impl HtmlPlugin {
    pub fn new(app: AppHandle, package: PluginPackage) -> Self {
        Self {
            app,
            proxy: PackageProxy::new(package),
        }
    }
}

impl Plugin for HtmlPlugin {
    fn id(&self) -> PluginId {
        self.proxy.plugin_id()
    }

    fn actions(&self) -> Vec<PluginAction> {
        vec![PluginAction::new(
            HTML_PLUGIN_ITEM_TYPE,
            ACTION_OPEN_HTML,
            OPEN_HTML_LABEL_KEY,
        )]
    }

    fn init(
        &self,
        cx: &dyn PluginContext,
        registrar: &mut dyn ItemRegistrar,
    ) -> Result<(), PluginError> {
        let package = self
            .proxy
            .reload()
            .map_err(|err| PluginError::new(format!("read html plugin manifest failed: {err}")))?;

        if !package.is_html() {
            return Err(PluginError::new("manifest type is not html"));
        }

        let plugin_id = PluginId(package.manifest.id.clone());

        registrar.register(vec![PackageProxy::item(&package, HTML_PLUGIN_ITEM_TYPE)]);

        cx.log_info(&format!("html plugin ready: {plugin_id}"));

        Ok(())
    }

    fn run_action(
        &self,
        cx: &dyn PluginContext,
        _item: &PluginItem,
        _handle: &ItemHandle,
        _action_id: &ActionId,
    ) -> ActionOutcome {
        let manifest = self.proxy.snapshot();

        let path = match plugin_package::resolve_in_package(self.proxy.dir(), &manifest.html) {
            Ok(path) => path,
            Err(err) => {
                cx.log_warn(&format!("html plugin page path rejected: {err}"));
                return ActionOutcome::Failed;
            }
        };

        match crate::plugin_window::open(&self.app, &manifest.id, &path, &manifest.name, 0.0, 0.0) {
            Ok(()) => ActionOutcome::Done,
            Err(err) => {
                cx.log_warn(&format!("html plugin open page failed: {err}"));
                ActionOutcome::Failed
            }
        }
    }
}
