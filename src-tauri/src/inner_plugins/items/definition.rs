use crate::inner_plugins::{base::KeyWord, common::{ItemDesc, ItemType}};

#[derive(Debug, Clone)]
pub struct Item {
    pub the_type: ItemType,
    pub key_word: KeyWord,
    pub name: String,
    pub desc: ItemDesc,
}

impl Item {
    pub fn new<S: Into<ItemDesc>>(the_type: ItemType, key_word: KeyWord, name: String, desc: S) -> Self {
        Self {
            the_type,
            key_word,
            name,
            desc: desc.into(),
        }
    }
}
