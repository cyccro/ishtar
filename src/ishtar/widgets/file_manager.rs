use std::path::PathBuf;

use isht::CmdTask;
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::helpers::{terminal_size, AreaOrder, FileTree, IshtarColors};

use super::IshtarSelectable;

#[derive(Debug, Eq, PartialEq)]
pub enum ManagingMode {
    Deleting,
    Renaming,
    Creating,
    Searching,
}
pub struct Searcher {
    preview: bool,
    orientation: Direction,
    path: PathBuf,
    in_dir_paths: Vec<PathBuf>,
    current_idx: usize,
    colors: [Color; 3],
    writing_idx: usize,
    cursor: (usize, usize),
}
impl Searcher {
    pub fn new(orientation: Direction, path: PathBuf, colors: [Color; 3]) -> Self {
        let size = terminal_size();
        Self {
            writing_idx: 0,
            cursor: ((size.0 / 4) as usize + 1, (size.1 / 4 + 1) as usize),
            preview: false,
            current_idx: 0,
            orientation,
            in_dir_paths: FileTree::new(&path).unwrap().read_paths().unwrap(),
            path,
            colors,
        }
    }
    pub fn cursor(&self) -> (usize, usize) {
        (self.cursor.0 + self.writing_idx, self.cursor.1)
    }
    pub fn render(&mut self, content: &str, area: Rect, buf: &mut Buffer) {
        self.writing_idx = self.writing_idx.min(content.len());
        let areas = Layout::new(
            Direction::Vertical,
            [Constraint::Length(3), Constraint::Fill(1)],
        )
        .areas::<2>(area);
        {
            let area_search = areas[0];
            let block = Block::new()
                .border_style(self.colors[1])
                .borders(Borders::ALL)
                .title_style(self.colors[0])
                .title("Searching");
            Paragraph::new(Span::from(content)).block(block).render(
                Rect {
                    x: area.width / 4,
                    y: area.height / 4,
                    width: area_search.width / 2,
                    height: area_search.height,
                },
                buf,
            );
        }
        {
            let block = Block::new()
                .border_style(self.colors[1])
                .borders(Borders::ALL);
            let searching_areas = areas[1];
            let mut lines: Vec<Line> = Vec::with_capacity(self.in_dir_paths.len());
            let mut idx = 0;
            for buf in &self.in_dir_paths {
                lines.push(Line::styled(
                    buf.display().to_string(),
                    Style::default().fg(if self.current_idx == idx {
                        Color::Black
                    } else {
                        Color::Red
                    }),
                ));
                idx += 1;
            }
            Paragraph::new(lines).block(block).render(
                Rect {
                    width: searching_areas.width / 2,
                    height: searching_areas.height / 2,
                    x: searching_areas.width / 4,
                    y: searching_areas.height / 4 + areas[0].height,
                },
                buf,
            );
        }
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
    ///Searcher Color, Searcher Border, Searcher Field\n
    ///Color extends for both title of the block and the color of the input
    fn get_colors_from(colors: IshtarColors) -> [Color; 3] {
        let searcher_color =
            Color::from_u32(colors.get("seracher_title").cloned().unwrap_or(0xffffff));
        let searcher_border_color = colors
            .get("seracher_border")
            .map(|n| Color::from_u32(*n))
            .unwrap_or(searcher_color);
        let searcher_field_color = colors
            .get("searcher_field")
            .map(|n| Color::from_u32(*n))
            .unwrap_or(searcher_color);
        [searcher_color, searcher_border_color, searcher_field_color]
    }
    pub fn cursor(&self) -> (usize, usize) {
        match self.mode {
            ManagingMode::Searching => self.searcher.cursor(),
            _ => todo!(),
        }
    }
    pub fn new_horizontal(see_file: bool, path: PathBuf, colors: IshtarColors) -> Self {
        Self {
            searcher: Searcher::new(Direction::Horizontal, path, Self::get_colors_from(colors)),
            seeing_file: see_file,
            mode: ManagingMode::Searching,
            opened: false,
            buffer: String::with_capacity(32),
        }
    }
    pub fn new_vertical(see_file: bool, path: PathBuf, colors: IshtarColors) -> Self {
        Self {
            searcher: Searcher::new(Direction::Vertical, path, Self::get_colors_from(colors)),
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
    #[inline]
    pub fn open(&mut self) {
        self.opened = true;
    }
    #[inline]
    pub fn close(&mut self) {
        self.opened = false;
    }
}
impl Widget for &mut FileManager {
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
        match key {
            KeyCode::Left => {
                self.searcher.writing_idx = self.searcher.writing_idx.saturating_sub(1)
            }
            KeyCode::Right => self.move_left(),
            KeyCode::Char(c) => {
                self.move_left();
                self.buffer.push(c);
            }
            KeyCode::Delete => self.delete(),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Esc => return CmdTask::StopSearch,
            _ => {}
        }
        CmdTask::Null
    }
    fn can_render(&self) -> bool {
        self.opened
    }
    fn renderize(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        self.render(area, buf);
    }
}
