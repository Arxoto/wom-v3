//! launcher 插件自己的常量
//!
//! 放本模块自己的文件里，**不新增到** `src-tauri/src/constants.rs`：那个文件是旧体系的常量表。

/// manifest 文件名，放在应用数据目录下（与现有内建设置文件同处）
pub const LAUNCHER_MANIFEST_FILE_NAME: &str = "launcher-manifest.txt";
