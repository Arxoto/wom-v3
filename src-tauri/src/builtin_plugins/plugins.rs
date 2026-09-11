//! 插件的可调用函数实现

use std::sync::Mutex;

use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime,
};

use tauri_plugin_log::log;

use crate::builtin_plugins::{
    persistence::load::{self, ItemCollection},
    search::ItemSearchResult,
};

pub const INNER_PLUGIN_NAME: &str = "inner-plugin";

/// 插件运行期状态
///
/// [`ItemCollection`] 与 [`ItemSearchResult`] 总是一起使用
/// （检索需要回表取 [`crate::builtin_plugins::common::Item`] ，重载设置需要同时替换两者），
/// 因此合并到同一把 [`Mutex`] ，不需要维护锁顺序，也不存在死锁
pub struct PluginStat(pub Mutex<PluginStatInner>);

impl PluginStat {
    pub fn new(item_collection: ItemCollection) -> Self {
        Self(Mutex::new(PluginStatInner {
            item_collection,
            item_search_result: ItemSearchResult::default(),
        }))
    }
}

/// [`PluginStat`] 的受保护内容
pub struct PluginStatInner {
    /// 根据设置文件得到的全量的 items
    pub item_collection: ItemCollection,
    /// 缓存的搜索结果
    pub item_search_result: ItemSearchResult,
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new(INNER_PLUGIN_NAME)
        .invoke_handler(tauri::generate_handler![cmds::search, cmds::search_page])
        .setup(|app, _api| {
            let item_collection = load::load_settings(app)?;
            app.manage(PluginStat::new(item_collection));

            Ok(())
        })
        .build()
}

pub fn reload_setting(app: &AppHandle) {
    let Ok(item_collection) = load::load_settings(app) else {
        log::warn!("io error when load settings");
        return;
    };

    let state = app
        .try_state::<PluginStat>()
        .expect("PluginStat not be managed");
    let mut stat = state.0.lock().unwrap();

    stat.item_collection = item_collection;
    stat.item_search_result = ItemSearchResult::default();
}

mod cmds {
    use tauri::State;

    use super::PluginStat;
    use crate::builtin_plugins::search::ItemSearchPage;

    /// 使用关键字进行检索
    #[tauri::command]
    pub async fn search(plugin_stat: State<'_, PluginStat>, k: &str) -> Result<ItemSearchPage, ()> {
        let mut stat = plugin_stat.0.lock().unwrap(); // 无法处理异常

        // cache
        if !stat.item_search_result.is_current_result(k) {
            let new_search_result = stat.item_collection.search(k);
            stat.item_search_result = new_search_result;
        }

        Ok(stat.item_search_result.page(0, &stat.item_collection))
    }

    /// 对检索结果进行翻页
    #[tauri::command]
    pub async fn search_page(
        plugin_stat: State<'_, PluginStat>,
        index: usize,
    ) -> Result<ItemSearchPage, ()> {
        let stat = plugin_stat.0.lock().unwrap();

        Ok(stat.item_search_result.page(index, &stat.item_collection))
    }
}
