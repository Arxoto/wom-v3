use crate::builtin_plugins::{
    base::ItemType,
    common::Item,
    persistence::parse_core::{ItemParseErr, ItemParsed},
};

pub const IS_RECURSIVE: &str = "r";

/// [`ItemParsedScan::base`] 变量名由 [`tauri::path::BaseDirectory`] 解析，
/// 配置只写变量名即可，不必关心平台差异，各平台按系统惯例落到不同目录
/// （Windows 走 `FOLDERID_*`，Linux 走 `XDG_*`，macOS 走 `~/Library` 等）
/// 
///
/// 桌面三端通用的变量：
///
/// 用户目录：
/// - `$HOME` 用户主目录
/// - `$DESKTOP` 桌面
/// - `$DOWNLOAD` 下载目录
/// - `$DOCUMENT` 文档目录
/// - `$PICTURE` 图片目录
/// - `$AUDIO` 音乐目录
/// - `$VIDEO` 视频目录（macOS 下的落点是 `~/Movies`）
///
/// 系统目录：
/// - `$CONFIG` 用户配置目录，所有应用共用
/// - `$DATA` 用户数据目录，所有应用共用（Windows 上与 `$CONFIG` 同路径）
/// - `$LOCALDATA` 用户本机数据目录，所有应用共用且不随账户漫游
///   （Linux 上与 `$DATA` 同路径，macOS 上三者同为 `~/Library/Application Support`）
/// - `$RESOURCE` 应用只读资源目录，随安装位置走，开发时即构建产物目录
///
/// 应用目录，即上述同名目录下以应用标识符分隔的子目录，用于存放本应用自己的数据：
/// - `$APPCONFIG` 应用配置目录
/// - `$APPDATA` 应用数据目录
/// - `$APPLOCALDATA` 应用本机数据目录，不随账户漫游（Linux 上与 `$APPDATA` 同路径，macOS 上三者同路径）
pub struct ItemParsedScan {
    pub file_types: Vec<String>,
    pub file_suffix: Vec<String>,
    pub black_list: Vec<String>,
    pub recursive: bool,
    /// 根路径变量，为空时 [`Self::path`] 原样使用（此时应当为绝对路径）
    ///
    /// 非空但无法识别的变量会得到 [`ItemParseErr::ItemValueParsedFailed`]，
    /// 不会退化成相对路径
    pub base: String,
    pub path: String,
}

impl Item {
    /// - 第一个固定为 [`ItemType`]
    /// - 第二个表示匹配的文件类型，以 [`super::parse_core::SPLIT_KEY`] 分割
    /// - 第三个表示匹配的文件后缀，以 [`super::parse_core::SPLIT_KEY`] 分割
    /// - 第四个表示黑名单关键字，以 [`super::parse_core::SPLIT_KEY`] 分割
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
