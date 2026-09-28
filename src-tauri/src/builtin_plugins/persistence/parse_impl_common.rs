//! 公共类型（片段、笔记、命令、网页）的转换

use crate::builtin_plugins::{
    common::Item,
    persistence::parse_core::{ItemParseErr, ItemParsed},
};

impl ItemParsed {
    pub(super) fn into_snippets(self) -> Result<Item, ItemParseErr> {
        Ok(Item::Snippets(self.common_data()?))
    }

    pub(super) fn into_note(self) -> Result<Item, ItemParseErr> {
        Ok(Item::Note(self.common_data()?))
    }

    pub(super) fn into_cmd(self) -> Result<Item, ItemParseErr> {
        Ok(Item::Cmd(self.common_data()?))
    }

    pub(super) fn into_web(self) -> Result<Item, ItemParseErr> {
        Ok(Item::Web(self.common_data()?))
    }
}
