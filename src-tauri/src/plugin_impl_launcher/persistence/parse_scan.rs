//! `scan` 类型的扫描配置
//!
//! 它就是 manifest 里 `desc` 那一行 JSON 的形状（照抄现状，这一轮不改文件语法）：
//! 字段、默认值与现有 `parse_impl_scan.rs` 的 `ScanConfigJson` 一致。

use std::path::Path;

use serde::{Deserialize, Serialize};

/// 一行 `scan` 的扫描配置
///
/// `Deserialize` 是manifest 那一行的读入方向；`Serialize` 只在测试里用来造一个
/// 已知转义形式的 `desc`（写回方向由 [`Self::to_json`] 手写，见那里的注释）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct LauncherScanConfig {
    #[serde(default)]
    pub file_types: Vec<String>,
    #[serde(default)]
    pub file_suffix: Vec<String>,
    #[serde(default)]
    pub black_list: Vec<String>,
    /// 递归最大层数，直接交给 walkdir：0 表示只取 `path` 本身
    #[serde(default)]
    pub max_depth: usize,
    /// 根路径变量，为空时 `path` 原样使用
    #[serde(default)]
    pub base: String,
    #[serde(default)]
    pub path: String,
}

impl LauncherScanConfig {
    /// 解析 `desc` 里的那一行 JSON
    pub fn parse(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|err| format!("parse scan desc as json failed: {err}"))
    }

    /// 写回 `desc` 里的那一行 JSON
    ///
    /// 手写而不是 `serde_json::to_string` + derive：`Display` 要走的是
    /// "字段固定、顺序固定、一行"这条确定性路径，不需要跟着结构体定义的顺序走。
    /// 字符串本身仍然交给 serde_json 转义，不自己拼引号。
    pub fn to_json(&self, path: &Path) -> String {
        let file_types = json_str_list(&self.file_types);
        let file_suffix = json_str_list(&self.file_suffix);
        let black_list = json_str_list(&self.black_list);
        let max_depth = self.max_depth;
        let base = json_str(&self.base);
        let path = json_str(&path.to_string_lossy());

        format!(
            "{{\"file_types\":[{file_types}],\"file_suffix\":[{file_suffix}],\"black_list\":[{black_list}],\"max_depth\":{max_depth},\"base\":{base},\"path\":{path}}}"
        )
    }
}

/// 一个字符串的 JSON 形式
fn json_str(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

/// 一组字符串的 JSON 形式
fn json_str_list(values: &[String]) -> String {
    values
        .iter()
        .map(|value| json_str(value))
        .collect::<Vec<String>>()
        .join(",")
}
