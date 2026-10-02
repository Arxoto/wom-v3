use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex},
};

use serde::Serialize;
use tauri_plugin_log::log::{info, warn};

use super::{
    error::PluginError,
    identity::{ActionId, ItemHandle, PluginId},
    item::{ActionOutcome, PluginAction, PluginItem},
    plugin::{ItemRegistrar, Plugin, PluginContext},
    search::{ItemSearchPage, ItemSearchResult, MatchMode, PluginItemDisplay, PAGE_SIZE},
};

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
pub fn actions_of<'a>(
    table: &'a ActionTableView,
    the_type: &str,
) -> Option<&'a [PluginActionView]> {
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
/// 不需要维护锁顺序，也不存在死锁。
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
    /// 不 panic、不做版本校验。
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

    // priority 在**注册时**稳定排序一次，检索时只做分组
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
