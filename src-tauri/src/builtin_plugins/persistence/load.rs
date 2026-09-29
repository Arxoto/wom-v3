//! 统一的加载实现

use std::{
    fs,
    io::{self, BufRead},
    path::PathBuf,
    str::FromStr,
};

use tauri::{AppHandle, Runtime};

use tauri_plugin_log::log;

use crate::{
    builtin_plugins::{
        base::ItemType,
        common::Item,
        persistence::{
            parse_core::{ItemParseErr, ItemParsed},
            scans_helper,
        },
    },
    constants::SETTING_FILE_NAME,
};

/// 根据设置文件得到的全量的 items
///
/// 由 [`crate::builtin_plugins::stat::BuiltinStat`] 持有
pub struct ItemCollection {
    pub item_list: Vec<Item>,
}

impl ItemCollection {
    /// 通过下标获取 [`Item`]
    ///
    /// 下标即 [`ItemCollection::item_list`] 中的位置，
    /// 因此直接按下标取出
    pub fn get_by_index(&self, index: usize) -> Option<&Item> {
        self.item_list.get(index)
    }
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
    log::debug!("get items: {:?}", item_list);

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

        let r = ItemParsed::from_str(line_content);
        match r {
            Ok(parsed) => {
                let r = add_items(app, parsed, &mut item_list);
                let _ = r.map_err(|e| handle_parsed_error(e, line_content));
            }
            Err(e) => handle_parsed_error(e, line_content),
        }

        line.clear();
    }

    // 稳定排序：priority 相同的条目保持它们在设置文件里的相对顺序
    item_list.sort_by_key(|item| item.priority());

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
    let the_type = ItemType::from_str(&parsed.the_type)?;

    match the_type {
        ItemType::Sys => item_list.push(parsed.into_system()?),
        ItemType::Cmd => item_list.push(parsed.into_cmd()?),
        ItemType::Web => item_list.push(parsed.into_web()?),
        ItemType::Scan => {
            let mut items = scans_helper::scan_files(app, parsed.into_scan_config()?)?;
            item_list.append(&mut items);
        }
        ItemType::Note => item_list.push(parsed.into_note()?),
        ItemType::Snippets => item_list.push(parsed.into_snippets()?),
    }

    Ok(())
}
