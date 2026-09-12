use std::{
    fs,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, OnceLock},
};

use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tauri_plugin_log::log::warn;

use crate::{
    constants::CONFIG_FILE_NAME,
    shortcuts::ShortcutChar,
    window_effect::{self, WindowEffect},
};

/// 配置变更事件，前端监听它重新读取配置
pub const CONFIG_CHANGED_EVENT: &str = "config_changed";

pub static CONFIG: OnceLock<ArcSwap<Config>> = OnceLock::new();

pub fn load_data(app: &tauri::AppHandle) {
    let config = Config::load(app).expect("load config failed");

    CONFIG
        .set(ArcSwap::from(Arc::new(config)))
        .expect("staticize config failed");
}

pub fn reload_data(app: &tauri::AppHandle) {
    let config = Config::load(app).expect("load config failed");

    CONFIG
        .get()
        .expect("config must be initialized before use")
        .store(Arc::new(config));

    // 通知前端重读配置：托盘重新加载与配置窗口保存都会走到这里
    if let Err(err) = app.emit(CONFIG_CHANGED_EVENT, ()) {
        warn!("emit {} failed: {}", CONFIG_CHANGED_EVENT, err);
    }
}

pub fn get_data() -> Arc<Config> {
    CONFIG
        .get()
        .expect("config must be initialized before use")
        .load_full()
}

/// 配置
///
/// 与持久化文件一一对应，也是运行时配置的唯一依据；
/// 派生值（窗口尺寸、生效的窗口效果）都由方法现算，不再另存一份运行时结构。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    // ========= 界面基础设置 =========
    /// 应用打开时自动显示主窗口（见 [`Config::show_main_auto`]）
    pub main_window_mode: MainWindowMode,
    /// 窗口背景与外观；`None` 表示跟随当前平台的推荐值
    #[serde(deserialize_with = "deserialize_window_effect")]
    pub window_effect: Option<WindowEffect>,
    /// 始终置顶
    pub always_on_top: bool,
    // ========= 界面布局 =========
    /// head 高度
    pub main_head_h: f64,
    /// tail 高度
    pub main_tail_h: f64,
    /// item 高度
    pub main_item_h: f64,
    /// item 数量
    pub main_item_n: i64,
    // ========= 全局快捷键 =========
    pub hot_key_alt: bool,
    pub hot_key_ctrl: bool,
    pub hot_key_meta: bool,
    pub hot_key_shift: bool,
    pub hot_key_char: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            main_window_mode: MainWindowMode::HideAndShow,
            window_effect: None,
            always_on_top: true,
            main_head_h: 60.0,
            main_tail_h: 24.0,
            main_item_h: 40.0,
            main_item_n: 10,
            hot_key_alt: true,
            hot_key_ctrl: false,
            hot_key_meta: false,
            hot_key_shift: false,
            hot_key_char: ShortcutChar::default().to_string(),
        }
    }
}

impl Config {
    /// 应用启动时自动显示主窗口
    pub fn show_main_auto(&self) -> bool {
        matches!(self.main_window_mode, MainWindowMode::Always)
    }

    /// 主窗口失去焦点时隐藏
    pub fn hide_main_unfocused(&self) -> bool {
        !self.show_main_auto()
    }

    /// 当前平台实际生效的窗口效果：未选择时取推荐值，选中的效果不可用时降级
    pub fn effect(&self) -> WindowEffect {
        window_effect::resolve(self.window_effect)
    }

    /// 窗口宽度：面板宽度 + 两侧 1px 发丝线
    pub fn window_width(&self) -> f64 {
        layout_coupling::PANEL_WIDTH + layout_coupling::EDGE_BORDER
    }

    /// 窗口高度：1px 发丝线 + 分割线 + head / tail / item 的高度
    pub fn window_height(&self) -> f64 {
        layout_coupling::EDGE_BORDER
            + layout_coupling::DIVIDER_H * (layout_coupling::DIVIDER_N as f64)
            + self.main_head_h
            + self.main_tail_h
            + self.main_item_h * (self.main_item_n as f64)
    }

    /// 窗口尺寸
    pub fn window_size(&self) -> (f64, f64) {
        (self.window_width(), self.window_height())
    }

    /// 用前端提交的可编辑配置覆盖对应字段，其余字段保持不动
    pub fn apply(&mut self, edit: &EditableConfig) {
        self.main_window_mode = edit.main_window_mode.clone();
        self.window_effect = edit.window_effect;
        self.always_on_top = edit.always_on_top;
        self.main_head_h = edit.main_head_h;
        self.main_tail_h = edit.main_tail_h;
        self.main_item_h = edit.main_item_h;
        self.main_item_n = edit.main_item_n;
        self.hot_key_alt = edit.hot_key_alt;
        self.hot_key_ctrl = edit.hot_key_ctrl;
        self.hot_key_meta = edit.hot_key_meta;
        self.hot_key_shift = edit.hot_key_shift;
        self.hot_key_char = edit.hot_key_char.clone();
    }

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
            Ok(serde_json::from_str::<Config>(&content)?)
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

    /// 写盘用的 JSON 文本
    ///
    /// 末尾补一个换行：手写 / diff 更友好，也让金样本文件与写盘内容逐字节一致。
    pub fn to_file_json(&self) -> tauri::Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(self)?))
    }

    /// 更新并持久化保存
    pub fn save(&self, app: &tauri::AppHandle) -> tauri::Result<()> {
        let path = Self::get_path(app);
        fs::write(path, self.to_file_json()?)?;
        Ok(())
    }
}

/// 前端用户可编辑的配置
///
/// 是 [`Config`] 的子集：界面能改多少就只暴露多少。
/// 读用 [`From<&Config>`]，写用 [`Config::apply`]；这里不加 `default`，
/// 前端必须整份回传（漏字段直接报错），避免漏传的字段被静默重置。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditableConfig {
    pub main_window_mode: MainWindowMode,
    pub window_effect: Option<WindowEffect>,
    pub always_on_top: bool,
    pub main_head_h: f64,
    pub main_tail_h: f64,
    pub main_item_h: f64,
    pub main_item_n: i64,
    pub hot_key_alt: bool,
    pub hot_key_ctrl: bool,
    pub hot_key_meta: bool,
    pub hot_key_shift: bool,
    pub hot_key_char: String,
}

impl EditableConfig {
    /// 保存前的校验：写盘是显式动作，宁可报错也不静默改用户的值
    pub fn validate(&self) -> Result<(), String> {
        if self.main_head_h <= 0.0 {
            return Err(format!("head 高度必须大于 0，当前 {}", self.main_head_h));
        }
        if self.main_tail_h < 0.0 {
            return Err(format!("tail 高度不能为负，当前 {}", self.main_tail_h));
        }
        if self.main_item_h <= 0.0 {
            return Err(format!("item 高度必须大于 0，当前 {}", self.main_item_h));
        }
        if self.main_item_n <= 0 {
            return Err(format!("item 数量必须大于 0，当前 {}", self.main_item_n));
        }
        if ShortcutChar::from_str(&self.hot_key_char).is_err() {
            return Err(format!("无法识别的快捷键字符：{}", self.hot_key_char));
        }
        if let Some(effect) = self.window_effect {
            if !window_effect::available().contains(&effect) {
                return Err(format!("当前平台不支持窗口效果 {:?}", effect));
            }
        }
        Ok(())
    }
}

impl From<&Config> for EditableConfig {
    fn from(value: &Config) -> Self {
        Self {
            main_window_mode: value.main_window_mode.clone(),
            window_effect: value.window_effect,
            always_on_top: value.always_on_top,
            main_head_h: value.main_head_h,
            main_tail_h: value.main_tail_h,
            main_item_h: value.main_item_h,
            main_item_n: value.main_item_n,
            hot_key_alt: value.hot_key_alt,
            hot_key_ctrl: value.hot_key_ctrl,
            hot_key_meta: value.hot_key_meta,
            hot_key_shift: value.hot_key_shift,
            hot_key_char: value.hot_key_char.clone(),
        }
    }
}

/// 与前端布局耦合的数据，需要同步修改
mod layout_coupling {
    /// 面板宽度，窗口宽度再额外加上两侧的发丝线
    pub const PANEL_WIDTH: f64 = 800.0;
    /// 面板左右/上下各 1px 的发丝线
    pub const EDGE_BORDER: f64 = 2.0;
    pub const DIVIDER_H: f64 = 8.0;
    pub const DIVIDER_N: i64 = 2;
}

/// 容忍配置文件里出现未知的效果名（手写配置），一律视为"未选择"
fn deserialize_window_effect<'de, D>(deserializer: D) -> Result<Option<WindowEffect>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<WindowEffect>::deserialize(deserializer).unwrap_or(None))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MainWindowMode {
    Always,
    HideAndShow,
}

/// 前端渲染窗口所需的信息
#[derive(Debug, Clone, Serialize)]
pub struct EffectInfo {
    /// 用户选择的效果；`None` 表示跟随平台推荐值
    pub configured: Option<WindowEffect>,
    /// 当前实际生效的效果（推荐值或降级后的值）
    pub effective: WindowEffect,
    /// 当前平台与系统版本可选的效果，供配置界面使用
    pub available: Vec<WindowEffect>,
    /// 面板底色的透明度，对应前端 js 写入的 `--color-bg-alpha`
    pub alpha: f64,
}

#[tauri::command]
pub async fn fetch_editable_config() -> EditableConfig {
    EditableConfig::from(&*get_data())
}

#[tauri::command]
pub async fn fetch_effect_info() -> EffectInfo {
    let config = get_data();
    let effective = config.effect();

    EffectInfo {
        configured: config.window_effect,
        effective,
        available: window_effect::available(),
        alpha: window_effect::alpha(effective),
    }
}

/// 保存前端提交的可编辑配置，并让它立刻生效
///
/// 同步命令：窗口效果只能在主线程应用（见 [`crate::window_effect::apply`]）。
#[tauri::command]
pub fn set_editable_config(app: tauri::AppHandle, edit: EditableConfig) -> Result<(), String> {
    edit.validate()?;

    let previous = get_data();
    let mut config = (*previous).clone();
    config.apply(&edit);

    // 文件与运行时始终是同一份内容
    config.save(&app).map_err(|err| err.to_string())?;
    reload_data(&app);

    // 窗口效果只能在建窗时应用，变了就重建；其余情况同步一次窗口尺寸即可
    if config.effect() != previous.effect() {
        crate::window_utils::recreate_main_window(&app).map_err(|err| err.to_string())?;
    } else {
        crate::window_utils::resize_main_window(&app, config.window_size());
    }

    // 快捷键可能变了，重新注册（内部会与当前注册值比对）
    #[cfg(desktop)]
    crate::global_shortcut::register_global_shortcut(&app).map_err(|err| err.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 磁盘格式的金样本：字段名、字段顺序、`null` 的写法都在这里钉住
    const GOLDEN: &str = include_str!("../tests/fixtures/config.golden.json");

    fn object_keys(value: &serde_json::Value) -> Vec<String> {
        let object = value.as_object().expect("需要是 JSON 对象");
        let mut keys: Vec<String> = object.keys().cloned().collect();
        keys.sort();
        keys
    }

    #[test]
    fn default_config_matches_golden_file() {
        assert_eq!(Config::default().to_file_json().unwrap(), GOLDEN);
    }

    #[test]
    fn golden_file_loads_as_default_config() {
        let config: Config = serde_json::from_str(GOLDEN).unwrap();
        assert_eq!(config, Config::default());
    }

    /// 新增 Config 字段时必须同步 EditableConfig（否则这条会红）
    #[test]
    fn editable_config_mirrors_every_config_field() {
        let config = Config::default();
        let config_keys = object_keys(&serde_json::to_value(&config).unwrap());
        let edit_keys = object_keys(&serde_json::to_value(EditableConfig::from(&config)).unwrap());

        assert_eq!(edit_keys, config_keys);
    }

    #[test]
    fn apply_overwrites_the_editable_fields() {
        let mut config = Config::default();
        let mut edit = EditableConfig::from(&config);
        edit.main_item_n = 3;
        edit.hot_key_char = "K".to_string();
        edit.window_effect = Some(WindowEffect::Solid);
        edit.always_on_top = false;

        config.apply(&edit);

        assert_eq!(config.main_item_n, 3);
        assert_eq!(config.hot_key_char, "K");
        assert_eq!(config.window_effect, Some(WindowEffect::Solid));
        assert!(!config.always_on_top);
    }

    /// 窗口尺寸由布局字段算出，不再持久化宽度
    #[test]
    fn window_size_follows_the_layout_fields() {
        let config = Config::default();
        // 800 + 1px × 2
        assert_eq!(config.window_width(), 802.0);
        // 1px × 2 + 分割线 8 × 2 + 60 + 24 + 40 × 10
        assert_eq!(config.window_height(), 502.0);

        let mut taller = config.clone();
        taller.main_item_n = 5;
        assert_eq!(taller.window_height(), 302.0);
    }

    #[test]
    fn validate_rejects_broken_values() {
        let config = Config::default();
        assert!(EditableConfig::from(&config).validate().is_ok());

        let mut edit = EditableConfig::from(&config);
        edit.main_item_n = 0;
        assert!(edit.validate().is_err());

        let mut edit = EditableConfig::from(&config);
        edit.main_item_h = 0.0;
        assert!(edit.validate().is_err());

        let mut edit = EditableConfig::from(&config);
        edit.hot_key_char = "F1".to_string();
        assert!(edit.validate().is_err());

        // 当前平台不可用的效果不能保存
        let unavailable = [
            WindowEffect::Mica,
            WindowEffect::Acrylic,
            WindowEffect::Vibrancy,
        ]
        .into_iter()
        .find(|effect| !window_effect::available().contains(effect));
        if let Some(unavailable) = unavailable {
            let mut edit = EditableConfig::from(&config);
            edit.window_effect = Some(unavailable);
            assert!(edit.validate().is_err());
        }
    }

    #[test]
    fn window_effect_round_trips_as_string() {
        for (json, effect) in [
            (r#""Solid""#, WindowEffect::Solid),
            (r#""Framed""#, WindowEffect::Framed),
            (r#""Mica""#, WindowEffect::Mica),
        ] {
            let actual: Option<WindowEffect> = serde_json::from_str(json).unwrap();
            assert_eq!(actual, Some(effect));
            assert_eq!(serde_json::to_string(&Some(effect)).unwrap(), json);
        }
    }

    /// 手写或换平台遗留的效果名不应该让配置加载失败
    #[test]
    fn unknown_window_effect_is_unset() {
        let config: Config = serde_json::from_str(r#"{"window_effect": "Blur"}"#).unwrap();
        assert_eq!(config.window_effect, None);
    }

    /// 开发期允许破坏性升级：旧文件只要不崩即可，未知键忽略、缺失字段用默认
    #[test]
    fn stale_config_file_loads_with_defaults() {
        let config: Config = serde_json::from_str(
            r#"{"main_window_mode": "Always", "custom_shadow": true, "window_frame": true, "main_width": 800.0}"#,
        )
        .unwrap();

        assert_eq!(config.main_window_mode, MainWindowMode::Always);
        assert!(config.show_main_auto());
        assert_eq!(config.window_effect, None);
        assert_eq!(config.main_item_n, Config::default().main_item_n);
    }
}
