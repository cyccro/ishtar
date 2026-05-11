use std::path::Path;

use wasmtime::{Config, Engine, Instance, Module, Store};

pub struct PluginId(usize);

pub struct PluginState {
    plugin_id: usize,
}

pub struct LoadedPlugin {
    id: PluginId,
    instance: Instance,
    store: Store<PluginState>,
    module: Module,
}

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
    pub fn create_store(&self) -> Store<PluginState>{
        Store::new(&self.engine, PluginState { plugin_id: self.plugins.len() })
    }

    pub fn load_plgins(&mut self, plugins_path: &Path) {
        let Ok(entries) = std::fs::read_dir(plugins_path) else {
            return;
        };
        for entry in entries {
            let Ok(path) = entry else {
                continue;
            };
            if matches!(path.path().extension(), Some(ext) if ext == "wasm") && let Ok(module) = Module::from_file(&self.engine, path.path()) {
                let mut store = self.create_store();
                let Ok(instance) = Instance::new(&mut store, &module, &[]) else {
                    continue;
                };
                let plugin = LoadedPlugin {
                    id: PluginId(self.plugins.len()),
                    instance,
                    store,
                    module
                };
                self.plugins.push(plugin);
            }
        }
    }
}
