use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, OnceLock},
};

use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};

use crate::{
    constants::CONFIG_FILE_NAME,
    shortcuts::ShortcutChar,
    window_effect::{self, WindowEffect},
};

pub static CONFIG: OnceLock<ArcSwap<Config>> = OnceLock::new();

pub fn load_data(app: &tauri::AppHandle) {
    let config = Config::load(app).expect("load config failed");

    CONFIG
        .set(ArcSwap::from(Arc::new(config)))
        .expect("staticize config failed");
}

/// 重新读取配置文件，返回配置是否发生变化
///
/// 只更新运行时配置；要不要重建窗口由触发方决定（见 [`crate::window_utils::recreate_main_window`]），
/// 免得 configs 反向依赖 window_utils。
pub fn reload_data(app: &tauri::AppHandle) -> bool {
    let config = Config::load(app).expect("load config failed");
    let previous = get_data();

    if *previous == config {
        return false;
    }

    CONFIG
        .get()
        .expect("config must be initialized before use")
        .store(Arc::new(config));
    true
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
    #[serde(deserialize_with = "deserialize_tolerant")]
    pub main_window_mode: MainWindowMode,
    /// 窗口背景与外观；`None` 表示跟随当前平台的推荐值
    #[serde(deserialize_with = "deserialize_tolerant")]
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
            main_window_mode: MainWindowMode::default(),
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
            // 解析失败（如文件损坏）直接报错，由调用方决定怎么处理；
            // 未知的键会被忽略，未知的枚举名退回默认值（见 deserialize_tolerant）
            Ok(serde_json::from_str::<Config>(&content)?)
        } else {
            // 如果目录不存在，先创建目录
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            // 文件不存在，生成默认值并保存
            let default_config = if cfg!(debug_assertions) {
                Self::dev_default()
            } else {
                Self::default()
            };
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

    /// 开发 / 测试环境（debug 构建）的默认配置
    ///
    /// 只在创建配置文件时用到：默认把快捷键字符设成 X，方便调试；
    /// 发布环境仍用 [`ShortcutChar::default`]（Space）。
    fn dev_default() -> Self {
        Self {
            hot_key_char: "X".to_string(),
            ..Self::default()
        }
    }

    /// 更新并持久化保存
    pub fn save(&self, app: &tauri::AppHandle) -> tauri::Result<()> {
        let path = Self::get_path(app);
        fs::write(path, self.to_file_json()?)?;
        Ok(())
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

/// 容忍配置文件里出现未知的枚举名（换平台、换版本、手写）
///
/// 解析失败一律退回该类型的默认值：解析失败会让应用启动 panic（见 [`load_data`]），
/// 不该由一个残留的效果名或窗口模式背这个锅。只用在枚举字段上，其它字段保持严格。
///
/// 先读成一个完整的 JSON 值再转换，而不是直接 `T::deserialize`：
/// 后者在枚举名不认识时会留下未消费的 token，让外层解析报出莫名其妙的错误。
fn deserialize_tolerant<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::de::DeserializeOwned + Default,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(serde_json::from_value(value).unwrap_or_default())
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum MainWindowMode {
    Always,
    #[default]
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

/// 校验前端提交的 JSON 是不是"整份配置"
///
/// 读盘要容忍未知键与缺失字段（见 ADR-0004），但写盘必须整份回传：
/// 漏字段意味着前端出错，宁可报错也不要把用户的值静默重置成默认值。
pub(crate) fn parse_full_config(payload: serde_json::Value) -> Result<Config, String> {
    if !payload.is_object() {
        return Err("配置必须是 JSON 对象".to_string());
    }

    // 期望的键从 Config 现算，不另立一份手写清单
    let default = serde_json::to_value(Config::default()).map_err(|err| err.to_string())?;
    let expected = key_set(&default);
    let actual = key_set(&payload);

    if actual != expected {
        let missing: Vec<&String> = expected.difference(&actual).collect();
        let extra: Vec<&String> = actual.difference(&expected).collect();
        return Err(format!("配置字段不匹配，缺少 {missing:?}，多出 {extra:?}"));
    }

    // 写盘不接受读盘能容忍的枚举名或类型（见 deserialize_tolerant）
    strict::<MainWindowMode>(&payload, "main_window_mode")?;
    strict::<Option<WindowEffect>>(&payload, "window_effect")?; // null 表示未选择

    serde_json::from_value(payload).map_err(|err| err.to_string())
}

/// 对象的键集合；不是对象时为空
fn key_set(value: &serde_json::Value) -> BTreeSet<String> {
    value
        .as_object()
        .map(|object| object.keys().cloned().collect())
        .unwrap_or_default()
}

/// `key` 对应的值必须能严格解析成 `T`，不走宽容分支
fn strict<T: serde::de::DeserializeOwned>(
    payload: &serde_json::Value,
    key: &str,
) -> Result<(), String> {
    let Some(value) = payload.get(key) else {
        return Err(format!("缺少字段 {key}"));
    };
    serde_json::from_value::<T>(value.clone())
        .map(|_| ())
        .map_err(|err| format!("{key} 无法解析：{err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 磁盘格式的金样本：字段名、字段顺序、`null` 的写法都在这里钉住
    const GOLDEN: &str = include_str!("../tests/fixtures/config.golden.json");

    #[test]
    fn default_config_matches_golden_file() {
        assert_eq!(Config::default().to_file_json().unwrap(), GOLDEN);
    }

    /// 开发环境新建配置时默认用 X 作为快捷键字符，发布环境用 Space
    #[test]
    fn dev_default_uses_x_as_hot_key_char() {
        assert_eq!(Config::dev_default().hot_key_char, "X");
        assert_eq!(
            Config::default().hot_key_char,
            ShortcutChar::default().to_string()
        );
        assert!(Config::dev_default().validate().is_ok());
    }

    #[test]
    fn golden_file_loads_as_default_config() {
        let config: Config = serde_json::from_str(GOLDEN).unwrap();
        assert_eq!(config, Config::default());
    }

    /// 写盘必须整份回传：漏字段或多字段都报错，不静默重置
    #[test]
    fn write_path_requires_the_whole_config() {
        let payload = serde_json::to_value(Config::default()).unwrap();
        assert!(parse_full_config(payload).is_ok());

        let mut partial = serde_json::to_value(Config::default()).unwrap();
        partial.as_object_mut().unwrap().remove("hot_key_char");
        let err = parse_full_config(partial).unwrap_err();
        assert!(err.contains("hot_key_char"), "{err}");

        let mut extra = serde_json::to_value(Config::default()).unwrap();
        extra
            .as_object_mut()
            .unwrap()
            .insert("custom_shadow".to_string(), true.into());
        let err = parse_full_config(extra).unwrap_err();
        assert!(err.contains("custom_shadow"), "{err}");

        assert!(parse_full_config(serde_json::json!("不是对象")).is_err());
    }

    /// 写路径对枚举同样严格，哪怕读盘容忍这些值（见 ADR-0004）
    #[test]
    fn write_path_rejects_values_the_read_path_tolerates() {
        for (key, value) in [
            ("main_window_mode", serde_json::json!("OnCurrentScreen")),
            ("main_window_mode", serde_json::json!(null)),
            ("window_effect", serde_json::json!("Blur")),
            ("window_effect", serde_json::json!(42)),
        ] {
            let mut payload = serde_json::to_value(Config::default()).unwrap();
            payload[key] = value.clone();
            assert!(parse_full_config(payload).is_err(), "{key} = {value}");
        }

        // 同一份内容走读盘仍然宽容，不该让启动失败
        let config: Config =
            serde_json::from_str(r#"{"main_window_mode": "OnCurrentScreen", "window_effect": 42}"#)
                .unwrap();
        assert_eq!(config.main_window_mode, MainWindowMode::default());
        assert_eq!(config.window_effect, None);
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
        assert!(Config::default().validate().is_ok());

        let config = Config {
            main_item_n: 0,
            ..Config::default()
        };
        assert!(config.validate().is_err());

        let config = Config {
            main_item_h: 0.0,
            ..Config::default()
        };
        assert!(config.validate().is_err());

        let config = Config {
            hot_key_char: "F1".to_string(),
            ..Config::default()
        };
        assert!(config.validate().is_err());

        // 当前平台不可用的效果不能保存
        let unavailable = [
            WindowEffect::Mica,
            WindowEffect::Acrylic,
            WindowEffect::Vibrancy,
        ]
        .into_iter()
        .find(|effect| !window_effect::available().contains(effect));
        if let Some(unavailable) = unavailable {
            let config = Config {
                window_effect: Some(unavailable),
                ..Config::default()
            };
            assert!(config.validate().is_err());
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

    /// 未知枚举名（换平台、换版本、手写）退回默认值，不让整份配置解析失败
    #[test]
    fn unknown_enum_names_fall_back_to_defaults() {
        let config: Config = serde_json::from_str(
            r#"{"main_window_mode": "OnCurrentScreen", "window_effect": "Blur"}"#,
        )
        .unwrap();

        assert_eq!(config.main_window_mode, MainWindowMode::default());
        assert_eq!(config.window_effect, None);
    }

    /// 枚举字段类型写错同样退回默认值
    #[test]
    fn wrong_type_enum_falls_back_to_default() {
        let config: Config =
            serde_json::from_str(r#"{"main_window_mode": 1, "window_effect": 42}"#).unwrap();

        assert_eq!(config.main_window_mode, MainWindowMode::default());
        assert_eq!(config.window_effect, None);
    }

    /// 非枚举字段保持严格：类型写错要报错，而不是静默用默认值
    #[test]
    fn wrong_type_on_plain_fields_is_an_error() {
        assert!(serde_json::from_str::<Config>(r#"{"main_item_n": "10"}"#).is_err());
        assert!(serde_json::from_str::<Config>(r#"{"always_on_top": "yes"}"#).is_err());
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
