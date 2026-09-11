//! 内建插件的基础字段定义
//!
//! 包括 [`ItemId`] [`KeyWord`] [`ItemType`] [`ItemDesc`]

use std::{fmt::Display, path::PathBuf, str::FromStr};

pub type ItemId = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyWord(pub String);

pub struct ItemTypeParseFailed;

// region: ItemType

/// todo
/// - Clipboard 剪贴板增强，纯文本复制、预览，不做历史管理，太重了，历史管理和增强有
///   - Windows 原生
///   - Ditto(Windows) https://github.com/sabrogden/Ditto
///   - CopyQ(Win/Mac/Linux) https://github.com/hluk/CopyQ
///   - Maccy(macOS) https://github.com/p0deje/Maccy
///   - FlowLauncher/Raycast 等启动软件集成
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    /// 片段 仅允许复制
    Snippets,
    /// 内置实现的系统命令
    System,
    /// 笔记 MarkDownLite 自定义简化语法，窗口渲染
    /// - 使用 React 组件属性 dangerouslySetInnerHTML 实现注入 html 语法
    /// - 使用 React useEffect 对渲染的内容增加事件监听（如最下面的实现）
    /// - 使用 Tauri convertFileSrc 将本地路径转换（或使用自定义协议，需要自己读取文件并根据后缀添加 Response 头）
    /// - 文件变更通知
    /// - 默认样式限制图片显示
    Note,
    /// 命令 可复制并自动打开终端、后台执行
    /// - 更自动化一点 AutoHotkey(Windows) / AppleScript(macOS)
    /// - 更集成一点的方案 enigo （仅控制输入无法识别聚焦的窗口），注意必须 app_handle.run_on_main_thread 主线程执行
    Cmd,
    /// 网页 支持使用默认浏览器打开、复制连接
    Web,
    /// 文件/文件夹/应用 支持默认方式打开、在文件夹中选中、复制完整路径 行为一样所以合并了
    File,
    /// 基于路径扫描得到文件
    /// - 可以自建数据库索引
    /// - 配合文件变更通知实时更新索引
    /// - 支持排除规则
    Scan,
}

const ITEM_SNIPPETS: &str = "snip";
const ITEM_SYSTEM: &str = "sys";
const ITEM_NOTE: &str = "note";
const ITEM_CMD: &str = "cmd";
const ITEM_WEB: &str = "web";
const ITEM_FILE: &str = "file";
const ITEM_SCAN: &str = "scan";

impl ItemType {
    pub fn as_str(&self) -> &str {
        match self {
            ItemType::Snippets => ITEM_SNIPPETS,
            ItemType::System => ITEM_SYSTEM,
            ItemType::Note => ITEM_NOTE,
            ItemType::Cmd => ITEM_CMD,
            ItemType::Web => ITEM_WEB,
            ItemType::File => ITEM_FILE,
            ItemType::Scan => ITEM_SCAN,
        }
    }
}

// 自动实现 to_string
impl Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "{}", self.as_str())
        f.write_str(self.as_str())
    }
}

impl FromStr for ItemType {
    type Err = ItemTypeParseFailed;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ITEM_SNIPPETS => Ok(Self::Snippets),
            ITEM_SYSTEM => Ok(Self::System),
            ITEM_NOTE => Ok(Self::Note),
            ITEM_CMD => Ok(Self::Cmd),
            ITEM_WEB => Ok(Self::Web),
            ITEM_FILE => Ok(Self::File),
            ITEM_SCAN => Ok(Self::Scan),
            _ => Err(ItemTypeParseFailed),
        }
    }
}

impl TryFrom<String> for ItemType {
    type Error = ItemTypeParseFailed;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

// endregion

// region: ItemDesc

#[derive(Debug, Clone)]
pub enum ItemDesc {
    Str(String),
    Path(PathBuf),
}

impl From<String> for ItemDesc {
    fn from(value: String) -> Self {
        Self::Str(value)
    }
}

// 必定成功 所以不是实现 FromStr
impl From<&str> for ItemDesc {
    fn from(value: &str) -> Self {
        Self::Str(value.to_string())
    }
}

impl From<PathBuf> for ItemDesc {
    fn from(value: PathBuf) -> Self {
        Self::Path(value)
    }
}

impl Display for ItemDesc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: &str = match self {
            ItemDesc::Str(s) => s,
            ItemDesc::Path(path_buf) => &path_buf.to_string_lossy(),
        };
        f.write_str(s)
    }
}

// endregion
