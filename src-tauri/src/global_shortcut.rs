use std::{str::FromStr, sync::Mutex};

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, RunEvent, Wry,
};
use tauri_plugin_global_shortcut::{
    Code, Error, GlobalShortcutExt, Modifiers, Shortcut, ShortcutEvent,
};
use tauri_plugin_log::log::warn;

use crate::{configs, shortcuts::ShortcutChar, window_utils};

/// 全局快捷键插件名称
pub const PLUGIN_NAME: &str = "wom-global-shortcut";

/// 系统级快捷键注册能力初始化
///
/// 由 `tauri-plugin-global-shortcut` 提供系统级注册能力，
/// 该插件必须在 [`global_shortcut_startup_register`] 之前注册。
pub fn global_shortcut_handler_init() -> TauriPlugin<Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(handler_global_shortcut)
        .build()
}

/// 全局快捷键插件
///
/// 必须先注册 [`global_shortcut_handler_init`]
pub fn global_shortcut_startup_register() -> TauriPlugin<Wry> {
    Builder::new(PLUGIN_NAME)
        .setup(|app, _api| {
            // 托管当前已注册快捷键的状态
            app.manage(GlobalShortcutStat::new());
            // 插件 `setup` 早于应用自身的 `setup`，此时 `configs` 尚未加载，无法读取用户配置的快捷键
            Ok(())
        })
        .on_event(|app, event| {
            if matches!(event, RunEvent::Ready) {
                // 应用就绪后注册全局快捷键
                let res = register_global_shortcut(app);
                if let Err(e) = res {
                    warn!("Failed to register global shortcut on startup: {}", e);
                }
            }
        })
        .build()
}

/// 状态由 [`global_shortcut_startup_register`] 在插件初始化时托管
fn get_stat(app: &tauri::AppHandle) -> tauri::State<'_, GlobalShortcutStat> {
    app.state::<GlobalShortcutStat>()
}

/// 运行时的注册动作可以手动触发
pub fn register_global_shortcut(app: &tauri::AppHandle) -> Result<(), Error> {
    let shortcut = get_shortcut();
    let stat = get_stat(app);

    if let Some(registered) = stat.get() {
        // 已保存的快捷键与最新的快捷键一致，无需重新注册
        if registered == shortcut {
            return Ok(());
        }

        // 与最新的快捷键不一致，先注销旧的
        // 注销失败不阻断注册（此时旧快捷键可能已不在注册状态）
        let res = app.global_shortcut().unregister(registered);
        if let Err(e) = res {
            warn!("Failed to unregister old global shortcut: {}", e);
        }
        stat.set(None);
    }

    // 注册并保存最新的快捷键
    app.global_shortcut().register(shortcut)?;
    stat.set(Some(shortcut));
    Ok(())
}

/// 运行时的注销动作可以手动触发
pub fn unregister_global_shortcut(app: &tauri::AppHandle) -> Result<(), Error> {
    let stat = get_stat(app);

    // 未保存任何快捷键，说明当前没有注册，无需注销
    let Some(registered) = stat.get() else {
        return Ok(());
    };

    app.global_shortcut().unregister(registered)?;
    stat.set(None);
    Ok(())
}

/// 当前已注册的全局快捷键状态
///
/// 注册 / 注销均由用户手动触发（托盘菜单），因此需要在内存中保存当前已注册的快捷键，
/// 用于判断是否需要重新注册，以及注销时确定注销哪一个快捷键。
struct GlobalShortcutStat(Mutex<Option<Shortcut>>);

impl GlobalShortcutStat {
    fn new() -> Self {
        Self(Mutex::new(None))
    }

    /// 获取当前已注册的快捷键，未注册时为 [`None`]
    fn get(&self) -> Option<Shortcut> {
        // 该状态只保存一个可复制的值，即使锁中毒也可以安全地取回其中的数据
        *self.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// 保存当前已注册的快捷键，未注册时保存 [`None`]
    fn set(&self, shortcut: Option<Shortcut>) {
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) = shortcut;
    }
}

impl Default for GlobalShortcutStat {
    fn default() -> Self {
        Self::new()
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
            let _ = window_utils::show_hide_main_window(app_handle);
        }
        tauri_plugin_global_shortcut::ShortcutState::Released => {}
    }
}

fn get_shortcut() -> Shortcut {
    let conf = configs::get_data();
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
