use serde::Serialize;

/// Must match the host's `KeybindRegistration` exactly.
#[derive(Serialize)]
struct KeybindRegistration {
    pattern: String,
    callback_id: u32,
}

/// Must match the host's `PluginCmd` exactly.
#[derive(Serialize)]
enum PluginCmd {
    Null,
    Exit,
    Write(String),
}

#[link(wasm_import_module = "host")]
extern "C" {
    fn register_keybind(ptr: *const u8, len: u32) -> i32;
    fn execute(ptr: *const u8, len: u32) -> i32;
}

/// Scratch buffer for serialization (max 4 KiB).
static mut BUF: [u8; 4096] = [0; 4096];

/// Called once on load. Registers `space+e` → callback 0.
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

/// Called when a registered keybind is triggered.
/// callback_id == 0 → write "plum! " then quit.
#[no_mangle]
pub extern "C" fn handle_command(callback_id: u32) {
    if callback_id != 0 {
        return;
    }
    let cmds = vec![PluginCmd::Write("plum! ".into())];
    unsafe {
        let buf: &mut [u8] = &mut BUF;
        let slice = postcard::to_slice(&cmds, buf).unwrap();
        execute(slice.as_ptr(), slice.len() as u32);
    }
}
