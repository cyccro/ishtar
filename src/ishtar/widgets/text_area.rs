use std::{
    ffi::OsStr,
    fmt::Debug,
    io::Read,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use ratatui::{
    prelude::Rect,
    text::Line,
    widgets::{Paragraph, Widget},
};
use tree_sitter::{Range, Tree, TreeCursor};
use tree_sitter_highlight::Highlighter;
use unicode_normalization::char::compose;

use crate::{helpers::terminal_line::TerminalLine, ishtar::logger::IshtarLogger};

use super::highlighter::TextAreaHighlighter;

pub struct TextArea {
    lines: Vec<TerminalLine>,
    pos_x: u16,
    pos_y: u16,
    x: usize,
    y: usize,
    w: u16,
    h: u16,
    punctuator: Option<char>,
    editing_file: Option<PathBuf>,
    timer: Instant,
    logger: IshtarLogger,
    highlighter: TextAreaHighlighter,
}

impl TextArea {
    pub fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Self {
            punctuator: None,
            pos_x: x,
            pos_y: y,
            x: 0,
            y: 0,
            w,
            h,
            lines: vec![TerminalLine::new()],
            editing_file: None,
            timer: Instant::now(),
            highlighter: TextAreaHighlighter::new(),
            logger: IshtarLogger::new(true, true).unwrap(),
        }
    }
    fn can_highlight(&self) -> bool {
        self.timer.elapsed() >= Duration::from_secs(1)
    }
    pub fn area(&self) -> Rect {
        Rect {
            x: self.pos_x,
            y: self.pos_y,
            width: self.w,
            height: self.h,
        }
    }
    pub fn line(&self) -> &TerminalLine {
        &self.lines[self.x]
    }
    pub fn file_position(&self) -> (u16, u16) {
        (0, self.h)
    }
    pub fn cursor_x(&self) -> usize {
        self.x % self.w as usize
    }
    pub fn cursor_y(&self) -> usize {
        self.y % (self.h - 1) as usize
    }
    pub fn posx(&self) -> u16 {
        self.pos_x
    }
    pub fn posy(&self) -> u16 {
        self.pos_y
    }
    pub fn set_posx(&mut self, x: u16) -> u16 {
        let old = self.pos_x;
        self.pos_x = x;
        old
    }
    pub fn set_posy(&mut self, y: u16) -> u16 {
        let old = self.pos_y;
        self.pos_y = y;
        old
    }
    pub fn set_w(&mut self, w: u16) -> u16 {
        let old = self.w;
        self.w = w;
        old
    }
    pub fn set_h(&mut self, h: u16) -> u16 {
        let old = self.h;
        self.h = h;
        old
    }
    pub fn x(&self) -> usize {
        self.x
    }
    pub fn y(&self) -> usize {
        self.y
    }
    pub fn w(&self) -> u16 {
        self.w
    }
    pub fn h(&self) -> u16 {
        self.h
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
        let h = (self.h - 1) as usize;
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
            self.highlighter.highlight(buf, &mut vec);
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
        let Some(line) = self.lines.get_mut(self.y) else {
            return;
        };
        if c.is_ascii_punctuation() {
            if let Some(c) = self.punctuator {
                self.punctuator = None;
                line.insert(self.x, c);
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
        if self.x > 0 {
            if let Some(line) = self.lines.get_mut(self.y as usize) {
                self.x -= 1;
                line.remove(self.x);
            }
        } else {
            if self.y > 0 {
                let mut line = self.lines.remove(self.y);
                self.y -= 1;
                self.x = self.lines[self.y].len();
                self.lines[self.y].append_line(&mut line);
            }
        }
    }
    pub fn newline(&mut self) {
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
            //5 + (-8) = -8 < 0 && 5 < --8, as typeof(y) = usize, set to 0
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
}
impl ToString for TextArea {
    fn to_string(&self) -> String {
        let mut buffer = String::new();
        for line in &self.lines {
            buffer.push_str(&line.to_string());
            buffer.push('\n')
        }
        buffer
    }
}
impl Widget for &mut TextArea {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let w = self.w as usize;
        let lines: Vec<Line> = self
            .visible_lines()
            .iter()
            .map(|(idx, content)| {
                let sidx = idx.to_string();
                let pos = w - sidx.len() - 1;
                format!(
                    "{sidx} {}",
                    if self.x > pos {
                        &content[pos * self.x / pos - 1..]
                    } else {
                        content
                    }
                )
                .into()
            })
            .collect();
        Paragraph::new(lines).render(area, buf);
        let file_name: Line = if let Some(name) = self.file_name() {
            name.to_string_lossy().into()
        } else {
            "Not a File".into()
        };
        let len = file_name.width();
        Paragraph::new(file_name).render(
            Rect {
                x: self.posx(),
                y: self.h - 1,
                width: len as u16,
                height: 1,
            },
            buf,
        )
    }
}
