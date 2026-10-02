//! manifest 的一行：[`LauncherItemSource`]
//!
//! 这是 launcher 持久化层的中间表示，也是它自己的公开面：`init` 只认它。
//! **手写 `FromStr` + `Display`**，不用 serde derive（Q46）——文件语法是既有用户的资产，
//! 派生宏会把它变成结构的影子，改一次结构就换一次语法。
//!
//! 一行是 `<类型> <-> <priority> <-> <key_words> <-> <name> [<-> <desc>]`：
//!
//! | 变体 | 字段数 | 说明 |
//! | --- | --- | --- |
//! | [`LauncherItemSource::Sys`] | **4** | 无 desc（Q46） |
//! | [`LauncherItemSource::Cmd`] / [`LauncherItemSource::Web`] | 5 | desc 是正文/链接 |
//! | [`LauncherItemSource::Scan`] | 5 | desc 是一行 JSON，priority 是逗号链 |
//!
//! `key_words` 以空格分隔，空项忽略（复刻 `split_key`）。
//! `desc` 里的 `<`、`>` **不做转义**（分隔符是 `<->`），照抄 `split_line` 的行为。

use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
    str::FromStr,
};

use crate::plugin_impl_launcher::persistence::parse_scan::LauncherScanConfig;

/// 行内的字段分隔符
///
/// 字段分隔符是既有用户的资产，不改（Q27）。
pub const SPLIT_LINE: &str = "<->";

/// 未配置 priority 时的默认值
pub const DEFAULT_PRIORITY: i32 = 0;

/// `key_words` 字段里的分隔符
pub const SPLIT_KEY: &str = " ";

/// 一行的字段数：Sys 只有 4 个（没有 desc），其余三种 5 个
const SYS_FIELD_COUNT: usize = 4;
const FULL_FIELD_COUNT: usize = 5;

/// manifest 一行的中间表示
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LauncherItemSource {
    /// 系统命令：每个命令的动作就是描述内容，所以第 5 个字段不必存在
    Sys {
        priority: i32,
        key_words: Vec<String>,
        name: String,
    },
    Cmd {
        priority: i32,
        key_words: Vec<String>,
        name: String,
        desc: String,
    },
    Web {
        priority: i32,
        key_words: Vec<String>,
        name: String,
        desc: String,
    },
    Scan {
        /// 每一递归层级的优先级：下标即层级，越靠后的层级缺省沿用前一个（链尾）
        priority_chain: Vec<i32>,
        key_words: Vec<String>,
        name: String,
        /// 扫描配置，来自 `desc` 里的一行 JSON
        config: LauncherScanConfig,
        /// 配置里写的扫描根路径，原样保存：`base` 非空时它要在 `Display` 里写回去
        path: PathBuf,
    },
}

/// 一行的解析失败
///
/// 这是**条目级**错误（Q18 的第二层）：launcher 自己消化它——记日志、跳过这一行——
/// 框架只看到最终的字符串。所以这个类型不进框架，框架也看不到"空行"与"字段不足"的区别。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LauncherParseError {
    /// 空行：调用方自己决定要不要当错误处理
    EmptyLine,
    /// 字段数不对（`sys` 要 4 个，其余要 5 个）
    ValueNotEnough(String),
    /// 认不出的类型名
    UnknownItemType(String),
    /// 某个字段本身解析不了（priority、priority 链、scan 的 desc JSON）
    FieldParseFailed(String),
}

impl Display for LauncherParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyLine => "empty line".fmt(f),
            Self::ValueNotEnough(msg) => write!(f, "not enough values: {msg}"),
            Self::UnknownItemType(the_type) => write!(f, "unknown item type: {the_type}"),
            Self::FieldParseFailed(msg) => write!(f, "parse field failed: {msg}"),
        }
    }
}

impl std::error::Error for LauncherParseError {}

// region: 类型名
//
// 类型名："sys" / "cmd" / "web" / "scan"。launcher 只支持这四种。

pub const ITEM_SYS: &str = "sys";
pub const ITEM_CMD: &str = "cmd";
pub const ITEM_WEB: &str = "web";
pub const ITEM_SCAN: &str = "scan";

/// launcher 支持的类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LauncherItemType {
    Sys,
    Cmd,
    Web,
    Scan,
}

impl LauncherItemType {
    /// 类型的字符串形式
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sys => ITEM_SYS,
            Self::Cmd => ITEM_CMD,
            Self::Web => ITEM_WEB,
            Self::Scan => ITEM_SCAN,
        }
    }
}

impl Display for LauncherItemType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl FromStr for LauncherItemType {
    type Err = LauncherParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ITEM_SYS => Ok(Self::Sys),
            ITEM_CMD => Ok(Self::Cmd),
            ITEM_WEB => Ok(Self::Web),
            ITEM_SCAN => Ok(Self::Scan),
            _ => Err(LauncherParseError::UnknownItemType(s.to_string())),
        }
    }
}

// endregion

impl FromStr for LauncherItemSource {
    type Err = LauncherParseError;

    /// 以行为单位解析：字段以 `<->` 分割
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        if line.trim().is_empty() {
            return Err(LauncherParseError::EmptyLine);
        }

        let mut values = split_line(line);
        // 至少要有类型与 priority 两个字段才谈得上后续
        if values.len() <= 1 {
            return Err(LauncherParseError::ValueNotEnough(
                "values must be at least 2 for item parsing".to_string(),
            ));
        }

        let the_type = LauncherItemType::from_str(&values[0])?;
        // `sys` 只有 4 个字段（Q46），其余三种是 5 个
        let field_count = match the_type {
            LauncherItemType::Sys => SYS_FIELD_COUNT,
            LauncherItemType::Cmd | LauncherItemType::Web | LauncherItemType::Scan => {
                FULL_FIELD_COUNT
            }
        };
        check_value_count(&values, field_count, the_type)?;

        // 类型之后字段的排布固定，取出时才需要区分变体
        let priority = std::mem::take(&mut values[1]);
        let key_words = std::mem::take(&mut values[2]);
        let name = std::mem::take(&mut values[3]);
        let desc = values.get(4).cloned().unwrap_or_default();

        match the_type {
            LauncherItemType::Sys => Ok(Self::Sys {
                priority: parse_priority(&priority)?,
                key_words: split_key(&key_words),
                name,
            }),
            LauncherItemType::Cmd => Ok(Self::Cmd {
                priority: parse_priority(&priority)?,
                key_words: split_key(&key_words),
                name,
                desc,
            }),
            LauncherItemType::Web => Ok(Self::Web {
                priority: parse_priority(&priority)?,
                key_words: split_key(&key_words),
                name,
                desc,
            }),
            LauncherItemType::Scan => {
                let config = LauncherScanConfig::parse(&desc)
                    .map_err(LauncherParseError::FieldParseFailed)?;

                Ok(Self::Scan {
                    priority_chain: parse_priority_chain(&priority)?,
                    key_words: split_key(&key_words),
                    name,
                    path: PathBuf::from(&config.path),
                    config,
                })
            }
        }
    }
}

impl Display for LauncherItemSource {
    /// 写回一行：字段顺序与字段数与 [`FromStr`] 对称
    ///
    /// 分隔符两侧各留一个空格（`sys <-> 1 <-> ...`）：解析时字段会 trim，
    /// 所以两种写法都读得进来，但写出去的那一行要跟既有 manifest 长得一样。
    ///
    /// 现在没有调用方（文件不落盘），但它是往返测试的前提，
    /// 也是将来配置页写回 manifest 必须有的东西。
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sys {
                priority,
                key_words,
                name,
            } => write!(
                f,
                "{ITEM_SYS} {SPLIT_LINE} {priority} {SPLIT_LINE} {} {SPLIT_LINE} {name}",
                join_key(key_words),
            ),
            Self::Cmd {
                priority,
                key_words,
                name,
                desc,
            } => write!(
                f,
                "{ITEM_CMD} {SPLIT_LINE} {priority} {SPLIT_LINE} {} {SPLIT_LINE} {name} {SPLIT_LINE} {desc}",
                join_key(key_words),
            ),
            Self::Web {
                priority,
                key_words,
                name,
                desc,
            } => write!(
                f,
                "{ITEM_WEB} {SPLIT_LINE} {priority} {SPLIT_LINE} {} {SPLIT_LINE} {name} {SPLIT_LINE} {desc}",
                join_key(key_words),
            ),
            Self::Scan {
                priority_chain,
                key_words,
                name,
                config,
                path,
            } => write!(
                f,
                "{ITEM_SCAN} {SPLIT_LINE} {} {SPLIT_LINE} {} {SPLIT_LINE} {name} {SPLIT_LINE} {}",
                join_priority_chain(priority_chain),
                join_key(key_words),
                config.to_json(path),
            ),
        }
    }
}

/// 按 `<->` 切一行并 trim 每个字段（复刻 `parse_core::split_line`）
fn split_line(line: &str) -> Vec<String> {
    line.split(SPLIT_LINE)
        .map(|s| s.trim().to_string())
        .collect()
}

/// 按空格切 `key_words`，空项忽略（复刻 `parse_core::split_key`）
fn split_key(key: &str) -> Vec<String> {
    key.split(SPLIT_KEY)
        .map(|k| k.trim())
        .filter(|k| !k.is_empty())
        .map(|v| v.to_string())
        .collect()
}

/// `key_words` 写回一行字段
fn join_key(key_words: &[String]) -> String {
    key_words.join(SPLIT_KEY)
}

/// `priority` 链写回一个字段
fn join_priority_chain(chain: &[i32]) -> String {
    chain
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<String>>()
        .join(",")
}

/// 解析单个 priority：空串取 [`DEFAULT_PRIORITY`]，认不出的值报错
fn parse_priority(raw: &str) -> Result<i32, LauncherParseError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(DEFAULT_PRIORITY);
    }

    raw.parse::<i32>()
        .map_err(|_| LauncherParseError::FieldParseFailed(format!("invalid priority: {raw}")))
}

/// 解析逐层 priority：以 `,` 分割，空项沿用前一个下标的值，首个缺省为 [`DEFAULT_PRIORITY`]
///
/// `1,2,,5` 这类写法是既有数据的资产（Q27）
pub fn parse_priority_chain(raw: &str) -> Result<Vec<i32>, LauncherParseError> {
    let mut chain: Vec<i32> = Vec::new();

    for part in raw.split(',') {
        let part = part.trim();
        let value = if part.is_empty() {
            chain.last().copied().unwrap_or(DEFAULT_PRIORITY)
        } else {
            part.parse::<i32>().map_err(|_| {
                LauncherParseError::FieldParseFailed(format!("invalid scan priority: {part}"))
            })?
        };

        chain.push(value);
    }

    Ok(chain)
}

/// 字段数必须与类型要求的一致
///
/// 多一个字段也算不符：`sys` 那第 5 个字段是 Q46 故意不认的（旧实现曾把它当成 desc），
/// 悄悄忽略它就等于把两个字段数混成一个。
fn check_value_count(
    values: &[String],
    count: usize,
    the_type: LauncherItemType,
) -> Result<(), LauncherParseError> {
    if values.len() != count {
        return Err(LauncherParseError::ValueNotEnough(format!(
            "type {} should have {} values but got {} for item parsing",
            the_type,
            count,
            values.len(),
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 把一行解析了再写回去，用于往返断言
    fn round_trip(line: &str) -> String {
        LauncherItemSource::from_str(line)
            .expect("should parse")
            .to_string()
    }

    // region: 四类型分派

    #[test]
    fn parse_dispatches_four_types() {
        assert!(matches!(
            LauncherItemSource::from_str("sys <-> 1 <-> a b <-> sys name").unwrap(),
            LauncherItemSource::Sys { .. }
        ));
        assert!(matches!(
            LauncherItemSource::from_str("cmd <-> 1 <-> a b <-> cmd name <-> cmd desc").unwrap(),
            LauncherItemSource::Cmd { .. }
        ));
        assert!(matches!(
            LauncherItemSource::from_str("web <-> 1 <-> a b <-> web name <-> https://a.b")
                .unwrap(),
            LauncherItemSource::Web { .. }
        ));
        assert!(matches!(
            LauncherItemSource::from_str(r#"scan <-> 1 <-> a b <-> scan name <-> {"path":"C:\\tmp"}"#)
                .unwrap(),
            LauncherItemSource::Scan { .. }
        ));
    }

    #[test]
    fn parse_field_values() {
        let source =
            LauncherItemSource::from_str("cmd <-> 7 <-> a   b  <->  cmd name <->  cmd desc  ")
                .unwrap();

        let LauncherItemSource::Cmd {
            priority,
            key_words,
            name,
            desc,
        } = source
        else {
            panic!("should be cmd");
        };

        assert_eq!(priority, 7);
        assert_eq!(key_words, vec!["a", "b"]);
        assert_eq!(name, "cmd name");
        assert_eq!(desc, "cmd desc");
    }

    #[test]
    fn parse_unknown_type_and_empty_line() {
        assert_eq!(
            LauncherItemSource::from_str("note <-> 1 <-> a <-> b <-> c"),
            Err(LauncherParseError::UnknownItemType("note".to_string()))
        );
        assert_eq!(
            LauncherItemSource::from_str("   "),
            Err(LauncherParseError::EmptyLine)
        );
    }

    // endregion

    // region: 字段数

    #[test]
    fn sys_needs_exactly_four_fields() {
        // 4 个字段是对的：sys 没有 desc
        assert!(LauncherItemSource::from_str("sys <-> 1 <-> a <-> b").is_ok());

        // 少一个
        assert!(matches!(
            LauncherItemSource::from_str("sys <-> 1 <-> a"),
            Err(LauncherParseError::ValueNotEnough(_))
        ));

        // 现存的 sys 行按旧实现是 5 个字段，新实现不认（Q46 的直接后果，见接入清单）
        assert!(matches!(
            LauncherItemSource::from_str("sys <-> 1 <-> a <-> b <-> c"),
            Err(LauncherParseError::ValueNotEnough(_))
        ));
    }

    #[test]
    fn other_types_need_exactly_five_fields() {
        for line in [
            "cmd <-> 1 <-> a <-> b",
            "web <-> 1 <-> a <-> b",
            r#"scan <-> 1 <-> a <-> b <-> {"path":"C:\\tmp"} <-> extra"#,
        ] {
            assert!(
                matches!(
                    LauncherItemSource::from_str(line),
                    Err(LauncherParseError::ValueNotEnough(_))
                ),
                "should reject: {line}"
            );
        }
    }

    // endregion

    // region: priority 链

    #[test]
    fn parse_priority_chain_fills_empty_items() {
        assert_eq!(parse_priority_chain("1,2,,5").unwrap(), vec![1, 2, 2, 5]);
        assert_eq!(parse_priority_chain(",,").unwrap(), vec![0, 0, 0]);
        assert_eq!(parse_priority_chain("").unwrap(), vec![0]);
        assert_eq!(parse_priority_chain(" 3 ,  ,4").unwrap(), vec![3, 3, 4]);
        assert!(matches!(
            parse_priority_chain("1,x"),
            Err(LauncherParseError::FieldParseFailed(_))
        ));
    }

    #[test]
    fn scan_keeps_its_priority_chain() {
        let source = LauncherItemSource::from_str(
            r#"scan <-> 1,2,,5 <-> k <-> name <-> {"max_depth":2,"path":"C:\\tmp"}"#,
        )
        .unwrap();

        let LauncherItemSource::Scan {
            priority_chain,
            config,
            path,
            ..
        } = source
        else {
            panic!("should be scan");
        };

        assert_eq!(priority_chain, vec![1, 2, 2, 5]);
        assert_eq!(config.max_depth, 2);
        assert_eq!(path, PathBuf::from(r"C:\tmp"));
    }

    // endregion

    // region: 往返

    #[test]
    fn round_trip_every_type() {
        for line in [
            "sys <-> 1 <-> a b <-> sys name",
            "cmd <-> 2 <-> a b <-> cmd name <-> cmd desc",
            "web <-> 3 <-> a b <-> web name <-> https://example.com/a?b=c&d=e",
            r#"scan <-> 1,2 <-> a b <-> scan name <-> {"file_types":["File"],"file_suffix":[".rs"],"black_list":["target"],"max_depth":3,"base":"$HOME","path":"src"}"#,
        ] {
            assert_eq!(round_trip(line), line);
            // 幂等：写回去的那一行再解析再写回，还是同一行
            assert_eq!(round_trip(&round_trip(line)), line);
        }
    }

    #[test]
    fn round_trip_scan_normalizes_the_priority_chain() {
        // 链里的空项在解析时就被摊平，所以写回去的链是等价的展开形式
        let line = r#"scan <-> 1,2,,5 <-> k <-> name <-> {"path":"C:\\tmp"}"#;
        let written = round_trip(line);

        assert!(written.starts_with("scan <-> 1,2,2,5 <-> k <-> name <-> "));
        assert_eq!(round_trip(&written), written);

        let source = LauncherItemSource::from_str(line).unwrap();
        let rewritten = LauncherItemSource::from_str(&written).unwrap();
        assert_eq!(source, rewritten);
    }

    #[test]
    fn round_trip_keeps_angle_brackets_in_desc() {
        // desc 里的 `<`、`>` 不做转义，也不参与分割
        let line = "cmd <-> 1 <-> a <-> echo <hello> <-> body";
        let written = round_trip(line);

        assert_eq!(written, line);
        assert_eq!(round_trip(&written), written);

        let LauncherItemSource::Cmd { name, desc, .. } = LauncherItemSource::from_str(line).unwrap()
        else {
            panic!("should be cmd");
        };
        assert_eq!(name, "echo <hello>");
        assert_eq!(desc, "body");
    }

    #[test]
    fn desc_can_not_contain_the_line_separator() {
        // `<->` 就是行内的字段分隔符，desc 里出现它就多出字段——这是既有语法的既有事实，
        // 不是新实现的选择（Q27 明确不改文件语法）
        let source = LauncherItemSource::from_str("cmd <-> 1 <-> a <-> echo <-> echo <-> ok");

        assert!(matches!(
            source,
            Err(LauncherParseError::ValueNotEnough(_))
        ));
    }

    #[test]
    fn round_trip_escapes_json_strings() {
        use crate::plugin_impl_launcher::persistence::parse_scan::LauncherScanConfig;

        // JSON 里的引号与反斜杠由 serde_json 处理：写回去的仍然是合法 JSON，
        // 且再解析一次得到同一个 scan 配置
        let config = LauncherScanConfig {
            path: r#"C:\a"b"#.to_string(),
            ..LauncherScanConfig::default()
        };
        let desc = serde_json::to_string(&config).expect("should stringify");

        let line = format!("scan <-> 1 <-> k <-> name <-> {desc}");
        let written = round_trip(&line);

        assert_eq!(round_trip(&written), written);
        assert_eq!(
            LauncherItemSource::from_str(&line).unwrap(),
            LauncherItemSource::from_str(&written).unwrap()
        );
    }

    // endregion
}
