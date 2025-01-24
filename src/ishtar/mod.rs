mod configuration;
mod enums;
mod logger;
mod widgets;
use std::{
    env,
    ops::{Deref, DerefMut},
    path::PathBuf,
    process::ExitStatus,
};

use isht::{CmdTask, ConfigStatment};
use logger::IshtarLogger;
use widgets::{clipboard::IshtarClipboard, keybind_handler::KeybindHandler};

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
    configuration::IshtarConfiguration,
    enums::{CmdResponse, IshtarMessage, IshtarMode},
    widgets::{command_interpreter::CommandInterpreter, writeable_area::WriteableArea},
};
///Editor instance containing all data required for working and managing buffers
pub struct Ishtar {
    current_path: PathBuf,
    exit: bool,
    initialized: bool,
    writer: WriteableArea,
    logger_area: IshtarLogger,
    cursor: (usize, usize),
    saved_cursor: (usize, usize),
    size: (u16, u16),
    mode: IshtarMode,
    cmd: CommandInterpreter,
    clipboard: IshtarClipboard,
    keybinds: KeybindHandler,
}
impl Default for Ishtar {
    fn default() -> Self {
        let size = terminal_size();
        let configs = Self::get_configs();
        Self {
            current_path: env::current_dir().unwrap(),
            exit: false,
            initialized: true,
            cursor: (0, size.1 as usize),
            saved_cursor: (0, size.1 as usize),
            cmd: CommandInterpreter::new(),
            size,
            writer: WriteableArea::new_vertical(size.0, size.1 - 1),
            logger_area: IshtarLogger::new().unwrap(),
            clipboard: IshtarClipboard::new(),
            mode: IshtarMode::Cmd,
            keybinds: KeybindHandler::new(configs.keybinds),
        }
    }
}
impl Ishtar {
    pub fn get_configs() -> IshtarConfiguration {
        let path_location = env::var("CONFIG_PATH").expect(
            "Must set CONFIG_PATH during compilation time. Try CONFIG_PATH=<path> cargo build --release",
        );
        let file_path = std::path::Path::new(&path_location);
        if file_path.exists() {
            let content = std::fs::read_to_string(file_path).unwrap();
            IshtarConfiguration::from_content(content).unwrap() //parses the file and creates a configutation
        } else {
            println!("The path for configuration does not exists. Using default instead");
            IshtarConfiguration::default()
        }
    }
    pub fn new() -> Self {
        Default::default()
    }
    fn draw(&mut self, f: &mut Frame) {
        f.set_cursor_position(self.cursor_position());
        f.render_widget(self, f.area());
    }
    pub fn run(&mut self) -> std::io::Result<()> {
        self.display("Initialize Process", logger::LogLevel::Info);
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
            terminal.draw(|f| self.draw(f))?;
            self.handle_event()?;
            self.update_cursor();
        }
        disable_raw_mode()?;
        terminal
            .backend_mut()
            .execute(LeaveAlternateScreen)
            .unwrap();
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
                IshtarMode::Modify | IshtarMode::Selection => self.writer.xoffset(),
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
            IshtarMode::Modify | IshtarMode::Selection => {
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
            IshtarMode::Modify => {
                self.cmd.set("Modifying");
                self.writer.enter_writing();
            }
            IshtarMode::Cmd => self.cmd.clear(),
            IshtarMode::Selection => {
                self.cmd.set("Selection");
                self.writer.enter_selection();
            }
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
    pub fn mode_id(&self) -> usize {
        match self.mode {
            IshtarMode::Cmd => 0,
            IshtarMode::Modify => 1,
            IshtarMode::Selection => 2,
        }
    }
    pub fn save_position(&mut self) {
        self.saved_cursor.0 = self.x_cursor_position() as usize;
        self.saved_cursor.1 = self.cursor.1;
    }
    pub fn exec_cmd(&mut self, cmd: &String) -> std::io::Result<ExitStatus> {
        std::process::Command::new(cmd)
            .spawn()
            .map(|mut child| child.wait())?
    }
    pub fn handle_task(&mut self, task: &CmdTask) {
        match task {
            CmdTask::CopySelection | CmdTask::CopyToSys | CmdTask::CopyToEditor => {
                let data = self
                    .writer
                    .copy_selection(&mut self.clipboard, matches!(task, CmdTask::CopyToEditor));
                self.handle_task(&data);
            }
            CmdTask::SelectLine => {
                self.writer.goto_init_of_line();
                self.change_mode(IshtarMode::Selection);
                self.writer.goto_end_of_line();
            }
            CmdTask::PasteSys | CmdTask::PasteEditor => {
                let content = if matches!(task, CmdTask::PasteEditor) {
                    self.clipboard.get_virtual()
                } else {
                    &self.clipboard.get()
                };
                let data = self.writer.paste(content);
                self.handle_task(&data);
            }

            CmdTask::DeleteLine => self.writer.delete_line(),

            CmdTask::SavePos => self.save_position(),
            CmdTask::MoveSaved => self.set_cursor_at(self.saved_cursor.0, self.saved_cursor.1),

            CmdTask::Write(content) => {
                self.writer.paste(content);
            }
            CmdTask::CreateWindow => {
                self.writer.create_area();
            }
            CmdTask::DeleteWindow => {
                self.writer.delete_current_area();
            }
            CmdTask::SetWindowUp => self.writer.set_focus_back(),
            CmdTask::SetWindowDown => self.writer.set_focus_next(),

            CmdTask::ExecCmd(cmd) => {
                let _ = self.exec_cmd(cmd);
            }
            CmdTask::ExecutePrompt(prompt) => {
                self.cmd.execute_cmd(prompt);
            }
            CmdTask::MoveIOL => self.writer.goto_init_of_line(),
            CmdTask::MoveEOL => self.writer.goto_end_of_line(),
            CmdTask::MoveIOB => self.writer.goto_init_of_file(),
            CmdTask::MoveEOB => self.writer.goto_end_of_file(),
            //CmdTask::MoveToLine(n) => self.writer.move_y(*n as i32),
            //CmdTask::MoveToRow(n) => self.writer.move_x(*n as i32),
            CmdTask::EnterNormal => self.change_mode(IshtarMode::Cmd),
            CmdTask::EnterModify => self.change_mode(IshtarMode::Modify),
            CmdTask::EnterSelection => self.change_mode(IshtarMode::Selection),

            CmdTask::SaveFile => self.writer.save(&self.current_path).unwrap(),
            e => panic!("Must implement {e:?}"),
        }
    }
    fn handle_tasks(&mut self, tasks: &Vec<ConfigStatment>) {
        for task in tasks {
            match task {
                ConfigStatment::Task(t) => self.handle_task(t),
                ConfigStatment::Cmd(c) => {
                    if let Some(tasks) = self.keybinds.get(c, self.mode_id()) {
                        self.handle_tasks(&tasks.clone());
                    }
                }
                _ => {}
            }
        }
    }
    fn keybinds_content(&self) -> String {
        self.keybinds
            .buffer
            .join("-")
            .split(' ')
            .collect::<Vec<&str>>()
            .join("")
            .to_string()
    }
    ///Handles keybind input. Returns Some if it made some change on the keybinds, None otherwhise.
    ///The returned message does not have relationship with the modification made.
    fn handle_keybind(&mut self, key: KeyEvent) -> Option<IshtarMessage> {
        // If got modifier start listening
        if !key.modifiers.is_empty() && !self.keybinds.listening() {
            self.keybinds.start_listening(key.code, key.modifiers);
            let content = self.keybinds_content();
            if let Some(tasks) = self.keybinds.get(&content, self.mode_id()).cloned() {
                self.handle_tasks(&tasks);
                self.keybinds.stop_listening();
            }
            return Some(IshtarMessage::Null);
        }
        // Stops listening when pressing enter
        if key.code == KeyCode::Enter && self.keybinds.listening() {
            let content = self.keybinds_content();
            self.display(format!("Exetuing cmd {content}"), logger::LogLevel::Info);
            if let Some(tasks) = self.keybinds.get(&content, self.mode_id()).cloned() {
                self.handle_tasks(&tasks);
            }
            self.keybinds.stop_listening();
            return Some(IshtarMessage::Null);
        }
        //Dont input but listens to key
        if self.keybinds.listening() {
            self.keybinds.handle(key.code);
            return Some(IshtarMessage::Null);
        }
        None
    }
    fn handle_key(&mut self, key: KeyEvent) -> std::io::Result<IshtarMessage> {
        if let Some(msg) = self.handle_keybind(key) {
            return Ok(msg);
        }
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
                        let line = self.writer.line().clone();
                        let gap = line.bytes().gap();
                        self.display(format!("{c} {gap} {line}",), logger::LogLevel::Info);
                        self.writer.write_char(c);
                    }
                    KeyCode::Backspace => self.writer.backspace(),
                    KeyCode::Delete => self.writer.del(),
                    KeyCode::Enter => self.writer.newline(),
                    KeyCode::Up => self.writer.move_up(),
                    KeyCode::Down => self.writer.move_down(),
                    KeyCode::Left => self.writer.move_left(),
                    KeyCode::Right => self.writer.move_right(),
                    KeyCode::End => self.writer.goto_end_of_line(),
                    KeyCode::Home => self.writer.goto_init_of_line(),

                    _ => {
                        self.initialized = false;
                    }
                }
                self.cursor = self.writer.cursor();
            }
            IshtarMode::Selection => {
                match key.code {
                    KeyCode::Esc => return Ok(IshtarMessage::ChangeMode(IshtarMode::Cmd)),
                    KeyCode::Up => self.writer.move_up(),
                    KeyCode::Down => self.writer.move_down(),
                    KeyCode::Left => self.writer.move_left(),
                    KeyCode::Right => self.writer.move_right(),
                    KeyCode::End => self.writer.goto_end_of_line(),
                    KeyCode::Home => self.writer.goto_init_of_line(),
                    _ => {}
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
