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

use crate::helpers::{
    char_size_backwards, char_size_init, min_max, terminal_line::TerminalLine, Vec2,
};

use super::clipboard::IshtarClipboard;

#[derive(Eq, PartialEq)]
pub enum TextAreaMode {
    Writing,
    Selecting,
}
///Writing buffer
pub struct TextArea {
    lines: Vec<TerminalLine>,
    position: Vec2,
    size: Vec2,
    selection_cursor: Vec2,
    x: usize,                 //cursorx
    y: usize,                 //cursory
    byte_offsets: Vec<usize>, //will be used for getting the offset received from multibyte chars and so,
    //for aligning the cursor
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
            byte_offsets: vec![0],
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
    ///Gets the area bounds of this writer
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
    ///Gets the X position of the cursor inside the bounds of the size
    #[inline]
    pub fn cursor_x(&self) -> usize {
        (self.x % self.size.x() as usize).saturating_sub(self.byte_offsets[self.y])
    }
    ///Gets the Y position of the cursor inside the bounds of the size
    #[inline]
    pub fn cursor_y(&self) -> usize {
        self.y % (self.size.y() - 1) as usize
    }
    ///Gets the position x of the area(left corner)
    #[inline]
    pub fn posx(&self) -> u16 {
        self.position.x()
    }
    ///Gets the y position of the area(top corner)
    #[inline]
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
    ///Gets the X position of the cursor without checking bounds
    #[inline]
    pub fn x(&self) -> usize {
        self.x
    }
    ///Gets the X position of the cursor without checking bounds
    #[inline]
    pub fn y(&self) -> usize {
        self.y
    }
    pub fn w(&self) -> u16 {
        self.size.x()
    }
    pub fn h(&self) -> u16 {
        self.size.y()
    }
    ///Gets X offset from the left due to line number
    pub fn xoffset(&self) -> usize {
        1 + self.y.to_string().len()
    }
    ///Gets the content of the line at the given index
    pub fn content(&self, idx: usize) -> Option<&TerminalLine> {
        self.lines.get(idx)
    }
    ///Clears the contents of the writer
    pub fn clear_content(&mut self) {
        self.lines.clear();
        self.lines.push(TerminalLine::new());
    }
    ////Sets the content of the writer to be the string. Splits all its lines and set them into
    ///each line and returns the amount of lines set
    pub fn set_content(&mut self, content: String) -> usize {
        self.lines.clear();
        let mut idx = 0;
        for line in content.lines() {
            self.lines.push(TerminalLine::from_str(line));
            idx += 1;
        }
        idx
    }
    //Gets all the visible lines on the current buffer.
    pub fn visible_lines(&mut self) -> Vec<(usize, String)> {
        let h = (self.h() - 1) as usize;
        let page = self.y / h; //no need for recalc everytime
        let mut bounds = {
            let dif = h * page;
            (dif, (dif + h).min(self.lines.len())) //init and finish of the 'page'; -1 due to the last be reserved to
                                                   //the file name
        };
        let mut vec = Vec::with_capacity(bounds.1 - bounds.0);
        for line in self.lines[bounds.0..bounds.1].iter() {
            vec.push((bounds.0, line.to_string()));
            bounds.0 += 1;
        }
        vec
    }
    ///Writes the given char and moves the cursor
    fn move_after_insert(&mut self, c: char) {
        self.lines[self.y].insert(self.x, c);
        if c.len_utf8() > 1 {
            self.byte_offsets[self.y] += c.len_utf8() - 1;
            self.x += c.len_utf8();
        } else {
            self.x += 1;
        }
    }
    ///Writes the given char checking for punctuators
    pub fn write_char(&mut self, c: char) {
        if self.mode == TextAreaMode::Selecting {
            return;
        }
        if c.is_ascii_punctuation() {
            if let Some(c) = self.punctuator {
                self.move_after_insert(c);
                self.punctuator = None;
            }
            self.move_after_insert(c);
            return;
        }
        if let Some(punc) = self.punctuator {
            if let Some(c) = compose(c, punc) {
                self.move_after_insert(c);
                self.punctuator = None;
                return;
            };
            self.move_after_insert(punc);
            self.move_after_insert(c);
        } else {
            self.move_after_insert(c);
        }
    }
    ///Removes the char at the current cursor position. Does nothing on selection mode
    pub fn backspace(&mut self) {
        if self.mode == TextAreaMode::Selecting {
            return;
        }
        if self.x > 0 {
            if let Some(line) = self.lines.get_mut(self.y) {
                if let Some((_, bsize)) = line.remove(self.x - 1, true) {
                    self.x -= bsize;
                    if bsize > 1 {
                        self.byte_offsets[self.y] -= bsize - 1;
                    }
                };
            }
        } else if self.y > 0 {
            let line = self.lines.remove(self.y);
            self.y -= 1;
            self.x = self.lines[self.y].len();
            self.lines[self.y].append_line(line);
        }
    }
    ///Removes the char forward to the current cursor position. Does nothing on selection
    ///mode.(Simply executes the Delete key usage)
    pub fn del(&mut self) {
        if self.mode == TextAreaMode::Selecting {
            return;
        }
        if self.line().is_empty() && self.lines.len() > 1 {
            self.lines.remove(self.y);
            self.byte_offsets.remove(self.y);
        } else if self.x == self.line().len() && self.y < self.lines.len() - 1 {
            let line = self.lines.remove(self.y + 1);
            self.lines[self.y].append_line(line);
            self.byte_offsets[self.y] += self.byte_offsets.remove(self.y + 1);
            return;
        } else if let Some((_, size)) = self.lines[self.y].remove(self.x, false) {
            if size > 1 {
                self.byte_offsets[self.y] -= size;
            }
        }
    }
    ///Creates a new line and makes the cursor go downwards. Does nothing on selection mode
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
            self.byte_offsets.push(0);
        } else {
            self.lines.insert(self.y, line);
            self.byte_offsets.insert(self.y, 0);
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
    pub fn move_down(&mut self) {
        self.y = (self.y + 1).min(self.lines.len() - 1);
        self.x += char_size_backwards(self.line().bytes(), self.x - 1);
        self.x = self.x.min(self.lines[self.y].len());
    }
    pub fn move_up(&mut self) {
        if self.y == 0 {
            self.x = 0;
            return;
        }
        self.y -= 1;
        self.x += char_size_backwards(self.line().bytes(), self.x - 1);
        self.x = self.x.min(self.lines[self.y].len());
    }
    pub fn move_left(&mut self) {
        if self.x == 0 {
            if self.y > 0 {
                self.y -= 1;
                self.x = self.lines[self.y].len();
            }
            return;
        }
        if let Some(b) = self.lines[self.y].get(self.x - 1) {
            if *b < 0x80 {
                self.x -= 1;
            } else {
                let csize = char_size_backwards(self.lines[self.y].bytes(), self.x - 1);
                self.x -= csize;
                if csize > 1 {
                    self.byte_offsets[self.y] -= csize - 1;
                }
            }
        };
        self.x = self.x.min(self.lines[self.y].len());
        self.lines[self.y].bytes_mut().set_gap(self.x);
    }
    pub fn move_right(&mut self) {
        if let Some(b) = self.lines[self.y].get(self.x) {
            if *b < 0x80 {
                self.x += 1;
            } else {
                self.x += char_size_init(*b) as usize;
                self.byte_offsets[self.y] += char_size_init(*b) as usize;
            }
        }
        self.x = self.x.min(self.lines[self.y].len());
    }
    pub fn move_x(&mut self, n: i16) {
        let mut dp = n - self.x as i16; //delta post. I really dont think anyone will create a line
                                        //with 32764 chars
        if dp > 0 {
            while dp > 0 {
                dp -= 1;
                self.move_right();
            }
        } else {
            while dp < 0 {
                dp += 1;
                self.move_left();
            }
        }
    }
    pub fn move_y(&mut self, n: i16) {
        let mut dp = n - self.x as i16; //delta post
        if dp > 0 {
            while dp > 0 {
                dp -= 1;
                self.move_up();
            }
        } else {
            while dp < 0 {
                dp += 1;
                self.move_down();
            }
        }
    }
    ///Totally resets the writer.
    pub fn reset(&mut self) {
        self.editing_file = None;
        self.lines.clear();
        self.lines.push(TerminalLine::new());
    }
    ///Opens the given file and set the writer content to be the file content. If the file does not
    ///exists, still sets the editing file to be the given path and when trying to write it will
    ///create a new file with the writer contents
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
    ///Close the files and clears the writer content
    pub fn close_file(&mut self) {
        self.clear_content();
        self.editing_file = None;
    }
    pub fn file_name(&self) -> Option<&OsStr> {
        if let Some(ref file) = self.editing_file {
            file.file_name()
        } else {
            None
        }
    }
    ///Saves into the file located as prefix + editiong_file the content of this writer. If the
    ///file does not exist, create a file and write it.
    pub fn save(&self, prefix: &Path) -> std::io::Result<()> {
        if let Some(ref file) = self.editing_file {
            let prefixed = prefix.join(file);
            std::fs::write(&prefixed, self.to_string())
        } else {
            Ok(())
        }
    }
    ///Pastes the given content at the current cursor location. If the content has multiple lines,
    ///append insert them as well
    pub fn paste(&mut self, content: &str) -> CmdTask {
        let lines = content.split('\n').collect::<Vec<&str>>();
        if lines.len() == 1 {
            self.lines[self.y].push_str_back(content);
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
    ///Copies to the clipboard(virtual or not if given) the content of the selection
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
    fn render(self, _: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
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
