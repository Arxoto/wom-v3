use std::path::{Path, PathBuf};

use super::{
    error::PluginError,
    identity::{ActionId, ItemHandle, PluginId},
    item::{ActionOutcome, PluginAction, PluginItem},
};

/// Plugin Context：插件从宿主取能力的**窄**接口，也是插件不依赖 Tauri 的原因（Q15）
///
/// 第一轮只给了最小集合：应用数据目录解析（含 [`Self::resolve_base`] 这个扩展面）、写日志、
/// 动作结果回传。接入时按接入清单第 4 条补上了剪贴板写入与打开能力——launcher 的三个动作
/// 需要它们。事件下发仍然**不在**这里：那是前端插件真正落地时的事（Q15）。
///
/// 实现只能落在宿主那一侧（见 `crate::plugin_host`）：本模块不依赖 Tauri，
/// 插件也不该知道底层是哪一个剪贴板或文件管理器。
///
/// 能力一律以 `Result<(), String>` 报错，错误串是 ASCII 的诊断文本；插件负责记日志与折算成
/// [`ActionOutcome`]，框架不替它解释。
pub trait PluginContext: Send + Sync {
    /// 应用数据目录（Q24）
    fn app_data_dir(&self) -> Result<PathBuf, String>;

    /// 解析一个扫描根路径变量
    ///
    /// `base` 是配置里手写的变量名（如 `$DESKTOP`、`$TEMP`）：识别不了就返回 [`None`]，
    /// 由调用方决定是报错还是退化成普通路径。变量名表**由宿主提供**，插件不自己维护
    /// 一份映射，免得与平台差异脱节（spec §2.5）。
    ///
    /// 名字里的 "base" 是配置字段 `Scan::base` 的沿用，不是"基目录"的意思。
    fn resolve_base(&self, base: &str) -> Option<PathBuf>;

    /// 把一段文本写进系统剪贴板
    fn write_clipboard(&self, text: &str) -> Result<(), String>;

    /// 用系统默认方式打开一个 URL
    fn open_url(&self, url: &str) -> Result<(), String>;

    /// 用系统默认方式打开一个路径
    fn open_path(&self, path: &Path) -> Result<(), String>;

    /// 在文件管理器里选中一个路径
    fn reveal(&self, path: &Path) -> Result<(), String>;

    fn log_info(&self, msg: &str);

    fn log_warn(&self, msg: &str);
}

/// 插件推条目给框架的唯一入口（Q30）
///
/// 句柄由框架分配，插件只负责把条目推全：`init` 里**一次性推完**，重载 = 重新 `init`，
/// 框架按 `plugin_id` 整块替换。
pub trait ItemRegistrar {
    /// 注册该插件的一批条目
    ///
    /// 一个插件手上可能有多个类型的条目，分几次 `register` 推上来即可，
    /// 块内顺序即注册顺序。
    fn register(&mut self, plugin_id: &PluginId, items: Vec<PluginItem>);
}

/// 一个插件（Q2）
///
/// 结构体尽量无状态，`init` / `run_action` 都取 `&self`（Q12）：框架要在派发时同时读表与
/// 调插件，`&self` 省掉一整层借用冲突。真需要缓存时插件自己用 `OnceLock`/`Mutex` 包字段。
pub trait Plugin: Send + Sync {
    /// 稳定 ASCII 标识
    fn id(&self) -> PluginId;

    /// 动作表：类型 → 动作 → label_key
    ///
    /// 顺序即优先级。表相对插件与类型是固定的，所以不加 revision（Q22），
    /// 前端维持"挂载时拉一次"的语义。
    fn actions(&self) -> Vec<PluginAction>;

    /// 一次性推完全部条目（`&self`，见 Q12）
    fn init(
        &self,
        cx: &dyn PluginContext,
        registrar: &mut dyn ItemRegistrar,
    ) -> Result<(), PluginError>;

    /// 把一个句柄解析成条目：框架**注册时没拿到**的那一类条目由插件自己交出来
    ///
    /// 注册推上来的条目框架手上都有，所以默认实现是 [`None`]。`Plugin Search Result`
    /// 是每次查询现算的、不注册进框架（Q10），派发它的动作时框架手上没有那一行，
    /// 于是回头问插件（实现见 `plugin_proxy_js::JsPlugin`）。
    ///
    /// 只有 [`PluginRegistry::run_action`] 会问它，而且只在自己那一份里找不到时才问。
    fn resolve_item(&self, _handle: &ItemHandle) -> Option<PluginItem> {
        None
    }

    /// 跑一个动作
    ///
    /// `item` 一定是本插件名下的条目——注册过的那个，或者 [`Plugin::resolve_item`] 给出的
    /// 那一个；`handle` 是它的身份。
    fn run_action(
        &self,
        cx: &dyn PluginContext,
        item: &PluginItem,
        handle: &ItemHandle,
        action_id: &ActionId,
    ) -> ActionOutcome;
}
