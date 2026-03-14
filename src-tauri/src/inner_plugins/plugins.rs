use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

use crate::inner_plugins::generator::{self, ItemsStat};

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new(generator::INNER_PLUGIN_NAME)
        .setup(|app, _api| {
            let items = generator::load_settings(app)?;
            app.manage(ItemsStat::new(items));
            Ok(())
        })
        .build()
}
