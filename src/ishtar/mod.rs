mod enums;
mod logger;
mod widgets;
use std::{
    env,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use logger::IshtarLogger;

use crate::helpers::terminal_size;
use ratatui::{
    crossterm::{
        event::{self, KeyCode, KeyEvent, KeyModifiers},
        terminal::{disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    init,
    layout::Position,
    widgets::Widget,
    Frame,
};

use self::{
    enums::{CmdResponse, IshtarMessage, IshtarMode},
    widgets::{command_interpreter::CommandInterpreter, writeable_area::WriteableArea},
};
pub struct Ishtar {
    current_path: PathBuf,
    exit: bool,
    initialized: bool,
    writer: WriteableArea,
    logger_area: IshtarLogger,
    cursor: (usize, usize),
    size: (u16, u16),
    mode: IshtarMode,
    cmd: CommandInterpreter,
}

impl Ishtar {
    pub fn new(savelogs: bool) -> std::io::Result<Self> {
        let size = terminal_size();
        Ok(Self {
            current_path: env::current_dir().unwrap(),
            exit: false,
            initialized: true,
            cursor: (0, size.1 as usize),
            cmd: CommandInterpreter::new(),
            size,
            writer: WriteableArea::new_vertical(size.0, size.1 - 1),
            logger_area: IshtarLogger::new(savelogs, !cfg!(debug_assertions))?,
            mode: IshtarMode::Cmd,
        })
    }
    fn draw(&mut self, f: &mut Frame) {
        f.set_cursor_position(self.cursor_position());
        f.render_widget(self, f.area());
    }
    pub fn run(&mut self) -> std::io::Result<()> {
        self.display(format!("Initialize Process",), logger::LogLevel::Info);
        let mut terminal = init();
        terminal
            .backend_mut()
            .execute(EnterAlternateScreen)
            .unwrap();
        terminal.show_cursor()?;
        loop {
            if self.exit {
                break;
            }
            //self.display("Requested Drawing", logger::LogLevel::Info)?;
            terminal.draw(|f| self.draw(f))?;
            self.handle_event()?;
            self.update_cursor();
        }
        self.finish()?;
        disable_raw_mode()?;
        terminal
            .backend_mut()
            .execute(LeaveAlternateScreen)
            .unwrap();
        Ok(())
    }
    pub fn finish(&mut self) -> std::io::Result<()> {
        self.display("Finished Process", logger::LogLevel::Info);
        Ok(())
    }
    pub fn request_exit(&mut self) {
        self.exit = true;
    }
    pub fn set_cursor_at(&mut self, x: usize, y: usize) {
        self.cursor.0 = x;
        self.cursor.1 = y;
    }
    fn x_cursor_position(&self) -> u16 {
        (self.cursor.0
            + match self.mode {
                IshtarMode::Cmd => 0,
                IshtarMode::Modify => self.writer.xoffset(),
            }) as u16
    }
    fn cursor_position(&self) -> Position {
        Position::new(self.x_cursor_position(), self.cursor.1 as u16)
    }
    pub fn update_cursor(&mut self) {
        match self.mode {
            IshtarMode::Cmd => {
                self.cursor.0 = self.cmd.cursor();
                self.cursor.1 = (self.size.1 - 1) as usize;
            }
            IshtarMode::Modify => {
                let cursor = self.writer.cursor();
                self.cursor.0 = cursor.0;
                self.cursor.1 = cursor.1;
            }
        };
    }
    fn change_mode(&mut self, mode: IshtarMode) {
        self.display(
            format!("Entering Ishtar '{mode:?}' mode"),
            logger::LogLevel::Info,
        );
        match mode {
            IshtarMode::Modify => self.cmd.set("Modifying"),
            IshtarMode::Cmd => self.cmd.clear(),
        }
        self.mode = mode;
    }
    pub fn save_file(&self) -> std::io::Result<()> {
        self.writer.save(&self.current_path)?;
        Ok(())
    }
    pub fn handle_message(&mut self, msg: IshtarMessage) -> std::io::Result<()> {
        match msg {
            IshtarMessage::Cmd(response) => match response {
                CmdResponse::Exit => {
                    self.request_exit();
                }
                CmdResponse::ChangeMode(mode) => self.change_mode(mode),
                CmdResponse::SaveFile => self.save_file()?,
                CmdResponse::Reset => self.writer.reset(),
                CmdResponse::ModifyFile(s) => {
                    self.writer.open_file(PathBuf::from(s));
                    self.change_mode(IshtarMode::Modify);
                }
            },
            IshtarMessage::ChangeMode(mode) => self.change_mode(mode),
            IshtarMessage::Null => {}
        }
        Ok(())
    }
    pub fn handle_key(&mut self, key: KeyEvent) -> std::io::Result<IshtarMessage> {
        match self.mode {
            IshtarMode::Cmd => {
                match key.code {
                    KeyCode::Esc => self.cmd.clear(),
                    KeyCode::Char(c) => {
                        if let Some(res) = self.cmd.write(c) {
                            return Ok(IshtarMessage::Cmd(res));
                        }
                    }
                    KeyCode::Left => self.cmd.move_left(),
                    KeyCode::Right => self.cmd.move_right(),
                    KeyCode::Enter => {
                        {
                            let content = format!("Executing command {}", self.cmd.line());
                            self.display(content, logger::LogLevel::Info);
                        };
                        if let Some(cmd) = self.cmd.execute() {
                            return Ok(IshtarMessage::Cmd(cmd));
                        }
                    }
                    KeyCode::Backspace => self.cmd.backspace(),
                    _ => {}
                };
            }
            IshtarMode::Modify => {
                match key.code {
                    KeyCode::Esc => return Ok(IshtarMessage::ChangeMode(IshtarMode::Cmd)),
                    KeyCode::Char(c) => {
                        self.writer.write_char(c);
                    }
                    KeyCode::Backspace => self.writer.backspace(),
                    KeyCode::Enter => self.writer.newline(),
                    KeyCode::Up => self.writer.move_y(-1),
                    KeyCode::Down => self.writer.move_y(1),
                    KeyCode::Left => self.writer.move_x(-1),
                    KeyCode::Right => self.writer.move_x(1),
                    KeyCode::Insert => {
                        self.writer.create_area();
                    }
                    KeyCode::Delete => {
                        self.writer.delete_area(99);
                    }
                    KeyCode::PageDown => self.writer.set_focus(self.writer.focus() + 1),
                    KeyCode::PageUp => {
                        let focus = self.writer.focus();
                        if focus != 0 {
                            self.writer.set_focus(focus - 1)
                        }
                    }
                    KeyCode::End => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            self.writer.goto_end_of_file()
                        } else {
                            self.writer.goto_end_of_line()
                        }
                    }
                    KeyCode::Home => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            self.writer.goto_init_of_file()
                        } else {
                            self.writer.goto_init_of_line()
                        }
                    }
                    KeyCode::F(5) => {
                        self.logger_area.display(
                            format!("Wrote in file {:?}", self.writer.file_name()),
                            logger::LogLevel::Info,
                        );
                        self.writer.save(&self.current_path)?
                    }
                    _ => {
                        self.initialized = false;
                    }
                }
                self.cursor = self.writer.cursor();
            }
        }
        Ok(IshtarMessage::Null)
    }
    pub fn handle_event(&mut self) -> std::io::Result<()> {
        let msg = match event::read()? {
            event::Event::Key(k) => self.handle_key(k)?,
            _ => IshtarMessage::Null,
        };
        self.handle_message(msg)
    }
}
impl Deref for Ishtar {
    type Target = IshtarLogger;
    fn deref(&self) -> &Self::Target {
        &self.logger_area
    }
}
impl DerefMut for Ishtar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.logger_area
    }
}
impl Widget for &mut Ishtar {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.writer.render(area, buf);
        self.cmd.render(area, buf);
    }
}
