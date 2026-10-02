//! JS 插件宿主：一个 `Plugin Package` 在框架里的代理 [`Plugin`]
//!
//! 插件的"两半"（spec §2.1）在这里见面：
//!
//! - **Rust 这一半**（本模块）：占住框架里的一个插件块。`id` / `actions` / `init` 全部只读
//!   清单，所以**条目能不能被搜到、动作表长什么样，都不依赖插件的 JS 能否成功执行**；
//! - **webview 那一半**（`src/plugins/host.ts` + `src/plugins/plugin_worker.ts`）：为这个包起一个
//!   Worker，在里面装载 `index.js`、交出类型与动作的图标和文案、跑插件自己的搜索与动作。
//!
//! 两半共用同一个 `PluginId`——清单的 `id`。没有这个代理，插件注册的条目一旦被触发，
//! 框架会走到一个不存在的插件上（spec §2.1）。
//!
//! 与 `plugin_impl_launcher` 的差别是这里**持有一个 [`AppHandle`]**：搜索与结果行的动作
//! 最后要回到 webview 里跑，这是一条只能由宿主侧发起的路。框架层（`plugin_framework`）依旧
//! 不依赖 Tauri，宿主侧的那一半才是碰 webview 的地方。
//!
//! 分工：
//!
//! - [`crate::plugin_package::manifest`]：清单的一行长什么样、怎么校验；
//! - [`crate::plugin_package`]：`Plugin Folder` 怎么扫成一个包的列表；
//! - 本文件：代理 [`JsPlugin`] 本身。

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::{
    constants,
    plugin_framework::{
        ActionId, ActionOutcome, ItemHandle, ItemRegistrar, Plugin, PluginAction, PluginContext,
        PluginError, PluginId, PluginItem,
    },
    plugin_package::{self, manifest::PackageManifest, PluginPackage},
};

/// 插件条目（Plugin Item）的类型名
///
/// 它是**宿主自己定的**一个类型名，不是清单里的：清单的 `types` 是结果行的类型。
/// 框架对类型名永远不解释，所以宿主当然可以有一条自己的类型——前端注册表里
/// （`src/plugins/js_host.tsx`）给它配图标与文案。
pub const JS_PLUGIN_ITEM_TYPE: &str = "js_plugin";

/// 插件条目上唯一的动作：打开这个包的 `Plugin Search Page`
///
/// 真正打开搜索页由前端直接发起（Enter 走 `plugin_open_plugin_search`），
/// 这个动作 id 的作用是让那一行有动作可显示、可提示。
pub const ACTION_OPEN_SEARCH: &str = "open_search";

/// 插件条目动作的文案键
///
/// 四段式（Q35）里的"插件"这一段写宿主自己的名字：这个动作不是插件给的，
/// 是宿主给插件条目的。
pub const OPEN_SEARCH_LABEL_KEY: &str = "action.js_host.js_plugin.open_search";

/// 结果行的 `local_id` 从它开始：`0` 是插件条目自己（注册时推上去的那一条）
///
/// 句柄是"插件 + 插件内序号"，两套条目共用同一段序号会撞：框架在自己手上找不到时
/// 才回头问插件（[`Plugin::resolve_item`]），所以结果行的序号必须落在注册条目之后。
/// 一个包注册的条目只有一条，于是结果行的序号就是 `行号 + 1`。
pub const RESULT_LOCAL_ID_BASE: usize = 1;

/// 结果行（`Plugin Search Result`）的一条：插件自己报上来的纯数据（spec §3.3）
///
/// 全部字段都给缺省值：这是**从 webview 来的不可信数据**，缺一个字段不该让整次搜索炸掉，
/// 顶多是那一行画不出图标、点不出动作。
#[derive(Debug, Clone, Deserialize)]
pub struct SearchRow {
    /// 行类型名：前端据它查类型图标与动作文案
    #[serde(default)]
    pub the_type: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub desc: String,
    /// 这一行自己的动作 id 列表，顺序即优先级；空表示这一行没有动作（spec §4.4）
    #[serde(default)]
    pub action_ids: Vec<String>,
}

/// 发往 webview 的搜索请求
///
/// 带上入口的绝对路径：前端拿它 `convertFileSrc`，交给插件 Worker 用 `importScripts` 装载，
/// 于是取文件与执行都是 webview 自己的行为（ADR-0011）。
#[derive(Debug, Clone, Serialize)]
pub struct SearchRequest {
    pub plugin_id: String,
    pub entry: String,
    pub keyword: String,
}

/// 发往 webview 的动作请求
///
/// 只报身份（插件 + 行在结果里的下标 + 动作 id）：行内容在前端自己报上去的那一份里，
/// 不必来回搬。
#[derive(Debug, Clone, Serialize)]
pub struct ActionRequest {
    pub plugin_id: String,
    /// 行在最近一次搜索结果里的下标（前端的账本按它查那一行）
    pub row_id: usize,
    pub action_id: String,
}

/// 结果行的动作文案键：四段式 `action.<插件 id>.<行类型>.<动作 id>`（spec §1.3 末）
///
/// 清单能给出"有哪些类型、哪些动作"，但给不出中文——中文只住在前端（AGENTS.md）。
/// 前端按同一套键从插件**装载时**交上来的文案表里取中文（见 `src/plugins/host.ts`）。
pub fn result_label_key(plugin_id: &str, the_type: &str, action_id: &str) -> String {
    format!("action.{plugin_id}.{the_type}.{action_id}")
}

/// 一个包最近一次搜索结果的行缓存
///
/// `Plugin Search Result` 不注册进框架（Q10），所以框架手上没有它；结果行的动作要能派发，
/// 就得有人在派发时按 `local_id` 找回那一行。这份缓存由宿主建、代理与宿主共用同一份：
/// 宿主在回程里写入，代理在 [`Plugin::resolve_item`] 里读。
pub type RowCache = Arc<Mutex<Vec<PluginItem>>>;

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
            package.icon_path().map(|path| path.to_string_lossy().into_owned()).unwrap_or_default(),
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

        match self
            .app
            .emit_to(constants::LABEL_MAIN, constants::EVENT_PLUGIN_ACTION_REQUEST, request)
        {
            Ok(()) => ActionOutcome::Done,
            Err(err) => {
                cx.log_warn(&format!("js plugin action request failed: {err}"));
                ActionOutcome::Failed
            }
        }
    }
}
