//! 内建插件的基础字段定义
//!
//! 包括 [`KeyWords`] [`ItemType`] [`ItemDesc`]

use std::{fmt::Display, path::PathBuf, str::FromStr};

use serde::{Serialize, Serializer};

/// 关键字列表
///
/// 检索时取所有关键字中优先级最高的匹配模式（见 [`crate::builtin_plugins::search`]）
pub type KeyWords = Vec<String>;

// region: ItemType

// Clipboard 剪贴板增强，纯文本复制和预览没必要做，历史管理又涉及存储老化太重了
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    /// System 内置实现的系统命令
    Sys,
    /// 命令 可复制、后台静默执行（待 Note 完成后可复用以实现输出展示），
    /// 目前暂不规划实现自动化能力，但可以基于脚本间接实现：
    /// - 更自动化一点 AutoHotkey(Windows) / AppleScript(macOS)
    /// - 更集成一点的方案 enigo （仅控制输入无法识别聚焦的窗口），注意必须 app_handle.run_on_main_thread 主线程执行
    Cmd,
    /// 网页 支持使用默认浏览器打开、复制连接
    Web,
    /// 基于路径扫描得到文件
    /// - 暂时简单实现，不做索引、文件变更通知等高级能力
    Scan,
    /// 笔记 MarkDownLite 自定义简化语法，窗口渲染
    /// - 使用 React 组件属性 dangerouslySetInnerHTML 实现注入 html 语法
    /// - 使用 React useEffect 对渲染的内容增加事件监听（如最下面的实现）
    /// - 使用 Tauri convertFileSrc 将本地路径转换（或使用自定义协议，需要自己读取文件并根据后缀添加 Response 头）
    /// - 文件变更通知
    /// - 默认样式限制图片显示
    Note,
    /// 片段 仅允许复制
    Snippets,
}

pub const ITEM_SYS: &str = "sys";
pub const ITEM_CMD: &str = "cmd";
pub const ITEM_WEB: &str = "web";
pub const ITEM_SCAN: &str = "scan";
pub const ITEM_NOTE: &str = "note";
pub const ITEM_SNIPPETS: &str = "snip";

impl ItemType {
    /// 类型的字符串形式，也是它在设置文件与检索结果里的名字
    pub fn as_str(&self) -> &'static str {
        match self {
            ItemType::Sys => ITEM_SYS,
            ItemType::Cmd => ITEM_CMD,
            ItemType::Web => ITEM_WEB,
            ItemType::Scan => ITEM_SCAN,
            ItemType::Note => ITEM_NOTE,
            ItemType::Snippets => ITEM_SNIPPETS,
        }
    }
}

// 自动实现 to_string
impl Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

pub struct ItemTypeParseFailed;

impl FromStr for ItemType {
    type Err = ItemTypeParseFailed;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ITEM_SYS => Ok(Self::Sys),
            ITEM_CMD => Ok(Self::Cmd),
            ITEM_WEB => Ok(Self::Web),
            ITEM_SCAN => Ok(Self::Scan),
            ITEM_NOTE => Ok(Self::Note),
            ITEM_SNIPPETS => Ok(Self::Snippets),
            _ => Err(ItemTypeParseFailed),
        }
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
        match self {
            ItemDesc::Str(s) => s.fmt(f),
            ItemDesc::Path(p) => p.to_string_lossy().fmt(f),
        }
    }
}

impl Serialize for ItemDesc {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

// endregion
