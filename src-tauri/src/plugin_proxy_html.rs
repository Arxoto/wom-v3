use std::{path::PathBuf, sync::Mutex};

use tauri::AppHandle;

use crate::{
    plugin_framework::{
        ActionId, ActionOutcome, ItemHandle, ItemRegistrar, Plugin, PluginAction, PluginContext,
        PluginError, PluginId, PluginItem,
    },
    plugin_package::{self, manifest::PackageManifest, PluginPackage},
};

pub const HTML_PLUGIN_ITEM_TYPE: &str = "html_plugin";

pub const ACTION_OPEN_HTML: &str = "open_html";

pub const OPEN_HTML_LABEL_KEY: &str = "action.html_host.html_plugin.open_html";

pub struct HtmlPlugin {
    app: AppHandle,
    dir: PathBuf,
    manifest: Mutex<PackageManifest>,
}

impl HtmlPlugin {
    pub fn new(app: AppHandle, package: PluginPackage) -> Self {
        Self {
            app,
            dir: package.dir,
            manifest: Mutex::new(package.manifest),
        }
    }

    fn manifest_snapshot(&self) -> PackageManifest {
        self.manifest
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .clone()
    }
}

impl Plugin for HtmlPlugin {
    fn id(&self) -> PluginId {
        PluginId(self.manifest_snapshot().id)
    }

    fn actions(&self) -> Vec<PluginAction> {
        vec![PluginAction::new(
            HTML_PLUGIN_ITEM_TYPE,
            ACTION_OPEN_HTML,
            OPEN_HTML_LABEL_KEY,
        )]
    }

    fn init(
        &self,
        cx: &dyn PluginContext,
        registrar: &mut dyn ItemRegistrar,
    ) -> Result<(), PluginError> {
        let package = plugin_package::read_package(&self.dir)
            .map_err(|err| PluginError::Init(format!("read html plugin manifest failed: {err}")))?;

        if !package.is_html() {
            return Err(PluginError::Init("manifest type is not html".to_string()));
        }

        let plugin_id = PluginId(package.manifest.id.clone());

        let item = PluginItem::new(
            HTML_PLUGIN_ITEM_TYPE,
            0,
            package.manifest.keywords.clone(),
            package.manifest.name.clone(),
            package.manifest.desc.clone(),
            package
                .icon_path()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default(),
        );

        registrar.register(&plugin_id, vec![item]);

        *self.manifest.lock().unwrap_or_else(|err| err.into_inner()) = package.manifest;

        cx.log_info(&format!("html plugin ready: {plugin_id}"));

        Ok(())
    }

    fn run_action(
        &self,
        cx: &dyn PluginContext,
        _item: &PluginItem,
        handle: &ItemHandle,
        _action_id: &ActionId,
    ) -> ActionOutcome {
        let manifest = self.manifest_snapshot();

        match crate::plugin_host::open_html_window(
            &self.app,
            &handle.plugin_id.0,
            &manifest.html,
            "",
            0.0,
            0.0,
        ) {
            Ok(()) => ActionOutcome::Done,
            Err(err) => {
                cx.log_warn(&format!("html plugin open page failed: {err}"));
                ActionOutcome::Failed
            }
        }
    }
}
