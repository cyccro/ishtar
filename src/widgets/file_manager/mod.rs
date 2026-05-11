pub mod searcher;
use crate::widgets::file_manager::searcher::Searcher;
use ratatui::{crossterm::event::KeyCode, layout::Direction, style::Color, widgets::Widget, Frame};
use std::path::PathBuf;

use crate::{
    helpers::IshtarColors,
    widgets::{CmdTask, IshtarSelectable},
};

#[derive(Debug, Eq, PartialEq)]
pub enum ManagingMode {
    Deleting,
    Renaming,
    Creating,
    Searching,
}

pub struct FileManager {
    seeing_file: bool, //Flag for checking if file content is being shown while searching
    pub mode: ManagingMode,
    opened: bool,
    buffer: String,
    searcher: Searcher,
}

impl FileManager {
    ///Searcher Color, Searcher Border, Searcher Field\n
    ///Color extends for both title of the block and the color of the input

    pub fn cursor(&self) -> (usize, usize) {
        match self.mode {
            ManagingMode::Searching => self.searcher.cursor(),
            _ => todo!(),
        }
    }
    pub fn new(see_file: bool, path: PathBuf) -> Self {
        Self {
            searcher: Searcher::new(path),
            seeing_file: see_file,
            mode: ManagingMode::Searching,
            opened: false,
            buffer: String::with_capacity(32),
        }
    }

    pub fn move_left(&mut self) {
        match self.mode {
            ManagingMode::Searching => {
                self.searcher.writing_idx =
                    (self.searcher.writing_idx + 1).min(self.buffer.len() + 1)
            }
            _ => todo!(),
        }
    }

    ///Updates the searcher dir to be the given one
    pub fn update_searcher_dir(&mut self, dir: &std::path::Path) {
        self.searcher.update(Some(dir));
    }

    fn delete(&mut self) {
        match self.mode {
            ManagingMode::Searching => {
                if self.searcher.writing_idx == self.buffer.len() {
                    return;
                }
                self.buffer.remove(self.searcher.writing_idx);
            }
            _ => todo!(),
        }
    }
    fn backspace(&mut self) {
        match self.mode {
            ManagingMode::Searching => {
                if self.searcher.writing_idx == 0 {
                    return;
                }

                self.searcher.writing_idx -= 1;
                self.buffer.remove(self.searcher.writing_idx);
            }
            _ => todo!(),
        }
    }
    ///Opens the managing section and makes it visible
    #[inline]
    pub fn open(&mut self) {
        self.opened = true;
    }
    ///Closes the managing section and makes it invisible
    #[inline]
    pub fn close(&mut self) {
        self.opened = false;
    }
}

impl Widget for &FileManager {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if self.mode == ManagingMode::Searching {
            self.searcher.render(&self.buffer, area, buf)
        };
    }
}

impl IshtarSelectable for FileManager {
    fn keydown(&mut self, key: ratatui::crossterm::event::KeyCode) -> CmdTask {
        match key {
            KeyCode::End => self.searcher.current_idx = self.searcher.in_dir_paths.len(),
            KeyCode::Home => self.searcher.current_idx = 0,
            KeyCode::Left => {
                self.searcher.writing_idx = self.searcher.writing_idx.saturating_sub(1)
            }
            KeyCode::Right => self.move_left(),
            KeyCode::Down => {
                self.searcher.current_idx =
                    (self.searcher.current_idx + 1).min(self.searcher.in_dir_paths.len());
            }
            KeyCode::Up => {
                self.searcher.current_idx = self.searcher.current_idx.saturating_sub(1);
            }
            KeyCode::Char(c) => {
                self.move_left();
                self.buffer.push(c);
            }
            KeyCode::Delete => self.delete(),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Esc => return CmdTask::StopSearch,
            KeyCode::Enter => {
                let dir = self.searcher.selected_dir();
                return if dir.is_file() {
                    CmdTask::Multi(vec![
                        CmdTask::ModifyFile(dir.display().to_string()),
                        CmdTask::StopSearch,
                    ])
                } else {
                    self.searcher.update(None);
                    CmdTask::Null
                };
            }
            _ => {}
        }
        CmdTask::Null
    }

    fn can_render(&self) -> bool {
        self.opened
    }

    fn renderize(&self, frame: &mut Frame, area: ratatui::prelude::Rect) {
        frame.render_widget(self, area);
    }
}
