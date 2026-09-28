//! Item 行的统一解析

use std::str::FromStr;

use crate::builtin_plugins::{
    base::{ItemType, ItemTypeParseFailed},
    common::ItemData,
};

pub const SPLIT_LINE: &str = "<->";
pub const SPLIT_KEY: &str = " ";

/// 一行 Item 的字段数：type / priority / key_words / name / desc
pub const ITEM_FIELD_COUNT: usize = 5;

/// 未配置 priority 时的默认值
pub const DEFAULT_PRIORITY: i32 = 0;

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

/// 一行的原始字段，全部为 String
#[derive(Debug, Clone)]
pub struct ItemParsed {
    pub the_type: String,
    pub priority: String,
    pub key_words: String,
    pub name: String,
    pub desc: String,
}

impl FromStr for ItemParsed {
    type Err = ItemParseErr;

    /// 以行为单位解析，字段以 [`SPLIT_LINE`] 分割
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        if line.trim().is_empty() {
            return Err(ItemParseErr::EmptyLine);
        }

        let mut values = split_line(line);
        if values.len() <= 1 {
            return Err(ItemParseErr::ValueNotEnough(
                "values must be at least 2 for Item parsing".to_string(),
            ));
        }

        let the_type = ItemType::from_str(&values[0])?;
        check_value_count(&values, ITEM_FIELD_COUNT, the_type)?;

        Ok(ItemParsed {
            the_type: std::mem::take(&mut values[0]),
            priority: std::mem::take(&mut values[1]),
            key_words: std::mem::take(&mut values[2]),
            name: std::mem::take(&mut values[3]),
            desc: std::mem::take(&mut values[4]),
        })
    }
}

impl ItemParsed {
    /// 取出公共字段：priority 解析为 i32，key_words 按 [`SPLIT_KEY`] 切分
    pub(super) fn common_data(self) -> Result<ItemData, ItemParseErr> {
        Ok(ItemData {
            priority: parse_priority(&self.priority)?,
            key_words: split_key(&self.key_words),
            name: self.name,
            desc: self.desc,
        })
    }
}

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

/// 解析单个 priority：空串或纯空白取 [`DEFAULT_PRIORITY`]，认不出的值报错
pub(super) fn parse_priority(raw: &str) -> Result<i32, ItemParseErr> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(DEFAULT_PRIORITY);
    }
    raw.parse::<i32>()
        .map_err(|_| ItemParseErr::ItemValueParsedFailed(format!("invalid priority: {raw}")))
}

fn check_value_count(
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::builtin_plugins::common::Item;

    #[test]
    fn parsed_must_trim() {
        let s = " cmd <-> 1 <-> a   s d <->  asdasd aasd <-> asdsdada  ";
        let item = ItemParsed::from_str(s).unwrap().into_cmd().unwrap();

        let Item::Cmd(data) = item else {
            panic!("not this");
        };

        assert_eq!(data.priority, 1);
        assert_eq!(data.key_words, vec!["a", "s", "d"]);
        assert_eq!(data.name, "asdasd aasd");
        assert_eq!(data.desc, "asdsdada");
    }
}
