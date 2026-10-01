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
//! 2. [`actions_of`]：动作表查询函数。
//!
//! 其余类型（条目、动作、上下文、错误）在**类型层面**是 `pub` 的——插件的 trait 实现必须能
//! 指名它们——但本模块整体是私有的，所以它们对外不可达，等价于"只暴露两处入口"。
//! 注册表的块结构、`local_id` 的分配逻辑、[`PluginError`] 以外的内部类型都不对外可见。
//!
//! `allow(dead_code)` 是**接入前临时**的：这一轮不接 Tauri 托管状态、不注册命令、不接线前端，
//! 所以整块代码都是死代码，接入时删掉（Q17）。

#![allow(dead_code)]

use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::Serialize;
use tauri_plugin_log::log::{info, warn};

// region: 身份

/// 插件标识：稳定 ASCII 字符串
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PluginId(pub String);

impl PluginId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemHandle {
    pub plugin_id: PluginId,
    /// 该插件内**从 0 递增的注册序号**
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
}

impl PluginItem {
    pub fn new(
        the_type: impl Into<String>,
        priority: i32,
        key_words: Vec<String>,
        name: impl Into<String>,
        desc: impl Into<String>,
    ) -> Self {
        Self {
            the_type: the_type.into(),
            priority,
            key_words,
            name: name.into(),
            desc: desc.into(),
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
/// 这一轮只给最小集合：应用数据目录解析（含 [`Self::resolve_base`] 这个扩展面）、写日志、
/// 动作结果回传。剪贴板写入、打开路径/URL、事件下发都**不在**这里——那是 launcher 的动作
/// 实现与前端插件真正落地时的事（Q15）。
pub trait PluginContext {
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

    /// 跑一个动作
    ///
    /// `item` 一定是本插件注册过的条目（框架只把落在这个插件名下的条目交给它），
    /// `handle` 是它的身份。
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
/// 前端靠它寻址条目（跑动作），列表行号只是显示位置。
///
/// `action_ids` 是该条目可用的动作，顺序即优先级、第一个是默认动作；
/// 投影时由框架按条目的 [`PluginItem::the_type`] 查插件注册的动作表得出——
/// 框架不解释类型名，只是拿它做一次查表。
#[derive(Debug, Serialize)]
pub struct PluginItemDisplay {
    #[serde(flatten)]
    pub item: PluginItem,
    pub item_index: usize,
    pub action_ids: Vec<ActionId>,
}

// endregion

// region: 注册表

/// 动作表：类型名 → 动作 id → label_key
///
/// 外层用 [`BTreeMap`] 是为了确定性输出：类型名按字典序，同一类型下插件按注册顺序、
/// 动作按注册顺序。**没有遍历任何 `HashMap`**，哈希种子不影响这里。
pub type ActionTableView = BTreeMap<String, BTreeMap<ActionId, String>>;

/// 查某个类型的动作表；类型没有动作时返回 [`None`]
///
/// 这是公开面的第二处（Q29）
pub fn actions_of<'a>(
    table: &'a ActionTableView,
    the_type: &str,
) -> Option<&'a BTreeMap<ActionId, String>> {
    table.get(the_type)
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
pub struct PluginRegistry {
    /// 应用数据目录由框架从上下文取出来缓存一次（Q24）
    app_data_dir: PathBuf,
    cx: Arc<dyn PluginContext>,
    inner: Mutex<RegistryInner>,
}

impl PluginRegistry {
    /// 构造注册表；`cx` 是插件从宿主取能力的唯一入口（Q15）
    pub fn new(cx: Arc<dyn PluginContext>) -> Result<Self, PluginError> {
        let app_data_dir = cx
            .app_data_dir()
            .map_err(|err| PluginError::Init(format!("resolve app data dir failed: {err}")))?;

        info!("plugin registry created");

        Ok(Self {
            app_data_dir,
            cx,
            inner: Mutex::new(RegistryInner {
                plugin_list: Vec::new(),
                plugin_index: HashMap::new(),
                item_search_result: ItemSearchResult::default(),
            }),
        })
    }

    /// 应用数据目录，插件持久化层用它定位自己的文件
    pub fn app_data_dir(&self) -> &Path {
        &self.app_data_dir
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

        if inner.plugin_index.contains_key(&plugin_id) {
            warn!("plugin already registered, skip: {plugin_id}");
            return;
        }

        // 先落位、再 `init`：`init` 里推的条目要按注册顺序落进这个块
        let slot = inner.plugin_list.len();
        inner.plugin_list.push(PluginBlock {
            plugin,
            items: Vec::new(),
        });
        inner.plugin_index.insert(plugin_id.clone(), slot);

        init_plugin(&mut inner, slot, self.cx.as_ref());

        info!("plugin registered: {plugin_id}");
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

    /// 已注册插件的 id，顺序即注册顺序
    pub fn plugin_ids(&self) -> Vec<PluginId> {
        let inner = self.inner.lock().unwrap_or_else(|err| err.into_inner());

        inner
            .plugin_list
            .iter()
            .map(|block| block.plugin.id())
            .collect()
    }

    /// 注册表持有的全量条目：`(插件 id, 该插件的条目)`，顺序以 `Vec` 为准
    ///
    /// 按注册顺序逐插件给出，每个插件内部已按 priority 稳定排定。
    pub fn items(&self) -> Vec<(PluginId, Vec<PluginItem>)> {
        let inner = self.inner.lock().unwrap_or_else(|err| err.into_inner());

        inner
            .plugin_list
            .iter()
            .map(|block| (block.plugin.id(), block.items.clone()))
            .collect()
    }

    /// 类型名 → 该类型的全部动作，形状与现有 `action::table()` 的输出一致
    ///
    /// 类型名按字典序、同一类型下插件按注册顺序、动作按注册顺序。
    pub fn action_table(&self) -> ActionTableView {
        let inner = self.inner.lock().unwrap_or_else(|err| err.into_inner());

        let mut table: ActionTableView = BTreeMap::new();

        for block in &inner.plugin_list {
            for action in block.plugin.actions() {
                table
                    .entry(action.the_type)
                    .or_default()
                    .insert(action.id, action.label_key);
            }
        }

        table
    }

    /// 用关键字检索：重新生成一份结果并给出第一页
    pub fn search(&mut self, k: &str) -> ItemSearchPage {
        let mut inner = self.inner.lock().unwrap_or_else(|err| err.into_inner());

        // 令牌只由后端生成：每重新生成一份结果就在上一份的基础上 +1（溢出回绕）
        let token = inner.item_search_result.token.wrapping_add(1);
        inner.item_search_result = search_items(&inner.plugin_list, k, token);

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

    /// 按下标跑一个动作，返回它是否**真的执行了**
    ///
    /// 落下标、认不出的动作、没挂在条目类型上的动作，都当无操作并记 warn：
    /// 不 panic、不做版本校验（复刻 `action.rs:195` 的语义）。
    pub fn run_action(&self, item_index: usize, action_id: &ActionId) -> ActionOutcome {
        let inner = self.inner.lock().unwrap_or_else(|err| err.into_inner());

        let Some((plugin_id, local_id)) = handle_at(&inner, item_index) else {
            warn!("run action with out-of-range item index: {item_index}");
            return ActionOutcome::NoOp;
        };

        let Some(block) = block_of(&inner, &plugin_id) else {
            warn!("run action on an unregistered plugin: {plugin_id}");
            return ActionOutcome::NoOp;
        };

        let Some(item) = block.items.get(local_id) else {
            warn!("run action on a missing item: plugin {plugin_id}, local {local_id}");
            return ActionOutcome::NoOp;
        };

        if !has_action(&block.plugin.actions(), &item.the_type, action_id) {
            warn!(
                "action {} is not registered for type {} by plugin {}",
                action_id, item.the_type, plugin_id
            );
            return ActionOutcome::NoOp;
        }

        let handle = ItemHandle {
            plugin_id,
            local_id,
        };

        block
            .plugin
            .run_action(self.cx.as_ref(), item, &handle, action_id)
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
fn project(inner: &RegistryInner, item_indexes: &[usize]) -> Vec<PluginItemDisplay> {
    item_indexes
        .iter()
        .filter_map(|item_index| {
            let (plugin_id, local_id) = handle_at(inner, *item_index)?;
            let block = block_of(inner, &plugin_id)?;
            let item = block.items.get(local_id)?;

            // 动作列表按条目类型查插件注册的动作表：框架不解释类型名，只拿它查表
            let action_ids = block
                .plugin
                .actions()
                .into_iter()
                .filter(|action| action.the_type == item.the_type)
                .map(|action| action.id)
                .collect();

            Some(PluginItemDisplay {
                item: item.clone(),
                item_index: *item_index,
                action_ids,
            })
        })
        .collect()
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
