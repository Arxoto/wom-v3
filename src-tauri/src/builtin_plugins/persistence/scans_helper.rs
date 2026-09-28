//! 扫描插件的扫描逻辑实现

use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use tauri::{path::BaseDirectory, AppHandle, Manager, Runtime};

use walkdir::{DirEntry, WalkDir};

use crate::builtin_plugins::{
    common::{Item, ItemData},
    persistence::{
        parse_core::{ItemParseErr, DEFAULT_PRIORITY},
        parse_impl_scan::ScanConfig,
    },
};

pub(super) fn scan_files<R: Runtime>(
    app: &AppHandle<R>,
    config: ScanConfig,
) -> Result<Vec<Item>, ItemParseErr> {
    let ScanConfig {
        priority,
        key_words,
        name,
        file_types,
        file_suffix,
        black_list,
        max_depth,
        base,
        path,
    } = config;

    // 根路径变量交给 tauri 解析（支持的变量见 ScanConfig::base 的文档注释），
    // 不自行维护变量名与 BaseDirectory 的映射表，避免与 tauri 的平台差异脱节
    let resolved_path = match BaseDirectory::from_variable(&base) {
        Some(base_dir) => app.path().resolve(path, base_dir).map_err(|_| {
            ItemParseErr::ItemValueParsedFailed("resolve base path failed".to_string())
        })?,
        // 空 base 是合法写法，此时 path 原样使用（见 ScanConfig::path）
        None if base.is_empty() => PathBuf::from(&path),
        // 非空却认不出来，说明变量名写错了，直接报错而不是退化成相对路径静默扫不到文件
        None => {
            return Err(ItemParseErr::ItemValueParsedFailed(format!(
                "unknown base path variable: {base}"
            )));
        }
    };

    let real_path = std::path::absolute(&resolved_path).map_err(|_| {
        ItemParseErr::ItemValueParsedFailed("get scan root absolute path failed".to_string())
    })?;

    let target_types: Vec<FileType> = file_types
        .iter()
        .map(|s| FileType::from_str(s))
        .filter_map(|r| r.ok())
        .collect();

    let opts = ScanOptions {
        follow_links: false, // 默认不扫描文件夹的软连接
        target_types,
        ends_with_patterns: file_suffix,
        black_list,
    };

    let file_name_path = scan_path(real_path, max_depth, opts);

    let r: Vec<Item> = file_name_path
        .into_iter()
        .map(|(file_name, file_path, depth)| {
            // 第 0 层就是 path 本身，沿用配置的 name，其余层级用文件名
            let item_name = if depth == 0 && !name.is_empty() {
                name.clone()
            } else {
                file_name
            };
            let item_key_words = if key_words.is_empty() {
                vec![item_name.clone()]
            } else {
                key_words.clone()
            };
            Item::Scan(ItemData {
                priority: priority_at(&priority, depth),
                key_words: item_key_words,
                name: item_name,
                desc: file_path.to_string_lossy().into_owned(),
            })
        })
        .collect();

    Ok(r)
}

/// 取某一递归层级的优先级：越界时沿用链尾，链为空时取默认值
fn priority_at(chain: &[i32], index: usize) -> i32 {
    chain
        .get(index)
        .copied()
        .or_else(|| chain.last().copied())
        .unwrap_or(DEFAULT_PRIORITY)
}

#[derive(Debug, Clone, Copy)]
pub enum FileType {
    File,
    Dir,
    Symlink,
}

impl FileType {
    pub fn as_str(&self) -> &str {
        match self {
            FileType::File => "File",
            FileType::Dir => "Dir",
            FileType::Symlink => "Symlink",
        }
    }
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub struct FileTypeParseFailed;

impl FromStr for FileType {
    type Err = FileTypeParseFailed;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "File" => Ok(Self::File),
            "Dir" => Ok(Self::Dir),
            "Symlink" => Ok(Self::Symlink),
            _ => Err(FileTypeParseFailed),
        }
    }
}

struct ScanOptions {
    /// 是否追踪软链接
    pub follow_links: bool,
    /// 文件类型过滤
    pub target_types: Vec<FileType>,
    /// 后缀过滤
    pub ends_with_patterns: Vec<String>,
    /// 黑名单
    pub black_list: Vec<String>,
}

fn scan_path<P: AsRef<Path>>(
    root: P,
    max_depth: usize,
    opts: ScanOptions,
) -> Vec<(String, PathBuf, usize)> {
    let mut results = Vec::new();

    // max_depth 直接交给 walkdir：0 表示只取 root 本身（即原 file 类型）
    let walker = WalkDir::new(root)
        .follow_links(opts.follow_links)
        .max_depth(max_depth);

    // 跳过无权限的目录
    for entry in walker.into_iter().filter_map(|r| r.ok()) {
        let depth = entry.depth();
        let file_name = entry.file_name().to_string_lossy();

        // 第 0 层是显式配置的 path，不受过滤条件约束
        if depth == 0 || is_match(&entry, &file_name, &opts) {
            results.push((file_name.into_owned(), entry.path().to_path_buf(), depth));
        }
    }

    results
}

/// 核心过滤逻辑
fn is_match(entry: &DirEntry, file_name: &str, opts: &ScanOptions) -> bool {
    let ft = entry.file_type();

    // 类型过滤逻辑
    if opts.target_types.is_empty() {
        return false;
    }

    let type_matched = opts.target_types.iter().any(|t| match t {
        FileType::File => ft.is_file(),
        FileType::Dir => ft.is_dir(),
        FileType::Symlink => ft.is_symlink(),
    });

    if !type_matched {
        return false;
    }

    // 后缀过滤逻辑
    if opts.ends_with_patterns.is_empty() {
        return false;
    }

    let pattern_matched = opts
        .ends_with_patterns
        .iter()
        .any(|pattern| file_name.ends_with(pattern));

    if !pattern_matched {
        return false;
    }

    // 黑名单过滤
    if opts.black_list.is_empty() {
        return true;
    }

    let has_black_key = opts
        .black_list
        .iter()
        .any(|black_key| file_name.contains(black_key));
    !has_black_key
}
