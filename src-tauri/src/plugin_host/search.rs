//! 一次 `Plugin Search` 的回程：请求 → 回程 → 投影（spec §3.3）
//!
//! 时序不变量是**先等插件报「已注册」、再发起搜索**：这一条由 webview 那一半的装载器保证
//! ——装载完成是一个 Promise，串在插件自己的搜索函数之前，所以 Rust 收到回程时，
//! 图标与文案一定已经注册好了。Rust 这边只发一次请求、等一次回程。

use tauri::{AppHandle, Emitter, Manager};

use super::packages::PackageHost;
use crate::{
    constants,
    plugin_framework::{
        ActionId, ItemAddress, ItemHandle, ItemSearchPage, PluginId, PluginItem, PluginItemDisplay,
    },
    plugin_proxy_js::{SearchRequest, SearchRow},
};

/// 一次插件搜索回程的最长等待：插件装载卡死或窗口没了时，命令不能永远挂着
const SEARCH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// 触发一次 `Plugin Search`：请求 → 回程 → 投影
pub async fn open_plugin_search(
    app: &AppHandle,
    plugin_id: &str,
    keyword: &str,
) -> Result<ItemSearchPage, String> {
    // 取出要用的东西后立刻放掉 state 借用：下面要跨 await
    let (entry, mut receiver) = {
        let host = app.state::<PackageHost>();
        let entry = host.entry_of(plugin_id)?;
        let receiver = host.begin_search(plugin_id)?;
        (entry, receiver)
    };

    let request = SearchRequest {
        plugin_id: plugin_id.to_string(),
        entry,
        keyword: keyword.to_string(),
    };

    if let Err(err) = app.emit_to(
        constants::LABEL_MAIN,
        constants::EVENT_PLUGIN_SEARCH_REQUEST,
        request,
    ) {
        app.state::<PackageHost>().cancel_search(plugin_id);
        return Err(format!("emit plugin search request failed: {err}"));
    }

    let rows = match tokio::time::timeout(SEARCH_TIMEOUT, receiver.recv()).await {
        Ok(Some(rows)) => rows,
        Ok(None) => return Err(format!("plugin search request dropped: {plugin_id}")),
        Err(_elapsed) => {
            // 超时之后这条回程没人等了，撤掉登记，迟到的结果按"没有在飞的搜索"丢掉
            app.state::<PackageHost>().cancel_search(plugin_id);
            return Err(format!("plugin search timed out: {plugin_id}"));
        }
    };

    Ok(search_page_of(app, plugin_id, rows))
}

/// 回程：插件执行完搜索后把结果行交回来（命令
/// [`crate::commands::plugin::plugin_report_search_results`] 的实现，也是
/// [`open_plugin_search`] 等待的那一头）
pub fn report_search_results(
    app: &AppHandle,
    plugin_id: &str,
    rows: Vec<SearchRow>,
) -> Result<(), String> {
    app.state::<PackageHost>().report(plugin_id, rows)
}

/// 把结果行投影成 `Plugin Search Page`
///
/// 结果行**不是**注册条目（Q10）：它不注册进框架、不由框架持有，是每次查询现算的，
/// 所以它走宿主自己这一条投影（注册条目走框架的 `project`）。两者的相同之处只在
/// "都由框架画成一行"——前端拿到的是同一个 [`PluginItemDisplay`] 形状。
///
/// 动作 id 由插件在结果行里自带（spec §3.3 的行形状）：清单里 `types` × `actions` 是
/// **声明**（派发时的白名单与 `label_key` 的来源），行可以一个动作都不带（spec §4.4）。
fn search_page_of(app: &AppHandle, plugin_id: &str, rows: Vec<SearchRow>) -> ItemSearchPage {
    let plugin_id = PluginId(plugin_id.to_string());

    let items: Vec<PluginItem> = rows
        .iter()
        .map(|row| {
            PluginItem::new(
                row.the_type.clone(),
                // 结果行不参与主检索的排序：优先级与关键字在这里没有意义
                0,
                Vec::new(),
                row.name.clone(),
                row.desc.clone(),
                // 结果行没有自带图片：那一张是"包内相对路径"的清单字段，只有插件条目才有
                String::new(),
            )
        })
        .collect();

    app.state::<PackageHost>()
        .set_rows(&plugin_id.0, items.clone());

    let total = items.len();
    let item_list = items
        .into_iter()
        .zip(rows)
        .enumerate()
        .map(|(row_id, (item, row))| PluginItemDisplay {
            item,
            // 结果页里没有"整集"：这一栏是行号（动作下标记账按它记），寻址一律走 `handle`
            item_index: row_id,
            action_ids: row.action_ids.into_iter().map(ActionId).collect(),
            handle: ItemHandle {
                plugin_id: plugin_id.clone(),
                address: ItemAddress::Row { index: row_id },
            },
        })
        .collect();

    ItemSearchPage {
        // 结果页是一次查询的整份答案：没有翻页令牌，也不分匹配模式组
        token: 0,
        total,
        index: 0,
        item_list,
        index_eq: 0,
        index_starts_with: 0,
        index_contains: 0,
        index_match: 0,
    }
}
