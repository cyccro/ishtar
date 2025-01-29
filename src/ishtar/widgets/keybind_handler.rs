use std::collections::HashMap;

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use isht::ConfigStatment;

///A handler for keybinds. Starts listening when receiving a Control Key(Alt, Control, Shift, etc...) and stops when
///receiving an Enter key
pub struct KeybindHandler {
    pub initializer: KeyModifiers,
    pub listening: bool,
    pub buffer: Vec<String>,
    bindings: [HashMap<String, Vec<ConfigStatment>>; 3],
}
impl KeybindHandler {
    pub fn new(bindings: [HashMap<String, Vec<ConfigStatment>>; 3]) -> Self {
        Self {
            initializer: KeyModifiers::NONE,
            listening: false,
            buffer: Vec::new(),
            bindings,
        }
    }
    ///Sets the handler to start listening keys. Panics if called when already listening
    pub fn start_listening(&mut self, key: KeyCode, modifier: KeyModifiers) {
        assert!(!self.listening);
        assert!(self.initializer.is_empty());
        self.listening = true;
        self.initializer = modifier;
        self.buffer.push(modifier.to_string());
        self.buffer.push(key.to_string());
    }
    ///Stops listening keys and make start_listnening usable again
    pub fn stop_listening(&mut self) {
        self.buffer.clear();
        self.initializer = KeyModifiers::NONE;
        self.listening = false;
    }
    //Appends the given key into the keybind buffer if is listening
    pub fn handle(&mut self, key: KeyCode) {
        if self.listening {
            match key {
                KeyCode::Modifier(m) => self.buffer.push(m.to_string()),
                KeyCode::Char(c) => self.buffer.push(c.to_string()),
                k => self.buffer.push(k.to_string()),
            };
        }
    }
    pub fn listening(&self) -> bool {
        self.listening
    }
    pub fn get(&self, val: &String, mode: usize) -> Option<&Vec<ConfigStatment>> {
        self.bindings[mode].get(val)
    }
}
