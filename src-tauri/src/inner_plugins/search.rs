use std::sync::Mutex;

use serde::Serialize;

use crate::inner_plugins::common::Item;

/// 缓存搜索结果
/// 纯内存计算，直接使用 [`std::sync::Mutex`]
pub struct ItemSearchStat(pub Mutex<ItemSearchResult>);

impl ItemSearchStat {
    pub fn new() -> Self {
        Self(Mutex::new(ItemSearchResult::default()))
    }
}

/// 搜索结果（后端）
#[derive(Debug, Default)]
pub struct ItemSearchResult {
    pub key_word: String,
    pub item_list: Vec<Item>,
}

/// 分页查询搜索结果
pub const PAGE_SIZE: usize = 100;

/// 搜索结果（前端）
#[derive(Debug, Serialize)]
pub struct ItemSearchPage {
    pub total: usize,
    pub index: usize,
    pub item_list: Vec<ItemDisplay>,
}

impl ItemSearchResult {
    pub fn is_current_result(&self, k: &str) -> bool {
        self.key_word == k
    }

    pub fn page(&self, index: usize) -> ItemSearchPage {
        let start_index = self.item_list.len().min(index);
        let final_index = self.item_list.len().min(start_index + PAGE_SIZE);

        let item_list: Vec<ItemDisplay> = self.item_list[start_index..final_index]
            .iter()
            .map(|item| ItemDisplay::from(item))
            .collect();

        ItemSearchPage {
            total: self.item_list.len(),
            index: start_index,
            item_list,
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
    use crate::inner_plugins::{base::KeyWord, common::Item, persistence::load::ItemCollection};

    impl KeyWord {
        #[inline]
        pub fn find_eq(&self, k: &str) -> bool {
            self.0 == k
        }

        #[inline]
        pub fn find_starts_with(&self, k: &str) -> bool {
            self.0.starts_with(k)
        }

        #[inline]
        pub fn find_contains(&self, k: &str) -> bool {
            self.0.contains(k)
        }

        pub fn find_match(&self, k: &str) -> bool {
            let key_bytes = self.0.as_bytes();
            let k_bytes = k.as_bytes();

            if k_bytes.len() > key_bytes.len() {
                return false;
            }

            // 子序列匹配 两个迭代器依次步进
            let mut key_iter = key_bytes.iter();
            k_bytes.iter().all(|s| key_iter.any(|t| t == s))
        }
    }

    impl ItemCollection {
        pub fn search(&self, k: &str) -> Vec<Item> {
            let item_list = &self.item_list;

            let mut list_eq: Vec<Item> = vec![];
            let mut list_starts_with: Vec<Item> = vec![];
            let mut list_contains: Vec<Item> = vec![];
            let mut list_match: Vec<Item> = vec![];
            for item in item_list {
                let key_word = &item.key_word;
                if key_word.find_eq(k) {
                    list_eq.push(item.clone());
                } else if key_word.find_starts_with(k) {
                    list_starts_with.push(item.clone());
                } else if key_word.find_contains(k) {
                    list_contains.push(item.clone());
                } else if key_word.find_match(k) {
                    list_match.push(item.clone());
                }
            }

            let mut item_list = list_eq;
            item_list.append(&mut list_starts_with);
            item_list.append(&mut list_contains);
            item_list.append(&mut list_match);

            // todo 增加 id
            // item_list.dedup_by(|a, b| a == b); // todo 去重
            item_list
        }
    }
}
