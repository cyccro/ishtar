use serde::{Deserialize, Serialize};

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

extern "C" {
    fn register_keybind(ptr: *const u8, len: u32) -> i32;
}

/// Scratch buffer for serialization (max 4 KiB).
static mut BUF: [u8; 4096] = [0; 4096];

/// Called once on load. Registers `space+e` → callback 0.
#[no_mangle]
pub extern "C" fn init() -> i32 {
    let reg = KeybindRegistration {
        pattern: "space+e".into(),
        callback_id: 0,
    };
    let buf = unsafe { &mut BUF };
    let Ok(slice) = postcard::to_slice(&reg, buf) else {
        return -1;
    };
    let ret = unsafe { host_register_keybind(slice.as_ptr(), slice.len() as u32) };
    if ret != 0 {
        return -2;
    }
    0
}

/// Called when a registered keybind is triggered.
/// callback_id == 0 → emit Exit command.
#[no_mangle]
pub extern "C" fn handle_command(callback_id: u32) {
    if callback_id != 0 {
        return;
    }
    let cmds = vec![PluginCmd::Exit];
    let buf = unsafe { &mut BUF };
    if let Ok(slice) = postcard::to_slice(&cmds, buf) {}
}
