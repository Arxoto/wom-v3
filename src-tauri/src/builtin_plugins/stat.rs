//! 内建插件的运行期状态、检索与重载

use std::sync::Mutex;

use tauri::{AppHandle, Manager};

use tauri_plugin_log::log;

use crate::builtin_plugins::{
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
    let mut stat = state.0.lock().unwrap();

    stat.item_collection = item_collection;
    stat.item_search_result = ItemSearchResult::default();
}

/// 使用关键字进行检索
pub fn search(builtin_stat: &BuiltinStat, k: &str) -> ItemSearchPage {
    let mut stat = builtin_stat.0.lock().unwrap(); // 无法处理异常

    // cache
    if !stat.item_search_result.is_current_result(k) {
        let new_search_result = stat.item_collection.search(k);
        stat.item_search_result = new_search_result;
    }

    stat.item_search_result.page(0, &stat.item_collection)
}

/// 对检索结果进行翻页
pub fn search_page(builtin_stat: &BuiltinStat, index: usize) -> ItemSearchPage {
    let stat = builtin_stat.0.lock().unwrap();

    stat.item_search_result.page(index, &stat.item_collection)
}
