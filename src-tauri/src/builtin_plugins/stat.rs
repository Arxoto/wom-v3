//! 内建插件的运行期状态、检索、动作与重载

use std::sync::Mutex;

use tauri::{AppHandle, Manager};

use tauri_plugin_log::log;

use crate::builtin_plugins::{
    action,
    persistence::load::{self, ItemCollection},
    search::{ItemSearchPage, ItemSearchResult},
};

/// 内建插件的运行期状态
///
/// [`ItemCollection`] 与 [`ItemSearchResult`] 总是一起使用
/// （检索需要回表取 [`crate::builtin_plugins::common::Item`] ，重载设置需要同时替换两者），
/// 因此合并到同一把 [`Mutex`] ，不需要维护锁顺序，也不存在死锁
pub struct BuiltinStat(pub Mutex<BuiltinStatInner>);

impl BuiltinStat {
    pub fn new(item_collection: ItemCollection) -> Self {
        Self(Mutex::new(BuiltinStatInner {
            item_collection,
            item_search_result: ItemSearchResult::default(),
        }))
    }
}

/// [`BuiltinStat`] 的受保护内容
pub struct BuiltinStatInner {
    /// 根据设置文件得到的全量的 items
    pub item_collection: ItemCollection,
    /// 缓存的搜索结果
    pub item_search_result: ItemSearchResult,
}

/// 加载设置文件，并把运行期状态交给 tauri 托管
///
/// 由应用 `setup` 调用（早于任何窗口创建）：检索命令以 `State<BuiltinStat>` 取用该状态
pub fn load_stat(app: &AppHandle) -> tauri::Result<()> {
    let item_collection = load::load_settings(app)?;
    app.manage(BuiltinStat::new(item_collection));

    Ok(())
}

pub fn reload_setting(app: &AppHandle) {
    let Ok(item_collection) = load::load_settings(app) else {
        log::warn!("io error when load settings");
        return;
    };

    let state = app
        .try_state::<BuiltinStat>()
        .expect("BuiltinStat not be managed");
    let mut stat = state.0.lock().unwrap_or_else(|err| err.into_inner());

    stat.item_collection = item_collection;
    stat.item_search_result = ItemSearchResult::default();
}

/// 使用关键字进行检索
pub fn search(builtin_stat: &BuiltinStat, k: &str) -> ItemSearchPage {
    let mut stat = builtin_stat.0.lock().unwrap_or_else(|err| err.into_inner());

    // cache
    if !stat.item_search_result.is_current_key(k) {
        // 令牌只由后端生成：每重新生成一份结果就在上一份的基础上 +1 （溢出回绕）
        let token = stat.item_search_result.token.wrapping_add(1);
        stat.item_search_result = stat.item_collection.search(k, token);
    }

    stat.item_search_result.page(0, &stat.item_collection)
}

/// 对检索结果进行翻页
///
/// 校验 `token` ，过期报错
pub fn search_page(
    builtin_stat: &BuiltinStat,
    index: usize,
    token: u32,
) -> Result<ItemSearchPage, String> {
    let stat = builtin_stat.0.lock().unwrap_or_else(|err| err.into_inner());

    if !stat.item_search_result.is_current_token(token) {
        let cached = stat.item_search_result.token;
        let err = format!("search page for stale token: requested {token}, cached {cached}");
        return Err(err);
    }

    if index > stat.item_search_result.item_indexes.len() {
        let err = format!("search page out of range: {index}");
        return Err(err);
    }

    Ok(stat.item_search_result.page(index, &stat.item_collection))
}

/// 按下标取出条目并跑它的一个 Item Action，返回动作是否真的执行了
///
/// 与检索一样，取条目与执行在同一把锁里：并发重载设置时，下标不该落到另一个条目上。
/// 越界索引与认不出的动作在 [`action::run`] 里当无操作并记 warn。
pub fn run_item_action(
    app: &tauri::AppHandle,
    builtin_stat: &BuiltinStat,
    item_index: usize,
    action_id: &str,
) -> bool {
    let stat = builtin_stat.0.lock().unwrap_or_else(|err| err.into_inner());

    action::run(app, &stat.item_collection, item_index, action_id)
}
