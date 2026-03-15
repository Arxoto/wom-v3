use crate::inner_plugins::base::{ItemDesc, ItemId, ItemType, KeyWord};

#[derive(Debug, Clone)]
pub struct Item {
    pub the_id: ItemId,
    pub the_type: ItemType,
    pub key_word: KeyWord,
    pub name: String,
    pub desc: ItemDesc,
}

impl Item {
    // 传入 id 自增
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
}
