use std::sync::Mutex;

use serde::Serialize;

use crate::inner_plugins::items::Item;

/// 纯内存计算，直接使用 [`std::sync::Mutex`]
pub struct ItemSearchStat(pub Mutex<ItemSearchResult>);

#[derive(Debug)]
pub struct ItemSearchResult {
    pub key_word: String,
    pub item_list: Vec<Item>,
}

pub const PAGE_SIZE: usize = 100;

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
            item_list
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ItemDisplay {
    pub the_type: String,
    pub name: String,
    pub desc: String,
}

impl From<&Item> for ItemDisplay {
    fn from(value: &Item) -> Self {
        Self {
            the_type: String::from(value.the_type),
            name: value.name.clone(),
            desc: String::from(&value.desc),
        }
    }
}
