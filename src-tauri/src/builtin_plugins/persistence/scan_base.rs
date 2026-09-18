//! 扫描根路径变量：配置页选项与具体目录的映射
//!
//! 变量名与平台解析都交给 [`BaseDirectory`] ，这里只维护"哪些变量值得摆到配置页上"
//! 以及前端下拉用的 label，避免 tauri 的变量表在这里出现第二份。

use serde::Serialize;
use tauri::path::BaseDirectory;

/// 扫描根路径 [`ItemParsedScan::base`] 的可选项枚举类（仅作为前端交互时的辅助，不作为反序列化类型）
///
/// 变量名与各平台的具体目录都由 [`tauri::path::BaseDirectory`] 决定，设置文件只写变量名即可，
/// 不必关心平台差异：各平台按系统惯例落到不同目录
/// （Windows 走 `FOLDERID_*`，Linux 走 `XDG_*`，macOS 走 `~/Library` 等）。
///
/// 这里只收三端都存在、语义明确的目录，供配置页做下拉选项。
/// 设置文件本身由 tauri 解析，手写的其它变量（如 `$TEMP` ）照样有效，只是不出现在下拉里。
///
/// 序列化形式就是变量名本身（如 `"$DESKTOP"` ），前端拿到的值与写进设置文件的值一致。
///
/// [`ItemParsedScan::base`]: crate::builtin_plugins::persistence::parse_impl_scan::ItemParsedScan::base
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanBase {
    /// `$HOME` 用户主目录，其余用户目录都在它下面
    Home,
    /// `$DESKTOP` 桌面
    Desktop,
    /// `$DOWNLOAD` 下载目录
    Download,
    /// `$DOCUMENT` 文档目录
    Document,
    /// `$PICTURE` 图片目录
    Picture,
    /// `$AUDIO` 音乐目录
    Audio,
    /// `$VIDEO` 视频目录，macOS 下的落点是 `~/Movies`
    Video,
    /// `$CONFIG` 用户配置目录，所有应用共用
    Config,
    /// `$DATA` 用户数据目录，所有应用共用
    Data,
    /// `$LOCALDATA` 用户本机数据目录，所有应用共用且不随账户漫游
    LocalData,
    /// `$RESOURCE` 应用只读资源目录，随安装位置走，开发时即构建产物目录
    Resource,
    /// `$APPCONFIG` 应用配置目录，即 `$CONFIG` 下以应用标识符分隔的子目录
    AppConfig,
    /// `$APPDATA` 应用数据目录，即 `$DATA` 下以应用标识符分隔的子目录
    AppData,
    /// `$APPLOCALDATA` 应用本机数据目录，即 `$LOCALDATA` 下以应用标识符分隔的子目录
    AppLocalData,
}

impl ScanBase {
    /// 全部可选项，顺序即配置页下拉的顺序
    pub const ALL: [ScanBase; 14] = [
        Self::Home,
        Self::Desktop,
        Self::Download,
        Self::Document,
        Self::Picture,
        Self::Audio,
        Self::Video,
        Self::Config,
        Self::Data,
        Self::LocalData,
        Self::Resource,
        Self::AppConfig,
        Self::AppData,
        Self::AppLocalData,
    ];

    /// 对应的 tauri 目录枚举，变量名与各平台的具体路径都由它决定
    pub fn base_directory(self) -> BaseDirectory {
        match self {
            Self::Home => BaseDirectory::Home,
            Self::Desktop => BaseDirectory::Desktop,
            Self::Download => BaseDirectory::Download,
            Self::Document => BaseDirectory::Document,
            Self::Picture => BaseDirectory::Picture,
            Self::Audio => BaseDirectory::Audio,
            Self::Video => BaseDirectory::Video,
            Self::Config => BaseDirectory::Config,
            Self::Data => BaseDirectory::Data,
            Self::LocalData => BaseDirectory::LocalData,
            Self::Resource => BaseDirectory::Resource,
            Self::AppConfig => BaseDirectory::AppConfig,
            Self::AppData => BaseDirectory::AppData,
            Self::AppLocalData => BaseDirectory::AppLocalData,
        }
    }

    /// 配置页下拉里显示的名字
    pub fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Desktop => "Desktop",
            Self::Download => "Download",
            Self::Document => "Document",
            Self::Picture => "Picture",
            Self::Audio => "Audio",
            Self::Video => "Video",
            Self::Config => "Config(all_apps)",
            Self::Data => "Data(all_apps)",
            Self::LocalData => "LocalData(all_apps)",
            Self::Resource => "Resource(app_install_dir)",
            Self::AppConfig => "AppConfig",
            Self::AppData => "AppData",
            Self::AppLocalData => "AppLocalData",
        }
    }

    /// 设置文件里写的变量名，如 `$DESKTOP`
    pub fn variable(self) -> &'static str {
        self.base_directory().variable()
    }

    /// 配置页下拉的全部选项，带上每个变量在这台机器上解析出来的目录
    pub fn options() -> Vec<ScanBaseOption> {
        Self::ALL
            .iter()
            .map(|base| ScanBaseOption {
                value: base.variable(),
                label: base.label(),
            })
            .collect()
    }
}

/// 前端下拉的一项
#[derive(Debug, Clone, Serialize)]
pub struct ScanBaseOption {
    /// 写进设置文件的值，同时也是变量名
    pub value: &'static str,
    /// 下拉里显示的名字
    pub label: &'static str,
}
