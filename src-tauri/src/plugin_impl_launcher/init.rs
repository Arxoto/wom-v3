//! launcher 的加载：manifest → 框架条目
//!
//! 流程：
//!
//! 1. 用 `persistence` 的方法拿到 [`LauncherItemSource`] 迭代器；
//! 2. 逐行转成框架的 Plugin Item（`scan` 行按 walkdir 展开成多条）；
//! 3. 交给框架注册（动作表由 [`crate::plugin_impl_launcher::action::actions`] 给出）。
//!
//! 条目级失败的消化全在这里（Q19）：空行跳过、坏行记 warn 跳过、`scan` 展开失败也记 warn 跳过。
//! 框架只会在 `init` 整体失败时看到一条字符串——它看不到"空行"与"字段不足"的区别，
//! 那是 launcher 的语义。
//!
//! 日志一律走 [`PluginContext`]，不直接调日志宏：插件与宿主之间只有这一个横切面。

use std::path::Path;

use crate::{
    plugin_framework::{PluginContext, PluginError, PluginItem},
    plugin_impl_launcher::persistence::{
        item_source_iter,
        launcher_source::{LauncherItemSource, LauncherItemType, LauncherParseError},
        manifest::LauncherItemReadError,
        parse_scan::LauncherScanConfig,
        scans_helper,
    },
};

/// 把 manifest 的全部行转成框架条目
///
/// 单行失败只跳过这一行；只有 manifest 整个打不开才算插件级失败（[`PluginError`]）。
pub fn load_items(
    cx: &dyn PluginContext,
    app_data_dir: &Path,
) -> Result<Vec<PluginItem>, PluginError> {
    let sources = item_source_iter(app_data_dir)
        .map_err(|err| PluginError::new(format!("open launcher manifest failed: {err}")))?;

    let mut items: Vec<PluginItem> = Vec::new();

    for (line_index, source) in sources.enumerate() {
        // manifest 的行号从 1 数起，日志里报的就是用户能对上的行号
        let line_number = line_index + 1;

        let source = match source {
            Ok(source) => source,
            // 空行是正常写法，不值一条日志
            Err(LauncherItemReadError::Parse(LauncherParseError::EmptyLine)) => continue,
            Err(err) => {
                cx.log_warn(&format!("skip launcher manifest line {line_number}: {err}"));
                continue;
            }
        };

        match item_sources_of(cx, source) {
            Ok(mut source_items) => items.append(&mut source_items),
            Err(err) => cx.log_warn(&format!("skip launcher manifest line {line_number}: {err}")),
        }
    }

    Ok(items)
}

/// 一行 `scan` 展开成多条，其余类型一对一
fn item_sources_of(
    cx: &dyn PluginContext,
    source: LauncherItemSource,
) -> Result<Vec<PluginItem>, String> {
    match source {
        LauncherItemSource::Sys {
            priority,
            key_words,
            name,
        } => Ok(vec![PluginItem::new(
            LauncherItemType::Sys.as_str(),
            priority,
            key_words,
            name,
            // sys 没有 desc，也就没有可复制的正文
            String::new(),
            // launcher 的条目没有各自一张图片：图标按类型来（见前端注册表）
            String::new(),
        )]),
        LauncherItemSource::Cmd {
            priority,
            key_words,
            name,
            desc,
        } => Ok(vec![PluginItem::new(
            LauncherItemType::Cmd.as_str(),
            priority,
            key_words,
            name,
            desc,
            String::new(),
        )]),
        LauncherItemSource::Web {
            priority,
            key_words,
            name,
            desc,
        } => Ok(vec![PluginItem::new(
            LauncherItemType::Web.as_str(),
            priority,
            key_words,
            name,
            desc,
            String::new(),
        )]),
        LauncherItemSource::Scan {
            priority_chain,
            key_words,
            name,
            config,
            ..
        } => scan_items(cx, &priority_chain, &key_words, &name, &config),
    }
}

/// 把一行 `scan` 展开成多条条目
///
/// 根路径解析失败会一直冒到这里：它意味着这一行的配置写错了，
/// 静默跳过会让用户以为扫描本来就没有结果（spec §2.5）。
fn scan_items(
    cx: &dyn PluginContext,
    priority_chain: &[i32],
    key_words: &[String],
    name: &str,
    config: &LauncherScanConfig,
) -> Result<Vec<PluginItem>, String> {
    let entries = scans_helper::scan(cx, config)?;

    let items = entries
        .into_iter()
        .map(|entry| {
            // 第 0 层就是配置的 path 本身，沿用配置的 name；其余层级用文件名
            let item_name = if entry.depth == 0 && !name.is_empty() {
                name.to_string()
            } else {
                entry.file_name
            };

            PluginItem::new(
                LauncherItemType::Scan.as_str(),
                scans_helper::priority_at(priority_chain, entry.depth),
                merged_key_words(key_words, &item_name),
                item_name,
                entry.path.to_string_lossy().into_owned(),
                String::new(),
            )
        })
        .collect();

    Ok(items)
}

/// `key_words` 的合并规则：**配置关键字 ∪ 文件名，且不去重**（Q31/Q32）
///
/// 命中集合会变大——配置关键字与文件名都成为关键字，
/// 且每条扫出的文件都自带一个精确命中自己的关键字（因为它的 `name` 就是文件名）。
///
/// 只有 `scan` 走这条规则：其余类型的条目没有"文件名"这个维度，配置怎么写就怎么算。
fn merged_key_words(key_words: &[String], name: &str) -> Vec<String> {
    let mut merged = key_words.to_vec();
    merged.push(name.to_string());

    merged
}
