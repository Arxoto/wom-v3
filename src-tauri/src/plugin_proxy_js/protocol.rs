use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::plugin_framework::PluginItem;

/// 插件条目（Plugin Item）的类型名
///
/// 它是**宿主自己定的**一个类型名，不是清单里的：清单的 `types` 是结果行的类型。
/// 框架对类型名永远不解释，所以宿主当然可以有一条自己的类型——前端注册表里给它配图标与文案。
pub const JS_PLUGIN_ITEM_TYPE: &str = "js_plugin";

/// 插件条目上唯一的动作：打开这个包的 `Plugin Search Page`
///
/// 真正打开搜索页由前端直接发起（Enter 走 `plugin_open_plugin_search`），
/// 这个动作 id 的作用是让那一行有动作可显示、可提示。
pub const ACTION_OPEN_SEARCH: &str = "open_search";

/// 插件条目动作的文案键
///
/// 四段式（Q35）里的"插件"这一段写宿主自己的名字：这个动作不是插件给的，
/// 是宿主给插件条目的。
pub const OPEN_SEARCH_LABEL_KEY: &str = "action.js_host.js_plugin.open_search";

/// 结果行（`Plugin Search Result`）的一条：插件自己报上来的纯数据（spec §3.3）
///
/// 全部字段都给缺省值：这是**从 webview 来的不可信数据**，缺一个字段不该让整次搜索炸掉，
/// 顶多是那一行画不出图标、点不出动作。
#[derive(Debug, Clone, Deserialize)]
pub struct SearchRow {
    /// 行类型名：前端据它查类型图标与动作文案
    #[serde(default)]
    pub the_type: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub desc: String,
    /// 这一行自己的动作 id 列表，顺序即优先级；空表示这一行没有动作（spec §4.4）
    #[serde(default)]
    pub action_ids: Vec<String>,
}

/// 发往 webview 的搜索请求
///
/// 带上入口的绝对路径：前端拿它 `convertFileSrc`，交给插件 Worker 用 `importScripts` 装载，
/// 于是取文件与执行都是 webview 自己的行为（ADR-0011）。
#[derive(Debug, Clone, Serialize)]
pub struct SearchRequest {
    pub plugin_id: String,
    pub entry: String,
    pub keyword: String,
}

/// 发往 webview 的动作请求
///
/// 只报身份（插件 + 行在结果里的下标 + 动作 id）：行内容在前端自己报上去的那一份里，
/// 不必来回搬。
#[derive(Debug, Clone, Serialize)]
pub struct ActionRequest {
    pub plugin_id: String,
    /// 行在最近一次搜索结果里的下标（前端的账本按它查那一行）
    pub row_id: usize,
    pub action_id: String,
}

/// 结果行的动作文案键：四段式 `action.<插件 id>.<行类型>.<动作 id>`（spec §1.3 末）
///
/// 清单能给出"有哪些类型、哪些动作"，但给不出中文——中文只住在前端（AGENTS.md）。
/// 前端按同一套键从插件**装载时**交上来的文案表里取中文。
pub fn result_label_key(plugin_id: &str, the_type: &str, action_id: &str) -> String {
    format!("action.{plugin_id}.{the_type}.{action_id}")
}

/// 一个包最近一次搜索结果的行缓存
///
/// `Plugin Search Result` 不注册进框架（Q10），所以框架手上没有它；结果行的动作要能派发，
/// 就得有人在派发时按行下标找回那一行。这份缓存由宿主建、代理与宿主共用同一份：
/// 宿主在回程里写入，代理在 [`Plugin::resolve_item`] 里读。
pub type RowCache = Arc<Mutex<Vec<PluginItem>>>;
