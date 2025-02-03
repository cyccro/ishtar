use std::path::PathBuf;

use isht::CmdTask;
use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};

use crate::helpers::{AreaOrder, IshtarColors};

use super::IshtarSelectable;

#[derive(Debug, Eq, PartialEq)]
pub enum ManagingMode {
    Deleting,
    Renaming,
    Creating,
    Searching,
}
pub struct Searcher {
    orientation: AreaOrder,
    path: PathBuf,
    colors: [Color; 3],
}
impl Searcher {
    pub fn new(orientation: AreaOrder, path: PathBuf, colors: [Color; 3]) -> Self {
        Self {
            orientation,
            path,
            colors,
        }
    }
    pub fn render(&self, content: &String, area: Rect, buf: &mut Buffer) {
        //Use layout
    }
}
pub struct FileManager {
    seeing_file: bool, //Flag for checking if file content is being shown while searching
    pub mode: ManagingMode,
    opened: bool,
    buffer: String,
    searcher: Searcher,
}

impl FileManager {
    fn get_colors_from(colors: IshtarColors) -> [Color; 3] {
        let searcher_color =
            Color::from_u32(colors.get("seracher_title").cloned().unwrap_or(0xffffff));
        let searcher_boder_color = colors
            .get("seracher_boder")
            .map(|n| Color::from_u32(*n))
            .unwrap_or(searcher_color);
        let searcher_field_color = colors
            .get("searcher_field")
            .map(|n| Color::from_u32(*n))
            .unwrap_or(searcher_color);
        [searcher_color, searcher_boder_color, searcher_field_color]
    }
    pub fn new_horizontal(see_file: bool, path: PathBuf, colors: IshtarColors) -> Self {
        Self {
            searcher: Searcher::new(AreaOrder::Vertical, path, Self::get_colors_from(colors)),
            seeing_file: see_file,
            mode: ManagingMode::Searching,
            opened: false,
            buffer: String::with_capacity(32),
        }
    }
    pub fn new_vertical(see_file: bool, path: PathBuf, colors: IshtarColors) -> Self {
        Self {
            searcher: Searcher::new(AreaOrder::Vertical, path, Self::get_colors_from(colors)),
            seeing_file: see_file,
            mode: ManagingMode::Searching,
            opened: false,
            buffer: String::with_capacity(32),
        }
    }
    pub fn open(&mut self) {
        self.opened = true;
    }
    pub fn close(&mut self) {
        self.opened = false;
    }
}
impl Widget for &FileManager {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let (w, h) = (area.width >> 1, area.height >> 1);
        if self.mode == ManagingMode::Searching {
            self.searcher.render(&self.buffer, area, buf)
        };
    }
}
impl IshtarSelectable for FileManager {
    fn priority(&self) -> u8 {
        3
    }
    fn priority_static() -> u8
    where
        Self: Sized,
    {
        3
    }
    fn keydown(&mut self, key: ratatui::crossterm::event::KeyCode) -> isht::CmdTask {
        CmdTask::Null
    }
    fn can_render(&self) -> bool {
        self.opened
    }
    fn renderize(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        self.render(area, buf);
    }
}
