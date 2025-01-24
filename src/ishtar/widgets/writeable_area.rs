use std::ops::{Deref, DerefMut};

use ratatui::{prelude::Rect, widgets::Widget};

use super::text_area::TextArea;

#[derive(Debug)]
pub enum WriteableAreaOrder {
    Horizontal,
    Vertical,
}
pub struct WriteableArea {
    writers: Vec<TextArea>,
    order: WriteableAreaOrder,
    focused_writer: usize,
    area: (u16, u16),
}

impl WriteableArea {
    pub fn new_horizontal(w: u16, h: u16) -> Self {
        let mut s = Self {
            writers: Vec::new(),
            order: WriteableAreaOrder::Horizontal,
            focused_writer: 0,
            area: (w, h),
        };
        s.create_area();
        s
    }
    pub fn new_vertical(w: u16, h: u16) -> Self {
        let mut s = Self {
            writers: Vec::new(),
            order: WriteableAreaOrder::Vertical,
            focused_writer: 0,
            area: (w, h),
        };
        s.create_area();
        s
    }
    ///Gets the cursor position
    pub fn cursor(&self) -> (usize, usize) {
        let current_writer = &self.writers[self.focused_writer];
        let w = if let WriteableAreaOrder::Horizontal = self.order {
            self.area.0 as usize / self.len() * self.focused_writer + current_writer.cursor_x()
        } else {
            self.cursor_x()
        };
        let y = if let WriteableAreaOrder::Vertical = self.order {
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
    pub fn order(&self) -> &WriteableAreaOrder {
        &self.order
    }
    pub fn order_mut(&mut self) -> &mut WriteableAreaOrder {
        &mut self.order
    }
    pub fn focus(&self) -> usize {
        self.focused_writer
    }
    pub fn set_focus_next(&mut self) {
        self.set_focus(self.focused_writer + 1);
    }
    pub fn set_focus_back(&mut self) {
        if self.focused_writer > 0 {
            self.set_focus(self.focused_writer - 1);
        }
    }
    pub fn set_focus(&mut self, focus: usize) {
        self.focused_writer = focus.min(self.len() - 1);
    }
    pub fn current_area(&self) -> &TextArea {
        &self.writers[self.focused_writer]
    }
    pub fn write_char(&mut self, c: char) {
        self.current_area_mut().write_char(c);
    }
    pub fn current_area_mut(&mut self) -> &mut TextArea {
        &mut self.writers[self.focused_writer]
    }
    fn modify_areas(&mut self) {
        let writers_len = self.writers.len() as u16;
        match self.order {
            WriteableAreaOrder::Vertical => {
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
            WriteableAreaOrder::Horizontal => {
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
    pub fn create_area(&mut self) {
        self.writers.push(TextArea::new(0, 0, 0, 2));
        self.modify_areas();
    }
    pub fn delete_current_area(&mut self) -> Option<TextArea> {
        if !self.writers.is_empty() {
            let data = Some(self.writers.remove(self.focused_writer));
            self.focused_writer -= 1;
            data
        } else {
            None
        }
    }
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
impl Widget for &mut WriteableArea {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        for writer in &mut self.writers {
            writer.render(area, buf);
        }
    }
}
