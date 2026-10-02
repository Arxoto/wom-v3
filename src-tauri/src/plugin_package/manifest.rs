//! `manifest.yml` 的解析：手写 `key: value` 行格式（见 `.scratch/js-plugin-host/spec.md` §1.3）
//!
//! 不用 YAML 解析器：字段是平的，仓库里也没有 YAML 依赖，而 launcher 的 `LauncherItemSource`
//! 已经证明这个仓库养得起手写行格式。规则只有四条：
//!
//! 1. 一行一条 `key: value`，按**第一个** `:` 切开；
//! 2. 行首是 `#` 的行是注释，空行（含只有空白的行）忽略；
//! 3. 键与值两侧的空白去掉，值内部原样保留，不做引号与转义处理；
//! 4. 同一个键写两遍，后写的赢。
//!
//! 认不出的键**不报错**，只是忽略：清单里多写一行不是这个包坏掉的理由。
//! 错误是**这个包的错误**（Q18/Q19）：调用方记一条 warn 后跳过这个包，框架看不到这些区别。

use std::fmt::Display;

/// 清单的文件名，包内固定名字
pub const MANIFEST_FILE_NAME: &str = "manifest.yml";

/// 入口的缺省值：清单不写 `entry` 时用它
pub const DEFAULT_ENTRY: &str = "index.js";

/// 一个合法 `Plugin Package` 的清单内容
///
/// 字段与缺省值见 spec §1.3 的表。`icon` / `desc` 用空串表示"没有"——
/// 它们是给人看的正文，空串本来就是"没有内容"，不必再多一层 `Option`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManifest {
    /// `PluginId`：稳定 ASCII 标识，也是这个包的身份（目录名不是）
    pub id: String,
    /// 插件条目的名字，缺省是 `id`
    pub name: String,
    /// 插件条目的描述
    pub desc: String,
    /// 主检索据它匹配出插件条目，**必填**
    pub keywords: Vec<String>,
    /// 包内相对路径的图片，空串表示没有
    pub icon: String,
    /// 包内相对路径的 JS 入口，缺省 [`DEFAULT_ENTRY`]
    pub entry: String,
    /// 结果行的动作 id，**有序**（第一个是默认动作）
    pub actions: Vec<String>,
    /// 结果行的类型名，**有序**
    pub types: Vec<String>,
}

/// 清单的错误：都是"这个包"的错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    /// 第 n 行不是 `key: value`（行号从 1 数起，与用户能对上的行号一致）
    BadLine(usize),
    /// 缺必填的 `id`
    MissingId,
    /// `id` 不是稳定的 ASCII 标识
    BadId(String),
    /// 缺必填的 `keywords`（写了但一个关键字都不剩也算）
    MissingKeywords,
}

impl Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadLine(line_number) => {
                write!(f, "line {line_number} is not a `key: value` line")
            }
            Self::MissingId => write!(f, "missing required key `id`"),
            Self::BadId(id) => write!(f, "`id` is not a stable ASCII id: {id}"),
            Self::MissingKeywords => write!(f, "missing required key `keywords`"),
        }
    }
}

impl std::error::Error for ManifestError {}

/// 解析一份清单文本
///
/// 只做格式与必填项这两件事：路径存不存在、指向什么，是扫描那一层的事。
pub fn parse(text: &str) -> Result<PackageManifest, ManifestError> {
    let mut id: Option<String> = None;
    let mut name: Option<String> = None;
    let mut desc: Option<String> = None;
    let mut keywords: Option<String> = None;
    let mut icon: Option<String> = None;
    let mut entry: Option<String> = None;
    let mut actions: Option<String> = None;
    let mut types: Option<String> = None;

    for (line_index, raw_line) in text.lines().enumerate() {
        // 行号从 1 数起：日志里报的行号就是用户在编辑器里看到的行号
        let line_number = line_index + 1;

        let line = raw_line.trim();
        // 空行是正常写法（分组用），不值一条日志
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once(':') else {
            return Err(ManifestError::BadLine(line_number));
        };

        let key = key.trim();
        let value = value.trim();

        match key {
            "id" => id = Some(value.to_string()),
            "name" => name = Some(value.to_string()),
            "desc" => desc = Some(value.to_string()),
            "keywords" => keywords = Some(value.to_string()),
            "icon" => icon = Some(value.to_string()),
            "entry" => entry = Some(value.to_string()),
            "actions" => actions = Some(value.to_string()),
            "types" => types = Some(value.to_string()),
            // 认不出的键忽略：清单里多写一行不是包坏掉的理由
            _ => {}
        }
    }

    let Some(id) = id.filter(|id| !id.is_empty()) else {
        return Err(ManifestError::MissingId);
    };

    if !is_stable_ascii_id(&id) {
        return Err(ManifestError::BadId(id));
    }

    // keywords 必填：主检索就是靠它把插件条目匹配出来的（spec §1.3 末）
    let keywords = split_list(&keywords.unwrap_or_default());
    if keywords.is_empty() {
        return Err(ManifestError::MissingKeywords);
    }

    let entry = entry.filter(|entry| !entry.is_empty());

    Ok(PackageManifest {
        name: name
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| id.clone()),
        desc: desc.unwrap_or_default(),
        keywords,
        icon: icon.unwrap_or_default(),
        entry: entry.unwrap_or_else(|| DEFAULT_ENTRY.to_string()),
        actions: split_list(&actions.unwrap_or_default()),
        types: split_list(&types.unwrap_or_default()),
        id,
    })
}

/// 逗号分隔的列表：两侧空白去掉、空项丢掉、重复的只留第一个
///
/// 去重是为了让动作表干净：同一个动作在清单里写两遍，框架那边会记一条
/// "already registered"，不如在这里就收掉。
fn split_list(text: &str) -> Vec<String> {
    let mut list: Vec<String> = Vec::new();

    for item in text.split(',') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        if list.iter().any(|kept| kept == item) {
            continue;
        }
        list.push(item.to_string());
    }

    list
}

/// 稳定的 ASCII 标识：非空、且只由 ASCII 字母数字与 `-` `_` `.` 组成
///
/// 它会被拼进 `label_key`（`action.<插件 id>.<类型>.<动作>`）与日志，所以不放行
/// 非 ASCII 的字符，也不放行空白与 `:`。
fn is_stable_ascii_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}
