use tauri::{AppHandle, Emitter};

use crate::{
    constants,
    plugin_framework::{
        ActionId, ActionOutcome, ItemAddress, ItemHandle, ItemRegistrar, Plugin, PluginAction,
        PluginContext, PluginError, PluginId, PluginItem,
    },
    plugin_package::PluginPackage,
    plugin_proxy_package::PackageProxy,
};

use super::protocol::{
    result_label_key, ActionRequest, RowCache, ACTION_OPEN_SEARCH, JS_PLUGIN_ITEM_TYPE,
    OPEN_SEARCH_LABEL_KEY,
};

/// 一个 `Plugin Package` 在框架里的代理
///
/// 清单在 `init` 里重读，所以重扫（[`crate::plugin_host::reload_packages`]）能改到名字、
/// 关键字与动作：框架那边的 `reload_plugin` 就是重新 `init`（Q11），这条语义刚好够用。
pub struct JsPlugin {
    app: AppHandle,
    /// 目录、清单与"清单 → 条目"的换算：与前端插件共用（见 [`PackageProxy`]）
    proxy: PackageProxy,
    /// 与宿主共用的结果行缓存
    rows: RowCache,
}

impl JsPlugin {
    /// `package` 是扫描时已经解析过的那一份：构造时就要有 `id`，框架注册前会先问它
    pub fn new(app: AppHandle, package: PluginPackage, rows: RowCache) -> Self {
        Self {
            app,
            proxy: PackageProxy::new(package),
            rows,
        }
    }
}

impl Plugin for JsPlugin {
    fn id(&self) -> PluginId {
        self.proxy.plugin_id()
    }

    /// 动作表 = 插件条目自己的那一个动作 + 清单里 `types` × `actions` 的笛卡尔积
    ///
    /// 于是框架侧的动作表**只由清单决定**，与插件的 JS 能不能跑起来无关（spec §2.3）。
    fn actions(&self) -> Vec<PluginAction> {
        let manifest = self.proxy.snapshot();

        let mut actions = vec![PluginAction::new(
            JS_PLUGIN_ITEM_TYPE,
            ACTION_OPEN_SEARCH,
            OPEN_SEARCH_LABEL_KEY,
        )];

        for the_type in &manifest.types {
            for action_id in &manifest.actions {
                actions.push(PluginAction::new(
                    the_type,
                    action_id,
                    result_label_key(&manifest.id, the_type, action_id),
                ));
            }
        }

        actions
    }

    /// 一次性推入**它唯一的那一条** Plugin Item（spec §2.2）
    ///
    /// 走的是现成的 [`ItemRegistrar`]，框架的公开面一行不扩；条目全部来自清单，
    /// 所以"插件能不能被搜到"不依赖它的代码能否执行。
    fn init(
        &self,
        cx: &dyn PluginContext,
        registrar: &mut dyn ItemRegistrar,
    ) -> Result<(), PluginError> {
        // 重扫走这条：清单重读一遍，改过的名字 / 关键字 / 动作跟着生效（spec §2.5）
        let package = self
            .proxy
            .reload()
            .map_err(|err| PluginError::new(format!("read js plugin manifest failed: {err}")))?;

        let plugin_id = PluginId(package.manifest.id.clone());

        // 结果行属于上一次搜索：清单可能已经改了，留着就是过期的行
        self.rows
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .clear();

        registrar.register(vec![PackageProxy::item(&package, JS_PLUGIN_ITEM_TYPE)]);

        cx.log_info(&format!("js plugin ready: {plugin_id}"));

        Ok(())
    }

    /// 结果行的动作要派发时，框架手上没有那一行（它不注册进框架，Q10）：从宿主写进来的
    /// 最近一次搜索结果里按行下标取回它
    ///
    /// 取不到就是过期的句柄——那一次搜索已经被下一次替换，或者包被重扫过。
    fn resolve_item(&self, handle: &ItemHandle) -> Option<PluginItem> {
        let ItemAddress::Row { index } = handle.address else {
            return None;
        };

        self.rows
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .get(index)
            .cloned()
    }

    /// 结果行的动作送回 webview 跑
    ///
    /// 这里**不等回程**：[`Plugin::run_action`] 是同步的（第一轮的 trait），而插件的处理函数
    /// 在 webview 里。请求发出去就算"派发过了"，插件自己的失败由它记日志——
    /// 这一轮不加"动作失败原因"这条回程（Q6）。
    fn run_action(
        &self,
        cx: &dyn PluginContext,
        item: &PluginItem,
        handle: &ItemHandle,
        action_id: &ActionId,
    ) -> ActionOutcome {
        // 插件条目自己：打开搜索页由前端直接发起，这条路没有可跑的动作
        if item.the_type == JS_PLUGIN_ITEM_TYPE {
            cx.log_warn(&format!(
                "js plugin item has no dispatched action: {action_id}"
            ));
            return ActionOutcome::NoOp;
        }

        let ItemAddress::Row { index: row_id } = handle.address else {
            cx.log_warn(&format!("js plugin action on a non-row item: {action_id}"));
            return ActionOutcome::NoOp;
        };

        let request = ActionRequest {
            plugin_id: handle.plugin_id.0.clone(),
            row_id,
            action_id: action_id.0.clone(),
        };

        match self.app.emit_to(
            constants::LABEL_MAIN,
            constants::EVENT_PLUGIN_ACTION_REQUEST,
            request,
        ) {
            Ok(()) => ActionOutcome::Done,
            Err(err) => {
                cx.log_warn(&format!("js plugin action request failed: {err}"));
                ActionOutcome::Failed
            }
        }
    }
}
