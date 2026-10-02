//! `scan` 类型的展开：路径解析 + [`WalkDir`] 遍历 + 过滤
//!
//! 根路径变量经 [`PluginContext`] 解析（框架层不依赖 Tauri，插件也不该），
//! `key_words` 的合并规则见 `init.rs`。

use std::{
    fmt::{Display, Formatter},
    path::{Path, PathBuf},
    str::FromStr,
};

use walkdir::{DirEntry, WalkDir};

use crate::{
    plugin_framework::PluginContext,
    plugin_impl_launcher::persistence::{
        launcher_source::DEFAULT_PRIORITY, parse_scan::LauncherScanConfig,
    },
};

/// 扫描出来的一个条目：文件名、完整路径、递归层级
pub struct ScannedEntry {
    pub file_name: String,
    pub path: PathBuf,
    pub depth: usize,
}

/// 扫描一个 `scan` 条目
///
/// `follow_links(false)`、`max_depth` 直接交给 walkdir、第 0 层是配置的 `path` 本身
/// 且不受过滤条件约束。
pub fn scan(
    cx: &dyn PluginContext,
    config: &LauncherScanConfig,
) -> Result<Vec<ScannedEntry>, String> {
    let root = resolve_root(cx, config)?;

    let target_types: Vec<FileType> = config
        .file_types
        .iter()
        .filter_map(|s| FileType::from_str(s).ok())
        .collect();

    let options = ScanOptions {
        // 默认不扫文件夹的软连接
        follow_links: false,
        target_types,
        ends_with_patterns: config.file_suffix.clone(),
        black_list: config.black_list.clone(),
    };

    Ok(scan_path(root, config.max_depth, &options))
}

/// 取某一递归层级的优先级：越界时沿用链尾，链为空时取默认值
pub fn priority_at(chain: &[i32], depth: usize) -> i32 {
    chain
        .get(depth)
        .copied()
        .or_else(|| chain.last().copied())
        .unwrap_or(DEFAULT_PRIORITY)
}

/// 解析扫描根路径
///
/// 根路径变量交给宿主解析（`PluginContext::resolve_base`，变量名表由宿主提供），
/// **不自己维护变量名映射表**，免得与 Tauri 的平台差异脱节。
/// 空 `base` 时 `path` 原样使用；非空但认不出则报错，不退化成相对路径静默扫不到文件。
fn resolve_root(cx: &dyn PluginContext, config: &LauncherScanConfig) -> Result<PathBuf, String> {
    let base = config.base.trim();
    let path = config.path.trim();

    let resolved = if base.is_empty() {
        PathBuf::from(path)
    } else {
        let base_dir = cx
            .resolve_base(base)
            .ok_or_else(|| format!("unknown base path variable: {base}"))?;

        if path.is_empty() {
            return Err(format!("empty path for base path variable: {base}"));
        }

        base_dir.join(path)
    };

    // 相对路径落到当前工作目录会静默扫空，所以先取绝对路径
    std::path::absolute(&resolved)
        .map_err(|err| format!("get scan root absolute path failed: {err}"))
}

/// 文件类型过滤的三档
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Dir,
    Symlink,
}

impl FileType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Dir => "Dir",
            Self::Symlink => "Symlink",
        }
    }
}

impl Display for FileType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

/// 认不出的文件类型名直接当作没配，不复刻一个"解析失败"的类型
impl FromStr for FileType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "File" => Ok(Self::File),
            "Dir" => Ok(Self::Dir),
            "Symlink" => Ok(Self::Symlink),
            _ => Err(()),
        }
    }
}

/// 一次扫描的全部过滤条件
struct ScanOptions {
    follow_links: bool,
    target_types: Vec<FileType>,
    ends_with_patterns: Vec<String>,
    black_list: Vec<String>,
}

/// 遍历并过滤，产出 `(文件名, 路径, 层级)`
fn scan_path<P: AsRef<Path>>(
    root: P,
    max_depth: usize,
    options: &ScanOptions,
) -> Vec<ScannedEntry> {
    let mut results = Vec::new();

    // max_depth 直接交给 walkdir：0 表示只取 root 本身（即原 file 类型）
    let walker = WalkDir::new(root)
        .follow_links(options.follow_links)
        .max_depth(max_depth);

    // 跳过无权限的目录
    for entry in walker.into_iter().filter_map(|r| r.ok()) {
        let depth = entry.depth();
        let file_name = entry.file_name().to_string_lossy();

        // 第 0 层是显式配置的 path，不受过滤条件约束
        if depth == 0 || is_match(&entry, &file_name, options) {
            results.push(ScannedEntry {
                file_name: file_name.into_owned(),
                path: entry.path().to_path_buf(),
                depth,
            });
        }
    }

    results
}

/// 核心过滤逻辑：类型、后缀、黑名单三项都要过
fn is_match(entry: &DirEntry, file_name: &str, options: &ScanOptions) -> bool {
    let ft = entry.file_type();

    // 没配类型就什么都不过
    if options.target_types.is_empty() {
        return false;
    }

    let type_matched = options.target_types.iter().any(|t| match t {
        FileType::File => ft.is_file(),
        FileType::Dir => ft.is_dir(),
        FileType::Symlink => ft.is_symlink(),
    });

    if !type_matched {
        return false;
    }

    // 没配后缀也什么都不过
    if options.ends_with_patterns.is_empty() {
        return false;
    }

    let pattern_matched = options
        .ends_with_patterns
        .iter()
        .any(|pattern| file_name.ends_with(pattern));

    if !pattern_matched {
        return false;
    }

    if options.black_list.is_empty() {
        return true;
    }

    let has_black_key = options
        .black_list
        .iter()
        .any(|black_key| file_name.contains(black_key));

    !has_black_key
}
