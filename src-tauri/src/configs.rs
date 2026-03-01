use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

pub const CONFIG_FILE_NAME: &str = "config.json";

/// 配置数据（编码用）
#[derive(Debug, Default, Clone)]
pub struct ConfigInner {
    /// 主窗口丢失焦点时隐藏
    pub hide_when_lost_focused: bool,
    /// 主窗口丢失焦点时关闭
    pub close_when_lost_focused: bool,
    // /// 创建主窗口时位于鼠标所在的屏幕 todo
    // pub create_on_current_screen: bool,
    /// 应用打开时自动显示主窗口
    pub show_main_auto: bool,
    /// 使用自定义窗口阴影，在默认窗口非圆角的情况下使用
    pub custom_shadow: bool,
    /// 是否使用系统原生框架
    pub window_frame: bool,
    /// 始终置顶
    pub always_on_top: bool,
    /// 主窗口宽
    pub main_width: f64,
    /// 主窗口高
    pub main_height: f64,
}

impl From<ConfigSettings> for ConfigInner {
    fn from(value: ConfigSettings) -> Self {
        Self {
            hide_when_lost_focused: match value.main_window_mode {
                MainWindowMode::Always => false,
                MainWindowMode::HideAndShow => true,
                MainWindowMode::CloseAndOpen => false,
                MainWindowMode::OnCurrentScreen => false,
            },
            close_when_lost_focused: match value.main_window_mode {
                MainWindowMode::Always => false,
                MainWindowMode::HideAndShow => false,
                MainWindowMode::CloseAndOpen => true,
                MainWindowMode::OnCurrentScreen => true,
            },
            // create_on_current_screen: matches!(
            //     value.main_window_mode,
            //     MainWindowMode::OnCurrentScreen
            // ),
            show_main_auto: matches!(value.main_window_mode, MainWindowMode::Always),
            custom_shadow: value.custom_shadow,
            window_frame: value.window_frame,
            always_on_top: value.always_on_top,
            main_width: value.main_width,
            main_height: value.main_height,
        }
    }
}

/// 适配 Tauri 状态管理
///
/// 注意使用了 [`std::sync::RwLock`] 而不是 [`tauri::async_runtime::RwLock`] ，读性能较好、编码时需注意内部不能做其他异步操作
#[derive(Debug, Default)]
pub struct ConfigOuter(pub std::sync::RwLock<ConfigInner>);

impl From<ConfigInner> for ConfigOuter {
    fn from(value: ConfigInner) -> Self {
        Self(std::sync::RwLock::new(value))
    }
}

impl From<ConfigSettings> for ConfigOuter {
    fn from(value: ConfigSettings) -> Self {
        Self(std::sync::RwLock::new(ConfigInner::from(value)))
    }
}

/// 配置数据（前端交互）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfigSettings {
    pub main_window_mode: MainWindowMode,
    pub custom_shadow: bool,
    pub window_frame: bool,
    pub always_on_top: bool,
    pub main_width: f64,
    pub main_height: f64,
}

impl Default for ConfigSettings {
    fn default() -> Self {
        Self {
            main_window_mode: MainWindowMode::HideAndShow,
            custom_shadow: false,
            window_frame: false,
            always_on_top: true,
            main_width: 800.0,
            main_height: 500.0,
        }
    }
}

impl ConfigSettings {
    /// 获取配置文件路径 (通常在 AppData 目录下)
    fn get_path(app: &tauri::AppHandle) -> PathBuf {
        tauri::Manager::path(app)
            .app_data_dir()
            .expect("get app data dir failed")
            .join(CONFIG_FILE_NAME)
    }

    /// 加载配置：若不存在则创建并返回默认值
    pub fn load(app: &tauri::AppHandle) -> tauri::Result<Self> {
        let path = Self::get_path(app);

        if path.exists() {
            // 读取文件内容
            let content = fs::read_to_string(&path)?;
            // 尝试解析 JSON，解析失败（如文件损坏）则返回默认值
            Ok(serde_json::from_str::<ConfigSettings>(&content)?)
        } else {
            // 如果目录不存在，先创建目录
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            // 文件不存在，生成默认值并保存
            let default_config = Self::default();
            default_config.save(app)?;
            Ok(default_config)
        }
    }

    /// 更新并持久化保存
    pub fn save(&self, app: &tauri::AppHandle) -> tauri::Result<()> {
        let path = Self::get_path(app);
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MainWindowMode {
    Always,
    HideAndShow,
    CloseAndOpen,
    OnCurrentScreen,
}
