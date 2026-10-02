//! 插件包的宿主侧状态（[`JsHost`]）：`Plugin Package` 的扫描结果、JS 插件结果行的缓存，
//! 以及一次 `Plugin Search` 的回程通道。它是宿主自己的账本，插件看不见，框架也不需要知道。
//!
//! 扫描同时认两种包：清单 `type` 是 `html` 的是前端插件（[`HtmlPlugin`]，条目直接开页面），
//! 是 `js` 的是 JS 插件（[`JsPlugin`]，条目开搜索页）。

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use serde::Serialize;
use tauri::{path::BaseDirectory, AppHandle, Emitter, Manager};
use tauri_plugin_log::log::{info, warn};

use crate::{
    constants,
    plugin_framework::{
        ActionId, ItemHandle, ItemSearchPage, PluginId, PluginItem, PluginItemDisplay,
        PluginRegistry,
    },
    plugin_package::{self, manifest::PackageType, PluginPackage},
    plugin_proxy_html::HtmlPlugin,
    plugin_proxy_js::{JsPlugin, RowCache, SearchRequest, SearchRow, RESULT_LOCAL_ID_BASE},
};

/// `Plugin Folder`：随包分发的插件目录，运行期可以往里加 `Plugin Package`
///
/// 路径交给 `resolve_resource`——它按 `bundle.resources` 的落点解析（开发态是 target 下的
/// 可执行文件目录，打包态是安装目录 / `Contents/Resources`），宿主不自己拼。
///
/// 目录不存在**不算错误**：应用照常起来，只是没有插件可扫。解析结果记一条 info：
/// 开发态与打包态落点不同（spec §1.1），这是唯一能直接看到它的地方。
fn plugins_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .resolve("plugins", BaseDirectory::Resource)
        .map_err(|err| err.to_string())?;

    info!(
        "plugin folder: path={} exists={}",
        dir.display(),
        dir.exists()
    );

    Ok(dir)
}

/// 一个已发现的 `Plugin Package`
///
/// 形状就是 `plugin_list_packages` 的下发形状：验证与重扫后核对都用它。
#[derive(Debug, Clone, Serialize)]
pub struct PackageInfo {
    pub id: String,
    pub name: String,
    /// 清单声明的包形态（`"js"` / `"html"`）
    pub the_type: PackageType,
    /// JS 入口的绝对路径：开发态与打包态落在哪，看这一条
    pub entry: String,
    /// 入口文件在不在：清单把 `entry` 指向不存在的文件时是 `false`，
    /// 但包照样被扫到（坏的是它自己的入口，不是框架）
    pub entry_exists: bool,
    /// HTML 页面的绝对路径；空串表示这个包没有 `html`
    pub html: String,
    /// HTML 页面在不在：写了 `html` 但文件不存在时是 `false`
    pub html_exists: bool,
}

/// 插件包的宿主侧运行期状态
///
/// 三张表一起用、一起换，所以合并到同一把 [`Mutex`]（与 [`PluginRegistry`] 同理）：
/// 不需要维护锁顺序，也不存在死锁。
pub struct JsHost {
    inner: Mutex<JsHostInner>,
}

struct JsHostInner {
    /// 已发现的包，按 `id` 字典序（扫描给的就是这个顺序）
    packages: Vec<PackageInfo>,
    /// 包 → 包目录：插件要按"包内相对路径"寻址时从这里出发
    dirs: HashMap<String, PathBuf>,
    /// 包 → 结果行缓存：与代理共用同一份，结果行的动作派发按它寻址
    rows: HashMap<String, RowCache>,
    /// 在飞的搜索：插件 id → 回程通道。同一个插件同时只允许一次搜索
    pending: HashMap<String, tauri::async_runtime::Sender<Vec<SearchRow>>>,
}

impl JsHost {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(JsHostInner {
                packages: Vec::new(),
                dirs: HashMap::new(),
                rows: HashMap::new(),
                pending: HashMap::new(),
            }),
        }
    }

    /// 换掉整份扫描结果；在飞的搜索不动（它不属于某一次扫描）
    fn set_packages(
        &self,
        packages: Vec<PackageInfo>,
        dirs: HashMap<String, PathBuf>,
        rows: HashMap<String, RowCache>,
    ) {
        let mut inner = self.inner.lock().unwrap_or_else(|err| err.into_inner());

        inner.packages = packages;
        inner.dirs = dirs;
        inner.rows = rows;
    }

    /// 已发现的包
    pub fn packages(&self) -> Vec<PackageInfo> {
        self.inner
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .packages
            .clone()
    }

    /// 某个包的 JS 入口绝对路径
    fn entry_of(&self, plugin_id: &str) -> Result<String, String> {
        self.inner
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .packages
            .iter()
            .find(|package| package.id == plugin_id)
            .map(|package| package.entry.clone())
            .ok_or_else(|| format!("plugin package not found: {plugin_id}"))
    }

    /// 一个包的名字：窗口没给标题时的缺省值
    pub fn name_of(&self, plugin_id: &str) -> String {
        self.inner
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .packages
            .iter()
            .find(|package| package.id == plugin_id)
            .map(|package| package.name.clone())
            .unwrap_or_else(|| plugin_id.to_string())
    }

    /// 把一个包内相对路径解析成绝对路径（见 [`crate::plugin_package::resolve_in_package`]）
    ///
    /// 相对路径是插件给的不可信输入，所以解析只能落在包目录里；包不存在直接报错。
    pub fn resolve_file(&self, plugin_id: &str, relative: &str) -> Result<PathBuf, String> {
        let dir = self
            .inner
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .dirs
            .get(plugin_id)
            .cloned()
            .ok_or_else(|| format!("plugin package not found: {plugin_id}"))?;

        plugin_package::resolve_in_package(&dir, relative)
    }

    /// 登记一次搜索并拿到回程的接收端
    ///
    /// 同一个插件的旧请求直接**换掉**：它多半已经死了（窗口重建、用户退出搜索页），
    /// 换掉之后那一头会拿到 `RecvError` 并记一条；留着它反而会把后来的搜索永远挡住。
    fn begin_search(
        &self,
        plugin_id: &str,
    ) -> Result<tauri::async_runtime::Receiver<Vec<SearchRow>>, String> {
        let mut inner = self.inner.lock().unwrap_or_else(|err| err.into_inner());

        // 容量 1：回程只有一条，发完就取走
        let (sender, receiver) = tauri::async_runtime::channel(1);
        inner.pending.insert(plugin_id.to_string(), sender);

        Ok(receiver)
    }

    /// 撤销一次登记：请求发不出去（窗口没了）时用，免得留一条永远没人等的通道
    fn cancel_search(&self, plugin_id: &str) {
        self.inner
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .pending
            .remove(plugin_id);
    }

    /// 回程：插件执行完搜索，把结果行交回来
    ///
    /// 没有在飞的请求说明这一份结果已经过期（用户已经退出搜索页），记 warn 丢掉。
    fn report(&self, plugin_id: &str, rows: Vec<SearchRow>) -> Result<(), String> {
        let sender = self
            .inner
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .pending
            .remove(plugin_id)
            .ok_or_else(|| format!("no plugin search in flight: {plugin_id}"))?;

        sender
            .try_send(rows)
            .map_err(|err| format!("send plugin search result failed: {err}"))
    }

    /// 记下最近一次搜索的结果行
    ///
    /// 框架不持有结果行（Q10），所以结果行的动作要能派发，就得在这里留一份：
    /// 代理按 `local_id` 查它，见 `plugin_proxy_js::JsPlugin::resolve_item`。
    fn set_rows(&self, plugin_id: &str, items: Vec<PluginItem>) {
        let inner = self.inner.lock().unwrap_or_else(|err| err.into_inner());

        match inner.rows.get(plugin_id) {
            Some(cache) => *cache.lock().unwrap_or_else(|err| err.into_inner()) = items,
            // 重扫之后代理与缓存一起换过，这时候收到旧请求的结果：丢掉
            None => warn!("plugin search result for an unknown package: {plugin_id}"),
        }
    }
}

impl Default for JsHost {
    fn default() -> Self {
        Self::new()
    }
}

/// 扫一遍 `Plugin Folder` 并把结果同步进框架与宿主状态
///
/// 按清单的 `type` 分成两种包：`html` 是前端插件，注册 [`HtmlPlugin`]；`js` 注册 [`JsPlugin`]。
/// 已存在的 `id` 换一份代理（清单可能改过），新出现的 `id` 直接注册；
/// 消失的包**保留**它的条目——这一轮不做移除（spec §2.5 / §7 开放问题）。
pub(super) fn sync_packages(app: &AppHandle, registry: &PluginRegistry) -> Vec<PackageInfo> {
    let folder = match plugins_dir(app) {
        Ok(folder) => folder,
        Err(err) => {
            warn!("plugin folder cannot be resolved: {err}");
            return Vec::new();
        }
    };

    let mut infos: Vec<PackageInfo> = Vec::new();
    let mut dirs_by_id: HashMap<String, PathBuf> = HashMap::new();
    let mut rows_by_id: HashMap<String, RowCache> = HashMap::new();

    for package in plugin_package::scan(&folder) {
        let info = package_info(&package);

        dirs_by_id.insert(info.id.clone(), package.dir.clone());

        if package.is_html() {
            registry.register_or_reload_plugin(Box::new(HtmlPlugin::new(app.clone(), package)));
        } else {
            // 结果行缓存由宿主建、与代理共用：宿主在回程里写入，代理在派发里读
            let rows: RowCache = Arc::new(Mutex::new(Vec::new()));
            rows_by_id.insert(info.id.clone(), Arc::clone(&rows));

            registry.register_or_reload_plugin(Box::new(JsPlugin::new(app.clone(), package, rows)));
        }

        infos.push(info);
    }

    app.state::<JsHost>()
        .set_packages(infos.clone(), dirs_by_id, rows_by_id);

    infos
}

/// 一个包的对外信息
fn package_info(package: &PluginPackage) -> PackageInfo {
    let entry = package.entry_path();
    let html = package.html_path();

    PackageInfo {
        id: package.manifest.id.clone(),
        name: package.manifest.name.clone(),
        the_type: package.manifest.the_type,
        entry: entry.to_string_lossy().into_owned(),
        entry_exists: entry.is_file(),
        html: html
            .as_deref()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default(),
        html_exists: html.as_deref().is_some_and(std::path::Path::is_file),
    }
}

/// 列出已发现的 `Plugin Package`（命令 [`crate::commands::plugin::plugin_list_packages`] 的实现）
pub fn list_packages(app: &AppHandle) -> Vec<PackageInfo> {
    app.state::<JsHost>().packages()
}

/// 重扫 `Plugin Folder`（命令 [`crate::commands::plugin::plugin_reload_packages`] 的实现）
///
/// 挂在**临时**托盘项上，供打包后加插件这一步核对（spec §2.5 / issue 04）。
pub fn reload_packages(app: &AppHandle) -> Vec<PackageInfo> {
    let registry = app.state::<PluginRegistry>();

    sync_packages(app, &registry)
}

/// 触发一次 `Plugin Search`：请求 → 回程 → 投影（spec §3.3）
///
/// 时序不变量是**先等插件报「已注册」、再发起搜索**：这一条由 webview 那一半的装载器保证
/// ——装载完成是一个 Promise，串在插件自己的搜索函数之前，所以 Rust 收到回程时，
/// 图标与文案一定已经注册好了。Rust 这边只发一次请求、等一次回程。
pub async fn open_plugin_search(
    app: &AppHandle,
    plugin_id: &str,
    keyword: &str,
) -> Result<ItemSearchPage, String> {
    // 取出要用的东西后立刻放掉 state 借用：下面要跨 await
    let (entry, mut receiver) = {
        let host = app.state::<JsHost>();
        let entry = host.entry_of(plugin_id)?;
        let receiver = host.begin_search(plugin_id)?;
        (entry, receiver)
    };

    let request = SearchRequest {
        plugin_id: plugin_id.to_string(),
        entry,
        keyword: keyword.to_string(),
    };

    if let Err(err) = app.emit_to(
        constants::LABEL_MAIN,
        constants::EVENT_PLUGIN_SEARCH_REQUEST,
        request,
    ) {
        app.state::<JsHost>().cancel_search(plugin_id);
        return Err(format!("emit plugin search request failed: {err}"));
    }

    let rows = receiver
        .recv()
        .await
        .ok_or_else(|| format!("plugin search request dropped: {plugin_id}"))?;

    Ok(search_page_of(app, plugin_id, rows))
}

/// 回程：插件执行完搜索后把结果行交回来
///
/// 命令 [`crate::commands::plugin::plugin_report_search_results`] 的实现，也是
/// [`open_plugin_search`] 等待的那一头。
pub fn report_search_results(
    app: &AppHandle,
    plugin_id: &str,
    rows: Vec<SearchRow>,
) -> Result<(), String> {
    app.state::<JsHost>().report(plugin_id, rows)
}

/// 把结果行投影成 `Plugin Search Page`
///
/// 结果行**不是**注册条目（Q10）：它不注册进框架、不由框架持有，是每次查询现算的，
/// 所以它走宿主自己这一条投影（注册条目走框架的 `project`）。两者的相同之处只在
/// "都由框架画成一行"——前端拿到的是同一个 [`PluginItemDisplay`] 形状。
///
/// 动作 id 由插件在结果行里自带（spec §3.3 的行形状）：清单里 `types` × `actions` 是
/// **声明**（派发时的白名单与 `label_key` 的来源），行可以一个动作都不带（spec §4.4）。
fn search_page_of(app: &AppHandle, plugin_id: &str, rows: Vec<SearchRow>) -> ItemSearchPage {
    let plugin_id = PluginId(plugin_id.to_string());

    let items: Vec<PluginItem> = rows
        .iter()
        .map(|row| {
            PluginItem::new(
                row.the_type.clone(),
                // 结果行不参与主检索的排序：优先级与关键字在这里没有意义
                0,
                Vec::new(),
                row.name.clone(),
                row.desc.clone(),
                // 结果行没有自带图片：那一张是"包内相对路径"的清单字段，只有插件条目才有
                String::new(),
            )
        })
        .collect();

    app.state::<JsHost>().set_rows(&plugin_id.0, items.clone());

    let total = items.len();
    let item_list = items
        .into_iter()
        .zip(rows)
        .enumerate()
        .map(|(row_id, (item, row))| PluginItemDisplay {
            item,
            // 结果页里没有"整集"：这一栏是行号（动作下标记账按它记），寻址一律走 `handle`
            item_index: row_id,
            action_ids: row.action_ids.into_iter().map(ActionId).collect(),
            handle: ItemHandle {
                plugin_id: plugin_id.clone(),
                // 结果行的序号排在注册条目之后：插件条目占 0，见 `RESULT_LOCAL_ID_BASE`
                local_id: RESULT_LOCAL_ID_BASE + row_id,
            },
        })
        .collect();

    ItemSearchPage {
        // 结果页是一次查询的整份答案：没有翻页令牌，也不分匹配模式组
        token: 0,
        total,
        index: 0,
        item_list,
        index_eq: 0,
        index_starts_with: 0,
        index_contains: 0,
        index_match: 0,
    }
}
