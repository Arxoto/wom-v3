//! 插件框架：插件与条目的注册、检索、前端投影与动作派发
//!
//! 框架**不知道任何内容类型**：条目的类型名（[`PluginItem::the_type`]）只是一个不透明字符串，
//! 框架从不解释、匹配或列举它，只透传。框架只负责注册、检索、投影与把动作派回插件。
//!
//! 本模块不依赖 `launcher`、**不直接依赖 Tauri**：
//! 插件要用的宿主能力一律走 [`PluginContext`]。
//!
//! 公开面只有两处（见 `.scratch/plugin-system/spec.md` §1.8）：
//!
//! 1. [`PluginRegistry`]：构造 + 注册插件 + 检索 + 翻页 + 跑动作；
//! 2. 动作表查询函数 [`actions_of`]——它现在只服务于页面投影：投影按条目的类型名查表，
//!    把动作 id 列表写进 [`PluginItemDisplay::action_ids`]。
//!
//! 跑动作按 [`ItemHandle`] 寻址（Q23），不按条目在整集里的下标：主列表与
//! `Plugin Search Page` 两层列表因此走同一条派发路（见 [`PluginItemDisplay::handle`]）。
//!
//! 动作表**不再有对外的取用面**：界面那一侧的动作顺序、图标与文案由前端的插件注册表给出
//! （见 `src/plugins/registry.tsx`），Rust 只在投影时用它算条目自带的 `action_ids`。
//! 因此 [`PluginActionView`] 与 [`ActionTableView`] 是本模块内部类型，`pub` 只为了让
//! trait 实现与投影函数能指名它们。
//!
//! 其余类型（条目、动作、上下文、错误）在**类型层面**是 `pub` 的——插件的 trait 实现必须能
//! 指名它们——但本模块整体是私有的，所以它们对外不可达，等价于"只暴露两处入口"。
//! 注册表的块结构、`local_id` 的分配逻辑、[`PluginError`] 以外的内部类型都不对外可见。
//!
//! 本模块**已经接入应用**（接入前那份临时的 `allow(dead_code)` 已删除）：宿主侧见
//! `crate::plugin_host`（[`PluginContext`] 的实现与注册表的托管），命令层见
//! `crate::commands` 里那几个 `plugin_*` 命令。

mod error;
mod identity;
mod item;
mod plugin;
mod registry;
mod search;

pub use error::PluginError;
pub use identity::{ActionId, ItemHandle, PluginId};
pub use item::{ActionOutcome, PluginAction, PluginItem};
pub use plugin::{ItemRegistrar, Plugin, PluginContext};
pub use registry::PluginRegistry;
pub use search::{ItemSearchPage, PluginItemDisplay};
