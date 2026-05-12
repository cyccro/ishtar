mod plugin;
pub use plugin::*;
use std::path::Path;
use wasmtime::{Config, Engine, Store};

pub struct PluginManager {
    engine: Engine,
    plugins: Vec<LoadedPlugin>,
}
impl PluginManager {
    pub fn config() -> Config {
        let mut config = Config::new();
        config.shared_memory(true);
        config
    }

    pub fn new(path: &Path) -> Result<Self, wasmtime::Error> {
        let mut this = Self {
            engine: Engine::new(&Self::config())?,
            plugins: Vec::new(),
        };
        this.load_plgins(path);
        Ok(this)
    }

    ///Creates a store for a plugin
    pub fn create_store(&self) -> Store<PluginState> {
        Store::new(
            &self.engine,
            PluginState::new(PluginId::new(self.plugins.len())),
        )
    }

    pub fn load_plgins(&mut self, plugins_path: &Path) {
        let Ok(entries) = std::fs::read_dir(plugins_path) else {
            return;
        };
        for entry in entries {
            let Ok(path) = entry else {
                continue;
            };
            if matches!(path.path().extension(), Some(ext) if ext == "wasm")
                && let Ok(mut plugin) = LoadedPlugin::new(
                    PluginId::new(self.plugins.len()),
                    &self.engine,
                    &path.path(),
                )
                && let Ok(_) = plugin.execute_func::<_, ()>("init", ())
            {
                self.plugins.push(plugin);
            }
        }
    }
}
