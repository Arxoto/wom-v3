//! Item Action 的动作表与执行
//!
//! 每种 [`ItemType`] 支持哪些动作在这里写死，顺序即优先级——第一个是该条目的默认动作，
//! 也是行内显示的那一个（见 spec §4.2）。
//!
//! 表里只放元数据：ASCII 的 [`ItemActionId`] 给前端查图标、给派发动作那条命令认动作；
//! `label_key` 给前端查文案，所以中文一个字都不进 Rust，同一个 `copy` 在 `File` 与 `Web`
//! 上才能各有各的说法。
//!
//! 执行只有一个入口 [`run`]，与表放在一起：加一条动作要同时动到「动作叫什么」与
//! 「动作做什么」，分在两个文件里就总会漏掉一半。本 effort 只有 `copy` 真的做事情。

use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{Serialize, Serializer};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_log::log::warn;

use super::{base::ItemType, common::Item, persistence::load::ItemCollection};

/// Item Action 的标识
///
/// 字符串形式（[`ItemActionId::as_str`]）是它在前端与 `run_item_action` 命令里的名字，
/// 前端手写镜像这个联合类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemActionId {
    /// 复制
    Copy,
    /// 用默认浏览器打开
    OpenUrl,
    /// 用默认方式打开
    OpenPath,
    /// 在文件夹中选中
    Reveal,
    /// 打开笔记
    OpenNote,
}

const ACTION_COPY: &str = "copy";
const ACTION_OPEN_URL: &str = "open_url";
const ACTION_OPEN_PATH: &str = "open_path";
const ACTION_REVEAL: &str = "reveal";
const ACTION_OPEN_NOTE: &str = "open_note";

pub struct ItemActionIdParseFailed;

impl ItemActionId {
    /// 动作的字符串形式，也是它在元数据与 `run_item_action` 载荷里的名字
    pub fn as_str(&self) -> &'static str {
        match self {
            ItemActionId::Copy => ACTION_COPY,
            ItemActionId::OpenUrl => ACTION_OPEN_URL,
            ItemActionId::OpenPath => ACTION_OPEN_PATH,
            ItemActionId::Reveal => ACTION_REVEAL,
            ItemActionId::OpenNote => ACTION_OPEN_NOTE,
        }
    }
}

impl Display for ItemActionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 认不出的动作名就是解析失败，由调用方当无操作处理（见 [`run`]）
impl FromStr for ItemActionId {
    type Err = ItemActionIdParseFailed;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ACTION_COPY => Ok(Self::Copy),
            ACTION_OPEN_URL => Ok(Self::OpenUrl),
            ACTION_OPEN_PATH => Ok(Self::OpenPath),
            ACTION_REVEAL => Ok(Self::Reveal),
            ACTION_OPEN_NOTE => Ok(Self::OpenNote),
            _ => Err(ItemActionIdParseFailed),
        }
    }
}

/// 序列化成 [`ItemActionId::as_str`] 那一份字符串
///
/// 手写而不是 `rename_all`：名字只有一处（那些 `ACTION_*` 常量），
/// 派生宏另写一份就多一处要跟着改
impl Serialize for ItemActionId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// 一个 Item Action 的元数据
///
/// `label_key` 按 [`ItemType`] + 动作分套：同一个 `copy` 在 `File` 上是「复制完整路径」、
/// 在 `Web` 上是「复制链接」，所以文案键不能只按动作分。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ItemAction {
    pub id: ItemActionId,
    pub label_key: &'static str,
}

/// 系统命令不挂动作，每个命令的动作就是描述内容
const SYS_ACTIONS: &[ItemAction] = &[];

/// 片段只能复制，没有别的动作可做
const SNIP_ACTIONS: &[ItemAction] = &[ItemAction {
    id: ItemActionId::Copy,
    label_key: "action.snip.copy",
}];

/// 笔记目前只有占位动作，真实的笔记窗口另起一条线
const NOTE_ACTIONS: &[ItemAction] = &[ItemAction {
    id: ItemActionId::OpenNote,
    label_key: "action.note.open_note",
}];

/// todo 命令支持「可复制并自动打开终端、后台执行」
const CMD_ACTIONS: &[ItemAction] = &[ItemAction {
    id: ItemActionId::Copy,
    label_key: "action.cmd.copy",
}];

const WEB_ACTIONS: &[ItemAction] = &[
    ItemAction {
        id: ItemActionId::OpenUrl,
        label_key: "action.web.open_url",
    },
    ItemAction {
        id: ItemActionId::Copy,
        label_key: "action.web.copy",
    },
];

const FILE_ACTIONS: &[ItemAction] = &[
    ItemAction {
        id: ItemActionId::OpenPath,
        label_key: "action.file.open_path",
    },
    ItemAction {
        id: ItemActionId::Reveal,
        label_key: "action.file.reveal",
    },
    ItemAction {
        id: ItemActionId::Copy,
        label_key: "action.file.copy",
    },
];

/// 扫描得到的条目与 [`ItemType::File`] 同类，行为一样
const SCAN_ACTIONS: &[ItemAction] = &[
    ItemAction {
        id: ItemActionId::OpenPath,
        label_key: "action.scan.open_path",
    },
    ItemAction {
        id: ItemActionId::Reveal,
        label_key: "action.scan.reveal",
    },
    ItemAction {
        id: ItemActionId::Copy,
        label_key: "action.scan.copy",
    },
];

/// 某一种 [`ItemType`] 支持的动作，顺序即优先级（第一个是默认动作）
///
/// 不按平台剔除动作：桌面三平台上这些动作都有官方路径，真缺 API 时再说。
pub fn of(the_type: ItemType) -> &'static [ItemAction] {
    match the_type {
        ItemType::Snippets => SNIP_ACTIONS,
        ItemType::System => SYS_ACTIONS,
        ItemType::Note => NOTE_ACTIONS,
        ItemType::Cmd => CMD_ACTIONS,
        ItemType::Web => WEB_ACTIONS,
        ItemType::File => FILE_ACTIONS,
        ItemType::Scan => SCAN_ACTIONS,
    }
}

/// [`table`] 的形状：[`ItemType::as_str`] → 该类型的动作表
pub type ItemActionTable = BTreeMap<&'static str, &'static [ItemAction]>;

/// 全部 [`ItemType`] 的动作表
///
/// 前端只在挂载时拉一次（配置重载会重建窗口，不需要热更新），所以一次给全，
/// 连空表的 `System` 也算上，前端不必自己补空数组。
pub fn table() -> ItemActionTable {
    [
        ItemType::Snippets,
        ItemType::System,
        ItemType::Note,
        ItemType::Cmd,
        ItemType::Web,
        ItemType::File,
        ItemType::Scan,
    ]
    .into_iter()
    .map(|the_type| (the_type.as_str(), of(the_type)))
    .collect()
}

/// 按下标取出条目、跑它的一个 Item Action
///
/// 返回动作是否**真的执行了**：只有执行了，调用方才轮到「按 `main_window_mode`
/// 决定要不要隐藏窗口」那一问（见 spec §4.3）。
/// 越界索引、认不出的动作名、以及还没实现的动作，都当无操作并记 warn——
/// 不 panic、不隐藏、不做版本校验；前端不提示失败。
pub fn run(
    app: &AppHandle,
    item_collection: &ItemCollection,
    item_index: usize,
    action_id: &str,
) -> bool {
    let Some(item) = item_collection.get_by_index(item_index) else {
        warn!("run item action with out-of-range item index: {item_index}");
        return false;
    };

    let Ok(id) = ItemActionId::from_str(action_id) else {
        warn!("run item action with unknown action: {action_id}");
        return false;
    };

    match id {
        ItemActionId::Copy => copy_to_clipboard(app, item),
        // 其余动作只出现在表里，本 effort 不实现：当无操作，也不让窗口在
        // 一件什么都没发生的事上消失。写全每一个变体，加新动作时会在这里被拦一下
        ItemActionId::OpenUrl
        | ItemActionId::OpenPath
        | ItemActionId::Reveal
        | ItemActionId::OpenNote => {
            warn!("item action not implemented yet: {id}");
            false
        }
    }
}

/// 把条目内容写进系统剪贴板
///
/// 能复制的条目，要复制的正文都在 `desc` 里：片段是内容本身、网页是链接、
/// 命令是命令行、文件与扫描条目是完整路径。
fn copy_to_clipboard(app: &AppHandle, item: &Item) -> bool {
    match app.clipboard().write_text(item.desc.to_string()) {
        Ok(()) => true,
        Err(err) => {
            warn!("write text to clipboard failed: {err}");
            false
        }
    }
}
