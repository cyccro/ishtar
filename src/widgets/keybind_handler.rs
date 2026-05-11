use std::{collections::HashMap, ops::Deref, sync::Arc};

use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};

use crate::widgets::CmdTask;

use super::IshtarSelectable;

/// Keybind map: `"MODIFIER-key"` string → list of tasks per mode index.
pub type Keybinds = HashMap<String, Vec<Vec<CmdTask>>>;

/// Handles modifier-initiated keybind sequences (e.g. `Ctrl-w`, `Alt-x`).
///
/// Starts listening when a key event carries a modifier and stops on `Enter` or `Esc`.
/// While listening, renders a small overlay showing the keys captured so far.
pub struct KeybindHandler {
    /// The modifier that started the current sequence.
    pub initializer: KeyModifiers,
    /// Whether the handler is currently capturing a sequence.
    pub listening: bool,
    /// Keys captured since listening started, stored as strings.
    pub buffer: Vec<String>,
    /// Index of the current editor mode, used to look up the right binding.
    pub current_mode: usize,
    /// Registered keybinds: sequence string → per-mode task lists.
    keybinds: Keybinds,
}

impl KeybindHandler {
    pub fn new(keybinds: Keybinds) -> Self {
        Self {
            initializer: KeyModifiers::NONE,
            listening: false,
            buffer: Vec::new(),
            current_mode: 0,
            keybinds,
        }
    }

    /// Returns `true` if the handler is currently capturing a key sequence.
    pub fn listening(&self) -> bool {
        self.listening
    }

    /// Begins capturing a key sequence initiated by `modifier` + `key`.
    ///
    /// # Panics
    /// Panics if called while already listening.
    pub fn start_listening(&mut self, key: KeyCode, modifier: KeyModifiers) {
        assert!(
            !self.listening,
            "start_listening called while already listening"
        );
        assert!(self.initializer.is_empty());
        self.listening = true;
        self.initializer = modifier;
        self.buffer.push(modifier.to_string());
        self.buffer.push(key.to_string());
    }

    /// Stops capturing and resets the buffer so a new sequence can begin.
    pub fn stop_listening(&mut self) {
        self.buffer.clear();
        self.initializer = KeyModifiers::NONE;
        self.listening = false;
    }

    /// Appends `key` to the buffer if currently listening.
    pub fn handle(&mut self, key: KeyCode) {
        if !self.listening {
            return;
        }
        let s = match key {
            KeyCode::Modifier(m) => m.to_string(),
            KeyCode::Char(c) => c.to_string(),
            k => k.to_string(),
        };
        self.buffer.push(s);
    }

    /// Returns the current buffer joined as a single `"MOD-key1-key2"` string.
    pub fn content(&self) -> String {
        self.buffer
            .join("-")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("")
    }

    /// Looks up the task list for `sequence` in the given `mode`.
    pub fn get(&self, sequence: &str, mode: usize) -> Option<&Vec<CmdTask>> {
        self.keybinds
            .get(sequence)
            .and_then(|modes| modes.get(mode))
    }
}

impl Widget for &KeybindHandler {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title_color = 0xffffff;
        let keys_color = 0xffffff;
        let border_color = 0xffffff;

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
        .alignment(Alignment::Center);

        paragraph.render(
            Rect {
                width: len + 2,
                height: 3,
                x: (area.width / 2) - len / 2,
                y: area.height / 8,
            },
            buf,
        );
    }
}

impl IshtarSelectable for KeybindHandler {
    fn can_render(&self) -> bool {
        self.listening
    }

    fn keydown(&mut self, key: KeyCode) -> CmdTask {
        match key {
            KeyCode::Enter => {
                let content = self.content();
                let task = self
                    .get(&content, self.current_mode)
                    .cloned()
                    .map(|tasks| CmdTask::Multi(tasks))
                    .unwrap_or(CmdTask::Null);
                self.stop_listening();
                // Always return to the saved mode after a sequence completes.
                CmdTask::Multi(vec![CmdTask::ReturnSavedMode, task])
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
