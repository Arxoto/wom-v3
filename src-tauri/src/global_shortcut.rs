use std::{fmt::Display, str::FromStr, sync::Mutex};

use tauri::Manager;
use tauri_plugin_global_shortcut::{
    Code, Error, GlobalShortcutExt, Modifiers, Shortcut, ShortcutEvent,
};
use tauri_plugin_log::log::warn;

/// 当前已注册的全局快捷键状态
///
/// 注册 / 注销均由用户手动触发（托盘菜单），因此需要在内存中保存当前已注册的快捷键，
/// 用于判断是否需要重新注册，以及注销时确定注销哪一个快捷键。
pub struct GlobalShortcutStat(Mutex<Option<Shortcut>>);

impl GlobalShortcutStat {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }

    /// 获取当前已注册的快捷键，未注册时为 [`None`]
    pub fn get(&self) -> Option<Shortcut> {
        // 该状态只保存一个可复制的值，即使锁中毒也可以安全地取回其中的数据
        *self.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// 保存当前已注册的快捷键，未注册时保存 [`None`]
    pub fn set(&self, shortcut: Option<Shortcut>) {
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) = shortcut;
    }
}

impl Default for GlobalShortcutStat {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ShortcutChar {
    Space,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
}

impl Default for ShortcutChar {
    fn default() -> Self {
        Self::Space
    }
}

impl ShortcutChar {
    pub fn as_str(&self) -> &str {
        match self {
            ShortcutChar::Space => "Space",
            ShortcutChar::A => "A",
            ShortcutChar::B => "B",
            ShortcutChar::C => "C",
            ShortcutChar::D => "D",
            ShortcutChar::E => "E",
            ShortcutChar::F => "F",
            ShortcutChar::G => "G",
            ShortcutChar::H => "H",
            ShortcutChar::I => "I",
            ShortcutChar::J => "J",
            ShortcutChar::K => "K",
            ShortcutChar::L => "L",
            ShortcutChar::M => "M",
            ShortcutChar::N => "N",
            ShortcutChar::O => "O",
            ShortcutChar::P => "P",
            ShortcutChar::Q => "Q",
            ShortcutChar::R => "R",
            ShortcutChar::S => "S",
            ShortcutChar::T => "T",
            ShortcutChar::U => "U",
            ShortcutChar::V => "V",
            ShortcutChar::W => "W",
            ShortcutChar::X => "X",
            ShortcutChar::Y => "Y",
            ShortcutChar::Z => "Z",
        }
    }
}

impl Display for ShortcutChar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub struct ShortcutParseFailed;

impl FromStr for ShortcutChar {
    type Err = ShortcutParseFailed;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Space" => Ok(Self::Space),
            "A" => Ok(Self::A),
            "B" => Ok(Self::B),
            "C" => Ok(Self::C),
            "D" => Ok(Self::D),
            "E" => Ok(Self::E),
            "F" => Ok(Self::F),
            "G" => Ok(Self::G),
            "H" => Ok(Self::H),
            "I" => Ok(Self::I),
            "J" => Ok(Self::J),
            "K" => Ok(Self::K),
            "L" => Ok(Self::L),
            "M" => Ok(Self::M),
            "N" => Ok(Self::N),
            "O" => Ok(Self::O),
            "P" => Ok(Self::P),
            "Q" => Ok(Self::Q),
            "R" => Ok(Self::R),
            "S" => Ok(Self::S),
            "T" => Ok(Self::T),
            "U" => Ok(Self::U),
            "V" => Ok(Self::V),
            "W" => Ok(Self::W),
            "X" => Ok(Self::X),
            "Y" => Ok(Self::Y),
            "Z" => Ok(Self::Z),
            _ => Err(ShortcutParseFailed),
        }
    }
}

impl From<ShortcutChar> for Code {
    fn from(value: ShortcutChar) -> Self {
        match value {
            ShortcutChar::Space => Code::Space,
            ShortcutChar::A => Code::KeyA,
            ShortcutChar::B => Code::KeyB,
            ShortcutChar::C => Code::KeyC,
            ShortcutChar::D => Code::KeyD,
            ShortcutChar::E => Code::KeyE,
            ShortcutChar::F => Code::KeyF,
            ShortcutChar::G => Code::KeyG,
            ShortcutChar::H => Code::KeyH,
            ShortcutChar::I => Code::KeyI,
            ShortcutChar::J => Code::KeyJ,
            ShortcutChar::K => Code::KeyK,
            ShortcutChar::L => Code::KeyL,
            ShortcutChar::M => Code::KeyM,
            ShortcutChar::N => Code::KeyN,
            ShortcutChar::O => Code::KeyO,
            ShortcutChar::P => Code::KeyP,
            ShortcutChar::Q => Code::KeyQ,
            ShortcutChar::R => Code::KeyR,
            ShortcutChar::S => Code::KeyS,
            ShortcutChar::T => Code::KeyT,
            ShortcutChar::U => Code::KeyU,
            ShortcutChar::V => Code::KeyV,
            ShortcutChar::W => Code::KeyW,
            ShortcutChar::X => Code::KeyX,
            ShortcutChar::Y => Code::KeyY,
            ShortcutChar::Z => Code::KeyZ,
        }
    }
}

pub fn handler_global_shortcut(
    app_handle: &tauri::AppHandle,
    _shortcut: &Shortcut,
    event: ShortcutEvent,
) {
    // 目前只注册一个快捷键，因此这里无需识别
    match event.state() {
        tauri_plugin_global_shortcut::ShortcutState::Pressed => {
            let _ = crate::window::show_hide_main_window(app_handle);
        }
        tauri_plugin_global_shortcut::ShortcutState::Released => {}
    }
}

fn get_shortcut() -> Shortcut {
    let conf = crate::configs::get_data();
    let hot_key_alt = conf.hot_key_alt;
    let hot_key_ctrl = conf.hot_key_ctrl;
    let hot_key_meta = conf.hot_key_meta;
    let hot_key_shift = conf.hot_key_shift;

    let mods = {
        let mut mods = Modifiers::empty();
        if hot_key_alt {
            mods |= Modifiers::ALT;
        }
        if hot_key_ctrl {
            mods |= Modifiers::CONTROL;
        }
        if hot_key_meta {
            mods |= Modifiers::META;
        }
        if hot_key_shift {
            mods |= Modifiers::SHIFT;
        }
        if mods.is_empty() {
            None
        } else {
            Some(mods)
        }
    };

    let shortcut_char = ShortcutChar::from_str(&conf.hot_key_char);
    let shortcut_char = match shortcut_char {
        Ok(sc) => sc,
        Err(_) => {
            warn!("parse shortcut_char failed, use default");
            ShortcutChar::default()
        }
    };

    let hot_key_char = Code::from(shortcut_char);
    Shortcut::new(mods, hot_key_char)
}

fn get_stat(app: &tauri::AppHandle) -> tauri::State<'_, GlobalShortcutStat> {
    match app.try_state::<GlobalShortcutStat>() {
        Some(stat) => stat,
        // 兜底：正常情况下已在应用启动时注册，此处避免因未托管而 panic
        None => {
            app.manage(GlobalShortcutStat::new());
            app.state::<GlobalShortcutStat>()
        }
    }
}

pub fn register_global_shortcut(app: &tauri::AppHandle) -> std::result::Result<(), Error> {
    let shortcut = get_shortcut();
    let stat = get_stat(app);

    if let Some(registered) = stat.get() {
        // 已保存的快捷键与最新的快捷键一致，无需重新注册
        if registered == shortcut {
            return Ok(());
        }

        // 与最新的快捷键不一致，先注销旧的
        // 注销失败不阻断注册（此时旧快捷键可能已不在注册状态）
        if let Err(e) = app.global_shortcut().unregister(registered) {
            warn!("Failed to unregister old global shortcut: {}", e);
        }
        stat.set(None);
    }

    // 注册并保存最新的快捷键
    app.global_shortcut().register(shortcut)?;
    stat.set(Some(shortcut));
    Ok(())
}

pub fn unregister_global_shortcut(app: &tauri::AppHandle) -> std::result::Result<(), Error> {
    let stat = get_stat(app);

    // 未保存任何快捷键，说明当前没有注册，无需注销
    let Some(registered) = stat.get() else {
        return Ok(());
    };

    app.global_shortcut().unregister(registered)?;
    stat.set(None);
    Ok(())
}
