use crate::builtin_plugins::{
    base::ItemType,
    common::Item,
    persistence::parse_core::{ItemParseErr, ItemParsed},
};

pub struct ItemParsedCommon {
    pub the_type: ItemType,
    pub key_words: Vec<String>,
    pub name: String,
    pub desc: String,
}

impl Item {
    /// - 第一个固定为 [`ItemType`]
    /// - 第二个为 [`crate::builtin_plugins::base::KeyWords`] （同一个 item 的多个关键字），以 [`super::parse_core::SPLIT_KEY`] 分割
    /// - 第三个为 item_name
    /// - 第四个为 item_desc
    pub(super) fn parse_str_common(
        item_type: ItemType,
        mut item_parsed_values: Vec<String>,
    ) -> Result<ItemParsed, ItemParseErr> {
        Self::check_value_count(&item_parsed_values, 4, item_type)?;

        let item_key_word_list = Self::split_key(&item_parsed_values[1]);

        let name = std::mem::take(&mut item_parsed_values[2]);
        let desc = std::mem::take(&mut item_parsed_values[3]);

        Ok(ItemParsed::Common(ItemParsedCommon {
            the_type: item_type,
            key_words: item_key_word_list,
            name,
            desc,
        }))
    }
}
