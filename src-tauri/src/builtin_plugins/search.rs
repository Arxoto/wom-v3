//! 检索逻辑

use serde::Serialize;

use crate::builtin_plugins::{common::Item, persistence::load::ItemCollection};

/// 搜索结果（后端）
///
/// 只保存下标，渲染用的 [`Item`] 在翻页时再从 [`ItemCollection`] 取出，
/// 避免为每条命中结果克隆一次 [`Item`]
///
/// 由 [`crate::builtin_plugins::plugins::PluginStat`] 持有
#[derive(Debug, Default)]
pub struct ItemSearchResult {
    /// 产生该结果的检索输入
    ///
    /// `None` 表示尚未进行过检索
    ///
    /// 使用 [`Option`] 而不是空串，避免“检索空串”和“未检索”互相混淆
    pub input_key: Option<String>,
    /// 按匹配模式分组后的 item 下标
    pub item_indexes: Vec<usize>,
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
        self.input_key.as_deref() == Some(k)
    }

    pub fn page(&self, index: usize, item_collection: &ItemCollection) -> ItemSearchPage {
        let start_index = self.item_indexes.len().min(index);
        let final_index = self.item_indexes.len().min(start_index + PAGE_SIZE);

        let item_list: Vec<ItemDisplay> = self.item_indexes[start_index..final_index]
            .iter()
            .filter_map(|index| item_collection.get_by_index(*index))
            .map(ItemDisplay::from)
            .collect();

        ItemSearchPage {
            total: self.item_indexes.len(),
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
    use crate::builtin_plugins::{persistence::load::ItemCollection, search::ItemSearchResult};

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

    /// 匹配模式，声明顺序即为优先级（越靠前优先级越高）
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    enum MatchMode {
        Eq,
        StartsWith,
        Contains,
        Match,
    }

    impl MatchMode {
        /// 得到单个关键字对输入的匹配模式，未命中返回 [`None`]
        #[inline]
        fn of(keyword: &str, k_input: &str) -> Option<Self> {
            if find_eq(keyword, k_input) {
                Some(Self::Eq)
            } else if find_starts_with(keyword, k_input) {
                Some(Self::StartsWith)
            } else if find_contains(keyword, k_input) {
                Some(Self::Contains)
            } else if find_match(keyword, k_input) {
                Some(Self::Match)
            } else {
                None
            }
        }
    }

    impl ItemCollection {
        /// 依次按照 精确 > 前缀 > 包含 > 子序列 检索
        ///
        /// 遍历全部关键字，只保留优先级最高的那次匹配对应的分组
        pub fn search(&self, k: &str) -> ItemSearchResult {
            let item_list = &self.item_list;

            // 四种匹配模式分别收集下标，既保证输出分组，又避免收集时克隆 [`Item`]
            let mut indexes_eq: Vec<usize> = vec![];
            let mut indexes_starts_with: Vec<usize> = vec![];
            let mut indexes_contains: Vec<usize> = vec![];
            let mut indexes_match: Vec<usize> = vec![];

            for (index, item) in item_list.iter().enumerate() {
                // 遍历该 item 的全部关键字，取优先级最高的匹配模式
                // 未命中任何关键字时不占用下标
                let mode = item
                    .key_words
                    .iter()
                    .filter_map(|keyword| MatchMode::of(keyword, k))
                    .min();

                let target = match mode {
                    Some(MatchMode::Eq) => &mut indexes_eq,
                    Some(MatchMode::StartsWith) => &mut indexes_starts_with,
                    Some(MatchMode::Contains) => &mut indexes_contains,
                    Some(MatchMode::Match) => &mut indexes_match,
                    None => continue,
                };

                target.push(index);
            }

            let index_eq = 0;
            let index_starts_with = index_eq + indexes_eq.len();
            let index_contains = index_starts_with + indexes_starts_with.len();
            let index_match = index_contains + indexes_contains.len();

            // 按优先级顺序拼接
            let mut item_indexes = Vec::with_capacity(index_match + indexes_match.len());
            item_indexes.extend(indexes_eq);
            item_indexes.extend(indexes_starts_with);
            item_indexes.extend(indexes_contains);
            item_indexes.extend(indexes_match);

            ItemSearchResult {
                input_key: Some(k.to_string()),
                item_indexes,
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
            base::{ItemDesc, ItemType},
            common::Item,
        };

        /// 下标由 `vec!` 中的位置决定
        fn item(key_words: &[&str]) -> Item {
            Item {
                the_type: ItemType::Snippets,
                key_words: key_words.iter().map(|s| s.to_string()).collect(),
                name: key_words.join(" "),
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
                    item(&["abc"]),   // 0: eq
                    item(&["abcde"]), // 1: starts_with
                    item(&["xabc"]),  // 2: contains
                    item(&["axbxc"]), // 3: 子序列
                    item(&["zzz"]),   // 4: 不匹配
                ],
            };

            let result = item_collection.search("abc");

            assert_eq!(result.item_indexes, vec![0, 1, 2, 3]);
            assert_eq!(result.index_eq, 0);
            assert_eq!(result.index_starts_with, 1);
            assert_eq!(result.index_contains, 2);
            assert_eq!(result.index_match, 3);
            assert_eq!(result.input_key.as_deref(), Some("abc"));
        }

        /// 同一个 item 的多个 key_words 命中不同模式时，只保留优先级最高的那次
        #[test]
        fn multi_key_word_keeps_the_highest_priority() {
            let item_collection = ItemCollection {
                item_list: vec![
                    item(&["xabc", "abc"]),   // 0: contains + eq ，取 eq
                    item(&["zzz", "axbxc"]),  // 1: 不匹配 + 子序列，取子序列
                    item(&["abcde", "xabc"]), // 2: starts_with + contains ，取 starts_with
                    item(&["zzz"]),           // 3: 全部不匹配，不占用下标
                ],
            };

            let result = item_collection.search("abc");

            assert_eq!(result.item_indexes, vec![0, 2, 1]);
            assert_eq!(result.index_eq, 0);
            assert_eq!(result.index_starts_with, 1);
            assert_eq!(result.index_contains, 2);
            assert_eq!(result.index_match, 2);
        }

        #[test]
        fn empty_key_word_matches_all() {
            let item_collection = ItemCollection {
                item_list: vec![item(&["a"]), item(&["b"])],
            };

            let result = item_collection.search("");

            assert_eq!(result.item_indexes, vec![0, 1]);
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
                item_list: vec![item(&["a"])],
            }
            .search("");
            assert!(result.is_current_result(""));
            assert!(!result.is_current_result("a"));
        }

        #[test]
        fn page_resolve_item_index() {
            let item_collection = ItemCollection {
                item_list: vec![item(&["abc"]), item(&["abcd"]), item(&["xabc"])],
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
