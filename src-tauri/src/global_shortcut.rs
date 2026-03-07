use tauri_plugin_global_shortcut::{
    Code, Error, GlobalShortcutExt, Modifiers, Shortcut, ShortcutEvent,
};

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

impl ShortcutChar {
    pub fn to_string(&self) -> String {
        String::from(self)
    }
}

impl From<&String> for ShortcutChar {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&str> for ShortcutChar {
    fn from(value: &str) -> Self {
        match value {
            "Space" => Self::Space,
            "A" => Self::A,
            "B" => Self::B,
            "C" => Self::C,
            "D" => Self::D,
            "E" => Self::E,
            "F" => Self::F,
            "G" => Self::G,
            "H" => Self::H,
            "I" => Self::I,
            "J" => Self::J,
            "K" => Self::K,
            "L" => Self::L,
            "M" => Self::M,
            "N" => Self::N,
            "O" => Self::O,
            "P" => Self::P,
            "Q" => Self::Q,
            "R" => Self::R,
            "S" => Self::S,
            "T" => Self::T,
            "U" => Self::U,
            "V" => Self::V,
            "W" => Self::W,
            "X" => Self::X,
            "Y" => Self::Y,
            "Z" => Self::Z,
            _ => Self::Space,
        }
    }
}

impl From<&ShortcutChar> for String {
    fn from(value: &ShortcutChar) -> Self {
        match value {
            ShortcutChar::Space => "Space".to_string(),
            ShortcutChar::A => "A".to_string(),
            ShortcutChar::B => "B".to_string(),
            ShortcutChar::C => "C".to_string(),
            ShortcutChar::D => "D".to_string(),
            ShortcutChar::E => "E".to_string(),
            ShortcutChar::F => "F".to_string(),
            ShortcutChar::G => "G".to_string(),
            ShortcutChar::H => "H".to_string(),
            ShortcutChar::I => "I".to_string(),
            ShortcutChar::J => "J".to_string(),
            ShortcutChar::K => "K".to_string(),
            ShortcutChar::L => "L".to_string(),
            ShortcutChar::M => "M".to_string(),
            ShortcutChar::N => "N".to_string(),
            ShortcutChar::O => "O".to_string(),
            ShortcutChar::P => "P".to_string(),
            ShortcutChar::Q => "Q".to_string(),
            ShortcutChar::R => "R".to_string(),
            ShortcutChar::S => "S".to_string(),
            ShortcutChar::T => "T".to_string(),
            ShortcutChar::U => "U".to_string(),
            ShortcutChar::V => "V".to_string(),
            ShortcutChar::W => "W".to_string(),
            ShortcutChar::X => "X".to_string(),
            ShortcutChar::Y => "Y".to_string(),
            ShortcutChar::Z => "Z".to_string(),
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

    let hot_key_char = Code::from(ShortcutChar::from(&conf.hot_key_char));

    Shortcut::new(mods, hot_key_char)
}

pub fn register_global_shortcut(app: &tauri::AppHandle) -> std::result::Result<(), Error> {
    app.global_shortcut().register(get_shortcut())
}

pub fn unregister_global_shortcut(app: &tauri::AppHandle) -> std::result::Result<(), Error> {
    app.global_shortcut().unregister(get_shortcut())
}
