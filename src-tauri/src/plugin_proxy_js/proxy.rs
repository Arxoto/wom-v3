use std::{path::PathBuf, sync::Mutex};

use tauri::{AppHandle, Emitter};

use crate::{
    constants,
    plugin_framework::{
        ActionId, ActionOutcome, ItemHandle, ItemRegistrar, Plugin, PluginAction, PluginContext,
        PluginError, PluginId, PluginItem,
    },
    plugin_package::{self, manifest::PackageManifest, PluginPackage},
};

use super::protocol::{
    result_label_key, ActionRequest, RowCache, ACTION_OPEN_SEARCH, JS_PLUGIN_ITEM_TYPE,
    OPEN_SEARCH_LABEL_KEY, RESULT_LOCAL_ID_BASE,
};

/// 一个 `Plugin Package` 在框架里的代理
///
/// 清单在 `init` 里重读，所以重扫（[`crate::plugin_host::reload_packages`]）能改到名字、
/// 关键字与动作：框架那边的 `reload_plugin` 就是重新 `init`（Q11），这条语义刚好够用。
pub struct JsPlugin {
    app: AppHandle,
    /// 包的目录：清单每次 `init` 从这里重读
    dir: PathBuf,
    /// 当前生效的清单
    manifest: Mutex<PackageManifest>,
    /// 与宿主共用的结果行缓存
    rows: RowCache,
}

impl JsPlugin {
    /// `package` 是扫描时已经解析过的那一份：构造时就要有 `id`，框架注册前会先问它
    pub fn new(app: AppHandle, package: PluginPackage, rows: RowCache) -> Self {
        Self {
            app,
            dir: package.dir,
            manifest: Mutex::new(package.manifest),
            rows,
        }
    }

    /// 当前清单的一份快照
    ///
    /// 一律取快照再干活：`registrar.register` 里还会问一次 `id`，
    /// 抱着锁调用就是自己和自己抢锁。
    fn manifest_snapshot(&self) -> PackageManifest {
        self.manifest
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .clone()
    }
}

impl Plugin for JsPlugin {
    fn id(&self) -> PluginId {
        PluginId(self.manifest_snapshot().id)
    }

    /// 动作表 = 插件条目自己的那一个动作 + 清单里 `types` × `actions` 的笛卡尔积
    ///
    /// 于是框架侧的动作表**只由清单决定**，与插件的 JS 能不能跑起来无关（spec §2.3）。
    fn actions(&self) -> Vec<PluginAction> {
        let manifest = self.manifest_snapshot();

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
        let package = plugin_package::read_package(&self.dir)
            .map_err(|err| PluginError::Init(format!("read js plugin manifest failed: {err}")))?;

        let plugin_id = PluginId(package.manifest.id.clone());

        // 结果行属于上一次搜索：清单可能已经改了，留着就是过期的行
        self.rows
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .clear();

        let item = PluginItem::new(
            JS_PLUGIN_ITEM_TYPE,
            // 插件条目没有"优先级链"可谈：一个包就这一条，0 是它唯一可能的排序
            0,
            package.manifest.keywords.clone(),
            package.manifest.name.clone(),
            package.manifest.desc.clone(),
            // 清单里的图片是**包内相对路径**：这里拼成绝对路径再交给前端，
            // 前端把它转成 asset URL（宿主不碰文件内容，只给路径）
            package
                .icon_path()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default(),
        );

        registrar.register(&plugin_id, vec![item]);

        *self.manifest.lock().unwrap_or_else(|err| err.into_inner()) = package.manifest;

        cx.log_info(&format!("js plugin ready: {plugin_id}"));

        Ok(())
    }

    /// 结果行的动作要派发时，框架手上没有那一行（它不注册进框架，Q10）：从宿主写进来的
    /// 最近一次搜索结果里按 `local_id`（即行在结果里的下标）取回它
    ///
    /// 取不到就是过期的句柄——那一次搜索已经被下一次替换，或者包被重扫过。
    fn resolve_item(&self, handle: &ItemHandle) -> Option<PluginItem> {
        let row_id = handle.local_id.checked_sub(RESULT_LOCAL_ID_BASE)?;

        self.rows
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .get(row_id)
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

        let request = ActionRequest {
            plugin_id: handle.plugin_id.0.clone(),
            // 句柄是"插件条目 0 + 结果行 1.."，前端只认结果行自己的下标
            row_id: handle.local_id.saturating_sub(RESULT_LOCAL_ID_BASE),
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
