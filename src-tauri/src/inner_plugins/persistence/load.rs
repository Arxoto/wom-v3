use std::{
    fs,
    io::{self, BufRead},
    path::PathBuf,
    sync::Mutex,
};

use tauri::{AppHandle, Runtime};

use tauri_plugin_log::log;

use crate::inner_plugins::{
    base::{ItemType, KeyWord},
    common::Item,
    persistence::{
        parse_core::{ItemParseErr, ItemParsed},
        scans_helper,
    },
};

pub const SETTING_FILE_NAME: &str = "inner_plugins.txt";

/// 纯内存计算，直接使用 [`std::sync::Mutex`]
pub struct ItemsStat(pub Mutex<ItemCollection>);

impl ItemsStat {
    pub fn new(items: ItemCollection) -> Self {
        Self(Mutex::new(items))
    }
}

pub struct ItemCollection {
    pub item_list: Vec<Item>,
}

pub fn load_settings<R: Runtime>(app: &AppHandle<R>) -> io::Result<ItemCollection> {
    log::info!("begin to load settings");

    let setting_path = tauri::Manager::path(app)
        .app_data_dir()
        .expect("get app data dir failed")
        .join(SETTING_FILE_NAME);

    if !setting_path.exists() {
        if let Some(parent) = setting_path.parent() {
            fs::create_dir_all(parent)?;
            fs::write(&setting_path, "")?;
        }
    }

    let item_list: Vec<Item> = gen_item_list(app, setting_path)?;

    log::info!("end to load settings");
    Ok(ItemCollection { item_list })
}

fn gen_item_list<R: Runtime>(app: &AppHandle<R>, setting_path: PathBuf) -> io::Result<Vec<Item>> {
    let mut item_list: Vec<Item> = Vec::new();

    let file = fs::File::open(setting_path)?;
    let mut reader = io::BufReader::new(file);
    let mut line = String::new();
    loop {
        let len = reader.read_line(&mut line)?;
        if len == 0 {
            break;
        }
        let line_content = line.trim_end();

        let r = Item::parse_str(line_content);
        match r {
            Ok(parsed) => {
                let r = add_items(app, parsed, &mut item_list);
                let _ = r.map_err(|e| handle_parsed_error(e, line_content));
            }
            Err(e) => handle_parsed_error(e, line_content),
        }

        line.clear();
    }

    Ok(item_list)
}

fn handle_parsed_error(e: ItemParseErr, line_content: &str) {
    match e {
        ItemParseErr::EmptyLine => { /* empty */ }
        ItemParseErr::ValueNotEnough(err_msg) => {
            log::warn!("load inner db error with: {}", err_msg);
        }
        ItemParseErr::ItemTypeParsedFailed => {
            log::warn!("load inner db error by unknown type");
            log::debug!("unknown type by line: {}", line_content);
        }
        ItemParseErr::ItemValueParsedFailed(err_msg) => {
            log::warn!("parse value failed with: {}", err_msg);
            log::debug!("parse value failed by line: {}", line_content);
        }
    }
}

fn add_items<R: Runtime>(
    app: &AppHandle<R>,
    parsed: ItemParsed,
    item_list: &mut Vec<Item>,
) -> Result<(), ItemParseErr> {
    match parsed {
        ItemParsed::Common(item_parsed_common) => {
            let mut ll: Vec<Item> = item_parsed_common
                .key_words
                .into_iter()
                .map(|k| KeyWord(k))
                .map(|k| {
                    Item::new(
                        item_parsed_common.the_type,
                        k,
                        item_parsed_common.name.clone(),
                        item_parsed_common.desc.clone(),
                    )
                })
                .collect();
            item_list.append(&mut ll);
            Ok(())
        }
        ItemParsed::System(item_parsed_system) => {
            let mut ll: Vec<Item> = item_parsed_system
                .key_words
                .into_iter()
                .map(|k| KeyWord(k))
                .map(|k| Item::new(ItemType::System, k, item_parsed_system.name.clone(), ""))
                .collect();
            item_list.append(&mut ll);
            Ok(())
        }
        ItemParsed::Scan(item_parsed_scan) => {
            let mut ll = scans_helper::scan_files(app, item_parsed_scan)?;

            item_list.append(&mut ll);
            Ok(())
        }
    }
}
