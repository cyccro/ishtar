use std::{collections::HashMap, ops::Deref, sync::Arc};

use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};

use super::IshtarSelectable;
use isht::{CmdTask, ConfigStatment};

///A handler for keybinds. Starts listening when receiving a Control Key(Alt, Control, Shift, etc...) and stops when
///receiving an Enter key
pub struct KeybindHandler {
    pub initializer: KeyModifiers,
    pub listening: bool,
    pub buffer: Vec<String>,
    pub current_mode: usize,
    bindings: [HashMap<String, Vec<ConfigStatment>>; 3],
    colors: Arc<HashMap<String, u32>>,
}

impl KeybindHandler {
    pub fn new(
        bindings: [HashMap<String, Vec<ConfigStatment>>; 3],
        colors: Arc<HashMap<String, u32>>,
    ) -> Self {
        Self {
            initializer: KeyModifiers::NONE,
            listening: false,
            buffer: Vec::new(),
            bindings,
            current_mode: 0,
            colors,
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

    pub fn content(&self) -> String {
        self.buffer
            .join("-")
            .split(' ')
            .collect::<Vec<&str>>()
            .join("")
            .to_string()
    }
}

impl Widget for &KeybindHandler {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let colors = self.colors.deref();
        let title_color = colors
            .get("handled_keys_title")
            .cloned()
            .unwrap_or(0xffffff);
        let keys_color = colors
            .get("handled_keys_color")
            .cloned()
            .unwrap_or(0xffffff);
        let border_color = colors
            .get("handled_keys_border")
            .cloned()
            .unwrap_or(0xffffff);
        let content = self.buffer.join("-");
        let len: u16 = content.len() as u16 + 4;
        let paragraph = Paragraph::new(Text::styled(
            content,
            Style::default().fg(Color::from_u32(keys_color)),
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::from_u32(border_color)))
                .title("Handled Key")
                .title_style(Style::default().fg(Color::from_u32(title_color)))
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

impl IshtarSelectable for KeybindHandler {
    fn priority(&self) -> u8 {
        2
    }

    fn priority_static() -> u8
    where
        Self: Sized,
    {
        2
    }

    fn can_render(&self) -> bool {
        self.listening
    }

    fn keydown(&mut self, key: KeyCode) -> isht::CmdTask {
        match key {
            KeyCode::Enter => {
                let content = self.content();
                if let Some(mut data) = self.get(&content, self.current_mode).cloned() {
                    let mut multi = Vec::new();
                    let mut idx = 0;
                    while let Some(statment) = data.get(idx).cloned() {
                        match statment {
                            ConfigStatment::Task(t) => multi.push(t),
                            ConfigStatment::Cmd(s) => {
                                if let Some(ref mut d) = self.get(&s, self.current_mode).cloned() {
                                    data.append(d);
                                }
                            }
                            _ => {}
                        }
                        idx += 1;
                    }
                    self.stop_listening();
                    multi.push(CmdTask::ReturnSavedMode);
                    return CmdTask::Multi(multi);
                };
                self.stop_listening();
                CmdTask::ReturnSavedMode
            }
            KeyCode::Esc => {
                self.stop_listening();
                CmdTask::ReturnSavedMode
            }
            key => {
                self.handle(key);
                CmdTask::Null
            }
        }
    }

    fn renderize(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(self, area);
    }
}
