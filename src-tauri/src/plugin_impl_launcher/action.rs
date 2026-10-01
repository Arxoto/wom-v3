//! launcher 的动作表与派发
//!
//! 动作清单与顺序**原样收下**（Q33），第一个是默认动作：
//!
//! | 类型 | 动作 |
//! | --- | --- |
//! | `sys` | 空（每个命令的动作就是描述内容） |
//! | `cmd` | `copy` |
//! | `web` | `open_url`、`copy` |
//! | `scan` | `open_path`、`reveal`、`copy` |
//!
//! 这一轮**只实现 `copy`**（Q36），其余三个动作记 warn 并返回 [`ActionOutcome::NoOp`]：
//! "还没实现的动作不隐藏窗口"这条行为靠的就是 NoOp 与 Failed 分开（复刻 `action.rs:219`）。
//!
//! `label_key` 的命名规范是**四段式**：`action.<插件>.<类型>.<动作>`（Q35），
//! 于是现有键迁移为 `action.launcher.scan.open_path` 这类形状。框架不强制这个规范，
//! 只当不透明字符串透传——规范是 launcher 自己遵守的约定。

use crate::{
    plugin_framework::{ActionId, ActionOutcome, PluginAction, PluginContext, PluginItem},
    plugin_impl_launcher::persistence::launcher_source::LauncherItemType,
};

/// 插件标识：稳定 ASCII
pub const LAUNCHER_PLUGIN_ID: &str = "launcher";

/// 动作 id。字符串形式是它在前端与动作派发里的名字
pub const ACTION_COPY: &str = "copy";
pub const ACTION_OPEN_URL: &str = "open_url";
pub const ACTION_OPEN_PATH: &str = "open_path";
pub const ACTION_REVEAL: &str = "reveal";

/// launcher 的动作表
///
/// `sys` 是空表：每个系统命令的动作就是描述内容，不挂动作。
pub fn actions() -> Vec<PluginAction> {
    vec![
        PluginAction::new(
            LauncherItemType::Cmd.as_str(),
            ACTION_COPY,
            label_key(LauncherItemType::Cmd, ACTION_COPY),
        ),
        PluginAction::new(
            LauncherItemType::Web.as_str(),
            ACTION_OPEN_URL,
            label_key(LauncherItemType::Web, ACTION_OPEN_URL),
        ),
        PluginAction::new(
            LauncherItemType::Web.as_str(),
            ACTION_COPY,
            label_key(LauncherItemType::Web, ACTION_COPY),
        ),
        PluginAction::new(
            LauncherItemType::Scan.as_str(),
            ACTION_OPEN_PATH,
            label_key(LauncherItemType::Scan, ACTION_OPEN_PATH),
        ),
        PluginAction::new(
            LauncherItemType::Scan.as_str(),
            ACTION_REVEAL,
            label_key(LauncherItemType::Scan, ACTION_REVEAL),
        ),
        PluginAction::new(
            LauncherItemType::Scan.as_str(),
            ACTION_COPY,
            label_key(LauncherItemType::Scan, ACTION_COPY),
        ),
    ]
}

/// 四段式文案键：`action.<插件>.<类型>.<动作>`
///
/// 键是**静态字面量**，不拼运行时字符串：前端文案表是编译期资产，
/// 这里拼一个错的键，只有点下去才会发现。
pub fn label_key(the_type: LauncherItemType, action_id: &str) -> String {
    match (the_type, action_id) {
        (LauncherItemType::Cmd, ACTION_COPY) => "action.launcher.cmd.copy",
        (LauncherItemType::Web, ACTION_OPEN_URL) => "action.launcher.web.open_url",
        (LauncherItemType::Web, ACTION_COPY) => "action.launcher.web.copy",
        (LauncherItemType::Scan, ACTION_OPEN_PATH) => "action.launcher.scan.open_path",
        (LauncherItemType::Scan, ACTION_REVEAL) => "action.launcher.scan.reveal",
        (LauncherItemType::Scan, ACTION_COPY) => "action.launcher.scan.copy",
        // 表里没有的组合：返回一个明显挂在 launcher 名下的键，而不是拼一个像样的假名
        _ => "action.launcher.unknown",
    }
    .to_string()
}

/// 把条目类型串回 [`LauncherItemType`]
///
/// 类型名是框架透传过来的不透明字符串，只有 launcher 自己知道这几个值。
fn item_type_of(item: &PluginItem) -> Option<LauncherItemType> {
    use std::str::FromStr;

    LauncherItemType::from_str(&item.the_type).ok()
}

/// 跑一个动作
///
/// `item` 的正文都在 `desc` 里：命令是命令行、网页是链接、扫描条目是完整路径。
pub fn run(cx: &dyn PluginContext, item: &PluginItem, action_id: &ActionId) -> ActionOutcome {
    let Some(the_type) = item_type_of(item) else {
        cx.log_warn(&format!(
            "launcher run action on unknown item type: {}",
            item.the_type
        ));
        return ActionOutcome::NoOp;
    };

    match (the_type, action_id.as_str()) {
        // 系统命令的动作就是描述内容，没有可跑的动作
        (LauncherItemType::Sys, _) => {
            cx.log_warn(&format!("launcher sys item has no action: {action_id}"));
            ActionOutcome::NoOp
        }
        (_, ACTION_COPY) => copy_desc(cx, item),
        // 其余动作只出现在表里，本 effort 不实现：当无操作，也不让窗口在一件什么都没发生的事上消失。
        // 写全每一个变体，加新动作时会在这里被拦一下
        (_, ACTION_OPEN_URL) | (_, ACTION_OPEN_PATH) | (_, ACTION_REVEAL) => {
            cx.log_warn(&format!(
                "launcher action not implemented yet: {action_id}"
            ));
            ActionOutcome::NoOp
        }
        (_, _) => {
            cx.log_warn(&format!("launcher unknown action: {action_id}"));
            ActionOutcome::NoOp
        }
    }
}

/// 把条目正文写进系统剪贴板
///
/// [`PluginContext`] 这一轮**没有**剪贴板写入（Q15 的最小集合），所以这里如实返回
/// [`ActionOutcome::Failed`] 并记 warn：报 Done 会让调用方以为剪贴板真的被写了，
/// 接入期补上 `PluginContext::write_clipboard` 时，这里只剩一行替换（见接入清单第 4 条）。
fn copy_desc(cx: &dyn PluginContext, item: &PluginItem) -> ActionOutcome {
    cx.log_warn(&format!(
        "launcher copy action needs PluginContext clipboard support, not available yet: {}",
        item.name
    ));

    ActionOutcome::Failed
}
