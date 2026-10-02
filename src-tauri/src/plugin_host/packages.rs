//! 插件包的宿主侧状态（[`PackageHost`]）：`Plugin Package` 的扫描结果与 JS 插件结果行的缓存。
//! 它是宿主自己的账本，插件看不见，框架也不需要知道。
//!
//! 扫描同时认两种包：清单 `type` 是 `html` 的是前端插件（[`HtmlPlugin`]，条目直接开页面），
//! 是 `js` 的是 JS 插件（[`JsPlugin`]，条目开搜索页）。

use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use tauri::{path::BaseDirectory, AppHandle, Manager};
use tauri_plugin_log::log::{info, warn};

use crate::{
    plugin_framework::{PluginId, PluginItem, PluginRegistry},
    plugin_package::{self, PluginPackage},
    plugin_proxy_html::HtmlPlugin,
    plugin_proxy_js::{JsPlugin, RowCache, SearchRow},
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

/// 一个包在宿主侧的账目：名字、入口、目录与（JS 插件的）结果行
struct PackageEntry {
    /// 清单里的包名：窗口没给标题时的缺省值
    name: String,
    /// JS 入口的绝对路径：开发态与打包态落在哪，看这一条
    entry: String,
    /// 包目录：插件要按"包内相对路径"寻址时从这里出发
    dir: PathBuf,
    /// 结果行缓存：与代理共用同一份，结果行的动作派发按它寻址；前端插件没有
    rows: Option<RowCache>,
}

/// 插件包的宿主侧运行期状态
///
/// 条目按 `id` 字典序（扫描给的就是这个顺序），反查与结果行都挂在这一个键上，
/// 三份平行表会各自走丢，一份条目只可能一起换。
pub struct PackageHost {
    inner: Mutex<PackageHostInner>,
}

struct PackageHostInner {
    /// 已发现的包，按 `id` 字典序
    entries: BTreeMap<String, PackageEntry>,
    /// 在飞的搜索：插件 id → 回程通道。同一个插件同时只允许一次搜索
    pending: HashMap<String, tauri::async_runtime::Sender<Vec<SearchRow>>>,
}

impl PackageHost {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(PackageHostInner {
                entries: BTreeMap::new(),
                pending: HashMap::new(),
            }),
        }
    }

    /// 换掉整份扫描结果；在飞的搜索不动（它不属于某一次扫描）
    fn set_packages(&self, entries: BTreeMap<String, PackageEntry>) {
        let mut inner = self.inner.lock().unwrap_or_else(|err| err.into_inner());

        inner.entries = entries;
    }

    /// 已发现的包 id，按字典序
    fn ids(&self) -> Vec<String> {
        self.inner
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .entries
            .keys()
            .cloned()
            .collect()
    }

    /// 某个包的 JS 入口绝对路径
    pub(super) fn entry_of(&self, plugin_id: &str) -> Result<String, String> {
        self.inner
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .entries
            .get(plugin_id)
            .map(|entry| entry.entry.clone())
            .ok_or_else(|| format!("plugin package not found: {plugin_id}"))
    }

    /// 一个包的名字：窗口没给标题时的缺省值
    pub fn name_of(&self, plugin_id: &str) -> String {
        self.inner
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .entries
            .get(plugin_id)
            .map(|entry| entry.name.clone())
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
            .entries
            .get(plugin_id)
            .map(|entry| entry.dir.clone())
            .ok_or_else(|| format!("plugin package not found: {plugin_id}"))?;

        plugin_package::resolve_in_package(&dir, relative)
    }

    /// 登记一次搜索并拿到回程的接收端
    ///
    /// 同一个插件的旧请求直接**换掉**：它多半已经死了（窗口重建、用户退出搜索页），
    /// 换掉之后那一头会拿到 `RecvError` 并记一条；留着它反而会把后来的搜索永远挡住。
    pub(super) fn begin_search(
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
    pub(super) fn cancel_search(&self, plugin_id: &str) {
        self.inner
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .pending
            .remove(plugin_id);
    }

    /// 回程：插件执行完搜索，把结果行交回来
    ///
    /// 没有在飞的请求说明这一份结果已经过期（用户已经退出搜索页），记 warn 丢掉。
    pub(super) fn report(&self, plugin_id: &str, rows: Vec<SearchRow>) -> Result<(), String> {
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
    /// 代理按行下标查它，见 `plugin_proxy_js::JsPlugin::resolve_item`。
    pub(super) fn set_rows(&self, plugin_id: &str, items: Vec<PluginItem>) {
        let inner = self.inner.lock().unwrap_or_else(|err| err.into_inner());

        match inner
            .entries
            .get(plugin_id)
            .and_then(|entry| entry.rows.as_ref())
        {
            Some(cache) => *cache.lock().unwrap_or_else(|err| err.into_inner()) = items,
            // 重扫之后代理与缓存一起换过，这时候收到旧请求的结果：丢掉
            None => warn!("plugin search result for an unknown package: {plugin_id}"),
        }
    }
}

impl Default for PackageHost {
    fn default() -> Self {
        Self::new()
    }
}

/// 扫一遍 `Plugin Folder` 并把结果同步进框架与宿主状态，返回扫到的包数
///
/// 按清单的 `type` 分成两种包：`html` 是前端插件，注册 [`HtmlPlugin`]；`js` 注册 [`JsPlugin`]。
/// 已存在的 `id` 换一份代理（清单可能改过），新出现的 `id` 直接注册，
/// 扫不到的 `id` 连同它的条目一起摘掉（spec §2.5 / §7 开放问题）。
pub(super) fn sync_packages(app: &AppHandle, registry: &PluginRegistry) -> usize {
    let folder = match plugins_dir(app) {
        Ok(folder) => folder,
        Err(err) => {
            warn!("plugin folder cannot be resolved: {err}");
            return 0;
        }
    };

    let previous = app.state::<PackageHost>().ids();
    let mut entries: BTreeMap<String, PackageEntry> = BTreeMap::new();

    for package in plugin_package::scan(&folder) {
        let id = package.manifest.id.clone();

        // 结果行缓存由宿主建、与代理共用：宿主在回程里写入，代理在派发里读
        let rows = (!package.is_html()).then(|| Arc::new(Mutex::new(Vec::new())));
        let entry = package_entry(&package, rows.clone());

        match &rows {
            Some(rows) => registry.register_or_reload_plugin(Box::new(JsPlugin::new(
                app.clone(),
                package,
                Arc::clone(rows),
            ))),
            None => {
                registry.register_or_reload_plugin(Box::new(HtmlPlugin::new(app.clone(), package)))
            }
        }

        entries.insert(id, entry);
    }

    let removed: Vec<String> = previous
        .into_iter()
        .filter(|id| !entries.contains_key(id))
        .collect();
    let count = entries.len();
    let host = app.state::<PackageHost>();

    host.set_packages(entries);

    for id in removed {
        // 这个包的在飞搜索没人等了，撤掉登记；迟到的结果按"没有在飞的搜索"丢掉
        host.cancel_search(&id);

        if registry.remove_plugin(&PluginId(id.clone())) {
            info!("plugin package removed: {id}");
        }
    }

    count
}

/// 一个包在宿主侧的一行账
fn package_entry(package: &PluginPackage, rows: Option<RowCache>) -> PackageEntry {
    PackageEntry {
        name: package.manifest.name.clone(),
        entry: package.entry_path().to_string_lossy().into_owned(),
        dir: package.dir.clone(),
        rows,
    }
}

/// 重扫 `Plugin Folder`，返回扫到的包数
///
/// 挂在**临时**托盘项上，供打包后加插件这一步核对（spec §2.5 / issue 04）。
pub fn reload_packages(app: &AppHandle) -> usize {
    let registry = app.state::<PluginRegistry>();

    sync_packages(app, &registry)
}
