use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use isht::CmdTask;
use ratatui::{
    prelude::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};
use unicode_normalization::char::compose;

use crate::helpers::{min_max, terminal_line::TerminalLine, Vec2};

use super::clipboard::IshtarClipboard;
#[derive(Eq, PartialEq)]
pub enum TextAreaMode {
    Writing,
    Selecting,
}

pub struct TextArea {
    lines: Vec<TerminalLine>,
    position: Vec2,
    size: Vec2,
    selection_cursor: Vec2,
    x: usize,
    y: usize,
    punctuator: Option<char>,
    editing_file: Option<PathBuf>,
    mode: TextAreaMode,
}

impl TextArea {
    pub fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Self {
            punctuator: None,
            position: Vec2::new(x, y),
            size: Vec2::new(w, h),
            selection_cursor: Vec2::new(0, 0),
            x: 0,
            y: 0,
            lines: vec![TerminalLine::new()],
            editing_file: None,
            mode: TextAreaMode::Writing,
        }
    }
    pub fn enter_selection(&mut self) {
        self.mode = TextAreaMode::Selecting;
        *self.selection_cursor.x_mut() = self.x as u16;
        *self.selection_cursor.y_mut() = self.y as u16;
    }
    pub fn enter_writing(&mut self) {
        self.mode = TextAreaMode::Writing;
    }
    pub fn area(&self) -> Rect {
        Rect {
            x: self.position.x(),
            y: self.position.x(),
            width: self.size.x(),
            height: self.size.y(),
        }
    }
    pub fn line(&self) -> &TerminalLine {
        &self.lines[self.y]
    }
    pub fn file_position(&self) -> (u16, u16) {
        (0, self.size.y())
    }
    pub fn cursor_x(&self) -> usize {
        self.x % self.size.x() as usize
    }
    pub fn cursor_y(&self) -> usize {
        self.y % (self.size.y() - 1) as usize
    }
    pub fn posx(&self) -> u16 {
        self.position.x()
    }
    pub fn posy(&self) -> u16 {
        self.position.y()
    }
    pub fn set_posx(&mut self, x: u16) -> u16 {
        let old = self.posx();
        *self.position.x_mut() = x;
        old
    }
    pub fn set_posy(&mut self, y: u16) -> u16 {
        let old = self.posy();
        *self.position.y_mut() = y;
        old
    }
    pub fn set_w(&mut self, w: u16) -> u16 {
        let old = self.size.x();
        *self.size.x_mut() = w;
        old
    }
    pub fn set_h(&mut self, h: u16) -> u16 {
        let old = self.size.y();
        *self.size.y_mut() = h;
        old
    }
    pub fn x(&self) -> usize {
        self.x
    }
    pub fn y(&self) -> usize {
        self.y
    }
    pub fn w(&self) -> u16 {
        self.size.x()
    }
    pub fn h(&self) -> u16 {
        self.size.y()
    }
    pub fn xoffset(&self) -> usize {
        1 + self.y.to_string().len()
    }
    pub fn content(&self, idx: usize) -> Option<&TerminalLine> {
        self.lines.get(idx)
    }
    pub fn clear_content(&mut self) {
        self.lines.clear();
    }
    pub fn set_content(&mut self, content: String) -> usize {
        self.clear_content();
        let lines = content.split('\n');
        let mut idx = 0;
        for line in lines {
            self.lines.push(TerminalLine::from_str(line));
            idx += 1;
        }
        idx
    }
    pub fn visible_lines(&mut self) -> Vec<(usize, String)> {
        let mut vec = Vec::new();
        let h = (self.h() - 1) as usize;
        let page = self.y / h; //no need for recalc everytime
        let bounds = {
            let dif = h * page;
            (dif, dif + h) //init and finish of the 'page'; -1 due to the last be reserved to
                           //the file name
        };
        let mut idx = bounds.0;
        if true {
            let mut buf = String::new();
            while idx < bounds.1 {
                let Some(line) = self.lines.get(idx) else {
                    break;
                };
                let str = line.to_string();
                buf.push_str(&str);
                buf.push('\n');
                vec.push((idx, str));
                idx += 1;
            }
        } else {
            while idx < bounds.1 {
                let Some(line) = self.lines.get(idx) else {
                    break;
                };
                vec.push((idx, line.to_string()));
                idx += 1;
            }
        }
        vec
    }
    pub fn write_char(&mut self, c: char) {
        if self.mode == TextAreaMode::Selecting {
            return;
        }
        let Some(line) = self.lines.get_mut(self.y) else {
            return;
        };
        if c.is_ascii_punctuation() {
            if let Some(c) = self.punctuator {
                line.insert(self.x, c);
                self.punctuator = None;
                self.x += 1;
            }
            line.insert(self.x, c);
            self.x += 1;
            return;
        }
        if let Some(punc) = self.punctuator {
            if let Some(c) = compose(c, punc) {
                line.insert(self.x, c);
                self.punctuator = None;
                self.x += 1;
                return;
            };
            line.insert(self.x, punc);
            self.x += 1;
            line.insert(self.x, c);
            self.punctuator = None;
            self.x += 1;
        } else {
            line.insert(self.x, c);
            self.x += 1;
        }
    }
    pub fn backspace(&mut self) {
        if self.mode == TextAreaMode::Selecting {
            return;
        }
        if self.x > 0 {
            if let Some(line) = self.lines.get_mut(self.y) {
                self.x -= 1;
                line.remove(self.x);
            }
        } else if self.y > 0 {
            let mut line = self.lines.remove(self.y);
            self.y -= 1;
            self.x = self.lines[self.y].len();
            self.lines[self.y].append_line(&mut line);
        }
    }
    pub fn del(&mut self) {
        if self.mode == TextAreaMode::Selecting {
            return;
        }
        if self.line().is_empty() && self.lines.len() > 1 {
            self.lines.remove(self.y);
        } else if self.x == self.line().len() && self.y < self.lines.len() - 1 {
            let mut line = self.lines.remove(self.y + 1);
            self.lines[self.y].append_line(&mut line);
            return;
        } else {
            let x = self.x();
            self.lines[self.y].remove(x);
        }
    }
    pub fn newline(&mut self) {
        if self.mode == TextAreaMode::Selecting {
            return;
        }
        let line = if let Some(line) = self.lines.get_mut(self.y) {
            if line.is_empty() || self.x > line.len() {
                self.x = 0;
                TerminalLine::new()
            } else {
                let rest = line.split_off(self.x);
                self.x = 0;
                rest
            }
        } else {
            TerminalLine::new()
        };
        self.y += 1;
        if self.y > self.lines.len() {
            self.lines.push(line);
        } else {
            self.lines.insert(self.y, line);
        }
    }
    pub fn goto_init_of_line(&mut self) {
        self.x = 0;
    }
    pub fn goto_end_of_line(&mut self) {
        self.x = self.lines[self.y].len();
    }
    pub fn goto_init_of_file(&mut self) {
        self.x = 0;
        self.y = 0;
    }
    pub fn goto_end_of_file(&mut self) {
        self.y = self.lines.len() - 1;
        self.x = self.lines[self.y].len();
    }
    pub fn move_y(&mut self, n: i32) {
        let abs = n.abs() as usize;

        if n < 0 && self.y < abs {
            //n > y, y - n < 0, usize::min = 0.
            self.y = 0;
            return;
        }
        if n < 0 {
            self.y -= abs;
        } else {
            self.y += abs
        };
        self.y = self.y.min(self.lines.len() - 1);
        self.x = self.x.min(self.lines[self.y].len());
    }
    pub fn move_x(&mut self, n: i32) {
        if n < 0 && self.x < -n as usize {
            //5 + (-8) = -8 < 0 && 5 < --8, as typeof(y) = usize, set to 0
            self.x = 0;
            return;
        }
        if n < 0 {
            self.x -= -n as usize
        } else {
            self.x += n as usize
        };
        self.x = self.x.min(self.lines[self.y].len());
    }
    pub fn reset(&mut self) {
        self.editing_file = None;
        self.lines.clear();
    }
    pub fn open_file(&mut self, path: PathBuf) {
        let mut fpath = std::env::current_dir().unwrap();
        fpath.push(&path);
        if let Ok(content) = std::fs::read_to_string(fpath) {
            self.set_content(content);
        };
        self.editing_file = Some(path);
    }
    pub fn modify_file_name(&mut self, new_name: &str) {
        if let Some(ref file) = self.editing_file {
            self.editing_file = Some(file.with_file_name(new_name));
        }
    }
    pub fn modify_file_extension(&mut self, new_name: &str) {
        if let Some(ref file) = self.editing_file {
            self.editing_file = Some(file.with_extension(new_name));
        };
    }
    pub fn file_extension(&mut self) -> Option<&OsStr> {
        if let Some(ref file) = self.editing_file {
            file.extension()
        } else {
            None
        }
    }
    pub fn close_file(&mut self) {
        self.editing_file = None;
    }
    pub fn file_name(&self) -> Option<&OsStr> {
        if let Some(ref file) = self.editing_file {
            file.file_name()
        } else {
            None
        }
    }
    pub fn save(&self, prefix: &Path) -> std::io::Result<()> {
        if let Some(ref file) = self.editing_file {
            let prefixed = prefix.join(file);
            std::fs::write(&prefixed, self.to_string())
        } else {
            Ok(())
        }
    }
    pub fn paste(&mut self, content: &str) -> CmdTask {
        let lines = content.split('\n').collect::<Vec<&str>>();
        if lines.len() == 1 {
            for (idx, c) in lines[0].chars().enumerate() {
                self.lines[self.y].insert(self.x + idx, c);
            }
            return CmdTask::EnterModify;
        }
        for (idx, line) in content.lines().enumerate() {
            let curr_idx = self.y + idx;
            if curr_idx > self.lines.len() {
                self.lines.push(TerminalLine::from_str(line));
            } else {
                self.lines.insert(curr_idx, TerminalLine::from_str(line));
            }
        }
        CmdTask::EnterModify
    }
    pub fn delete_line(&mut self) {
        if self.lines.len() == 1 {
            self.lines[0].clear();
            return;
        }
        self.lines.remove(self.y);
        self.y = self.y.min(self.lines.len().saturating_sub(1));
        self.x = self.x.min(self.lines[self.y].len().saturating_sub(1));
    }
    pub fn copy_line(&self, clipboard: &mut IshtarClipboard, is_virtual: bool) -> CmdTask {
        if !self.is_selecting() {
            return CmdTask::EnterModify;
        }
        if is_virtual {
            clipboard.set_virtual(self.lines[self.y].to_string());
        } else {
            clipboard.set(self.lines[self.y].to_string());
        }
        CmdTask::EnterModify
    }
    pub fn copy_buffer(&self, clipboard: &mut IshtarClipboard, is_virtual: bool) -> CmdTask {
        if !self.is_selecting() {
            return CmdTask::EnterModify;
        }
        if is_virtual {
            clipboard.set_virtual(self.to_string());
        } else {
            clipboard.set(self.to_string());
        }
        CmdTask::EnterModify
    }
    pub fn copy_selection(&mut self, clipboard: &mut IshtarClipboard, is_virtual: bool) -> CmdTask {
        if !self.is_selecting() {
            return CmdTask::EnterModify;
        }
        let (mut min, max) = min_max(self.y, self.selection_cursor.y() as usize);
        let mut buffer = String::new();
        loop {
            if min > max {
                break;
            }
            buffer.push_str(&self.lines[min].to_string());
            buffer.push('\n');
            min += 1;
        }
        if is_virtual {
            clipboard.set_virtual(buffer);
        } else {
            clipboard.set(buffer);
        }
        CmdTask::EnterModify
    }
    fn is_in_selection_bounds(&self, idx: usize) -> bool {
        if self.selection_cursor.y() > self.y as u16 {
            //cursor moving up
            idx >= self.y && idx <= self.selection_cursor.y() as usize
        } else {
            //moving down
            idx <= self.y && idx >= self.selection_cursor.y() as usize
        }
    }
    pub fn is_selecting(&self) -> bool {
        matches!(self.mode, TextAreaMode::Selecting)
    }
}
impl std::fmt::Display for TextArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = String::new();
        for line in &self.lines {
            buffer.push_str(&line.to_string());
            buffer.push('\n')
        }
        buffer.pop();
        write!(f, "{buffer}")
    }
}
impl Widget for &mut TextArea {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let w = self.size.x() as usize;
        let lines: Vec<Line> = if self.is_selecting() {
            self.visible_lines()
                .into_iter()
                .map(|(idx, content)| {
                    let sidx = idx.to_string();
                    let pos = w - sidx.len() - 1;
                    if self.is_in_selection_bounds(idx) {
                        if idx != self.y {
                            let line_content = if self.x > pos {
                                &content[self.x - 1..]
                            } else {
                                &content
                            }
                            .to_string();
                            Line::from(vec![
                                Span::from(format!("{sidx} ")),
                                Span::styled(line_content, Style::default().bg(Color::Red)),
                            ])
                        } else {
                            let (min, max) = min_max(self.selection_cursor.x() as usize, self.x);
                            let min = min.min(self.lines[self.y].len().saturating_sub(1));
                            let max = max.min(self.lines[self.y].len().saturating_sub(1));
                            let begin = Span::from(content[..min].to_string());
                            let selected = Span::styled(
                                content[min..max].to_string(),
                                Style::default().bg(Color::Red),
                            );
                            let finish = if self.x > pos {
                                Span::from(content[max - 1..].to_string())
                            } else {
                                Span::from(content[max..].to_string())
                            };
                            Line::from(vec![sidx.into(), " ".into(), begin, selected, finish])
                        }
                    } else {
                        format!(
                            "{sidx} {}",
                            if self.x > pos {
                                &content[self.x - 1..]
                            } else {
                                &content
                            }
                        )
                        .into()
                    }
                })
                .collect()
        } else {
            self.visible_lines()
                .iter()
                .map(|(idx, content)| {
                    let sidx = idx.to_string();
                    let pos = w - sidx.len() - 1;

                    format!(
                        "{sidx} {}",
                        if self.x > pos {
                            &content[self.x - 1..]
                        } else {
                            content
                        }
                    )
                    .into()
                })
                .collect()
        };
        Paragraph::new(lines).render(self.area(), buf);
        let file_name: Line = if let Some(name) = self.file_name() {
            name.to_string_lossy().into()
        } else {
            "Not a File".into()
        };
        let len = file_name.width();
        Paragraph::new(file_name).render(
            Rect {
                width: len as u16,
                x: self.posx(),
                y: self.h() + self.posy() - 1,
                height: 1,
            },
            buf,
        )
    }
}
