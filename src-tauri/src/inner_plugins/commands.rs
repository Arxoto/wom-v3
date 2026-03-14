use tauri::{AppHandle, Manager, State};
use tauri_plugin_log::log;

use crate::inner_plugins::{
    generator::{self, ItemsStat},
    items::Item,
    search::{ItemSearchPage, ItemSearchResult, ItemSearchStat},
};

pub fn reload_setting(app: &AppHandle) {
    let Ok(items) = generator::load_settings(&app) else {
        log::warn!("io error when load settings");
        return;
    };

    let state = app
        .try_state::<ItemsStat>()
        .expect("ItemsStat not be managed");

    let mut item_collection = state.0.lock().unwrap();
    *item_collection = items;
}

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
    let item_list = {
        let item_collection = item_stat.0.lock().unwrap();
        let item_list = &item_collection.item_list;

        let mut list_eq: Vec<Item> = vec![];
        let mut list_starts_with: Vec<Item> = vec![];
        let mut list_contains: Vec<Item> = vec![];
        let mut list_match: Vec<Item> = vec![];
        for item in item_list {
            let key_word = &item.key_word;
            if key_word.find_eq(k) {
                list_eq.push(item.clone());
            } else if key_word.find_starts_with(k) {
                list_starts_with.push(item.clone());
            } else if key_word.find_contains(k) {
                list_contains.push(item.clone());
            } else if key_word.find_match(k) {
                list_match.push(item.clone());
            }
        }

        let mut item_list = list_eq;
        item_list.append(&mut list_starts_with);
        item_list.append(&mut list_contains);
        item_list.append(&mut list_match);
        item_list
    };

    // restore
    let mut item_search_result = item_search_stat.0.lock().unwrap();
    *item_search_result = ItemSearchResult {
        key_word: k.to_string(),
        item_list: item_list,
    };

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
