//! 具体的 [`Item`] 实现

use crate::builtin_plugins::base::{ItemDesc, ItemId, ItemType, KeyWord};

#[derive(Debug, Clone)]
pub struct Item {
    pub the_id: ItemId,
    pub the_type: ItemType,
    pub key_word: KeyWord,
    pub name: String,
    pub desc: ItemDesc,
}

impl Item {
    /// 传入 id 自增
    pub fn new<S: Into<ItemDesc>>(
        the_id: &mut ItemId,
        the_type: ItemType,
        key_word: KeyWord,
        name: String,
        desc: S,
    ) -> Self {
        *the_id += 1;
        Self {
            the_id: *the_id,
            the_type,
            key_word,
            name,
            desc: desc.into(),
        }
    }

    /// 传入 id 自增，生成多个不同 key_word 的副本
    pub fn new_list<S: Into<ItemDesc> + Clone, K: Iterator<Item = KeyWord>>(
        the_id: &mut ItemId,
        the_type: ItemType,
        key_words: K,
        name: String,
        desc: S,
    ) -> Vec<Self> {
        *the_id += 1;
        key_words
            .map(|key_word| Self {
                the_id: *the_id,
                the_type,
                key_word,
                name: name.clone(),
                desc: desc.clone().into(),
            })
            .collect()
    }
}
