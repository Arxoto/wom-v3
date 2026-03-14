pub mod impl_scan;
pub mod impl_system;

use crate::inner_plugins::{
    common::{ItemType, ItemTypeParsedFailed},
    items::{
        impl_persistence::{impl_scan::ItemParsedScan, impl_system::ItemParsedSystem},
        Item,
    },
};

pub const SPLIT_LINE: &str = "<->";
pub const SPLIT_KEY: &str = " ";

#[derive(Debug)]
pub enum ItemParseErr {
    EmptyLine,
    ValueNotEnough(String),
    ItemTypeParsedFailed,
    ItemValueParsedFailed(String),
}

impl From<ItemTypeParsedFailed> for ItemParseErr {
    fn from(_value: ItemTypeParsedFailed) -> Self {
        Self::ItemTypeParsedFailed
    }
}

pub enum ItemParsed {
    Common(ItemParsedCommon),
    System(ItemParsedSystem),
    Scan(ItemParsedScan),
}

pub struct ItemParsedCommon {
    pub the_type: ItemType,
    pub key_words: Vec<String>,
    pub name: String,
    pub desc: String,
}

impl Item {
    fn split_line(line: &str) -> Vec<String> {
        line.split(SPLIT_LINE)
            .map(|s| s.trim().to_string())
            .collect()
    }

    fn split_key(key: &str) -> Vec<String> {
        key.split(SPLIT_KEY)
            .map(|k| k.trim())
            .filter(|k| *k != "")
            .map(|v| v.to_string())
            .collect()
    }

    pub(super) fn check_value_count(
        values: &[String],
        count: usize,
        the_type: ItemType,
    ) -> Result<(), ItemParseErr> {
        if values.len() != count {
            Err(ItemParseErr::ValueNotEnough(format!(
                "type {} should have {} values for Item parsing",
                the_type, count,
            )))
        } else {
            Ok(())
        }
    }

    /// 以行为单位进行解析，以 [`SPLIT_LINE`] 进行分割
    pub fn parse_str(line: &str) -> Result<ItemParsed, ItemParseErr> {
        if line.trim().is_empty() {
            return Err(ItemParseErr::EmptyLine);
        }

        let item_parsed_values = Self::split_line(line);
        if item_parsed_values.len() <= 1 {
            return Err(ItemParseErr::ValueNotEnough(format!(
                "values must be at least 2 for Item parsing"
            )));
        }

        let item_type_str = &item_parsed_values[0];
        let item_type = ItemType::try_from(item_type_str)?;

        match item_type {
            ItemType::System => Self::parse_str_system(item_parsed_values),
            ItemType::Cmd
            | ItemType::Snippets
            | ItemType::Note
            | ItemType::Web
            | ItemType::File => Self::parse_str_common(item_type, item_parsed_values),
            ItemType::Scan => Self::parse_str_scan(item_parsed_values),
        }
    }

    /// - 第一个固定为 [`ItemType`]
    /// - 第二个为 [`crate::inner_plugins::base::KeyWord`] 的复数形式，以 [`SPLIT_KEY`] 分割
    /// - 第三个为 item_name
    /// - 第四个为 item_desc
    fn parse_str_common(
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
