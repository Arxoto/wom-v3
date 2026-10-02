use serde::Serialize;

use super::{
    identity::{ActionId, ItemHandle},
    item::PluginItem,
};

/// 一页的条目数
pub const PAGE_SIZE: usize = 100;

/// 匹配模式，声明顺序即为优先级（越靠前优先级越高）
///
/// 四种匹配模式：精确 > 前缀 > 包含 > 子序列（Q8）
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
/// 四种匹配模式按优先级分组，
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
/// 条目是 Plugin Item 的投影
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
