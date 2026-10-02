use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// 插件标识：稳定 ASCII 字符串
///
/// 它随 [`ItemHandle`] 一起下发到前端，所以两端都认这一个形状（`Serialize` / `Deserialize`）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PluginId(pub String);

impl Display for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// 动作标识，只在插件自己的动作表里有意义
///
/// 下发形状就是那个字符串本身（`transparent`）：前端拿到的动作名与注册时写的一字不差
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct ActionId(pub String);

impl ActionId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for ActionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// 条目身份：插件 + 插件内的注册序号
///
/// 由框架分配（Q30），插件不自己造。框架与插件之间用它寻址条目，
/// 与"条目在加载出的整集里的下标"不是一回事。
///
/// **它随条目一起下发到前端**（Q23）：`Plugin Search Result` 不在框架持有的那一集里，
/// 用 `item_index` 寻址对它不成立，所以两层列表的寻址统一到 handle 上——
/// 管你在第几层，动作派发都能找到那一行。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ItemHandle {
    pub plugin_id: PluginId,
    /// 该插件内**从 0 递增的注册序号**；结果行则是它在最近一次搜索结果里的下标
    pub local_id: usize,
}
