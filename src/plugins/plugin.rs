use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use wasmtime::{Caller, Engine, Instance, Linker, Module, Store};

/// A command a plugin emits via the `execute` host function.
///
/// The plugin serialises a `Vec<PluginCmd>` with postcard and calls
/// `execute(ptr, len)`. The host deserialises and converts each to a
/// `CmdTask` for execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginCmd {
    /// No operation.
    Null,
    /// Close the editor.
    Exit,
    /// Insert text at the current cursor position.
    Write(String),
}

/// A keybind registration the plugin sends during `init()`.
///
/// The plugin serialises this with postcard and calls
/// `register_keybind(ptr, len)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindRegistration {
    /// Key sequence pattern, e.g. `"space+e"`. Joined with `+`.
    pub pattern: String,
    /// Opaque identifier passed back in `handle_command(callback_id)`.
    pub callback_id: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct PluginId(usize);

impl PluginId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

/// Per-plugin state stored inside the wasmtime `Store`.
///
/// Holds the keybind map populated by `register_keybind` calls from the
/// plugin, and the command buffer written by `execute` calls.
pub struct PluginState {
    plugin_id: PluginId,
    keybinds: HashMap<String, u32>,
    /// Commands written by the plugin during `handle_command`, consumed
    /// by the host after the call returns.
    pending_cmds: Vec<PluginCmd>,
}

impl PluginState {
    pub fn new(id: PluginId) -> Self {
        Self {
            plugin_id: id,
            keybinds: HashMap::new(),
            pending_cmds: Vec::new(),
        }
    }

    pub fn keybinds(&self) -> &HashMap<String, u32> {
        &self.keybinds
    }

    pub fn drain_pending(&mut self) -> Vec<PluginCmd> {
        std::mem::take(&mut self.pending_cmds)
    }
}

/// A single loaded WASM plugin instance.
///
/// Wraps a wasmtime `Instance` and `Store<PluginState>` and provides
/// typed access to the plugin's exported functions.
pub struct LoadedPlugin {
    id: PluginId,
    instance: Instance,
    store: Store<PluginState>,
    module: Module,
}

impl LoadedPlugin {
    /// Load a `.wasm` file and instantiate it with host function imports.
    ///
    /// Creates a linker registering the two host functions
    /// (`"host".register_keybind` and `"host".execute`), instantiates
    /// the module, and returns the loaded plugin ready for `init()`.
    pub fn new(id: PluginId, engine: &Engine, path: &Path) -> Result<Self, wasmtime::Error> {
        let module = Module::from_file(engine, path)?;
        let mut store = Store::new(engine, PluginState::new(id));

        let mut linker = Linker::<PluginState>::new(engine);

        // Host function: register_keybind(ptr, len) -> i32
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

        // Host function: execute(ptr, len) -> i32
        // Plugin calls this from inside handle_command to emit serialised PluginCmds.
        linker.func_wrap(
            "host",
            "execute",
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
                let cmds: Vec<PluginCmd> = match postcard::from_bytes(bytes) {
                    Ok(c) => c,
                    Err(_) => return -1,
                };
                caller.data_mut().pending_cmds = cmds;
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

    /// Call the plugin's exported `handle_command(callback_id)`.
    /// Returns the commands the plugin emitted via `host_execute`.
    pub fn handle_command(&mut self, callback_id: u32) -> Result<Vec<PluginCmd>, wasmtime::Error> {
        self.execute_func::<(u32,), ()>("handle_command", (callback_id,))?;
        Ok(self.store.data_mut().drain_pending())
    }

    pub fn keybinds(&self) -> &HashMap<String, u32> {
        self.store.data().keybinds()
    }

    pub fn plugin_id(&self) -> PluginId {
        self.id
    }
}
