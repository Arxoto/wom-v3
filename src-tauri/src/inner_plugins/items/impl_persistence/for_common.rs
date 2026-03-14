use crate::inner_plugins::{
    common::ItemType,
    items::{
        impl_persistence::core::{ItemParseErr, ItemParsed},
        Item,
    },
};

pub struct ItemParsedCommon {
    pub the_type: ItemType,
    pub key_words: Vec<String>,
    pub name: String,
    pub desc: String,
}

impl Item {
    /// - 第一个固定为 [`ItemType`]
    /// - 第二个为 [`crate::inner_plugins::base::KeyWord`] 的复数形式，以 [`crate::inner_plugins::items::impl_persistence::core::SPLIT_KEY`] 分割
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsed_must_trim() {
        let s = " Cmd <-> a   s d <->  asdasd aasd <-> asdsdada  ";
        let ll = Item::parse_str(s).unwrap();

        let item_parsed = match ll {
            ItemParsed::Common(item_parsed_common) => item_parsed_common,
            ItemParsed::System(_) => panic!("not this"),
            ItemParsed::Scan(_) => panic!("not this"),
        };

        assert_eq!(item_parsed.the_type, ItemType::Cmd);
        assert_eq!(item_parsed.key_words, vec!["a", "s", "d"]);
        assert_eq!(item_parsed.name, "asdasd aasd");
        assert_eq!(item_parsed.desc, "asdsdada");
    }
}
