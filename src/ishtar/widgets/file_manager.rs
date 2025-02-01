use std::path::{Path, PathBuf};

use isht::CmdTask;
use ratatui::{
    style::{Color, Style},
    text::Line,
    widgets::Widget,
};

use crate::helpers::{AreaOrder, FileTree, IshtarColors};

use super::IshtarSelectable;

pub enum ManagingMode {
    Deleting,
    Renaming,
    Creating,
    Searching,
}

pub struct FileManager {
    seeing_file: bool, //Flag for checking if file content is being shown while searching
    orientation: AreaOrder,
    tree: Vec<PathBuf>,
    colors: IshtarColors,
    pub mode: ManagingMode,
    opened: bool,
    last_opened_dir: Option<PathBuf>,
    writing_buf: String,
}

impl FileManager {
    pub fn new_horizontal(see_file: bool, path: PathBuf, colors: IshtarColors) -> Self {
        Self {
            seeing_file: see_file,
            orientation: AreaOrder::Horizontal,
            tree: FileTree::Dir(path).read_paths().unwrap(),
            colors,
            mode: ManagingMode::Searching,
            last_opened_dir: None,
            opened: false,
            writing_buf: String::with_capacity(64),
        }
    }
    pub fn new_vertical(see_file: bool, path: PathBuf, colors: IshtarColors) -> Self {
        Self {
            seeing_file: see_file,
            orientation: AreaOrder::Vertical,
            tree: FileTree::Dir(path).read_paths().unwrap(), //Panics due, if fails its a bug and i
            //gotta solve it
            colors,
            mode: ManagingMode::Searching,
            last_opened_dir: None,
            opened: false,
            writing_buf: String::with_capacity(64),
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
        Line::styled("Eitalasqueira", Style::default().bg(Color::Black)).render(
            ratatui::layout::Rect {
                x: 40,
                y: 40,
                width: 40,
                height: 40,
            },
            buf,
        );
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
