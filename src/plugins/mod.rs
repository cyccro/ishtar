mod plugin;
pub use plugin::*;
use std::path::{Path, PathBuf};
use wasmtime::{Config, Engine, Store};

/// Manages all loaded WASM plugins.
///
/// Scans a directory for `*.wasm` files, instantiates each one with
/// wasmtime, calls `init()` to let plugins register keybinds, and
/// dispatches keybind sequences to the owning plugin.
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

    pub fn new() -> Result<Self, wasmtime::Error> {
        let this = Self {
            engine: Engine::new(&Self::config())?,
            plugins: Vec::new(),
        };
        Ok(this)
    }

    /// Creates a store for a plugin.
    pub fn create_store(&self) -> Store<PluginState> {
        Store::new(
            &self.engine,
            PluginState::new(PluginId::new(self.plugins.len())),
        )
    }

    pub fn load_plgins(&mut self, plugins_path: &Path) -> Vec<(wasmtime::Error, PathBuf)> {
        let Ok(entries) = std::fs::read_dir(plugins_path) else {
            return vec![];
        };
        let mut out = Vec::new();
        for entry in entries {
            let Ok(path) = entry else {
                continue;
            };
            if matches!(path.path().extension(), Some(ext) if ext == "wasm") {
                let mut plugin = match LoadedPlugin::new(
                    PluginId::new(self.plugins.len()),
                    &self.engine,
                    &path.path(),
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        out.push((e, path.path()));
                        continue;
                    }
                };
                match plugin.execute_func::<(), ()>("init", ()) {
                    Ok(v) => v,
                    Err(e) => {
                        out.push((e, path.path()));
                        continue;
                    }
                }
                self.plugins.push(plugin);
            }
        }
        out
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
        self.plugins[idx].handle_command(callback_id).ok()
    }
}
