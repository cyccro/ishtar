use std::{collections::HashMap, process::Command, sync::Arc};

use isht::CmdTask;
use ratatui::{
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Color, Style},
    text::ToLine,
    widgets::Widget,
};

use crate::{
    helpers::terminal_line::TerminalLine,
    ishtar::enums::{CmdResponse, IshtarMode},
};

use super::IshtarSelectable;
pub struct CommandInterpreter {
    line: TerminalLine,
    cursor: usize,
    builtins: HashMap<String, CmdTask>,
    colors: Arc<HashMap<String, u32>>,
}
impl CommandInterpreter {
    pub fn new(colors: Arc<HashMap<String, u32>>) -> Self {
        Self {
            line: TerminalLine::new(),
            cursor: 0,
            builtins: {
                let mut builtins = HashMap::new();
                builtins.insert(":l".into(), CmdTask::Exit);
                builtins.insert(":r".into(), CmdTask::Reset);
                builtins
            },
            colors,
        }
    }
    ///Gets the current position of the cursor
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
    ///Sets interpreter content to be the given content
    pub fn set(&mut self, content: &str) {
        self.line.clear();
        self.line.push_str_back(content);
    }
    fn check_for_unique(&self, c: char) -> CmdTask {
        match c {
            'm' => CmdTask::EnterModify,
            's' => CmdTask::EnterSelection,
            _ => CmdTask::Null,
        }
    }
    ///Writes the char into the interpreter and checks if the the written key is defined as
    ///shortcut, if so returns its task
    pub fn write(&mut self, c: char) -> CmdTask {
        if self.is_empty() {
            let task = self.check_for_unique(c);
            if !matches!(&task, CmdTask::Null) {
                return task;
            }
        }
        self.line.insert(self.cursor, c);
        self.cursor += 1;
        CmdTask::Null
    }
    pub fn backspace(&mut self) {
        if self.line.is_empty() {
            return;
        }
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        self.line.remove(self.cursor, true);
    }
    //Clears the interpreter content
    pub fn clear(&mut self) {
        self.line.clear();
        self.cursor = 0;
    }
    fn execute_internal(&mut self, target: &str) -> CmdTask {
        if let Some(builtin) = self.builtins.get(target) {
            return builtin.clone();
        }
        let mut result = CmdTask::Null;
        if let Some('!') = target.chars().nth(0) {
            for cmd in target[1..].split(';') {
                if let Err(e) = Command::new(cmd).spawn() {
                    self.set(&format!("{:?}", e));
                };
            }
            self.clear();
            return CmdTask::Null;
        }
        let broken_cmd = target.split(' ').collect::<Vec<_>>();
        if broken_cmd.len() > 1 {
        } else {
            match broken_cmd[0] {
                ":s" => result = CmdTask::SaveFile,
                _ => {}
            }
        }
        self.clear();
        result
    }
    pub fn execute_cmd(&mut self, cmd: &str) -> CmdTask {
        self.execute_internal(cmd)
    }
    ///Executes the command that was written into the interpreter
    pub fn execute(&mut self) -> CmdTask {
        /*if let Some(builtin) = self.builtins.get(&self.line.to_string()) {
            return builtin.clone();
        }
        let mut result = ;
        let string = self.to_string();
        if let Some('!') = string.chars().nth(0) {
            for cmd in string[1..].split(';') {
                if let Err(e) = Command::new(cmd).spawn() {
                    self.set(&format!("{:?}", e));
                };
            }
            self.clear();
            return CmdTask::Null;
        }
        let broken_cmd = string.split(' ').collect::<Vec<_>>();
        if broken_cmd.len() > 1 {
            match broken_cmd[0] {
                ":m" => result = CmdTask::ModifyFile Some(CmdResponse::ModifyFile(broken_cmd[1].to_string())),
                _ => {}
            }
        } else {
            match broken_cmd[0] {
                ":m" => result = Some(CmdResponse::ChangeMode(IshtarMode::Modify)),
                ":s" => result = Some(CmdResponse::SaveFile),
                ":q" => result = Some(CmdResponse::Exit),
                ":sl" => {
                    result = Some(CmdResponse::ChangeMode(
                        crate::ishtar::enums::IshtarMode::Selection,
                    ))
                }
                _ => {}
            }
        }
        self.clear();*/
        self.execute_internal(&self.line.to_string())
    }
    pub fn goto_end(&mut self) {
        self.cursor = self.line.len() - 1;
    }
    pub fn is_empty(&self) -> bool {
        self.line.len() == 0
    }
}
impl std::fmt::Display for CommandInterpreter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.line)
    }
}
impl Widget for &CommandInterpreter {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let line = self
            .line
            .to_line()
            .style(Style::default().fg(Color::from_u32(
                (*self.colors).get("cmd").cloned().unwrap_or(0xffffff),
            )));
        let rect = Rect {
            x: 0,
            y: area.height - 1,
            width: line.width() as u16,
            height: 1,
        };
        line.render(rect, buf)
    }
}
impl IshtarSelectable for CommandInterpreter {
    fn priority(&self) -> u8 {
        0
    }
    fn priority_static() -> u8
    where
        Self: Sized,
    {
        0
    }
    fn keydown(&mut self, key: ratatui::crossterm::event::KeyCode) -> isht::CmdTask {
        match key {
            KeyCode::Esc => self.clear(),
            KeyCode::Char(c) => return self.write(c),
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Enter => return self.execute(),
            KeyCode::Backspace => self.backspace(),
            _ => return CmdTask::Null,
        };
        CmdTask::Null
    }
    fn renderize(&mut self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        self.render(area, buf);
    }
}
