//! launcher 插件：内置的第一个插件，支持 `sys` / `cmd` / `web` / `scan` 四种类型
//!
//! 三个子模块分工：
//!
//! - `persistence`：manifest 的一行长什么样、文件怎么读写（自己的常量、自己的解析）；
//! - `action`：动作表与动作派发；
//! - `init`：manifest → 框架条目。
//!
//! 对外只出一个 [`LauncherPlugin`]：接入时 `lib.rs` 唯一要碰的东西。
//! 三个子模块互相可见、对外私有。类型名是这里自己的枚举，框架只当不透明字符串透传。
//!
//! 本模块**已经接入应用**（接入前那份临时的 `allow(dead_code)` 已删除）：注册表由
//! `crate::plugin_host` 组装并托管，条目经 `plugin_*` 命令下发给前端。

// 三个子模块只在本模块内部协作，对外私有（Q37）：外面只该看见 [`LauncherPlugin`]
mod action;

mod init;

mod persistence;

use crate::plugin_framework::{
    ActionId, ActionOutcome, ItemHandle, ItemRegistrar, Plugin, PluginAction, PluginContext,
    PluginError, PluginId, PluginItem,
};

/// launcher 插件
///
/// 结构体无状态（Q12）：manifest 的读取结果由框架持有，缓存与句柄都不在这里。
/// 插件这一轮就是一张常量动作表加一个"读文件、推条目"的 `init`。
pub struct LauncherPlugin;

impl LauncherPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LauncherPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for LauncherPlugin {
    fn id(&self) -> PluginId {
        PluginId(action::LAUNCHER_PLUGIN_ID.to_string())
    }

    fn actions(&self) -> Vec<PluginAction> {
        action::actions()
    }

    /// 一次性推完全部条目（Q11）
    fn init(
        &self,
        cx: &dyn PluginContext,
        registrar: &mut dyn ItemRegistrar,
    ) -> Result<(), PluginError> {
        let app_data_dir = cx
            .app_data_dir()
            .map_err(|err| PluginError::Init(format!("resolve app data dir failed: {err}")))?;

        cx.log_info(&format!("{} load manifest start", action::LAUNCHER_PLUGIN_ID));

        let items = init::load_items(cx, &app_data_dir)?;

        cx.log_info(&format!(
            "{} load manifest end, items: {}",
            action::LAUNCHER_PLUGIN_ID,
            items.len()
        ));

        registrar.register(&self.id(), items);

        Ok(())
    }

    fn run_action(
        &self,
        cx: &dyn PluginContext,
        item: &PluginItem,
        handle: &ItemHandle,
        action_id: &ActionId,
    ) -> ActionOutcome {
        // 框架只把落在这个插件名下的条目交给它（见 PluginRegistry::run_action 的按块查找），
        // 这一条断言是那句话的运行时凭据
        debug_assert_eq!(&handle.plugin_id, &self.id());

        action::run(cx, item, action_id)
    }
}
