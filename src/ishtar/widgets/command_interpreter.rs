use std::{collections::HashMap, process::Command};

use ratatui::{
    layout::Rect,
    text::{Line, ToLine},
    widgets::Widget,
};

use crate::{helpers::terminal_line::TerminalLine, ishtar::enums::CmdResponse};
pub struct CommandInterpreter {
    line: TerminalLine,
    cursor: usize,
    builtins: HashMap<String, CmdResponse>,
}

impl CommandInterpreter {
    pub fn new() -> Self {
        Self {
            line: TerminalLine::new(),
            cursor: 0,
            builtins: {
                let mut builtins = HashMap::new();
                builtins.insert(":l".into(), CmdResponse::Exit);
                builtins.insert(":r".into(), CmdResponse::Reset);
                builtins
            },
        }
    }
    pub fn cursor(&self) -> usize {
        self.cursor
    }
    pub fn line(&self) -> &TerminalLine {
        &self.line
    }
    pub fn move_right(&mut self) {
        if self.line.is_empty() {
            return;
        }
        self.cursor = (self.cursor + 1).min(self.line.len());
    }
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }
    pub fn set(&mut self, content: &str) {
        self.line.clear();
        self.line.push_str_back(content);
    }
    fn check_for_unique(&self, c: char) -> Option<CmdResponse> {
        match c {
            'm' => Some(CmdResponse::ChangeMode(
                crate::ishtar::enums::IshtarMode::Modify,
            )),
            _ => None,
        }
    }
    pub fn write(&mut self, c: char) -> Option<CmdResponse> {
        if self.is_empty() {
            if let Some(e) = self.check_for_unique(c) {
                return Some(e);
            };
        }
        self.line.insert(self.cursor, c);
        self.cursor += 1;
        None
    }
    pub fn backspace(&mut self) {
        if self.line.is_empty() {
            return;
        }
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        self.line.remove(self.cursor);
    }
    pub fn clear(&mut self) {
        self.line.clear();
        self.cursor = 0;
    }
    pub fn execute(&mut self) -> Option<CmdResponse> {
        if let Some(builtin) = self.builtins.get(&self.line.to_string()) {
            return Some(builtin.clone());
        }
        let mut result = None;
        let string = self.to_string();
        if let Some('!') = string.chars().nth(0) {
            for cmd in (&string[1..]).split(';') {
                if let Err(e) = Command::new(cmd).spawn() {
                    self.set(&format!("{:?}", e));
                };
            }
            self.clear();
            return None;
        }
        let broken_cmd = string.split(' ').collect::<Vec<_>>();
        if broken_cmd.len() > 1 {
            match broken_cmd[0] {
                ":m" => result = Some(CmdResponse::ModifyFile(broken_cmd[1].to_string())),
                _ => {}
            }
        } else {
            match broken_cmd[0] {
                ":s" => result = Some(CmdResponse::SaveFile),
                ":q" => result = Some(CmdResponse::Exit),
                _ => {}
            }
        }
        self.clear();
        result
    }
    pub fn goto_end(&mut self) {
        self.cursor = self.line.len() - 1;
    }
    pub fn is_empty(&self) -> bool {
        self.line.len() == 0
    }
}
impl ToLine for CommandInterpreter {
    fn to_line(&self) -> Line<'_> {
        Line::raw(self.line.to_string())
    }
}
impl std::fmt::Display for CommandInterpreter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.line.to_string())
    }
}
impl Widget for &CommandInterpreter {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let line = self.line.to_line();
        let rect = Rect {
            x: 0,
            y: area.height - 1,
            width: line.width() as u16,
            height: 1,
        };
        line.render(rect, buf)
    }
}
