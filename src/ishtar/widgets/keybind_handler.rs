use std::collections::HashMap;

use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{block::Position, Block, Borders, Padding, Paragraph, Widget},
};

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
impl Widget for &KeybindHandler {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let content = self.buffer.join("-");
        let len: u16 = content.len() as u16 + 4;
        let paragraph = Paragraph::new(Text::styled(
            content,
            Style::default().fg(Color::from_u32(0x00519cf7)),
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .title("Handled Key")
                .title_style(Style::default().fg(Color::Yellow))
                .title_alignment(Alignment::Center),
        )
        .alignment(ratatui::layout::Alignment::Center);
        paragraph.render(
            Rect {
                width: len + 2, //16 == 12 + 4 == sizeof("Handled Keys") + 4
                height: 3,
                x: (area.width / 2) - len / 2,
                y: area.height / 8,
            },
            buf,
        );
    }
}
