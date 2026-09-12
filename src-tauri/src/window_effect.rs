use serde::{Deserialize, Serialize};
use tauri::WebviewWindow;
use tauri_plugin_log::log::{debug, warn};

/// 窗口背景与外观
///
/// 原生效果与经典外观（Solid Panel）是互斥的两条路：用原生效果时窗口必须透明、无原生框架，
/// 所以"是否使用系统原生框架"只体现在经典外观的两种形态上。
/// 窗口阴影一律用系统原生阴影，不做自绘。
/// 
/// todo Liquid Glass (macOS 26+) 目前 API 不够稳定，
/// 且 Tauri 下必须用 with_webview 把 WKWebView 交给 NSGlassEffectView 的 contentView 。
/// 待稳定再加回来：恢复 macOS 的 objc2-app-kit 依赖。
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
        .find(|effect| is_available(*effect))
        .unwrap_or_default()
}

/// 当前平台与系统版本下可选的效果，末尾是经典外观的两种形态，供配置界面使用
pub fn available() -> Vec<WindowEffect> {
    candidates()
        .filter(|effect| is_available(*effect))
        .collect()
}

/// 解析用户配置：未选择时用推荐值；选中的效果不可用时沿候选链降级
pub fn resolve(configured: Option<WindowEffect>) -> WindowEffect {
    let Some(configured) = configured else {
        return recommended();
    };

    // 从选中项开始沿候选链向下找第一个可用项；链尾的经典外观永远可用
    chain_from(configured)
        .find(|effect| is_available(*effect))
        .unwrap_or_default()
}

/// 面板底色的透明度，前端写入 css 变量 `--color-bg-alpha`（见 index_main.css 的底色层）
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
    // 从配置的效果开始沿链向下，先成功的胜出；经典外观在链尾，不需要系统调用
    for candidate in chain_from(effect) {
        match apply_one(window, candidate) {
            Ok(()) => {
                debug!("window effect {:?} applied", candidate);
                return candidate;
            }
            Err(err) => warn!("apply window effect {:?} failed: {}", candidate, err),
        }
    }

    // 只有 effect 不在本平台的候选链上时才会走到这里
    warn!("window effect {:?} is off chain, fallback to solid", effect);
    WindowEffect::default()
}

/// 经典外观的两种形态：不依赖原生材质，任何平台都能用，所以排在候选链末尾兜底
const CLASSIC_APPEARANCE: [WindowEffect; 2] = [WindowEffect::Solid, WindowEffect::Framed];

/// 当前平台的候选链，越靠前越优先，降级时沿此顺序向下找；链尾是经典外观
fn candidates() -> impl Iterator<Item = WindowEffect> {
    #[cfg(target_os = "windows")]
    let native: &'static [WindowEffect] = &[WindowEffect::Mica, WindowEffect::Acrylic];

    #[cfg(target_os = "macos")]
    let native: &'static [WindowEffect] = &[WindowEffect::Vibrancy];

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let native: &'static [WindowEffect] = &[];

    native.iter().copied().chain(CLASSIC_APPEARANCE)
}

/// 候选链从 `effect` 起（含）的尾部；`effect` 不在链上（换平台带来的效果）时为空
fn chain_from(effect: WindowEffect) -> impl Iterator<Item = WindowEffect> {
    candidates().skip_while(move |candidate| *candidate != effect)
}

#[cfg(target_os = "windows")]
fn is_available(effect: WindowEffect) -> bool {
    // build 22000 = Windows 11 21H2，17763 = Windows 10 v1809
    // 与 window-vibrancy 内部的版本判断保持一致（见其 windows.rs 的 is_swca_supported
    // 与 is_undocumented_mica_supported），升级该依赖时要一起核对
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

    /// 链从选中项本身开始；不在链上的效果（如 Windows 上的 Vibrancy）拿不到链
    #[test]
    fn chain_starts_at_the_selected_effect() {
        for effect in candidates() {
            assert_eq!(chain_from(effect).next(), Some(effect), "{:?}", effect);
        }

        for effect in [
            WindowEffect::Mica,
            WindowEffect::Acrylic,
            WindowEffect::Vibrancy,
        ] {
            if candidates().any(|candidate| candidate == effect) {
                continue;
            }
            assert_eq!(chain_from(effect).next(), None, "{:?}", effect);
            assert_eq!(resolve(Some(effect)), WindowEffect::Solid, "{:?}", effect);
        }
    }

    /// 每条链都以经典外观收尾且链尾可用：降级总有着落
    #[test]
    fn every_chain_ends_in_the_classic_appearance() {
        for effect in candidates() {
            let last = chain_from(effect).last().expect("chain is never empty");
            assert!(
                matches!(last, WindowEffect::Solid | WindowEffect::Framed),
                "{:?} ends with {:?}",
                effect,
                last
            );
            assert!(is_available(last), "{:?}", last);
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
