use serde::{Deserialize, Serialize};
use tauri::WebviewWindow;
use tauri_plugin_log::log::{debug, warn};

/// 窗口背景与外观
///
/// 原生效果与经典外观（Solid Panel）是互斥的两条路：用原生效果时窗口必须透明、无原生框架，
/// 所以"是否使用系统原生框架"只体现在经典外观的两种形态上。
/// 窗口阴影一律用系统原生阴影，不做自绘。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowEffect {
    /// 不使用原生效果、不使用原生框架：无边框的经典面板
    #[default]
    Solid,
    /// 不使用原生效果、使用系统原生框架：经典面板 + 原生标题栏
    Framed,
    /// Windows 11：桌面壁纸着色
    Mica,
    /// Windows 10 v1809+：亚克力模糊
    Acrylic,
    /// macOS：NSVisualEffectView 毛玻璃
    Vibrancy,
    // todo Liquid Glass（macOS 26+）：window-vibrancy 0.7 引入、0.8 就换掉了 API（LiquidGlassOptions），
    //      而且 Tauri 下必须用 with_webview 把 WKWebView 交给 NSGlassEffectView 的 contentView，
    //      失败只能异步回传。等它在 macOS 26 真机上验证稳定再加回来：恢复 macOS 的 objc2-app-kit 依赖，
    //      并在此补变体 + candidates() / is_available() / alpha() / apply_one() 各一条。
}

impl WindowEffect {
    /// 是否使用系统原生框架；使用原生效果时始终为 false
    pub fn window_frame(&self) -> bool {
        matches!(self, Self::Framed)
    }

    /// 窗口是否需要透明：原生效果只有透过透明窗口才能看到，经典外观跟随原生框架
    pub fn transparent(&self) -> bool {
        !self.window_frame()
    }
}

/// 未做选择时的效果：跟随当前平台的推荐值
pub fn recommended() -> WindowEffect {
    candidates()
        .iter()
        .copied()
        .find(|effect| is_available(*effect))
        .unwrap_or_default()
}

/// 当前平台与系统版本下可选的效果，末尾是经典外观的两种形态，供配置界面使用
pub fn available() -> Vec<WindowEffect> {
    let mut effects: Vec<WindowEffect> = candidates()
        .iter()
        .copied()
        .filter(|effect| is_available(*effect))
        .collect();
    effects.push(WindowEffect::Solid);
    effects.push(WindowEffect::Framed);
    effects
}

/// 解析用户配置：未选择时用推荐值；选中的效果不可用时沿候选链降级
pub fn resolve(configured: Option<WindowEffect>) -> WindowEffect {
    let Some(configured) = configured else {
        return recommended();
    };

    // 经典外观永远可用
    if is_available(configured) {
        return configured;
    }

    // 从选中项开始沿候选链向下找第一个可用项
    let candidates = candidates();
    candidates
        .iter()
        .position(|effect| *effect == configured)
        .and_then(|index| {
            candidates[index..]
                .iter()
                .copied()
                .find(|effect| is_available(*effect))
        })
        .unwrap_or_default()
}

/// 面板底色的透明度，对应前端 js 写入的 `--color-bg-alpha`
pub fn alpha(effect: WindowEffect) -> f64 {
    match effect {
        WindowEffect::Solid | WindowEffect::Framed => 1.0,
        WindowEffect::Mica => 0.35,
        WindowEffect::Acrylic => 0.55,
        WindowEffect::Vibrancy => 0.6,
    }
}

/// 应用效果，返回实际生效的效果（降级后会与传入值不同）
pub fn apply(window: &WebviewWindow, effect: WindowEffect) -> WindowEffect {
    // 经典外观不需要任何系统调用
    if matches!(effect, WindowEffect::Solid | WindowEffect::Framed) {
        return effect;
    }

    // 从配置的效果开始沿链向下，先成功的胜出
    let candidates = candidates();
    let start = candidates
        .iter()
        .position(|item| *item == effect)
        .unwrap_or(0);

    for candidate in &candidates[start..] {
        match apply_one(window, *candidate) {
            Ok(()) => {
                debug!("window effect {:?} applied", candidate);
                return *candidate;
            }
            Err(err) => warn!("apply window effect {:?} failed: {}", candidate, err),
        }
    }

    warn!("no window effect applied, fallback to solid");
    WindowEffect::default()
}

/// 当前平台的原生效果候选，越靠前越优先，降级时沿此顺序向下找
fn candidates() -> &'static [WindowEffect] {
    #[cfg(target_os = "windows")]
    let candidates = &[WindowEffect::Mica, WindowEffect::Acrylic];

    #[cfg(target_os = "macos")]
    let candidates = &[WindowEffect::Vibrancy];

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let candidates: &'static [WindowEffect] = &[];

    candidates
}

#[cfg(target_os = "windows")]
fn is_available(effect: WindowEffect) -> bool {
    // build 22000 = Windows 11 21H2，17763 = Windows 10 v1809
    let build = windows_version::OsVersion::current().build;
    match effect {
        WindowEffect::Mica => build >= 22000,
        WindowEffect::Acrylic => build >= 17763,
        WindowEffect::Solid | WindowEffect::Framed => true,
        _ => false,
    }
}

#[cfg(target_os = "macos")]
fn is_available(effect: WindowEffect) -> bool {
    matches!(
        effect,
        WindowEffect::Vibrancy | WindowEffect::Solid | WindowEffect::Framed
    )
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn is_available(effect: WindowEffect) -> bool {
    // Linux 等平台的效果由合成器决定，不做处理
    matches!(effect, WindowEffect::Solid | WindowEffect::Framed)
}

#[cfg(target_os = "windows")]
fn apply_one(window: &WebviewWindow, effect: WindowEffect) -> Result<(), String> {
    let result = match effect {
        // dark 同时决定原生标题栏的明暗，与当前浅色配色保持一致
        WindowEffect::Mica => window_vibrancy::apply_mica(window, Some(false)),
        // 颜色留空，面板色调交给 css 的半透明底色（build >= 22523 时该参数会被忽略）
        WindowEffect::Acrylic => window_vibrancy::apply_acrylic(window, None),
        WindowEffect::Solid | WindowEffect::Framed => return Ok(()),
        _ => return Err(format!("{:?} is not supported on Windows", effect)),
    };
    result.map_err(|err| err.to_string())
}

#[cfg(target_os = "macos")]
fn apply_one(window: &WebviewWindow, effect: WindowEffect) -> Result<(), String> {
    let result = match effect {
        // 面板是直角+1px 边框，材质本身不做圆角
        WindowEffect::Vibrancy => window_vibrancy::apply_vibrancy(
            window,
            window_vibrancy::NSVisualEffectMaterial::Popover,
            Some(window_vibrancy::NSVisualEffectState::Active),
            None,
        ),
        WindowEffect::Solid | WindowEffect::Framed => return Ok(()),
        _ => return Err(format!("{:?} is not supported on macOS", effect)),
    };
    result.map_err(|err| err.to_string())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn apply_one(_window: &WebviewWindow, effect: WindowEffect) -> Result<(), String> {
    match effect {
        WindowEffect::Solid | WindowEffect::Framed => Ok(()),
        _ => Err(format!("{:?} is not supported on this platform", effect)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 原生效果必须透明且没有原生框架
    #[test]
    fn native_effect_has_no_window_frame() {
        for effect in [
            WindowEffect::Mica,
            WindowEffect::Acrylic,
            WindowEffect::Vibrancy,
        ] {
            assert!(!effect.window_frame(), "{:?}", effect);
            assert!(effect.transparent(), "{:?}", effect);
        }
    }

    /// 经典外观的两种形态只差原生框架
    #[test]
    fn classic_appearance_follows_the_variant() {
        assert!(!WindowEffect::Solid.window_frame());
        assert!(WindowEffect::Solid.transparent());

        assert!(WindowEffect::Framed.window_frame());
        assert!(!WindowEffect::Framed.transparent());
    }

    #[test]
    fn classic_appearance_is_always_available() {
        for effect in [WindowEffect::Solid, WindowEffect::Framed] {
            assert!(available().contains(&effect), "{:?}", effect);
            assert_eq!(resolve(Some(effect)), effect);
        }
        assert_eq!(WindowEffect::default(), WindowEffect::Solid);
    }

    #[test]
    fn unset_follows_recommended() {
        assert_eq!(resolve(None), recommended());
        assert!(available().contains(&recommended()));
    }

    #[test]
    fn available_effects_are_kept_as_is() {
        for effect in available() {
            assert_eq!(resolve(Some(effect)), effect);
        }
    }

    #[test]
    fn unavailable_effect_downgrades_to_an_available_one() {
        for effect in [
            WindowEffect::Mica,
            WindowEffect::Acrylic,
            WindowEffect::Vibrancy,
        ] {
            let resolved = resolve(Some(effect));
            assert!(
                available().contains(&resolved),
                "{:?} resolved to unavailable {:?}",
                effect,
                resolved
            );
        }
    }

    #[test]
    fn classic_appearance_is_opaque() {
        assert_eq!(alpha(WindowEffect::Solid), 1.0);
        assert_eq!(alpha(WindowEffect::Framed), 1.0);
        for effect in [
            WindowEffect::Mica,
            WindowEffect::Acrylic,
            WindowEffect::Vibrancy,
        ] {
            assert!(alpha(effect) < 1.0, "{:?} should be translucent", effect);
        }
    }
}
