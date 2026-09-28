//! 扫描类型的转换
//!
//! 类型、priority、key_words、name 取自 [`ItemParsed`] 的对应字段，
//! 其余扫描配置从 desc 的单行 json 中解析

use serde::Deserialize;

use crate::builtin_plugins::{
    base::KeyWords,
    persistence::parse_core::{split_key, ItemParseErr, ItemParsed, DEFAULT_PRIORITY},
};

/// `scan` 类型条目的解析结果
///
/// 根路径变量（[`Self::base`]）的取值范围与含义见
/// [`ScanBase`](crate::builtin_plugins::persistence::scan_base::ScanBase)，
/// 设置文件里手写的其它变量（如 `$TEMP`）同样有效，只是不出现在配置页下拉里
pub struct ScanConfig {
    /// 每一递归层级的优先级：下标即层级，越靠后的层级缺省沿用前一个下标的值
    pub priority: Vec<i32>,
    pub key_words: KeyWords,
    pub name: String,
    pub file_types: Vec<String>,
    pub file_suffix: Vec<String>,
    pub black_list: Vec<String>,
    /// 递归最大层数，直接作为 walkdir 的 max_depth：0 表示只取 [`Self::path`] 本身（即原 file 类型）
    pub max_depth: usize,
    /// 根路径变量，为空时 [`Self::path`] 原样使用（此时应当为绝对路径）
    ///
    /// 非空但无法识别的变量会得到 [`ItemParseErr::ItemValueParsedFailed`]，
    /// 不会退化成相对路径
    pub base: String,
    pub path: String,
}

/// desc 里单行 json 的形状
#[derive(Debug, Deserialize)]
struct ScanConfigJson {
    #[serde(default)]
    file_types: Vec<String>,
    #[serde(default)]
    file_suffix: Vec<String>,
    #[serde(default)]
    black_list: Vec<String>,
    #[serde(default)]
    max_depth: usize,
    #[serde(default)]
    base: String,
    #[serde(default)]
    path: String,
}

impl ItemParsed {
    pub(super) fn into_scan_config(self) -> Result<ScanConfig, ItemParseErr> {
        let json: ScanConfigJson = serde_json::from_str(&self.desc).map_err(|err| {
            ItemParseErr::ItemValueParsedFailed(format!("parse scan desc as json failed: {err}"))
        })?;

        Ok(ScanConfig {
            priority: parse_priority_chain(&self.priority)?,
            key_words: split_key(&self.key_words),
            name: self.name,
            file_types: json.file_types,
            file_suffix: json.file_suffix,
            black_list: json.black_list,
            max_depth: json.max_depth,
            base: json.base,
            path: json.path,
        })
    }
}

/// 解析逐层 priority：以 `,` 分割，空项沿用前一个下标的值，首个缺省为 [`DEFAULT_PRIORITY`]
fn parse_priority_chain(raw: &str) -> Result<Vec<i32>, ItemParseErr> {
    let mut chain: Vec<i32> = Vec::new();
    for part in raw.split(',') {
        let part = part.trim();
        let value = if part.is_empty() {
            chain.last().copied().unwrap_or(DEFAULT_PRIORITY)
        } else {
            part.parse::<i32>().map_err(|_| {
                ItemParseErr::ItemValueParsedFailed(format!("invalid scan priority: {part}"))
            })?
        };
        chain.push(value);
    }
    Ok(chain)
}
