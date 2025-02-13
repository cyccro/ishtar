mod enums;
mod logger;
mod widget_manager;
mod widgets;
use isht::{configuration::IshtarConfiguration, CmdTask, ConfigStatment};
use logger::{IshtarLogger, LogLevel};
use std::{
    env,
    ops::{Deref, DerefMut},
    path::PathBuf,
    process::ExitStatus,
};
use widget_manager::WidgetManager;
use widgets::{
    clipboard::IshtarClipboard,
    file_manager::{FileManager, ManagingMode},
    keybind_handler::KeybindHandler,
    IshtarSelectable,
};

use crate::helpers::terminal_size;

use self::{
    enums::IshtarMode,
    widgets::{command_interpreter::CommandInterpreter, writeable_area::WriteableArea},
};
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

///Editor instance containing all data required for working and managing buffers
pub struct Ishtar {
    current_path: PathBuf,
    exit: bool,
    logger_area: IshtarLogger,
    cursor: (usize, usize),
    saved_cursor: (usize, usize),
    handler: WidgetManager,
    priority: (u8, u8), //current | saved
    mode: IshtarMode,
    clipboard: IshtarClipboard,
    size: (u16, u16),
}
impl Default for Ishtar {
    fn default() -> Self {
        let size = terminal_size();
        Self {
            size,
            current_path: env::current_dir().unwrap(),
            exit: false,
            cursor: (0, size.1 as usize),
            saved_cursor: (0, size.1 as usize),
            priority: (
                CommandInterpreter::priority_static(),
                CommandInterpreter::priority_static(),
            ),
            logger_area: IshtarLogger::new().unwrap(),
            clipboard: IshtarClipboard::new(),
            mode: IshtarMode::Cmd,
            handler: WidgetManager::new(),
        }
    }
}
impl Ishtar {
    ///Gets the configurations based on the configuration file located as
    ///~/.config/ishtar/config.isht. If not given,
    ///uses default.
    pub fn get_configs() -> IshtarConfiguration {
        let username = std::env::var("USER").unwrap();
        let path = format!("/home/{username}/.config/ishtar/config.isht");
        let file_path = std::path::Path::new(&path);
        if let Ok(content) = std::fs::read_to_string(file_path) {
            //parses the file and creates a configutation
            IshtarConfiguration::from_content(content).unwrap()
        } else {
            IshtarConfiguration::new()
        }
    }
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }
    fn draw(&mut self, f: &mut Frame) {
        f.set_cursor_position(self.cursor_position());
        f.render_widget(self, f.area());
    }
    pub fn run(&mut self) -> std::io::Result<()> {
        let display = self.current_path.clone();
        self.display(display.display(), logger::LogLevel::Info);
        self.display("Initialize Process", logger::LogLevel::Info);
        let mut terminal = init();
        terminal.show_cursor()?;
        loop {
            if self.exit {
                break;
            }
            terminal.draw(|f| self.draw(f))?;
            self.handle_event()?;
            self.update_cursor();
        }
        ratatui::restore();
        Ok(())
    }
    #[inline]
    pub fn set_priority<T: IshtarSelectable>(&mut self) {
        self.priority.0 = T::priority_static();
    }
    #[inline]
    pub fn is_priority_of<T: IshtarSelectable>(&self) -> bool {
        self.priority.0 == T::priority_static()
    }
    #[inline]
    pub fn set_cursor_at(&mut self, x: usize, y: usize) {
        self.cursor.0 = x;
        self.cursor.1 = y;
    }
    ///Gets the x position of the cursor
    fn x_cursor_position(&self) -> u16 {
        (self.cursor.0
            + match self.mode {
                IshtarMode::Cmd => 0,
                IshtarMode::Modify | IshtarMode::Selection => self.handler.writer().xoffset(),
            }) as u16
    }
    #[inline]
    fn cursor_position(&self) -> Position {
        Position::new(self.x_cursor_position(), self.cursor.1 as u16)
    }

    ///Updates the cursor position to be on the current writing area
    pub fn update_cursor(&mut self) {
        if self.is_priority_of::<FileManager>() {
            self.cursor = self.handler.file_manager().cursor();
            return;
        }
        match self.mode {
            IshtarMode::Cmd => {
                self.cursor.0 = self.handler.cmd().cursor();
                self.cursor.1 = (self.size.1 - 1) as usize;
            }
            IshtarMode::Modify | IshtarMode::Selection => {
                self.cursor = self.handler.writer().cursor();
            }
        };
    }
    fn change_mode(&mut self, mode: IshtarMode) {
        self.display(
            format!("Entering Ishtar '{mode:?}' mode"),
            logger::LogLevel::Info,
        );
        match mode {
            IshtarMode::Modify | IshtarMode::Selection => {
                self.handler.cmd_mut().set(&format!("{mode:?}"));
                let x = self.saved_cursor.0;
                let y = self.saved_cursor.1;
                let writer = self.handler.writer_mut();
                if matches!(mode, IshtarMode::Modify) {
                    writer.enter_writing()
                } else {
                    writer.enter_selection();
                };
                writer.set_cursor_x(x);
                writer.set_cursor_y(y);
                self.set_priority::<WriteableArea>();
            }
            IshtarMode::Cmd => {
                self.save_position();
                self.handler.cmd_mut().clear();
                self.set_priority::<CommandInterpreter>();
            }
        }
        self.mode = mode;
    }
    ///Saves the content of the current area at current_path + current_area_file
    pub fn save_file(&self) -> std::io::Result<()> {
        self.handler.writer().save(&self.current_path)?;
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
        self.saved_cursor = self.handler.writer().cursor();
    }
    pub fn exec_cmd(&mut self, cmd: &String) -> std::io::Result<ExitStatus> {
        std::process::Command::new(cmd)
            .spawn()
            .map(|mut child| child.wait())?
    }
    pub fn request_search(&mut self) {
        self.set_priority::<FileManager>();
        self.handler.file_manager_mut().mode = ManagingMode::Searching;
        self.handler.file_manager_mut().open();
    }
    pub fn stop_search(&mut self) {
        self.priority.0 = self.priority.1;
        self.handler.file_manager_mut().close();
    }
    ///Some will not be handled here. Probably Req ones due to them being a request and depending
    ///on the state of the terminal, being better to pass to a widget handle it instead
    pub fn handle_task(&mut self, task: &CmdTask) {
        if matches!(task, CmdTask::Null | CmdTask::Continue) {
            return;
        };
        self.display(format!("{:?}", task), LogLevel::Info);
        match task {
            CmdTask::SaveMode => self.priority.1 = self.priority.0,
            CmdTask::ReturnSavedMode => {
                self.priority.0 = self.priority.1;
                self.change_mode(self.mode.clone()); //Assert it enters the mode again;
            }
            CmdTask::SetPriority(n) => self.priority.0 = *n,
            CmdTask::CopySelection | CmdTask::CopyToSys | CmdTask::CopyToEditor => {
                let data = self.handler.writer_mut().get_selection();
                if let Some(data) = data {
                    if matches!(task, CmdTask::CopyToSys | CmdTask::CopySelection) {
                        self.clipboard.set(data);
                    } else {
                        self.clipboard.set_virtual(data);
                    };
                }
                self.handle_task(&CmdTask::EnterModify);
            }
            CmdTask::SelectLine => {
                self.handler.writer_mut().goto_init_of_line();
                self.change_mode(IshtarMode::Selection);
                self.handler.writer_mut().goto_end_of_line();
            }
            CmdTask::PasteSys | CmdTask::PasteEditor => {
                let content = if matches!(task, CmdTask::PasteEditor) {
                    self.clipboard.get_virtual().clone()
                } else {
                    self.clipboard.get()
                };
                let data = self.handler.writer_mut().paste(&content);
                self.handle_task(&data);
            }

            CmdTask::DeleteLine => self.handler.writer_mut().delete_line(),

            CmdTask::SavePos => self.save_position(),
            CmdTask::MoveSaved => self.set_cursor_at(self.saved_cursor.0, self.saved_cursor.1),

            CmdTask::Write(content) => {
                self.handler.writer_mut().paste(content);
            }
            CmdTask::CreateWindow => {
                self.handler.writer_mut().create_area();
            }
            CmdTask::DeleteWindow => {
                self.handler.writer_mut().delete_current_area();
            }
            CmdTask::SetWindowUp => self.handler.writer_mut().set_focus_back(),
            CmdTask::SetWindowDown => self.handler.writer_mut().set_focus_next(),

            CmdTask::ExecCmd(cmd) => {
                let _ = self.exec_cmd(cmd);
            }
            CmdTask::ExecutePrompt(prompt) => {
                self.handler.cmd_mut().execute_cmd(prompt);
            }
            CmdTask::MoveIOL => self.handler.writer_mut().goto_init_of_line(),
            CmdTask::MoveEOL => self.handler.writer_mut().goto_end_of_line(),
            CmdTask::MoveIOB => self.handler.writer_mut().goto_init_of_file(),
            CmdTask::MoveEOB => self.handler.writer_mut().goto_end_of_file(),
            CmdTask::MoveToLine(n) => self.handler.writer_mut().move_y(*n as i16),
            CmdTask::MoveToRow(n) => self.handler.writer_mut().move_x(*n as i16),
            CmdTask::EnterNormal => self.change_mode(IshtarMode::Cmd),
            CmdTask::EnterModify => self.change_mode(IshtarMode::Modify),
            CmdTask::EnterSelection => self.change_mode(IshtarMode::Selection),

            CmdTask::ModifyFile(f) => {
                self.handler.writer_mut().open_file(f.into());
            }
            CmdTask::SaveFile => {
                if self.handler.writer().file_name().is_some() {
                    self.handler.writer().save(&self.current_path).unwrap();
                } else {
                    self.set_priority::<CommandInterpreter>();
                    self.handler
                        .cmd_mut()
                        .request_data("Give the file a name ", CmdTask::ReqSaveFile);
                };
            }
            CmdTask::SaveFileAs(msg) => {
                let writer = self.handler.writer_mut();
                writer.modify_file_name(msg);
                writer.save(&self.current_path).unwrap();
            }
            CmdTask::Multi(tasks) => {
                for task in tasks {
                    self.handle_task(task);
                }
            }
            CmdTask::Log(s) => {
                self.display(s, logger::LogLevel::Info);
            }
            CmdTask::Warn(s) => {
                self.display(s, logger::LogLevel::Warn);
            }
            CmdTask::ReqSearchFile => self.request_search(),
            CmdTask::StopSearch => self.stop_search(),
            CmdTask::Exit => self.exit = true,
            e => panic!("Must implement {e:?} or should not be here"),
        }
    }
    fn handle_tasks(&mut self, tasks: &Vec<ConfigStatment>) {
        for task in tasks {
            match task {
                ConfigStatment::Task(t) => self.handle_task(t),
                ConfigStatment::Cmd(c) => {
                    if let Some(tasks) = self.handler.keybind().get(c, self.mode_id()) {
                        self.handle_tasks(&tasks.clone());
                    }
                }
                _ => {}
            }
        }
    }
    ///Handles keybind input. Returns CmdTask::Null if the caller, handler_key, must stop its
    ///execution; CmdTask::Continue otherwhite
    ///The returned task does not have relationship with the modification made.
    fn should_init_keybind(&mut self, key: KeyEvent) -> CmdTask {
        // If got modifier start listening
        if !key.modifiers.is_empty() && !self.handler.keybind().listening() {
            self.set_priority::<KeybindHandler>();
            self.handler
                .keybind_mut()
                .start_listening(key.code, key.modifiers);
            let content = self.handler.keybind().content();
            if let Some(tasks) = self
                .handler
                .keybind()
                .get(&content, self.mode_id())
                .cloned()
            {
                self.handle_tasks(&tasks);
                self.handler.keybind_mut().stop_listening();
            }
            return CmdTask::Null;
        }
        //Dont input but listens to key
        if self.is_priority_of::<KeybindHandler>() {
            self.handler.keybind_mut().current_mode = self.mode_id();
            match self.handler.keybind_mut().keydown(key.code) {
                CmdTask::Null => return CmdTask::Null,
                CmdTask::Continue => return CmdTask::Continue,
                task => self.handle_task(&task),
            }
        }
        CmdTask::Continue
    }
    fn handle_key(&mut self, key: KeyEvent) {
        if let KeyCode::Char(c) = key.code {
            if self.is_priority_of::<WriteableArea>()
                && c.is_uppercase()
                && key.modifiers == KeyModifiers::SHIFT
            {
                self.handler.writer_mut().write_char(c);
                return;
            }
        }
        if !self.is_priority_of::<FileManager>() {
            if let CmdTask::Null = self.should_init_keybind(key) {
                return;
            }
        }
        let widget = self.handler.find_widget_mut(self.priority.0);
        let task = widget.keydown(key.code);
        self.handle_task(&task);
        match self.mode {
            IshtarMode::Modify | IshtarMode::Selection => {
                self.cursor = self.handler.writer().cursor();
            }

            _ => {}
        }
    }
    pub fn handle_event(&mut self) -> std::io::Result<()> {
        if let event::Event::Key(k) = event::read()? {
            self.handle_key(k);
        }
        Ok(())
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
        for i in 0..self.handler.widgets.len() {
            if self.handler.widgets[i].can_render() {
                self.handler.widgets[i].renderize(area, buf);
            }
        }
    }
}
