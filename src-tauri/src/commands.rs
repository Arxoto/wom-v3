//! 前端调用的命令
//!
//! 实现的归属：能留在各自模块里的就只在这里转发（如检索命令，见 [`crate::plugin_framework`]）；
//! 配置相关的命令要同时指挥 configs / window_utils / global_shortcut，实现直接落在这里——
//! 放进 `configs` 会让它反向依赖 `window_utils`，而依赖只能单向流动。
//!
//! 应用只有一张命令注册表：`lib.rs` 的 `invoke_handler`。
//!
//! 检索与动作走插件注册表那一套（`plugin_search`、`plugin_run_item_action` …）。
//! 这一套**没有**取动作表的命令：界面用的动作顺序、图标与文案都在前端的插件注册表里
//! （见 `docs/adr/0010`）；条目自带的动作 id 列表随检索结果一起下来。

pub mod app;
pub mod plugin;
