//! launcher 插件的持久化层
//!
//! 目录与文件名：应用数据目录下的 `launcher-manifest.txt`（Q24）。
//! 文件名常量放本模块自己的 `constants` 里，不进 `src-tauri/src/constants.rs`。
//!
//! 分层：`launcher_source`（一行的中间表示）→ `parse_scan`（scan 的 JSON）
//! → `manifest`（文件访问与逐行迭代）→ `scans_helper`（walkdir 展开）。
//! 只有 `init` 是它们的调用方。

pub mod constants;

pub mod launcher_source;

pub mod manifest;

pub mod parse_scan;

pub mod scans_helper;

pub use manifest::item_source_iter;
