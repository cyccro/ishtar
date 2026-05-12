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

    /// Creates a store for a plugin.
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
                && let Ok(_) = plugin.execute_func::<(), ()>("init", ())
            {
                self.plugins.push(plugin);
            }
        }
    }

    /// Returns `true` if any loaded plugin registered a keybind that starts with `prefix`.
    pub fn has_prefix(&self, prefix: &str) -> bool {
        self.plugins
            .iter()
            .any(|p| p.keybinds().keys().any(|k| k.starts_with(prefix)))
    }

    /// Tries to dispatch a keybind sequence to the owning plugin.
    /// Returns the commands the plugin emitted, or `None` if no match.
    pub fn dispatch(&mut self, sequence: &str) -> Option<Vec<PluginCmd>> {
        let idx = self
            .plugins
            .iter()
            .position(|p| p.keybinds().contains_key(sequence))?;
        let callback_id = *self.plugins[idx].keybinds().get(sequence)?;
        self.plugins[idx]
            .handle_command(callback_id)
            .ok()
    }
}
