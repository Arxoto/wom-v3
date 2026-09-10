use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime,
};

use tauri_plugin_log::log;

use crate::builtin_plugins::{
    persistence::load::{self, ItemsStat},
    search::{ItemSearchResult, ItemSearchStat},
};

pub const INNER_PLUGIN_NAME: &str = "inner-plugin";

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new(INNER_PLUGIN_NAME)
        .invoke_handler(tauri::generate_handler![cmds::search, cmds::search_page])
        .setup(|app, _api| {
            // load
            let items = load::load_settings(app)?;
            app.manage(ItemsStat::new(items));

            // cache
            app.manage(ItemSearchStat::new());

            Ok(())
        })
        .build()
}

pub fn reload_setting(app: &AppHandle) {
    let Ok(items) = load::load_settings(app) else {
        log::warn!("io error when load settings");
        return;
    };

    // first to clear cache
    {
        let state = app
            .try_state::<ItemSearchStat>()
            .expect("ItemSearchStat not be managed");

        let mut items = state.0.lock().unwrap();
        *items = ItemSearchResult::default();
    }

    {
        let state = app
            .try_state::<ItemsStat>()
            .expect("ItemsStat not be managed");

        let mut item_collection = state.0.lock().unwrap();
        *item_collection = items;
    }
}

mod cmds {
    use tauri::State;

    use crate::builtin_plugins::{
        persistence::load::ItemsStat,
        search::{ItemSearchPage, ItemSearchStat},
    };

    #[tauri::command]
    pub async fn search(
        item_stat: State<'_, ItemsStat>,
        item_search_stat: State<'_, ItemSearchStat>,
        k: &str,
    ) -> Result<ItemSearchPage, ()> {
        // cache
        {
            let item_search_result = item_search_stat.0.lock().unwrap(); // 无法处理异常
            if item_search_result.is_current_result(k) {
                return Ok(item_search_result.page(0));
            }
        }

        // do search
        let new_search_result = {
            let item_collection = item_stat.0.lock().unwrap();
            item_collection.search(k)
        };

        // restore
        let mut item_search_result = item_search_stat.0.lock().unwrap();
        *item_search_result = new_search_result;

        Ok(item_search_result.page(0))
    }

    #[tauri::command]
    pub async fn search_page(
        item_search_stat: State<'_, ItemSearchStat>,
        index: usize,
    ) -> Result<ItemSearchPage, ()> {
        let item_search_result = item_search_stat.0.lock().unwrap();
        Ok(item_search_result.page(index))
    }
}
