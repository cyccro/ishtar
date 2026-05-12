use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use wasmtime::{Caller, Engine, Instance, Linker, Module, Store};

/// A keybind registration sent from a plugin via postcard-serialized bytes.
///
/// The plugin writes this into WASM linear memory and calls the host function
/// `register_keybind` with a pointer + length. The host deserializes it and
/// stores the mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindRegistration {
    /// Key sequence pattern, e.g. `"space+e"`.
    pub pattern: String,
    /// Opaque identifier the plugin uses to dispatch this binding in `handle_command`.
    pub callback_id: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct PluginId(usize);

impl PluginId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

pub struct PluginState {
    plugin_id: PluginId,
    keybinds: HashMap<String, u32>,
}

impl PluginState {
    pub fn new(id: PluginId) -> Self {
        Self {
            plugin_id: id,
            keybinds: HashMap::new(),
        }
    }

    pub fn keybinds(&self) -> &HashMap<String, u32> {
        &self.keybinds
    }
}

pub struct LoadedPlugin {
    id: PluginId,
    instance: Instance,
    store: Store<PluginState>,
    module: Module,
}

impl LoadedPlugin {
    pub fn new(id: PluginId, engine: &Engine, path: &Path) -> Result<Self, wasmtime::Error> {
        let module = Module::from_file(engine, path)?;
        let mut store = Store::new(engine, PluginState::new(id));

        let mut linker = Linker::<PluginState>::new(engine);

        // Host function: register_keybind(ptr: i32, len: i32) -> i32
        // Reads postcard-serialized KeybindRegistration from WASM memory.
        linker.func_wrap(
            "host",
            "register_keybind",
            |mut caller: Caller<'_, PluginState>, ptr: i32, len: i32| -> i32 {
                let memory = match caller.get_export("memory") {
                    Some(wasmtime::Extern::Memory(m)) => m,
                    _ => return -1,
                };
                let data = memory.data(&caller);
                let Some(bytes) = data
                    .get(ptr as usize..)
                    .and_then(|d| d.get(..len as usize))
                else {
                    return -1;
                };
                let reg: KeybindRegistration = match postcard::from_bytes(bytes) {
                    Ok(r) => r,
                    Err(_) => return -1,
                };
                caller
                    .data_mut()
                    .keybinds
                    .insert(reg.pattern, reg.callback_id);
                0
            },
        )?;

        let instance = linker.instantiate(&mut store, &module)?;

        Ok(Self {
            id,
            instance,
            store,
            module,
        })
    }

    pub fn execute_func<Args, Results>(
        &mut self,
        name: &'static str,
        args: Args,
    ) -> Result<Results, wasmtime::Error>
    where
        Args: wasmtime::WasmParams,
        Results: wasmtime::WasmResults,
    {
        let f = self
            .instance
            .get_typed_func::<Args, Results>(&mut self.store, name)?;
        f.call(&mut self.store, args)
    }

    pub fn keybinds(&self) -> &HashMap<String, u32> {
        self.store.data().keybinds()
    }

    pub fn plugin_id(&self) -> PluginId {
        self.id
    }
}
