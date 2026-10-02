use std::fmt::Display;

/// 框架级错误（Q18）
///
/// 只有 init 一类。条目级错误是插件私有的，框架只 `to_string()`，不进这个类型——
/// 框架因此看不到"空行"与"字段不足"的区别，那是插件的语义（Q19）。
#[derive(Debug)]
pub struct PluginError(String);

impl PluginError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }

    /// 诊断用的一行 ASCII 文本
    pub fn message(&self) -> &str {
        &self.0
    }
}

impl Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "plugin init error: {}", self.message())
    }
}

impl std::error::Error for PluginError {}
