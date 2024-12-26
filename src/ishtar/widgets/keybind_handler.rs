use std::collections::HashMap;

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use isht::ConfigStatment;

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
    pub fn get_from_buffer(&self, mode: usize) -> Option<&Vec<ConfigStatment>> {
        let s = self.buffer.join("-");
        if let Some(cfgs) = self.bindings[mode].get(&s) {
            Some(cfgs)
        } else {
            None
        }
    }
    pub fn is_initializer(&self, modifier: KeyModifiers) -> bool {
        modifier == self.initializer && self.initializer != KeyModifiers::NONE
    }
    pub fn start_listening(&mut self, key: KeyCode, modifier: KeyModifiers) {
        assert!(!self.listening);
        assert!(self.initializer.is_empty());
        self.listening = true;
        self.initializer = modifier;
        self.buffer.push(modifier.to_string());
        self.buffer.push(key.to_string());
    }
    pub fn stop_listening(&mut self) {
        self.buffer.clear();
        self.initializer = KeyModifiers::NONE;
        self.listening = false;
    }
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
