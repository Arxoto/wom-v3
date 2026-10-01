//! manifest 的文件访问：定位、不存在时建空文件、逐行迭代

use std::{
    fs,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

use crate::plugin_impl_launcher::persistence::{
    constants::LAUNCHER_MANIFEST_FILE_NAME,
    launcher_source::{LauncherItemSource, LauncherParseError},
};

/// manifest 的完整路径：应用数据目录下的固定文件名（Q24）
pub fn manifest_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(LAUNCHER_MANIFEST_FILE_NAME)
}

/// manifest 不存在时创建父目录与一个空文件（Q45，复刻 `load.rs:51`）
///
/// "不存在"与"内容为空"不合并成一种状态：前者要落盘一个空文件，后者不该再写一次。
/// 两者对加载的结果相同，但前者让配置页有一个可写的落点。
pub fn ensure_manifest(path: &Path) -> io::Result<()> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
        fs::write(path, "")?;
    }

    Ok(())
}

/// 迭代器逐行给出的一种失败
///
/// 两类要分开（Q18）：坏行是**条目级**错误，由 launcher 自己消化；
/// 文件读不出来只报在这里。后者没有行号可报，不伪装成"这一行有错"。
#[derive(Debug)]
pub enum LauncherItemReadError {
    /// 这一行的数据坏了
    Parse(LauncherParseError),
    /// 这一行没读出来
    Io(io::Error),
}

impl std::fmt::Display for LauncherItemReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(err) => err.fmt(f),
            Self::Io(err) => write!(f, "read line failed: {err}"),
        }
    }
}

/// 逐行产出 [`LauncherItemSource`]：**惰性**，一行一个结果
///
/// 空行与坏行都如实产出错误，由调用方决定怎么处理（Q47）：
/// "跳过空行、记日志跳过坏行"是 launcher 的语义，框架不该替它命名。
pub struct LauncherItemSourceIter {
    reader: BufReader<fs::File>,
    line: String,
}

impl LauncherItemSourceIter {
    /// 打开 manifest 并从头开始读
    pub fn new(path: &Path) -> io::Result<Self> {
        let file = fs::File::open(path)?;

        Ok(Self {
            reader: BufReader::new(file),
            line: String::new(),
        })
    }
}

impl Iterator for LauncherItemSourceIter {
    type Item = Result<LauncherItemSource, LauncherItemReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.line.clear();

        match self.reader.read_line(&mut self.line) {
            Ok(0) => None,
            Ok(_) => {
                let line = self.line.trim_end();
                Some(
                    line.parse::<LauncherItemSource>()
                        .map_err(LauncherItemReadError::Parse),
                )
            }
            Err(err) => Some(Err(LauncherItemReadError::Io(err))),
        }
    }
}

/// 拿到一个逐行迭代器；manifest 不存在就先落一个空文件
pub fn item_source_iter(app_data_dir: &Path) -> io::Result<LauncherItemSourceIter> {
    let path = manifest_path(app_data_dir);
    ensure_manifest(&path)?;

    LauncherItemSourceIter::new(&path)
}
