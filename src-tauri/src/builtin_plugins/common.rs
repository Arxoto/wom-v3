//! 具体的 [`Item`] 实现

use serde::Serialize;

use crate::builtin_plugins::base::KeyWords;

/// Item 的数据部分，各种类型共用
#[derive(Debug, Clone, Serialize)]
pub struct ItemData {
    /// 排序用的优先级，解析时若未配置则为 0
    pub priority: i32,
    pub key_words: KeyWords,
    pub name: String,
    pub desc: String,
}

/// 一个 Item
///
/// 以 `the_type` 为 serde 的内部 tag，数据字段与 tag 平铺在同一层，
/// 前端据 `the_type` 区分类型
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "the_type")]
pub enum Item {
    #[serde(rename = "snip")]
    Snippets(ItemData),
    #[serde(rename = "sys")]
    System(ItemData),
    #[serde(rename = "note")]
    Note(ItemData),
    #[serde(rename = "cmd")]
    Cmd(ItemData),
    #[serde(rename = "web")]
    Web(ItemData),
    #[serde(rename = "scan")]
    Scan(ItemData),
}

impl Item {
    /// 数据部分，各种类型通用
    pub fn data(&self) -> &ItemData {
        match self {
            Item::Snippets(data)
            | Item::System(data)
            | Item::Note(data)
            | Item::Cmd(data)
            | Item::Web(data)
            | Item::Scan(data) => data,
        }
    }

    /// 排序用的优先级
    pub fn priority(&self) -> i32 {
        self.data().priority
    }
}
