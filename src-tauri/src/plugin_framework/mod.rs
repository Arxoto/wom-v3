//! 插件框架：插件与条目的注册、检索、前端投影与动作派发
//!
//! 框架**不知道任何内容类型**：条目的类型名（[`PluginItem::the_type`]）只是一个不透明字符串，
//! 框架从不解释、匹配或列举它，只透传。框架只负责注册、检索、投影与把动作派回插件。
//!
//! 本模块不依赖 `launcher`、不依赖 `builtin_plugins`、**不直接依赖 Tauri**：
//! 插件要用的宿主能力一律走 [`PluginContext`]。
//!
//! 公开面只有两处（见 `.scratch/plugin-system/spec.md` §1.8）：
//!
//! 1. [`PluginRegistry`]：构造 + 注册插件 + 检索 + 翻页 + 跑动作；
//! 2. 动作表查询函数 [`actions_of`]——它现在只服务于页面投影：投影按条目的类型名查表，
//!    把动作 id 列表写进 [`PluginItemDisplay::action_ids`]。
//!
//! 跑动作按 [`ItemHandle`] 寻址（Q23），不按条目在整集里的下标：主列表与
//! `Plugin Search Page` 两层列表因此走同一条派发路（见 [`PluginItemDisplay::handle`]）。
//!
//! 动作表**不再有对外的取用面**：界面那一侧的动作顺序、图标与文案由前端的插件注册表给出
//! （见 `src/plugins/registry.tsx`），Rust 只在投影时用它算条目自带的 `action_ids`。
//! 因此 [`PluginActionView`] 与 [`ActionTableView`] 是本模块内部类型，`pub` 只为了让
//! trait 实现与投影函数能指名它们。
//!
//! 其余类型（条目、动作、上下文、错误）在**类型层面**是 `pub` 的——插件的 trait 实现必须能
//! 指名它们——但本模块整体是私有的，所以它们对外不可达，等价于"只暴露两处入口"。
//! 注册表的块结构、`local_id` 的分配逻辑、[`PluginError`] 以外的内部类型都不对外可见。
//!
//! 本模块**已经接入应用**（接入前那份临时的 `allow(dead_code)` 已删除）：宿主侧见
//! `crate::plugin_host`（[`PluginContext`] 的实现与注册表的托管），命令层见
//! `crate::commands` 里那几个 `plugin_*` 命令。

use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use tauri_plugin_log::log::{info, warn};

// region: 身份

/// 插件标识：稳定 ASCII 字符串
///
/// 它随 [`ItemHandle`] 一起下发到前端，所以两端都认这一个形状（`Serialize` / `Deserialize`）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PluginId(pub String);

impl Display for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// 动作标识，只在插件自己的动作表里有意义
///
/// 下发形状就是那个字符串本身（`transparent`）：前端拿到的动作名与注册时写的一字不差
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct ActionId(pub String);

impl ActionId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for ActionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// 条目身份：插件 + 插件内的注册序号
///
/// 由框架分配（Q30），插件不自己造。框架与插件之间用它寻址条目，
/// 与"条目在加载出的整集里的下标"不是一回事。
///
/// **它随条目一起下发到前端**（Q23）：`Plugin Search Result` 不在框架持有的那一集里，
/// 用 `item_index` 寻址对它不成立，所以两层列表的寻址统一到 handle 上——
/// 管你在第几层，动作派发都能找到那一行。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ItemHandle {
    pub plugin_id: PluginId,
    /// 该插件内**从 0 递增的注册序号**；结果行则是它在最近一次搜索结果里的下标
    pub local_id: usize,
}

// endregion

// region: 条目与动作

/// 一个 Plugin Item：不透明类型名 + 纯数据字段
///
/// 句柄不在数据里，由框架的条目下标给出，插件不做二次存储（Q6/Q30）。
/// 不留插件私有数据位（Q10）。
///
/// 以 `the_type` 为 serde 内部 tag、字段与 tag 平铺，下发形状与现有
/// `builtin_plugins::common::Item` 完全一致，前端手写的镜像类型不需要改动（Q34）。
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "the_type")]
pub struct PluginItem {
    /// 排序用的优先级
    pub priority: i32,
    pub key_words: Vec<String>,
    pub name: String,
    pub desc: String,
    /// 类型名。框架对它的值**永远不解释**，只透传
    pub the_type: String,
    /// 条目自带的图标：**绝对路径**，空串表示没有
    ///
    /// 与"类型图标"不是一回事：类型图标一张画给同类型的每一行（前端注册表里按类型名查），
    /// 而这一张是**这个条目自己**的图片——`Plugin Package` 的清单里写的那一张就是它。
    /// 框架不碰文件、也不解释路径，只把它原样透传给前端（前端再交给 asset protocol）。
    pub icon: String,
}

impl PluginItem {
    pub fn new(
        the_type: impl Into<String>,
        priority: i32,
        key_words: Vec<String>,
        name: impl Into<String>,
        desc: impl Into<String>,
        icon: impl Into<String>,
    ) -> Self {
        Self {
            the_type: the_type.into(),
            priority,
            key_words,
            name: name.into(),
            desc: desc.into(),
            icon: icon.into(),
        }
    }
}

/// 一个 Plugin Action 的元数据
///
/// 类型轴在这里：同一个 `copy` 在不同类型上要走不同的 `label_key`，
/// 所以 `label_key` 必须按"类型 + 动作"分套（Q35）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PluginAction {
    /// 该动作挂在哪个类型上，值同上不透明
    pub the_type: String,
    pub id: ActionId,
    /// 前端据它查文案。框架只当不透明字符串透传，**不强制**命名规范
    pub label_key: String,
}

impl PluginAction {
    pub fn new(
        the_type: impl Into<String>,
        id: impl Into<String>,
        label_key: impl Into<String>,
    ) -> Self {
        Self {
            the_type: the_type.into(),
            id: ActionId(id.into()),
            label_key: label_key.into(),
        }
    }
}

/// 一个动作跑完的结果
///
/// [`Self::NoOp`] 与 [`Self::Failed`] 必须分开（Q13）：前者是"什么都没发生"，
/// 调用方据此决定要不要隐藏窗口（复刻 `action.rs:219` 的行为）。这一轮不加错误文案字段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionOutcome {
    /// 真的执行了
    Done,
    /// 没有动作可做，也不算失败（动作未实现、动作不挂在该条目上等）
    NoOp,
    /// 尝试执行但失败了
    Failed,
}

// endregion

// region: 错误

/// 框架级错误（Q18）
///
/// 只有 init 与 action 两类。条目级错误是插件私有的，框架只 `to_string()`，不进这个枚举——
/// 框架因此看不到"空行"与"字段不足"的区别，那是插件的语义（Q19）。
#[derive(Debug)]
pub enum PluginError {
    /// 插件 init 失败
    Init(String),
    /// 插件 action 失败
    ///
    /// 这一轮没有构造点：跑动作只回 [`ActionOutcome`] 三态（Q13 明确此时不加错误文案字段），
    /// 失败归因留在插件自己的日志里。变体先留着——Q18 的三层错误表要求 init 与 action 分开，
    /// 等动作真的要把失败原因交回框架时（例如前端插件）就在这里构造。
    #[allow(dead_code)]
    Action(String),
}

impl PluginError {
    pub fn kind(&self) -> &'static str {
        match self {
            PluginError::Init(_) => "init",
            PluginError::Action(_) => "action",
        }
    }

    /// 诊断用的一行 ASCII 文本
    pub fn message(&self) -> &str {
        match self {
            PluginError::Init(msg) | PluginError::Action(msg) => msg,
        }
    }
}

impl Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "plugin {} error: {}", self.kind(), self.message())
    }
}

impl std::error::Error for PluginError {}

// endregion

// region: 插件与宿主之间的接口

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
    /// 应用数据目录，与现有内建设置文件同处（Q24）
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
    /// 于是回头问插件（实现见 `plugin_impl_js::JsPlugin`）。
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

// endregion

// region: 检索

/// 一页的条目数，与现有 `builtin_plugins::search::PAGE_SIZE` 一致
pub const PAGE_SIZE: usize = 100;

/// 匹配模式，声明顺序即为优先级（越靠前优先级越高）
///
/// 原样沿用现有四种模式：精确 > 前缀 > 包含 > 子序列（Q8）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchMode {
    Eq,
    StartsWith,
    Contains,
    Match,
}

impl MatchMode {
    /// 单个关键字对输入的匹配模式，未命中返回 [`None`]
    pub fn of(keyword: &str, k_input: &str) -> Option<Self> {
        if keyword == k_input {
            Some(Self::Eq)
        } else if keyword.starts_with(k_input) {
            Some(Self::StartsWith)
        } else if keyword.contains(k_input) {
            Some(Self::Contains)
        } else if is_sub_sequence(keyword, k_input) {
            Some(Self::Match)
        } else {
            None
        }
    }
}

/// 子序列匹配：两个迭代器依次步进
fn is_sub_sequence(keyword: &str, k_input: &str) -> bool {
    if k_input.len() > keyword.len() {
        return false;
    }
    let mut key_iter = keyword.bytes();
    k_input.bytes().all(|s| key_iter.any(|t| t == s))
}

/// 检索结果：只保存条目下标，渲染用的条目在翻页时再取
///
/// 与现有 `ItemSearchResult` 同义（`search.rs:14`）：四种匹配模式按优先级分组，
/// 四个分割索引即分组边界，前端据它们给结果分区。
#[derive(Debug, Default)]
pub struct ItemSearchResult {
    /// 这一份结果的身份令牌，前端翻页时校验是否与当前结果一致
    pub token: u32,
    /// 产生该结果的检索输入；`None` 表示尚未检索过（空串与"未检索"不能混淆）
    pub input_key: Option<String>,
    /// 按匹配模式分组后的条目下标
    pub item_indexes: Vec<usize>,
    pub index_eq: usize,
    pub index_starts_with: usize,
    pub index_contains: usize,
    pub index_match: usize,
}

impl ItemSearchResult {
    pub fn is_current_key(&self, k: &str) -> bool {
        self.input_key.as_deref() == Some(k)
    }

    pub fn is_current_token(&self, token: u32) -> bool {
        self.token == token
    }
}

/// 一页检索结果（前端）
///
/// 形状与现有 `ItemSearchPage` 一致（`search.rs:39`），只是条目换成了 Plugin Item 的投影
#[derive(Debug, Serialize)]
pub struct ItemSearchPage {
    pub token: u32,
    pub total: usize,
    pub index: usize,
    pub item_list: Vec<PluginItemDisplay>,
    pub index_eq: usize,
    pub index_starts_with: usize,
    pub index_contains: usize,
    pub index_match: usize,
}

impl ItemSearchPage {
    /// 一份空页：检索不出结果时用它，令牌照旧
    pub fn empty(token: u32) -> Self {
        Self {
            token,
            total: 0,
            index: 0,
            item_list: Vec::new(),
            index_eq: 0,
            index_starts_with: 0,
            index_contains: 0,
            index_match: 0,
        }
    }
}

/// 一条条目的渲染结构
///
/// 条目以内部 tag（`the_type`）序列化、字段与 tag 平铺；`item_index` 是条目在整集里的下标，
/// 列表行号只是显示位置——**动作派发一律用 `handle`**（见 [`ItemHandle`]），不再用下标。
///
/// `action_ids` 是该条目可用的动作，顺序即优先级、第一个是默认动作；
/// 投影时由框架按条目的 [`PluginItem::the_type`] 查插件注册的动作表得出——
/// 框架不解释类型名，只是拿它做一次查表。
#[derive(Debug, Serialize)]
pub struct PluginItemDisplay {
    #[serde(flatten)]
    pub item: PluginItem,
    /// 条目在整集里的下标：主列表翻页与预请求的记账用（`search_page` 的 `index`），
    /// **不是**动作派发的地址
    pub item_index: usize,
    pub action_ids: Vec<ActionId>,
    /// 条目身份：动作派发用它，两层列表（主列表 / `Plugin Search Page`）因此都是同一条路
    pub handle: ItemHandle,
}

// endregion

// region: 注册表

/// 动作表里的一条动作：动作 id + 前端据它查文案的 `label_key`
///
/// 与 [`PluginAction`] 只差一个 `the_type`——它在表的外层键上，不必在每条动作里再写一遍。
///
/// 只服务于投影，不下发：界面读的是前端注册表（见本模块头部的说明）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PluginActionView {
    pub id: ActionId,
    pub label_key: String,
}

/// 动作表：类型名 → 该类型的动作（**顺序即优先级**，第一个是默认动作）
///
/// 外层用 [`BTreeMap`] 是为了确定性输出：类型名按字典序。**没有遍历任何 `HashMap`**，
/// 哈希种子不影响这里。
///
/// 内层是 `Vec` 而不是按 id 排序的 `BTreeMap`：动作的顺序就是优先级、第一个是默认动作（§1.5），
/// 按 id 排会把 launcher 的 `scan` 排成 `copy`、`open_path`、`reveal`，默认动作就错了。
pub type ActionTableView = BTreeMap<String, Vec<PluginActionView>>;

/// 查某个类型的动作；类型没有动作时返回 [`None`]
///
/// 页面投影按条目的类型名查它——框架不解释类型名，只拿它查表。
pub fn actions_of<'a>(table: &'a ActionTableView, the_type: &str) -> Option<&'a [PluginActionView]> {
    table.get(the_type).map(Vec::as_slice)
}

/// 某个插件的注册块
///
/// `items` 的顺序就是迭代表：priority 在注册时排定一次，检索时只做分组。
struct PluginBlock {
    plugin: Arc<dyn Plugin>,
    items: Vec<PluginItem>,
}

/// 注册表受保护的内容
///
/// 插件块、下标反查、缓存结果总是一起用、一起换，所以合并到同一把 [`Mutex`]：
/// 不需要维护锁顺序，也不存在死锁（与 `builtin_plugins::stat::BuiltinStat` 同理）。
struct RegistryInner {
    /// 注册顺序即数组顺序
    plugin_list: Vec<PluginBlock>,
    /// 只做反查；**不用它的迭代顺序**：哈希种子会让顺序每次进程启动都可能不同，
    /// 而我们要的是确定性排序（Q25）
    plugin_index: HashMap<PluginId, usize>,
    /// 缓存的检索结果
    item_search_result: ItemSearchResult,
}

/// 插件注册表
///
/// 结构是 `Vec<PluginBlock>`（注册顺序即数组顺序）配 `HashMap<PluginId, usize>` 反查下标（Q21/Q25）。
/// `HashMap` **只**服务于"整块替换某插件的条目"与按 id 找块，任何迭代顺序都以 `Vec` 为准。
///
/// 本身可以 `manage` 进 Tauri（接入期补的）：条目与缓存都在一把私有 [`Mutex`] 后面，
/// 所有取用方法都只取 `&self`，所以它是 `Send + Sync` 的；只有装配期的
/// [`PluginRegistry::register_plugin`] 取 `&mut`，运行期的重扫走
/// [`PluginRegistry::register_or_reload_plugin`]（它只取 `&self`，因为 `manage` 之后
/// 宿主手上只有 `State`）。
pub struct PluginRegistry {
    cx: Arc<dyn PluginContext>,
    inner: Mutex<RegistryInner>,
}

impl PluginRegistry {
    /// 构造注册表；`cx` 是插件从宿主取能力的唯一入口（Q15）
    ///
    /// 顺手探一次应用数据目录（Q24）：拿不到就没必要往下走——落在应用数据目录下的插件
    /// 迟早都要用它落文件，早失败好过一个只有空条目的启动器。
    pub fn new(cx: Arc<dyn PluginContext>) -> Result<Self, PluginError> {
        cx.app_data_dir()
            .map_err(|err| PluginError::Init(format!("resolve app data dir failed: {err}")))?;

        info!("plugin registry created");

        Ok(Self {
            cx,
            inner: Mutex::new(RegistryInner {
                plugin_list: Vec::new(),
                plugin_index: HashMap::new(),
                item_search_result: ItemSearchResult::default(),
            }),
        })
    }

    /// 注册一个插件并立刻 `init` 它
    ///
    /// 插件级隔离（Q14）：`init` 失败只记 warn 并**跳过该插件**（它这一轮推的条目一并回滚），
    /// 其余插件照常注册、应用启动不受影响，一个坏插件不该让启动器起不来。
    ///
    /// 注册顺序即数组顺序：后注册的插件排在后面，这也是一条排序维度（Q26）。
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        let plugin: Arc<dyn Plugin> = Arc::from(plugin);
        let plugin_id = plugin.id();

        let mut inner = self.inner.lock().unwrap_or_else(|err| err.into_inner());

        // 装配期的重复注册是错的：同一个 id 出现两次，多半是代码写错了
        if inner.plugin_index.contains_key(&plugin_id) {
            warn!("plugin already registered, skip: {plugin_id}");
            return;
        }

        let slot = place_plugin(&mut inner, plugin);

        init_plugin(&mut inner, slot, self.cx.as_ref());

        info!("plugin registered: {plugin_id}");
    }

    /// 注册一个插件；`id` 已经存在时**换掉实现**并重新 `init`（运行期重扫走这条）
    ///
    /// 与 [`Self::register_plugin`] 的差别只有"已存在时怎么办"：装配期的重复注册是错的，
    /// 而重扫是常态——`Plugin Package` 的清单可能改过（名字、关键字、动作），代理要跟着换一份。
    ///
    /// 取 `&self`：注册表 `manage` 进 Tauri 之后拿到的是 `State`，重扫没有 `&mut` 可用。
    pub fn register_or_reload_plugin(&self, plugin: Box<dyn Plugin>) {
        let plugin: Arc<dyn Plugin> = Arc::from(plugin);
        let plugin_id = plugin.id();

        let mut inner = self.inner.lock().unwrap_or_else(|err| err.into_inner());
        let replaced = inner.plugin_index.contains_key(&plugin_id);

        let slot = place_plugin(&mut inner, plugin);

        init_plugin(&mut inner, slot, self.cx.as_ref());

        if replaced {
            info!("plugin replaced: {plugin_id}");
        } else {
            info!("plugin registered: {plugin_id}");
        }
    }

    /// 重新 `init` 一个已注册插件：按 `plugin_id` 整块替换它的条目（Q11）
    pub fn reload_plugin(&self, plugin_id: &PluginId) {
        let mut inner = self.inner.lock().unwrap_or_else(|err| err.into_inner());

        let Some(slot) = inner.plugin_index.get(plugin_id).copied() else {
            warn!("reload an unregistered plugin: {plugin_id}");
            return;
        };

        init_plugin(&mut inner, slot, self.cx.as_ref());
    }

    /// 用关键字检索：重新生成一份结果并给出第一页
    ///
    /// 同一个关键字不重新检索：令牌与结果都留在上一份上（复刻 `stat::search` 的缓存）。
    /// 前端已经按关键字去重过一次，这里再挡一道，免得同一份结论被算两遍。
    pub fn search(&self, k: &str) -> ItemSearchPage {
        let mut inner = self.inner.lock().unwrap_or_else(|err| err.into_inner());

        if !inner.item_search_result.is_current_key(k) {
            // 令牌只由后端生成：每重新生成一份结果就在上一份的基础上 +1（溢出回绕）
            let token = inner.item_search_result.token.wrapping_add(1);
            inner.item_search_result = search_items(&inner.plugin_list, k, token);
        }

        let token = inner.item_search_result.token;

        match page_of(&inner, 0, token) {
            Ok(page) => page,
            Err(err) => {
                warn!("{err}");
                ItemSearchPage::empty(token)
            }
        }
    }

    /// 翻页，校验令牌，过期报错
    pub fn page(&self, index: usize, token: u32) -> Result<ItemSearchPage, String> {
        let inner = self.inner.lock().unwrap_or_else(|err| err.into_inner());

        page_of(&inner, index, token)
    }

    /// 按 [`ItemHandle`] 跑一个动作，返回它是否**真的执行了**
    ///
    /// 认不出的插件、落不到的行、没挂在条目类型上的动作，都当无操作并记 warn：
    /// 不 panic、不做版本校验（复刻 `action.rs:195` 的语义）。
    ///
    /// 寻址用 handle 而不是下标（Q23）：主列表与 `Plugin Search Page` 两层列表因此
    /// 走的是同一条派发路，框架不必知道自己在哪一层。
    pub fn run_action(&self, handle: &ItemHandle, action_id: &ActionId) -> ActionOutcome {
        let inner = self.inner.lock().unwrap_or_else(|err| err.into_inner());

        let Some(block) = block_of(&inner, &handle.plugin_id) else {
            warn!("run action on an unregistered plugin: {}", handle.plugin_id);
            return ActionOutcome::NoOp;
        };

        // 自己手上没有就问插件：`Plugin Search Result` 不在注册表里（Q10）
        let resolved;
        let item = match block.items.get(handle.local_id) {
            Some(item) => item,
            None => {
                resolved = block.plugin.resolve_item(handle);

                let Some(item) = resolved.as_ref() else {
                    warn!(
                        "run action on a missing item: plugin {}, local {}",
                        handle.plugin_id, handle.local_id
                    );
                    return ActionOutcome::NoOp;
                };

                item
            }
        };

        if !has_action(&block.plugin.actions(), &item.the_type, action_id) {
            warn!(
                "action {} is not registered for type {} by plugin {}",
                action_id, item.the_type, handle.plugin_id
            );
            return ActionOutcome::NoOp;
        }

        block
            .plugin
            .run_action(self.cx.as_ref(), item, handle, action_id)
    }
}

/// 把一个插件的块放进注册表：已有的 `id` 换实现，没有的追加到末尾
///
/// 注册与替换共用这一段；调用方负责先决定"重复注册该怎么办"。
/// 先落位、再 `init`：`init` 里推的条目要按注册顺序落进这个块。
fn place_plugin(inner: &mut RegistryInner, plugin: Arc<dyn Plugin>) -> usize {
    let plugin_id = plugin.id();

    match inner.plugin_index.get(&plugin_id).copied() {
        Some(slot) => {
            inner.plugin_list[slot].plugin = plugin;
            slot
        }
        None => {
            let slot = inner.plugin_list.len();
            inner.plugin_list.push(PluginBlock {
                plugin,
                items: Vec::new(),
            });
            inner.plugin_index.insert(plugin_id, slot);
            slot
        }
    }
}


/// `init` 一个块里的插件，成功后整块替换它的条目；失败则回滚并跳过该插件
///
/// 注册与重载走同一条路：重载就是重新 `init`（Q11）。
fn init_plugin(inner: &mut RegistryInner, slot: usize, cx: &dyn PluginContext) {
    let plugin_id = inner.plugin_list[slot].plugin.id();
    let plugin = Arc::clone(&inner.plugin_list[slot].plugin);

    // 先清空：`init` 里推的条目就是这个插件这一轮的全部条目
    inner.plugin_list[slot].items.clear();

    // 已经定稿的条目数：一个块处理到这里就定稿，失败时把没定稿的丢掉
    let mut done_len = 0;
    let init_result = {
        let mut registrar = RegistryRegistrar {
            plugin_list: &mut inner.plugin_list,
            slot,
            plugin_id: &plugin_id,
            done_len: &mut done_len,
        };

        plugin.init(cx, &mut registrar)
    };

    if let Err(err) = init_result {
        // 插件级隔离：只回滚它自己这一块（Q14）
        inner.plugin_list[slot].items.truncate(done_len);
        warn!("plugin init failed, skip plugin {plugin_id}: {err}");
        return;
    }

    // priority 在**注册时**稳定排序一次（复刻 `load.rs:91`），检索时只做分组
    inner.plugin_list[slot]
        .items
        .sort_by_key(|item| item.priority);
}

/// 一页的结果：把下标翻成渲染结构
fn page_of(inner: &RegistryInner, index: usize, token: u32) -> Result<ItemSearchPage, String> {
    let result = &inner.item_search_result;

    if !result.is_current_token(token) {
        return Err(format!(
            "search page for stale token: requested {token}, cached {}",
            result.token
        ));
    }

    if index > result.item_indexes.len() {
        return Err(format!("search page out of range: {index}"));
    }

    let start_index = result.item_indexes.len().min(index);
    let final_index = result.item_indexes.len().min(start_index + PAGE_SIZE);
    let item_list = project(inner, &result.item_indexes[start_index..final_index]);

    Ok(ItemSearchPage {
        token,
        total: result.item_indexes.len(),
        index: start_index,
        item_list,
        index_eq: result.index_eq,
        index_starts_with: result.index_starts_with,
        index_contains: result.index_contains,
        index_match: result.index_match,
    })
}

/// 把条目下标投影成前端要的形状
///
/// 动作表一页只建一次：条目按类型名查表拿动作，不必为了每条条目再向插件要一遍动作表
/// （一页 100 条，旧写法每页要多要 100 次）。
fn project(inner: &RegistryInner, item_indexes: &[usize]) -> Vec<PluginItemDisplay> {
    let table = action_table_of(&inner.plugin_list);

    item_indexes
        .iter()
        .filter_map(|item_index| {
            let (plugin_id, local_id) = handle_at(inner, *item_index)?;
            let block = block_of(inner, &plugin_id)?;
            let item = block.items.get(local_id)?;

            // 动作列表按条目类型查动作表：框架不解释类型名，只拿它查表
            let action_ids = actions_of(&table, &item.the_type)
                .unwrap_or_default()
                .iter()
                .map(|action| action.id.clone())
                .collect();

            Some(PluginItemDisplay {
                item: item.clone(),
                item_index: *item_index,
                action_ids,
                handle: ItemHandle {
                    plugin_id,
                    local_id,
                },
            })
        })
        .collect()
}

/// 按注册顺序拼出动作表：插件注册顺序 > 动作注册顺序，类型名去重成外层键
///
/// 同一个类型名上的同一个动作只认可先注册的那一条（重复注册是插件自己的账，
/// 这里只记一条 warn，不让它在前端变成两个一模一样的动作）。
fn action_table_of(plugin_list: &[PluginBlock]) -> ActionTableView {
    let mut table: ActionTableView = BTreeMap::new();

    for block in plugin_list {
        for action in block.plugin.actions() {
            let views = table.entry(action.the_type.clone()).or_default();

            if views.iter().any(|view| view.id == action.id) {
                warn!(
                    "action {} on type {} is already registered, skip the later one",
                    action.id, action.the_type
                );
                continue;
            }

            views.push(PluginActionView {
                id: action.id,
                label_key: action.label_key,
            });
        }
    }

    table
}

/// 按 id 找块：`HashMap` 反查只在这里用
fn block_of<'a>(inner: &'a RegistryInner, plugin_id: &PluginId) -> Option<&'a PluginBlock> {
    inner
        .plugin_index
        .get(plugin_id)
        .and_then(|slot| inner.plugin_list.get(*slot))
}

/// 条目下标 → 身份：按注册顺序（插件顺序 > 条目顺序）累加各块的长度
///
/// 于是 `local_id` 就是块内下标，与条目被推给框架时的注册序号一致（排序是稳定排序）。
fn handle_at(inner: &RegistryInner, item_index: usize) -> Option<(PluginId, usize)> {
    let mut offset = item_index;

    for block in &inner.plugin_list {
        let len = block.items.len();
        if offset < len {
            return Some((block.plugin.id(), offset));
        }
        offset -= len;
    }

    None
}

/// 按四种匹配模式检索并按分组拼接
///
/// 条目顺序以 `Vec` 为准（插件注册顺序 > 插件内 priority 顺序），
/// 同一分组内因此天然满足 "匹配模式 > priority > 插件注册顺序 > 条目注册顺序"（Q26）。
fn search_items(plugin_list: &[PluginBlock], k: &str, token: u32) -> ItemSearchResult {
    let mut indexes_eq: Vec<usize> = Vec::new();
    let mut indexes_starts_with: Vec<usize> = Vec::new();
    let mut indexes_contains: Vec<usize> = Vec::new();
    let mut indexes_match: Vec<usize> = Vec::new();

    let mut item_index = 0;
    for block in plugin_list {
        for item in &block.items {
            // 遍历该条目的全部关键字，只保留优先级最高的那次匹配
            let mode = item
                .key_words
                .iter()
                .filter_map(|keyword| MatchMode::of(keyword, k))
                .min();

            match mode {
                Some(MatchMode::Eq) => indexes_eq.push(item_index),
                Some(MatchMode::StartsWith) => indexes_starts_with.push(item_index),
                Some(MatchMode::Contains) => indexes_contains.push(item_index),
                Some(MatchMode::Match) => indexes_match.push(item_index),
                None => {}
            }

            item_index += 1;
        }
    }

    let index_eq = 0;
    let index_starts_with = index_eq + indexes_eq.len();
    let index_contains = index_starts_with + indexes_starts_with.len();
    let index_match = index_contains + indexes_contains.len();

    let mut item_indexes = Vec::with_capacity(index_match + indexes_match.len());
    item_indexes.extend(indexes_eq);
    item_indexes.extend(indexes_starts_with);
    item_indexes.extend(indexes_contains);
    item_indexes.extend(indexes_match);

    ItemSearchResult {
        input_key: Some(k.to_string()),
        token,
        item_indexes,
        index_eq,
        index_starts_with,
        index_contains,
        index_match,
    }
}

/// 条目的类型上是否挂了该动作
fn has_action(actions: &[PluginAction], the_type: &str, action_id: &ActionId) -> bool {
    actions
        .iter()
        .any(|action| action.the_type == the_type && &action.id == action_id)
}

/// 框架交给插件的 registrar：插件只管推条目，句柄与存储都在框架这边（Q30）
///
/// 每条 `register` 调用推一批条目，块内顺序就是注册顺序，`local_id` 由框架按累加得出。
struct RegistryRegistrar<'a> {
    plugin_list: &'a mut Vec<PluginBlock>,
    /// 正在 `init` 的块的下标，由框架给出——插件不自己分配（Q30）
    slot: usize,
    /// 正在 `init` 的插件 id：只认它推上来的条目
    plugin_id: &'a PluginId,
    /// 已经定稿的条目数，`init` 失败时据此回滚
    done_len: &'a mut usize,
}

impl ItemRegistrar for RegistryRegistrar<'_> {
    fn register(&mut self, plugin_id: &PluginId, items: Vec<PluginItem>) {
        // 只认正在 init 的那个插件：别的插件名一律拒绝，免得条目串了块
        if plugin_id != self.plugin_id {
            warn!("registrar called with a foreign plugin id: {plugin_id}");
            return;
        }

        if let Some(block) = self.plugin_list.get_mut(self.slot) {
            block.items.extend(items);
            *self.done_len += 1;
        }
    }
}

// endregion
