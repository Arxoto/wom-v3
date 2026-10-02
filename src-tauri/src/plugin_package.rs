//! `Plugin Folder` 的扫描：目录 → [`PluginPackage`] 列表
//!
//! 目录名**不是**身份，`manifest.json` 的 `id` 才是；目录名只用于定位文件（spec §1.2）。
//! 坏包**跳过并记 warn**：目录读不了、清单打不开、清单坏掉、`id` 重复，都只影响那一个包，
//! 应用照常启动——一个坏插件不该让启动器起不来（成功判据第 5 条）。
//!
//! 扫描在应用 `setup` 里同步跑完：条目与动作表因此**不依赖插件的代码能否成功执行**，
//! 这是"坏插件不影响主窗口"的最强形式（spec §1.3 末）。

pub mod manifest;

use std::{
    collections::HashSet,
    path::{Component, Path, PathBuf},
};

use tauri_plugin_log::log::warn;

use manifest::{PackageManifest, PackageType};

/// 一个落在磁盘上的 `Plugin Package`：一个目录 + 它清单里声明的内容
#[derive(Debug, Clone)]
pub struct PluginPackage {
    /// 包的目录，定位入口与图标时的根
    pub dir: PathBuf,
    /// 清单内容
    pub manifest: PackageManifest,
}

impl PluginPackage {
    /// 这个包的形态是不是前端插件（清单 `type` 是 `html`）
    pub fn is_html(&self) -> bool {
        self.manifest.the_type == PackageType::Html
    }

    /// JS 入口的绝对路径：`entry` 是包内相对路径
    pub fn entry_path(&self) -> PathBuf {
        self.dir.join(&self.manifest.entry)
    }

    /// HTML 页面的绝对路径：清单没写 `html` 时是 [`None`]
    pub fn html_path(&self) -> Option<PathBuf> {
        if self.manifest.html.is_empty() {
            return None;
        }

        Some(self.dir.join(&self.manifest.html))
    }

    /// 条目图标的绝对路径：清单没写 `icon` 时是 [`None`]
    ///
    /// 只拼路径，不检查文件在不在：图标缺一张不该让这个包作废，界面认不出就不画。
    pub fn icon_path(&self) -> Option<PathBuf> {
        if self.manifest.icon.is_empty() {
            return None;
        }

        Some(self.dir.join(&self.manifest.icon))
    }
}

/// 把一个**包内相对路径**解析成绝对路径
///
/// 路径来自插件的代码，是**不可信输入**：只认普通路径段与 `.`，绝对路径、`..` 与
/// 驱动器前缀一律拒绝，于是插件只能碰到自己包目录里的文件。
pub fn resolve_in_package(dir: &Path, relative: &str) -> Result<PathBuf, String> {
    let relative_path = Path::new(relative);
    if relative_path.as_os_str().is_empty() {
        return Err("empty package-relative path".to_string());
    }

    let mut path = dir.to_path_buf();
    for component in relative_path.components() {
        match component {
            Component::Normal(part) => path.push(part),
            // `./a.html` 是正常写法，值本身没有意义
            Component::CurDir => {}
            _ => return Err(format!("not a package-relative path: {relative}")),
        }
    }

    Ok(path)
}

/// 读一个包目录
///
/// 没有 `manifest.json`、文件读不了、清单坏掉，都是这个包的错误——
/// 返回的字符串是给日志用的 ASCII 诊断文本。
pub fn read_package(dir: &Path) -> Result<PluginPackage, String> {
    let path = dir.join(manifest::MANIFEST_FILE_NAME);

    let text = std::fs::read_to_string(&path)
        .map_err(|err| format!("read {} failed: {err}", path.display()))?;

    let manifest = manifest::parse(&text).map_err(|err| err.to_string())?;

    Ok(PluginPackage {
        dir: dir.to_path_buf(),
        manifest,
    })
}

/// 扫一个 `Plugin Folder`：列子目录、逐个读清单
///
/// 目录不存在**不算错误**：应用照常起来，只是没有插件可扫。结果按 `id` 字典序，
/// 与文件系统的返回顺序无关（`read_dir` 的顺序不保证，注册顺序也就不能跟着它走）。
pub fn scan(folder: &Path) -> Vec<PluginPackage> {
    let entries = match std::fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(err) => {
            warn!(
                "plugin folder cannot be read, no plugin package loaded: {} ({err})",
                folder.display()
            );
            return Vec::new();
        }
    };

    let mut packages: Vec<PluginPackage> = Vec::new();
    // 身份是 `id` 不是目录名：两个目录写同一个 `id` 时只认先扫到的那个
    let mut ids: HashSet<String> = HashSet::new();

    for entry in entries.flatten() {
        let dir = entry.path();

        if !dir.is_dir() {
            continue;
        }

        match read_package(&dir) {
            Ok(package) => {
                if !ids.insert(package.manifest.id.clone()) {
                    warn!(
                        "duplicate plugin id, skip package: {} ({})",
                        package.manifest.id,
                        dir.display()
                    );
                    continue;
                }
                packages.push(package);
            }
            Err(err) => warn!("skip plugin package {}: {err}", dir.display()),
        }
    }

    packages.sort_by(|a, b| a.manifest.id.cmp(&b.manifest.id));

    packages
}
