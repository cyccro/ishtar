use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use isht::CmdTask;
use ratatui::{buffer::Buffer, crossterm::event::KeyCode, prelude::Rect, widgets::Widget, Frame};

use crate::helpers::AreaOrder;

use super::{text_area::TextArea, IshtarSelectable};
#[derive(Debug)]
pub struct WriteableArea {
    writers: Vec<TextArea>,
    order: AreaOrder,
    focused_writer: usize,
    area: (u16, u16),
    colors: Arc<HashMap<String, u32>>,
}

impl WriteableArea {
    pub fn new_horizontal(w: u16, h: u16, colors: Arc<HashMap<String, u32>>) -> Self {
        let mut s = Self {
            writers: Vec::new(),
            order: AreaOrder::Horizontal,
            focused_writer: 0,
            area: (w, h),
            colors,
        };
        s.create_area();
        s
    }
    pub fn new_vertical(w: u16, h: u16, colors: Arc<HashMap<String, u32>>) -> Self {
        let mut s = Self {
            writers: Vec::new(),
            order: AreaOrder::Vertical,
            focused_writer: 0,
            area: (w, h),
            colors,
        };
        s.create_area();
        s
    }
    ///Gets the cursor position based on the active text area
    pub fn cursor(&self) -> (usize, usize) {
        let current_writer = &self.writers[self.focused_writer];
        let w = if let AreaOrder::Horizontal = self.order {
            self.area.0 as usize / self.len() * self.focused_writer + current_writer.cursor_x()
        } else {
            self.cursor_x()
        };
        let y = if let AreaOrder::Vertical = self.order {
            self.area.1 as usize / self.len() * self.focused_writer + current_writer.cursor_y()
        } else {
            self.cursor_y()
        };
        (w, y)
    }
    ///Gets how many TexArea this Writeable is handling
    pub fn len(&self) -> usize {
        self.writers.len()
    }
    pub fn order(&self) -> &AreaOrder {
        &self.order
    }
    pub fn order_mut(&mut self) -> &mut AreaOrder {
        &mut self.order
    }
    pub fn focus(&self) -> usize {
        self.focused_writer
    }
    ///Focus the next text area, if vertical, goes one below, if horizontal goes one to the right
    pub fn set_focus_next(&mut self) {
        self.set_focus(self.focused_writer + 1);
    }
    ///Focus the back text area, if vertical, goes one up, if horizontal goes one to the left
    pub fn set_focus_back(&mut self) {
        if self.focused_writer > 0 {
            self.set_focus(self.focused_writer - 1);
        }
    }
    ///Set the focus on the are with the given index
    pub fn set_focus(&mut self, focus: usize) {
        self.focused_writer = focus.min(self.len() - 1);
    }
    pub fn current_area(&self) -> &TextArea {
        &self.writers[self.focused_writer]
    }
    ///Writes the given char into the current area
    pub fn write_char(&mut self, c: char) {
        self.current_area_mut().write_char(c);
    }
    pub fn current_area_mut(&mut self) -> &mut TextArea {
        &mut self.writers[self.focused_writer]
    }
    fn modify_areas(&mut self) {
        let writers_len = self.writers.len() as u16;
        match self.order {
            AreaOrder::Vertical => {
                //One
                //Two
                //Three
                let h = self.area.1 / writers_len;
                let mut yoffset = 0;
                for writer in &mut self.writers {
                    writer.set_posx(0);
                    writer.set_posy(yoffset);
                    writer.set_w(self.area.0);
                    writer.set_h(h);
                    yoffset += h;
                }
            }
            AreaOrder::Horizontal => {
                //One|Two|Three
                let w = self.area.0 / writers_len;
                let mut xoffset = 0;
                for writer in &mut self.writers {
                    writer.set_posx(xoffset);
                    writer.set_posy(0);
                    writer.set_w(w);
                    writer.set_h(self.area.1);
                    xoffset += w;
                }
            }
        }
    }
    ///Creates a new text area and modifies the view to handle the new one
    pub fn create_area(&mut self) {
        self.writers.push(TextArea::new(0, 0, 0, 2));
        self.modify_areas();
    }
    ///Deletes the current active area
    pub fn delete_current_area(&mut self) -> Option<TextArea> {
        if !self.writers.is_empty() {
            let data = Some(self.writers.remove(self.focused_writer));
            self.focused_writer -= 1;
            data
        } else {
            None
        }
    }
    ///Deletes the area based on it's index, if horizontal, +idx = right, if vertical,
    ///+idx = bottom
    pub fn delete_area(&mut self, idx: usize) -> Option<TextArea> {
        let idx = idx.min(self.len() - 1);
        if idx != 0 {
            if self.focused_writer == idx {
                self.focused_writer -= 1;
            }
            Some(self.writers.remove(idx))
        } else {
            None
        }
    }
}
impl Deref for WriteableArea {
    type Target = TextArea;
    fn deref(&self) -> &Self::Target {
        &self.writers[self.focused_writer]
    }
}
impl DerefMut for WriteableArea {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writers[self.focused_writer]
    }
}
impl IshtarSelectable for WriteableArea {
    fn priority_static() -> u8
    where
        Self: Sized,
    {
        1
    }
    fn priority(&self) -> u8 {
        1
    }
    fn keydown(&mut self, key: ratatui::crossterm::event::KeyCode) -> isht::CmdTask {
        match key {
            KeyCode::Esc => return CmdTask::EnterNormal,
            KeyCode::Char(c) => self.write_char(c),
            KeyCode::Delete => self.current_area_mut().del(),
            KeyCode::Backspace => self.current_area_mut().backspace(),
            KeyCode::Enter => self.current_area_mut().newline(),
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::End => self.goto_end_of_line(),
            KeyCode::Home => self.goto_init_of_line(),
            _ => {}
        }
        CmdTask::Null
    }
    fn renderize(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(self, area);
    }
}
impl Widget for &WriteableArea {
    fn render(self, _: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        for writer in &self.writers {
            writer.render_colored(&self.colors, buf);
        }
    }
}
