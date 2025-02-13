use std::{
    borrow::Cow,
    env::join_paths,
    ops::{Range, RangeBounds},
    path::PathBuf,
};

use isht::CmdTask;
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget},
};

use crate::helpers::{min_max, terminal_size, AreaOrder, FileTree, IshtarColors};

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
            in_dir_paths: std::fs::read_dir(&path)
                .unwrap()
                .map(|dir| dir.unwrap().path())
                .collect(),
            colors,
        }
    }
    pub fn cursor(&self) -> (usize, usize) {
        (self.cursor.0 + self.writing_idx, self.cursor.1)
    }
    pub fn update(&mut self) {
        let Ok(entry) = std::fs::read_dir(self.selected_dir()) else {
            return;
        };
        self.in_dir_paths.clear();
        entry.for_each(|entry| {
            self.in_dir_paths.push({
                let Ok(entry) = entry else {
                    return;
                };
                entry.path()
            })
        });
        self.current_idx = self.current_idx.min(self.in_dir_paths.len());
    }
    pub fn selected_dir(&self) -> &std::path::Path {
        if let Some(path) = self.in_dir_paths.get(self.current_idx) {
            path
        } else {
            self.in_dir_paths[0].parent().unwrap().parent().unwrap()
        }
    }

    fn all_file_names(&self) -> Vec<(Cow<str>, Cow<str>, usize)> {
        let mut vec = Vec::with_capacity(self.in_dir_paths.len());
        for (idx, entry) in self.in_dir_paths.iter().enumerate() {
            let parent_name = entry
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy();
            let file_name = entry.file_name().unwrap().to_string_lossy();
            vec.push((parent_name, file_name, idx))
        }
        vec.push(("..".into(), "".into(), vec.len()));
        vec
    }
    fn file_names_from(&self, n: usize) -> Vec<(Cow<str>, Cow<str>, usize)> {
        let mut vec = Vec::with_capacity(self.in_dir_paths.len());
        for (idx, entry) in self.in_dir_paths[n..].iter().enumerate() {
            let parent_name = entry
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy();
            let file_name = entry.file_name().unwrap().to_string_lossy();
            vec.push((parent_name, file_name, idx))
        }
        vec.push(("..".into(), "".into(), vec.len()));
        vec
    }
    fn file_names(&self, range: Range<usize>) -> Vec<(Cow<str>, Cow<str>, usize)> {
        let mut vec = Vec::with_capacity(self.in_dir_paths.len());
        for (idx, entry) in self.in_dir_paths[range].iter().enumerate() {
            let parent_name = entry
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy();
            let file_name = entry.file_name().unwrap().to_string_lossy();
            vec.push((parent_name, file_name, idx))
        }
        vec.push(("..".into(), "".into(), vec.len()));
        vec
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
                .border_set(symbols::border::Set {
                    top_left: symbols::line::ROUNDED.top_left,
                    top_right: symbols::line::ROUNDED.top_right,
                    bottom_left: symbols::line::ROUNDED.vertical_right,
                    bottom_right: symbols::line::ROUNDED.vertical_left,
                    ..symbols::border::PLAIN
                })
                .border_style(self.colors[1])
                .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP | Borders::BOTTOM)
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
            let set = symbols::border::Set {
                bottom_left: symbols::line::ROUNDED.bottom_left,
                bottom_right: symbols::line::ROUNDED.bottom_right,
                ..symbols::border::PLAIN
            };
            let block = Block::new()
                .border_set(set)
                .border_style(self.colors[1])
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM);
            let searching_areas = areas[1];

            let height = (searching_areas.height / 2) as usize;
            let len = self.in_dir_paths.len();

            let dif = {
                let (min, max) = min_max(len, height);
                max - min
            };
            let opt = 0;
            let mut lines: Vec<Line> = Vec::with_capacity(len.min(height));

            let style = Style::default();
            if height >= len {
                for (parent, name, idx) in self.all_file_names() {
                    lines.push(
                        Line::from(vec![Span::from(parent), Span::from("/"), Span::from(name)])
                            .style(if self.current_idx == idx {
                                style.fg(Color::Green).add_modifier(Modifier::ITALIC)
                            } else {
                                style.fg(Color::Red)
                            }),
                    )
                }
            } else if self.current_idx > dif {
                for (parent, name, idx) in self.file_names_from(dif + 1) {
                    lines.push(
                        Line::from(vec![Span::from(parent), Span::from("/"), Span::from(name)])
                            .style(if self.current_idx == dif + idx + 1 {
                                style.fg(Color::Green).add_modifier(Modifier::ITALIC)
                            } else {
                                style.fg(Color::Red)
                            }),
                    )
                }
            } else {
                for (parent, name, idx) in
                    self.file_names(self.current_idx..self.current_idx + height)
                {
                    lines.push(
                        Line::from(vec![Span::from(parent), Span::from("/"), Span::from(name)])
                            .style(if idx == 0 {
                                style.fg(Color::Green).add_modifier(Modifier::ITALIC)
                            } else {
                                style.fg(Color::Red)
                            }),
                    )
                }
            };
            Paragraph::new(lines).block(block).render(
                Rect {
                    width: searching_areas.width / 2,
                    height: height as u16,
                    x: searching_areas.width / 4,
                    y: searching_areas.height / 4 + areas[0].height + 1,
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
                    self.searcher.update();
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
    fn renderize(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        self.render(area, buf);
    }
}
