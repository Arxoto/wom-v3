use std::path::{Path, PathBuf};

use tauri::{path::BaseDirectory, AppHandle, Manager, Runtime};

use walkdir::{DirEntry, WalkDir};

use crate::inner_plugins::{
    base::KeyWord,
    common::ItemType,
    items::{
        impl_persistence::{impl_scan::ItemParsedScan, ItemParseErr},
        Item,
    },
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

    let base_dir = BaseDirectory::from_variable(&base);

    let real_path = match base_dir {
        Some(base_dir) => app.path().resolve(path, base_dir).map_err(|_| {
            ItemParseErr::ItemValueParsedFailed("resolve base path failed".to_string())
        })?,
        None => Path::new(&path).to_path_buf(),
    };

    let target_types: Vec<FileType> = file_types
        .iter()
        .map(|s| FileType::try_from(s))
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
                KeyWord(file_name.clone()),
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

impl From<&FileType> for &str {
    fn from(value: &FileType) -> Self {
        match value {
            FileType::File => "File",
            FileType::Dir => "Dir",
            FileType::Symlink => "Symlink",
        }
    }
}

impl TryFrom<&str> for FileType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "File" => Ok(Self::File),
            "Dir" => Ok(Self::Dir),
            "Symlink" => Ok(Self::Symlink),
            _ => Err(()),
        }
    }
}

impl TryFrom<&String> for FileType {
    type Error = ();

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        FileType::try_from(value.as_str())
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
            // 获取绝对路径
            if let Ok(abs_path) = std::fs::canonicalize(entry.path()) {
                results.push((file_name.into_owned(), abs_path));
            }
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
    return !has_black_key;
}
