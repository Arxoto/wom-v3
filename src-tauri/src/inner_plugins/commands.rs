use tauri::{AppHandle, Manager};
use tauri_plugin_log::log;

use crate::inner_plugins::generator::{self, ItemsStat};

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
