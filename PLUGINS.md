# Plugin System

Plugins are **WebAssembly modules** loaded from `./plugin/` at startup. Each plugin runs in a sandboxed wasmtime instance and communicates with the editor through two host functions and two exported functions.

## Architecture

```
┌─────────────────────────────────────────────────┐
│                   Editor (host)                  │
│  ┌──────────────────────────────────────────┐   │
│  │            PluginManager                  │   │
│  │  ┌──────────┐  ┌──────────┐              │   │
│  │  │ plugin 0 │  │ plugin 1 │  ...         │   │
│  │  │ .wasm    │  │ .wasm    │              │   │
│  │  └────┬─────┘  └──────────┘              │   │
│  └───────┼──────────────────────────────────┘   │
│          │ wasmtime sandbox                      │
│          │                                       │
│  ┌───────┴──────────────────────────────────┐   │
│  │         key event loop                    │   │
│  │  space → leader mode → collect keys       │   │
│  │  match "space+e" → plugin.handle_command()│   │
│  │  plugin.execute() → parse → CmdTask       │   │
│  └───────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
         ▲ host functions          │ exports
         │ (imported by WASM)      │ (called by host)
         │                         ▼
┌─────────────────────────────────────────────────┐
│               WASM Plugin (.wasm)                │
│  init()          → register_keybind()            │
│  handle_command() → execute()                    │
│  memory          ← linear memory for data        │
└─────────────────────────────────────────────────┘
```

## Host Functions (Plugin → Editor)

The editor provides these functions, imported by the plugin from module `"host"`:

### `register_keybind(ptr, len) -> i32`

Called during `init()` to register a keybind pattern.

| Param | Type | Description |
|-------|------|-------------|
| `ptr` | `*const u8` | Pointer to postcard-encoded `KeybindRegistration` in linear memory |
| `len` | `u32` | Number of bytes to read |
| return | `i32` | `0` on success, `-1` on failure |

**KeybindRegistration** (postcard-serialized):
```rust
struct KeybindRegistration {
    pattern: String,   // e.g. "space+e", "ctrl+p"
    callback_id: u32,  // opaque ID passed back in handle_command
}
```

### `execute(ptr, len) -> i32`

Called from inside `handle_command()` to emit commands back to the editor.

| Param | Type | Description |
|-------|------|-------------|
| `ptr` | `*const u8` | Pointer to postcard-encoded `Vec<PluginCmd>` in linear memory |
| `len` | `u32` | Number of bytes to read |
| return | `i32` | `0` on success, `-1` on failure |

**PluginCmd** variants (postcard-serialized):
```rust
enum PluginCmd {
    Null,             // no-op
    Exit,             // close the editor
    Write(String),    // insert text at cursor
}
```

## Plugin Exports (Editor → Plugin)

Every `.wasm` plugin must export these:

### `init()`

Called once after loading. The plugin should call `register_keybind()` for each keybind it wants to handle.

```rust
// Rust
#[no_mangle]
pub extern "C" fn init() {
    let reg = KeybindRegistration { pattern: "space+e".into(), callback_id: 0 };
    // serialize with postcard, call register_keybind
}
```

### `handle_command(callback_id: u32)`

Called when a registered keybind pattern is matched. The plugin should call `execute()` with the commands to run.

```rust
#[no_mangle]
pub extern "C" fn handle_command(callback_id: u32) {
    if callback_id != 0 { return; }
    let cmds = vec![PluginCmd::Write("hello ".into())];
    // serialize with postcard, call execute
}
```

### `memory`

The plugin's linear memory. The host reads from and writes to this memory to exchange serialized data.

```rust
// Rust automatically exports memory when using extern "C" exports
```

## Keybind Sequences (Space Leader)

Plugin keybinds use a **space leader** pattern, similar to Emacs or Helix:

1. Press `<space>` — enters leader mode (only works outside Insert mode)
2. Press the rest of the sequence (e.g. `e`)
3. If the full sequence matches a plugin registration, the plugin's `handle_command` fires
4. If the sequence doesn't match any prefix, leader mode cancels automatically

Sequences are joined with `+`, so pressing <kbd>space</kbd> then <kbd>e</kbd> produces the pattern `"space+e"`.

## Writing a Plugin

### 1. Create a Rust crate

```
test-plugin/
├── Cargo.toml
└── src/
    └── lib.rs
```

### 2. `Cargo.toml`

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1", features = ["derive"] }
postcard = "1.1.3"
```

### 3. `src/lib.rs`

```rust
use serde::Serialize;

// ── Must match the host definitions exactly ──

#[derive(Serialize)]
struct KeybindRegistration {
    pattern: String,
    callback_id: u32,
}

#[derive(Serialize)]
enum PluginCmd {
    Null,
    Exit,
    Write(String),
}

// ── Import host functions ──

#[link(wasm_import_module = "host")]
extern "C" {
    fn register_keybind(ptr: *const u8, len: u32) -> i32;
    fn execute(ptr: *const u8, len: u32) -> i32;
}

// ── Scratch buffer (max 4 KiB) ──

static mut BUF: [u8; 4096] = [0; 4096];

// ── Required exports ──

#[no_mangle]
pub extern "C" fn init() {
    let reg = KeybindRegistration {
        pattern: "space+e".into(),
        callback_id: 0,
    };
    unsafe {
        let buf: &mut [u8] = &mut BUF;
        let slice = postcard::to_slice(&reg, buf).unwrap();
        register_keybind(slice.as_ptr(), slice.len() as u32);
    }
}

#[no_mangle]
pub extern "C" fn handle_command(callback_id: u32) {
    if callback_id != 0 { return; }
    let cmds = vec![PluginCmd::Exit];
    unsafe {
        let buf: &mut [u8] = &mut BUF;
        let slice = postcard::to_slice(&cmds, buf).unwrap();
        execute(slice.as_ptr(), slice.len() as u32);
    }
}
```

### 4. Compile

```sh
cd test-plugin
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/my_plugin.wasm ../plugin/
```

Or use the Makefile:

```sh
make -C test-plugin install
```

## Plugin Lifecycle

```
Editor starts
  ↓
PluginManager::new()                         ← creates wasmtime Engine
  ↓
load_plugins("./plugin/")                    ← scans *.wasm files
  ↓
LoadedPlugin::new(id, engine, path)
  ├── Module::from_file                     ← parses .wasm
  ├── Linker::new(engine)                   ← registers host functions
  │   ├── "host".register_keybind           ← plugin calls this
  │   └── "host".execute                    ← plugin calls this
  └── linker.instantiate                    ← creates instance + linear memory
  ↓
execute_func::<(), ()>("init", ())          ← calls plugin's init()
  └── plugin calls register_keybind          ← stores pattern → callback_id mapping
  ↓
plugin added to PluginManager.plugins[]
  ↓
─── User presses keys ───
  ↓
handle_plugin_keybind(key)                   ← in event loop
  ├── space → start leader mode
  │   plugin_seq = Some(["space"])
  ├── e → append, join → "space+e"
  │   PluginManager::dispatch("space+e")
  │   ├── find plugin owning "space+e"
  │   ├── plugin.handle_command(callback_id)
  │   │   └── plugin.execute(cmds_bytes)
  │   │       └── host stores in pending_cmds
  │   └── return Vec<PluginCmd>
  ├── convert PluginCmd → CmdTask
  └── execute CmdTask (write text, exit, etc.)
```

## Adding New PluginCmd Variants

To add a new command that plugins can return:

1. Add a variant to `PluginCmd` in `src/plugins/plugin.rs` (both `Serialize` and `Deserialize`)
2. Add the conversion in `Ishtar::plugin_cmd_to_task()` in `src/ishtar/mod.rs`
3. Update the plugin-side enum to match
4. Handle the new `CmdTask` variant in `handle_task()` in `src/ishtar/tasks.rs`

## Files Reference

| File | Role |
|------|------|
| `src/plugins/mod.rs` | `PluginManager` — loads, queries, dispatches plugins |
| `src/plugins/plugin.rs` | `LoadedPlugin`, `PluginState`, `PluginCmd`, `KeybindRegistration`, host functions |
| `src/ishtar/mod.rs` | `handle_plugin_keybind()`, `plugin_cmd_to_task()`, `key_to_string()` |
| `test-plugin/` | Example plugin crate |
| `plugin/` | Directory scanned for `.wasm` files at startup |
| `Makefile` | `make plugin` to rebuild and install plugins |
