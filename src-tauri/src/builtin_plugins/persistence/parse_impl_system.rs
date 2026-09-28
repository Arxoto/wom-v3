//! 系统命令类型的转换

use crate::builtin_plugins::{
    common::Item,
    persistence::parse_core::{ItemParseErr, ItemParsed},
};

impl ItemParsed {
    pub(super) fn into_system(self) -> Result<Item, ItemParseErr> {
        Ok(Item::System(self.common_data()?))
    }
}
