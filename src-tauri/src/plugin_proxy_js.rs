//! JS 插件宿主：一个 `Plugin Package` 在框架里的代理 [`Plugin`]
//!
//! 插件的"两半"（spec §2.1）在这里见面：
//!
//! - **Rust 这一半**（本模块）：占住框架里的一个插件块。`id` / `actions` / `init` 全部只读
//!   清单，所以**条目能不能被搜到、动作表长什么样，都不依赖插件的 JS 能否成功执行**；
//! - **webview 那一半**：为这个包起一个
//!   Worker，在里面装载 `index.js`、交出类型与动作的图标和文案、跑插件自己的搜索与动作。
//!
//! 两半共用同一个 `PluginId`——清单的 `id`。没有这个代理，插件注册的条目一旦被触发，
//! 框架会走到一个不存在的插件上（spec §2.1）。
//!
//! 与 `plugin_impl_launcher` 的差别是这里**持有一个 [`AppHandle`]**：搜索与结果行的动作
//! 最后要回到 webview 里跑，这是一条只能由宿主侧发起的路。框架层（`plugin_framework`）依旧
//! 不依赖 Tauri，宿主侧的那一半才是碰 webview 的地方。
//!
//! 分工：
//!
//! - [`crate::plugin_package::manifest`]：清单的一行长什么样、怎么校验；
//! - [`crate::plugin_package`]：`Plugin Folder` 怎么扫成一个包的列表；
//! - 本文件：代理 [`JsPlugin`] 本身。

mod protocol;
mod proxy;

pub use protocol::{RowCache, SearchRequest, SearchRow, RESULT_LOCAL_ID_BASE};
pub use proxy::JsPlugin;
