pub const LABEL_CONFIG: &str = "config";
pub const LABEL_MAIN: &str = "main";

/// JS 插件打开的 HTML 窗口的标签前缀，后面接 `PluginId`
pub const PLUGIN_WINDOW_LABEL_PREFIX: &str = "plugin-";

/// 主窗口被显示时发给前端的事件
pub const EVENT_MAIN_SHOWN: &str = "main_shown";

/// 宿主请前端执行一次插件搜索（带上入口与关键字）
///
/// 这一条是 `plugin_open_plugin_search` 命令里的"去程"：前端装载插件、跑插件自己的搜索，
/// 再用 `plugin_report_search_results` 把结果行交回来（见 `plugin_proxy_js` 的说明）。
pub const EVENT_PLUGIN_SEARCH_REQUEST: &str = "plugin_search_request";

/// 宿主把结果行的一个动作送回插件执行
///
/// 结果行的动作处理函数在 webview 里，而 `Plugin::run_action` 是同步的，
/// 所以这是一条**单向**请求：发出去就算派发过了（见 `plugin_proxy_js::JsPlugin::run_action`）。
pub const EVENT_PLUGIN_ACTION_REQUEST: &str = "plugin_action_request";

pub const CONFIG_FILE_NAME: &str = "config.json";
