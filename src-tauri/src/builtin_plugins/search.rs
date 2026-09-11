//! 检索逻辑

use serde::Serialize;

use crate::builtin_plugins::{base::ItemId, common::Item, persistence::load::ItemCollection};

/// 搜索结果（后端）
///
/// 只保存 [`ItemId`] ，渲染用的 [`Item`] 在翻页时再从 [`ItemCollection`] 取出，
/// 避免为每条命中结果克隆一次 [`Item`]
///
/// 由 [`crate::builtin_plugins::plugins::PluginStat`] 持有
#[derive(Debug, Default)]
pub struct ItemSearchResult {
    /// `None` 表示尚未进行过检索
    ///
    /// 使用 [`Option`] 而不是空串，避免“检索空串”和“未检索”互相混淆
    pub key_word: Option<String>,
    pub item_ids: Vec<ItemId>,
    // 不同匹配模式的分割索引
    pub index_eq: usize,
    pub index_starts_with: usize,
    pub index_contains: usize,
    pub index_match: usize,
}

/// 分页查询搜索结果
pub const PAGE_SIZE: usize = 100;

/// 搜索结果（前端）
#[derive(Debug, Serialize)]
pub struct ItemSearchPage {
    pub total: usize,
    pub index: usize,
    pub item_list: Vec<ItemDisplay>,
    // 不同匹配模式的分割索引
    pub index_eq: usize,
    pub index_starts_with: usize,
    pub index_contains: usize,
    pub index_match: usize,
}

impl ItemSearchResult {
    pub fn is_current_result(&self, k: &str) -> bool {
        self.key_word.as_deref() == Some(k)
    }

    pub fn page(&self, index: usize, item_collection: &ItemCollection) -> ItemSearchPage {
        let start_index = self.item_ids.len().min(index);
        let final_index = self.item_ids.len().min(start_index + PAGE_SIZE);

        let item_list: Vec<ItemDisplay> = self.item_ids[start_index..final_index]
            .iter()
            .filter_map(|id| item_collection.get_by_id(*id))
            .map(ItemDisplay::from)
            .collect();

        ItemSearchPage {
            total: self.item_ids.len(),
            index: start_index,
            item_list,
            index_eq: self.index_eq,
            index_starts_with: self.index_starts_with,
            index_contains: self.index_contains,
            index_match: self.index_match,
        }
    }
}

/// [`Item`] 渲染结构
#[derive(Debug, Serialize)]
pub struct ItemDisplay {
    pub the_type: String,
    pub name: String,
    pub desc: String,
}

impl From<&Item> for ItemDisplay {
    fn from(value: &Item) -> Self {
        Self {
            the_type: value.the_type.to_string(),
            name: value.name.clone(),
            desc: value.desc.to_string(),
        }
    }
}

mod algorithm {
    use bitvec::prelude::*;

    use crate::builtin_plugins::{
        base::ItemId, persistence::load::ItemCollection, search::ItemSearchResult,
    };

    /// 精确相等
    #[inline]
    fn find_eq(keyword: &str, k_input: &str) -> bool {
        keyword == k_input
    }

    /// 前缀
    #[inline]
    fn find_starts_with(keyword: &str, k_input: &str) -> bool {
        keyword.starts_with(k_input)
    }

    /// 包含
    #[inline]
    fn find_contains(keyword: &str, k_input: &str) -> bool {
        keyword.contains(k_input)
    }

    /// 子序列匹配 两个迭代器依次步进
    fn find_match(keyword: &str, k_input: &str) -> bool {
        if k_input.len() > keyword.len() {
            return false;
        }
        let mut key_iter = keyword.bytes();
        k_input.bytes().all(|s| key_iter.any(|t| t == s))
    }

    impl ItemCollection {
        /// 依次按照 精确 > 前缀 > 包含 > 子序列 检索，同一个 item 只保留优先级最高的那次匹配
        ///
        /// 去重使用 `BitVec<u64, Lsb0>` 判断 id 是否已存在
        /// - 相比 `HashSet` ，无需哈希计算，内存占用低、缓存友好
        /// - 相比 `bit-set` ，`bitvec` 的维护活跃度更高
        /// - 相比 `roaring` ，更成熟、工业级，但更适用于海量数据，低数据量下容器结构反而更重
        ///
        /// 为优化性能， id 必须是纯数字，且尽量密集
        /// （见 [`crate::builtin_plugins::common::Item::new`] ）
        pub fn search(&self, k: &str) -> ItemSearchResult {
            let item_list = &self.item_list;

            // 预分配位图，id 最大为 max_id ，向上取整到 64 位便于内存对齐
            // 空集合时取 64 ，保证位图非空
            let max_id = item_list.iter().map(|item| item.the_id).max().unwrap_or(0);
            let bits = ((max_id as usize) + 64) & !63;
            let mut seen_ids = bitvec![u64, Lsb0; 0; bits];

            // 四种匹配模式分别收集 id ，既保证输出分组，又避免收集时克隆 [`Item`]
            let mut ids_eq: Vec<ItemId> = vec![];
            let mut ids_starts_with: Vec<ItemId> = vec![];
            let mut ids_contains: Vec<ItemId> = vec![];
            let mut ids_match: Vec<ItemId> = vec![];

            for item in item_list {
                let keyword = item.key_word.0.as_str();

                // 先分类，未命中的 item 不能占用 id
                // 同一个 item 可能有多个 key_word
                // （见 [`crate::builtin_plugins::common::Item::new_list`] ），
                // 因此同一个 id 可能命中多次，只保留优先级最高的那次
                let target = if find_eq(keyword, k) {
                    &mut ids_eq
                } else if find_starts_with(keyword, k) {
                    &mut ids_starts_with
                } else if find_contains(keyword, k) {
                    &mut ids_contains
                } else if find_match(keyword, k) {
                    &mut ids_match
                } else {
                    continue;
                };

                // replace(index, value) 会将该位设为 true ，并返回该位原来的值
                // 如果原来是 false ，说明是第一次遇见
                if !seen_ids.replace(item.the_id as usize, true) {
                    target.push(item.the_id);
                }
            }

            let index_eq = 0;
            let index_starts_with = index_eq + ids_eq.len();
            let index_contains = index_starts_with + ids_starts_with.len();
            let index_match = index_contains + ids_contains.len();

            // 按优先级顺序拼接
            let mut item_ids = Vec::with_capacity(index_match + ids_match.len());
            item_ids.extend(ids_eq);
            item_ids.extend(ids_starts_with);
            item_ids.extend(ids_contains);
            item_ids.extend(ids_match);

            ItemSearchResult {
                key_word: Some(k.to_string()),
                item_ids,
                index_eq,
                index_starts_with,
                index_contains,
                index_match,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use crate::builtin_plugins::{
            base::{ItemDesc, ItemType, KeyWord},
            common::Item,
        };

        fn item(the_id: ItemId, key_word: &str) -> Item {
            Item {
                the_id,
                the_type: ItemType::Snippets,
                key_word: KeyWord(key_word.to_string()),
                name: key_word.to_string(),
                desc: ItemDesc::Str(String::new()),
            }
        }

        #[test]
        fn find_match_by_sub_sequence() {
            assert!(find_match("hello world", "hwd"));
            assert!(!find_match("hello world", "hdr"));
            assert!(!find_match("abc", "abcd")); // 关键字比输入更短
            assert!(find_match("中文测试", "中测"));
        }

        #[test]
        fn search_by_priority() {
            let item_collection = ItemCollection {
                item_list: vec![
                    item(1, "abc"),   // eq
                    item(2, "abcde"), // starts_with
                    item(3, "xabc"),  // contains
                    item(4, "axbxc"), // 子序列
                    item(5, "zzz"),   // 不匹配
                ],
            };

            let result = item_collection.search("abc");

            assert_eq!(result.item_ids, vec![1, 2, 3, 4]);
            assert_eq!(result.index_eq, 0);
            assert_eq!(result.index_starts_with, 1);
            assert_eq!(result.index_contains, 2);
            assert_eq!(result.index_match, 3);
            assert_eq!(result.key_word.as_deref(), Some("abc"));
        }

        /// 同一个 item 的多个 key_word 命中不同模式时，只保留优先级最高的那次
        #[test]
        fn duplicate_id_keeps_the_first_match() {
            let item_collection = ItemCollection {
                item_list: vec![
                    item(1, "xabc"), // contains
                    item(1, "abc"),  // eq 同 id ，优先级更高，但排在后面
                    item(2, "zzz"),  // 不匹配，不能占用 id
                    item(2, "zabc"), // contains
                ],
            };

            let result = item_collection.search("abc");

            assert_eq!(result.item_ids, vec![1, 2]);
            assert_eq!(result.index_contains, 0);
            assert_eq!(result.index_match, 2);
        }

        #[test]
        fn empty_key_word_matches_all() {
            let item_collection = ItemCollection {
                item_list: vec![item(1, "a"), item(2, "b")],
            };

            let result = item_collection.search("");

            assert_eq!(result.item_ids, vec![1, 2]);
            assert_eq!(result.index_eq, 0);
            assert_eq!(result.index_starts_with, 0);
            assert_eq!(result.index_contains, 2);
            assert_eq!(result.index_match, 2);
        }

        /// 空串关键字与“未检索”不能混淆
        #[test]
        fn empty_key_word_is_not_cached() {
            let result = ItemSearchResult::default();
            assert!(!result.is_current_result(""));
            assert!(!result.is_current_result("abc"));

            let result = ItemCollection {
                item_list: vec![item(1, "a")],
            }
            .search("");
            assert!(result.is_current_result(""));
            assert!(!result.is_current_result("a"));
        }

        #[test]
        fn page_resolve_item_id() {
            let item_collection = ItemCollection {
                item_list: vec![item(1, "abc"), item(2, "abcd"), item(3, "xabc")],
            };
            let result = item_collection.search("abc");

            let page = result.page(0, &item_collection);
            assert_eq!(page.total, 3);
            assert_eq!(page.index, 0);
            assert_eq!(page.item_list.len(), 3);
            assert_eq!(page.item_list[0].name, "abc");
            assert_eq!(page.item_list[2].name, "xabc");
            assert_eq!(page.index_contains, 2);

            // 越界时返回空页
            let page = result.page(100, &item_collection);
            assert_eq!(page.index, 3);
            assert!(page.item_list.is_empty());
        }
    }
}
