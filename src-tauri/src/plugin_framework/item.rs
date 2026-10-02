use serde::Serialize;

use super::identity::ActionId;

/// 一个 Plugin Item：不透明类型名 + 纯数据字段
///
/// 句柄不在数据里，由框架的条目下标给出，插件不做二次存储（Q6/Q30）。
/// 不留插件私有数据位（Q10）。
///
/// 以 `the_type` 为 serde 内部 tag、字段与 tag 平铺，前端手写的镜像类型（`src/core.tsx`）
/// 与它对应（Q34）。
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "the_type")]
pub struct PluginItem {
    /// 排序用的优先级
    pub priority: i32,
    pub key_words: Vec<String>,
    pub name: String,
    pub desc: String,
    /// 类型名。框架对它的值**永远不解释**，只透传
    pub the_type: String,
    /// 条目自带的图标：**绝对路径**，空串表示没有
    ///
    /// 与"类型图标"不是一回事：类型图标一张画给同类型的每一行（前端注册表里按类型名查），
    /// 而这一张是**这个条目自己**的图片——`Plugin Package` 的清单里写的那一张就是它。
    /// 框架不碰文件、也不解释路径，只把它原样透传给前端（前端再交给 asset protocol）。
    pub icon: String,
}

impl PluginItem {
    pub fn new(
        the_type: impl Into<String>,
        priority: i32,
        key_words: Vec<String>,
        name: impl Into<String>,
        desc: impl Into<String>,
        icon: impl Into<String>,
    ) -> Self {
        Self {
            the_type: the_type.into(),
            priority,
            key_words,
            name: name.into(),
            desc: desc.into(),
            icon: icon.into(),
        }
    }
}

/// 一个 Plugin Action 的元数据
///
/// 类型轴在这里：同一个 `copy` 在不同类型上要走不同的 `label_key`，
/// 所以 `label_key` 必须按"类型 + 动作"分套（Q35）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PluginAction {
    /// 该动作挂在哪个类型上，值同上不透明
    pub the_type: String,
    pub id: ActionId,
    /// 前端据它查文案。框架只当不透明字符串透传，**不强制**命名规范
    pub label_key: String,
}

impl PluginAction {
    pub fn new(
        the_type: impl Into<String>,
        id: impl Into<String>,
        label_key: impl Into<String>,
    ) -> Self {
        Self {
            the_type: the_type.into(),
            id: ActionId(id.into()),
            label_key: label_key.into(),
        }
    }
}

/// 一个动作跑完的结果
///
/// [`Self::NoOp`] 与 [`Self::Failed`] 必须分开（Q13）：前者是"什么都没发生"，
/// 调用方据此决定要不要隐藏窗口。这一轮不加错误文案字段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionOutcome {
    /// 真的执行了
    Done,
    /// 没有动作可做，也不算失败（动作未实现、动作不挂在该条目上等）
    NoOp,
    /// 尝试执行但失败了
    Failed,
}
