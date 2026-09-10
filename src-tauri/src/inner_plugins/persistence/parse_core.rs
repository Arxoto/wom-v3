use std::str::FromStr;

use crate::inner_plugins::{
    base::{ItemType, ItemTypeParseFailed},
    common::Item,
    persistence::{
        parse_impl_common::ItemParsedCommon, parse_impl_scan::ItemParsedScan,
        parse_impl_system::ItemParsedSystem,
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

impl From<ItemTypeParseFailed> for ItemParseErr {
    fn from(_value: ItemTypeParseFailed) -> Self {
        Self::ItemTypeParsedFailed
    }
}

pub enum ItemParsed {
    Common(ItemParsedCommon),
    System(ItemParsedSystem),
    Scan(ItemParsedScan),
}

impl Item {
    pub(super) fn split_line(line: &str) -> Vec<String> {
        line.split(SPLIT_LINE)
            .map(|s| s.trim().to_string())
            .collect()
    }

    pub(super) fn split_key(key: &str) -> Vec<String> {
        key.split(SPLIT_KEY)
            .map(|k| k.trim())
            .filter(|k| !k.is_empty())
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
            return Err(ItemParseErr::ValueNotEnough(
                "values must be at least 2 for Item parsing".to_string(),
            ));
        }

        let item_type_str = &item_parsed_values[0];
        let item_type = ItemType::from_str(item_type_str)?;

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
