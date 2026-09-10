use crate::builtin_plugins::{
    base::ItemType,
    common::Item,
    persistence::parse_core::{ItemParseErr, ItemParsed},
};

pub struct ItemParsedSystem {
    pub key_words: Vec<String>,
    pub name: String,
}

impl Item {
    /// - 第一个固定为 [`ItemType`]
    /// - 第二个为 [`crate::builtin_plugins::base::KeyWord`] 的复数形式，以 [`super::parse_core::SPLIT_KEY`] 分割
    /// - 第三个为 item_name
    pub(super) fn parse_str_system(
        mut item_parsed_values: Vec<String>,
    ) -> Result<ItemParsed, ItemParseErr> {
        Self::check_value_count(&item_parsed_values, 3, ItemType::System)?;

        let item_key_word_list = Self::split_key(&item_parsed_values[1]);

        let name = std::mem::take(&mut item_parsed_values[2]);

        Ok(ItemParsed::System(ItemParsedSystem {
            key_words: item_key_word_list,
            name,
        }))
    }
}
