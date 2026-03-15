use std::{fmt::Display, str::FromStr};

use tauri_plugin_global_shortcut::{
    Code, Error, GlobalShortcutExt, Modifiers, Shortcut, ShortcutEvent,
};
use tauri_plugin_log::log::warn;

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

pub fn register_global_shortcut(app: &tauri::AppHandle) -> std::result::Result<(), Error> {
    app.global_shortcut().register(get_shortcut())
}

pub fn unregister_global_shortcut(app: &tauri::AppHandle) -> std::result::Result<(), Error> {
    app.global_shortcut().unregister(get_shortcut())
}
