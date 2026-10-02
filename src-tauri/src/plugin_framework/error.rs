use std::fmt::Display;

/// 框架级错误（Q18）
///
/// 只有 init 与 action 两类。条目级错误是插件私有的，框架只 `to_string()`，不进这个枚举——
/// 框架因此看不到"空行"与"字段不足"的区别，那是插件的语义（Q19）。
#[derive(Debug)]
pub enum PluginError {
    /// 插件 init 失败
    Init(String),
    /// 插件 action 失败
    ///
    /// 这一轮没有构造点：跑动作只回 [`ActionOutcome`] 三态（Q13 明确此时不加错误文案字段），
    /// 失败归因留在插件自己的日志里。变体先留着——Q18 的三层错误表要求 init 与 action 分开，
    /// 等动作真的要把失败原因交回框架时（例如前端插件）就在这里构造。
    #[allow(dead_code)]
    Action(String),
}

impl PluginError {
    pub fn kind(&self) -> &'static str {
        match self {
            PluginError::Init(_) => "init",
            PluginError::Action(_) => "action",
        }
    }

    /// 诊断用的一行 ASCII 文本
    pub fn message(&self) -> &str {
        match self {
            PluginError::Init(msg) | PluginError::Action(msg) => msg,
        }
    }
}

impl Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "plugin {} error: {}", self.kind(), self.message())
    }
}

impl std::error::Error for PluginError {}
