//! 扫描插件的扫描逻辑实现

use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use tauri::{path::BaseDirectory, AppHandle, Manager, Runtime};

use walkdir::{DirEntry, WalkDir};

use crate::builtin_plugins::{
    base::ItemType,
    common::Item,
    persistence::{parse_core::ItemParseErr, parse_impl_scan::ItemParsedScan},
};

pub(super) fn scan_files<R: Runtime>(
    app: &AppHandle<R>,
    parsed: ItemParsedScan,
) -> Result<Vec<Item>, ItemParseErr> {
    let ItemParsedScan {
        file_types,
        file_suffix,
        black_list,
        recursive,
        base,
        path,
    } = parsed;

    // 根路径变量交给 tauri 解析（支持的变量见 ItemParsedScan::base 的文档注释），
    // 不自行维护变量名与 BaseDirectory 的映射表，避免与 tauri 的平台差异脱节
    let resolved_path = match BaseDirectory::from_variable(&base) {
        Some(base_dir) => app.path().resolve(path, base_dir).map_err(|_| {
            ItemParseErr::ItemValueParsedFailed("resolve base path failed".to_string())
        })?,
        // 空 base 是合法写法，此时 path 原样使用（见 ItemParsedScan::path）
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
        recursive,
        follow_links: false, // 默认不扫描文件夹的软连接
        target_types,
        ends_with_patterns: file_suffix,
        black_list,
    };

    let file_name_path = scan_path(real_path, opts);

    let r: Vec<Item> = file_name_path
        .into_iter()
        .map(|(file_name, file_path)| {
            Item::new(
                ItemType::File,
                vec![file_name.clone()],
                file_name,
                file_path,
            )
        })
        .collect();

    Ok(r)
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
    /// 递归子目录
    pub recursive: bool,
    /// 是否追踪软链接
    pub follow_links: bool,
    /// 文件类型过滤
    pub target_types: Vec<FileType>,
    /// 后缀过滤
    pub ends_with_patterns: Vec<String>,
    /// 黑名单
    pub black_list: Vec<String>,
}

fn scan_path<P: AsRef<Path>>(root: P, opts: ScanOptions) -> Vec<(String, PathBuf)> {
    let mut results = Vec::new();

    // 基础配置
    let mut walker = WalkDir::new(root).follow_links(opts.follow_links);
    if !opts.recursive {
        walker = walker.max_depth(1);
    }

    // 跳过无权限的目录
    for entry in walker.into_iter().filter_map(|r| r.ok()) {
        // 跳过目录本身
        if entry.depth() == 0 {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy();
        if is_match(&entry, &file_name, &opts) {
            results.push((file_name.into_owned(), entry.path().to_path_buf()));
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
