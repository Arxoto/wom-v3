//! 具体的 [`Item`] 实现

use crate::builtin_plugins::base::{ItemDesc, ItemType, KeyWords};

#[derive(Debug, Clone)]
pub struct Item {
    pub the_type: ItemType,
    pub key_words: KeyWords,
    pub name: String,
    pub desc: ItemDesc,
}

impl Item {
    /// item 不持有下标，
    /// 其下标即它在 [`ItemCollection::item_list`] 中的位置，
    /// 由 [`ItemCollection`] 放入时确定
    ///
    /// [`ItemCollection`]: crate::builtin_plugins::persistence::load::ItemCollection
    /// [`ItemCollection::item_list`]: crate::builtin_plugins::persistence::load::ItemCollection::item_list
    pub fn new<S: Into<ItemDesc>, K: IntoIterator<Item = String>>(
        the_type: ItemType,
        key_words: K,
        name: String,
        desc: S,
    ) -> Self {
        Self {
            the_type,
            key_words: key_words.into_iter().collect(),
            name,
            desc: desc.into(),
        }
    }
}
