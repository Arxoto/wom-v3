//! `Plugin Package` 在框架里的公共一半：目录、清单缓存与"清单 → 一条条目"的换算
//!
//! JS 插件（`plugin_proxy_js`）与前端插件（`plugin_proxy_html`）共用这一段：两者都持有一个
//! 包目录、都在 `init` 里重读清单、都只注册一条宿主定类型的条目。
//!
//! 清单在 `init` 里重读，所以重扫（[`crate::plugin_host::reload_packages`]）能改到名字、
//! 关键字与动作：框架那边的 `reload_plugin` 就是重新 `init`（Q11），这条语义刚好够用。

use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use crate::{
    plugin_framework::{PluginId, PluginItem},
    plugin_package::{self, manifest::PackageManifest, PluginPackage},
};

/// 一个 `Plugin Package` 在框架里的公共一半
pub struct PackageProxy {
    /// 包的目录：清单每次 `init` 从这里重读
    dir: PathBuf,
    /// 当前生效的清单
    manifest: Mutex<PackageManifest>,
}

impl PackageProxy {
    /// `package` 是扫描时已经解析过的那一份：构造时就要有 `id`，框架注册前会先问它
    pub fn new(package: PluginPackage) -> Self {
        Self {
            dir: package.dir,
            manifest: Mutex::new(package.manifest),
        }
    }

    /// 包的目录：前端插件按它把清单里的页面拼成绝对路径
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// 当前清单的一份快照
    ///
    /// 一律取快照再干活：`id` / `actions` / `init` 都要读清单，
    /// 抱着锁去读文件就是自己和自己抢锁。
    pub fn snapshot(&self) -> PackageManifest {
        self.manifest
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .clone()
    }

    pub fn plugin_id(&self) -> PluginId {
        PluginId(self.snapshot().id)
    }

    /// 重新读一遍包目录，成功后换掉当前清单
    pub fn reload(&self) -> Result<PluginPackage, String> {
        let package = plugin_package::read_package(&self.dir)?;
        *self.manifest.lock().unwrap_or_else(|err| err.into_inner()) = package.manifest.clone();

        Ok(package)
    }

    /// 清单 → 一条条目：类型名由宿主定，其余字段照清单
    pub fn item(package: &PluginPackage, item_type: &str) -> PluginItem {
        PluginItem::new(
            item_type,
            // 插件条目没有"优先级链"可谈：一个包就这一条，0 是它唯一可能的排序
            0,
            package.manifest.keywords.clone(),
            package.manifest.name.clone(),
            package.manifest.desc.clone(),
            // 清单里的图片是**包内相对路径**：这里拼成绝对路径再交给前端，
            // 前端把它转成 asset URL（宿主不碰文件内容，只给路径）
            package
                .icon_path()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default(),
        )
    }
}
