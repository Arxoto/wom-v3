use super::*;

pub const IS_RECURSIVE: &str = "r";

pub struct ItemParsedScan {
    pub file_types: Vec<String>,
    pub file_suffix: Vec<String>,
    pub black_list: Vec<String>,
    pub recursive: bool,
    pub base: String,
    pub path: String,
}

impl Item {
    /// - 第一个固定为 [`ItemType`]
    /// - 第二个表示匹配的文件类型，以 [`SPLIT_KEY`] 分割
    /// - 第三个表示匹配的文件后缀，以 [`SPLIT_KEY`] 分割
    /// - 第四个表示黑名单关键字，以 [`SPLIT_KEY`] 分割
    /// - 第五个表示是否递归子目录，仅 [`IS_RECURSIVE`] 表示递归
    /// - 第六个表示根路径，可以为空
    /// - 第七个表示相对路径，当根路径为空时应该为绝对路径
    pub(super) fn parse_str_scan(
        mut item_parsed_values: Vec<String>,
    ) -> Result<ItemParsed, ItemParseErr> {
        Self::check_value_count(&item_parsed_values, 7, ItemType::Scan)?;

        let file_types = std::mem::take(&mut item_parsed_values[1]);
        let file_suffix = std::mem::take(&mut item_parsed_values[2]);
        let black_list = std::mem::take(&mut item_parsed_values[3]);
        let recursive = std::mem::take(&mut item_parsed_values[4]);
        let base = std::mem::take(&mut item_parsed_values[5]);
        let path = std::mem::take(&mut item_parsed_values[6]);

        let file_types = Self::split_key(&file_types);
        let file_suffix = Self::split_key(&file_suffix);
        let black_list = Self::split_key(&black_list);
        let recursive = recursive == IS_RECURSIVE;

        Ok(ItemParsed::Scan(ItemParsedScan {
            file_types,
            file_suffix,
            black_list,
            recursive,
            base,
            path,
        }))
    }
}
