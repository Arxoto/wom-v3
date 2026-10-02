//! `manifest.json` 的解析：serde 反序列化 + 一次校验（见 `docs/adr/0014`）
//!
//! 字段是平的、数量固定，所以直接 `serde_json` 反序列化到 [`PackageManifest`]：
//! 缺必填字段、类型不对、JSON 本身坏了都由它报错，报错自带行列号。
//! 反序列化之后再补三件 serde 表达不了的事：`id` 是稳定 ASCII、`keywords` 非空、
//! `type: html` 必须给出 `html` 页面。
//!
//! 认不出的键**不报错**，只是忽略（serde 的默认行为）：清单里多写一个键不是这个包坏掉的理由。
//! 错误是**这个包的错误**（Q18/Q19）：调用方记一条 warn 后跳过这个包，框架看不到这些区别。

use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// 清单的文件名，包内固定名字
pub const MANIFEST_FILE_NAME: &str = "manifest.json";

/// 入口的缺省值：清单不写 `entry` 时用它
pub const DEFAULT_ENTRY: &str = "index.js";

/// 包的形态，也就是清单里的 `type`
///
/// 它取代了"`html` 非空就是前端插件"那条隐式判据（见 `docs/adr/0014`）：
/// `Js` 读 `entry`（缺省 [`DEFAULT_ENTRY`]），`Html` 读 `html`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageType {
    Js,
    Html,
}

/// 一个合法 `Plugin Package` 的清单内容
///
/// 字段与缺省值见 `docs/adr/0014`。`icon` / `desc` / `html` 用空串表示"没有"——
/// 它们是给人看的正文，空串本来就是"没有内容"，不必再多一层 `Option`。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PackageManifest {
    /// `PluginId`：稳定 ASCII 标识，也是这个包的身份（目录名不是）
    pub id: String,
    /// 插件条目的名字，缺省是 `id`
    #[serde(default)]
    pub name: String,
    /// 插件条目的描述
    #[serde(default)]
    pub desc: String,
    /// 主检索据它匹配出插件条目，**必填**
    pub keywords: Vec<String>,
    /// 包内相对路径的图片，空串表示没有
    #[serde(default)]
    pub icon: String,
    /// 包的形态，**必填**
    #[serde(rename = "type")]
    pub the_type: PackageType,
    /// 包内相对路径的 JS 入口，缺省 [`DEFAULT_ENTRY`]
    #[serde(default = "default_entry")]
    pub entry: String,
    /// 包内相对路径的 HTML 页面，空串表示没有；`type: html` 时必填
    #[serde(default)]
    pub html: String,
    /// 结果行的动作 id，**有序**（第一个是默认动作）
    #[serde(default)]
    pub actions: Vec<String>,
    /// 结果行的类型名，**有序**
    #[serde(default)]
    pub types: Vec<String>,
}

fn default_entry() -> String {
    DEFAULT_ENTRY.to_string()
}

/// 清单的错误：都是"这个包"的错误
#[derive(Debug)]
pub enum ManifestError {
    /// JSON 本身坏了（serde 的报错自带行列号）
    BadJson(serde_json::Error),
    /// `id` 是空串
    MissingId,
    /// `id` 不是稳定的 ASCII 标识
    BadId(String),
    /// `keywords` 是空表（或全是空串）
    MissingKeywords,
    /// `type` 是 `html`，却没有给出 `html` 页面
    MissingHtmlPage,
}

impl Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadJson(err) => write!(f, "manifest.json is not valid: {err}"),
            Self::MissingId => write!(f, "`id` is empty"),
            Self::BadId(id) => write!(f, "`id` is not a stable ASCII id: {id}"),
            Self::MissingKeywords => write!(f, "`keywords` must not be empty"),
            Self::MissingHtmlPage => write!(f, "`type` is html but `html` is empty"),
        }
    }
}

impl std::error::Error for ManifestError {}

/// 解析一份清单文本
///
/// 只做格式与必填项这两件事：路径存不存在、指向什么，是扫描那一层的事。
pub fn parse(text: &str) -> Result<PackageManifest, ManifestError> {
    let mut manifest: PackageManifest =
        serde_json::from_str(text).map_err(ManifestError::BadJson)?;

    if manifest.id.is_empty() {
        return Err(ManifestError::MissingId);
    }

    if !is_stable_ascii_id(&manifest.id) {
        return Err(ManifestError::BadId(manifest.id));
    }

    if manifest.name.is_empty() {
        manifest.name = manifest.id.clone();
    }

    normalize_list(&mut manifest.keywords);
    if manifest.keywords.is_empty() {
        return Err(ManifestError::MissingKeywords);
    }

    normalize_list(&mut manifest.actions);
    normalize_list(&mut manifest.types);

    if manifest.the_type == PackageType::Html && manifest.html.is_empty() {
        return Err(ManifestError::MissingHtmlPage);
    }

    Ok(manifest)
}

/// 列表字段的规整：两侧空白去掉、空项丢掉、重复的只留第一个
///
/// 去重是为了让动作表干净：同一个动作在清单里写两遍，框架那边会记一条
/// "already registered"，不如在这里就收掉。JSON 已经是数组，这一步只做规整，不做切分。
fn normalize_list(list: &mut Vec<String>) {
    let mut kept: Vec<String> = Vec::new();

    for item in list.drain(..) {
        let item = item.trim().to_string();

        if item.is_empty() {
            continue;
        }
        if kept.iter().any(|existing| existing == &item) {
            continue;
        }

        kept.push(item);
    }

    *list = kept;
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
