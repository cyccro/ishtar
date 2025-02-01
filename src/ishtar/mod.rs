mod enums;
mod logger;
mod widgets;
use std::{
    env,
    ops::{Deref, DerefMut},
    path::PathBuf,
    process::ExitStatus,
};

use isht::{configuration::IshtarConfiguration, CmdTask, ConfigStatment};
use logger::IshtarLogger;
use widgets::{
    clipboard::IshtarClipboard,
    file_manager::{FileManager, ManagingMode},
    keybind_handler::KeybindHandler,
    IshtarSelectable,
};

use self::{
    enums::IshtarMode,
    widgets::{command_interpreter::CommandInterpreter, writeable_area::WriteableArea},
};
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
///Editor instance containing all data required for working and managing buffers
pub struct Ishtar {
    current_path: PathBuf,
    exit: bool,
    logger_area: IshtarLogger,
    cursor: (usize, usize),
    saved_cursor: (usize, usize),
    size: (u16, u16),
    widgets: Vec<Box<dyn IshtarSelectable>>,
    priority: u8,
    mode: IshtarMode,
    clipboard: IshtarClipboard,
}
impl Default for Ishtar {
    fn default() -> Self {
        let size = terminal_size();
        let configs = Self::get_configs();
        let colors = std::sync::Arc::new(configs.colors);
        Self {
            current_path: env::current_dir().unwrap(),
            exit: false,
            cursor: (0, size.1 as usize),
            saved_cursor: (0, size.1 as usize),
            size,
            widgets: vec![
                Box::new(WriteableArea::new_vertical(
                    size.0,
                    size.1 - 1,
                    colors.clone(),
                )),
                Box::new(CommandInterpreter::new(colors.clone())),
                Box::new(KeybindHandler::new(configs.keybinds, colors.clone())),
                Box::new(FileManager::new_vertical(
                    true,
                    env::current_dir().unwrap(),
                    colors,
                )),
            ],
            priority: CommandInterpreter::priority_static(),
            logger_area: IshtarLogger::new().unwrap(),
            clipboard: IshtarClipboard::new(),
            mode: IshtarMode::Cmd,
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
    pub fn set_priority<T: IshtarSelectable>(&mut self) {
        self.priority = T::priority_static();
    }
    pub fn is_priority_of<T: IshtarSelectable>(&self) -> bool {
        self.priority == T::priority_static()
    }
    pub fn set_cursor_at(&mut self, x: usize, y: usize) {
        self.cursor.0 = x;
        self.cursor.1 = y;
    }
    ///Gets the x position of the cursor
    fn x_cursor_position(&self) -> u16 {
        (self.cursor.0
            + match self.mode {
                IshtarMode::Cmd => 0,
                IshtarMode::Modify | IshtarMode::Selection => self.writer().xoffset(),
            }) as u16
    }
    fn cursor_position(&self) -> Position {
        Position::new(self.x_cursor_position(), self.cursor.1 as u16)
    }
    fn find_widget(&self, priority: u8) -> &Box<dyn IshtarSelectable> {
        for idx in 0..self.widgets.len() {
            if self.widgets[idx].priority() == priority {
                return &self.widgets[idx];
            }
        }
        panic!("Failed to find widget with given priority");
    }
    fn find_widget_mut(&mut self, priority: u8) -> &mut Box<dyn IshtarSelectable> {
        for idx in 0..self.widgets.len() {
            if self.widgets[idx].priority() == priority {
                return &mut self.widgets[idx];
            }
        }
        panic!("Failed to find widget with given priority");
    }
    pub fn get_widget<T: IshtarSelectable>(&self) -> &T {
        for idx in 0..self.widgets.len() {
            if self.widgets[idx].priority() == T::priority_static() {
                return self.widgets[idx].downcast_ref::<T>().unwrap();
            }
        }
        panic!("Could not find writeable area, you are dumb")
    }
    pub fn get_widget_mut<T: IshtarSelectable>(&mut self) -> &mut T {
        for idx in 0..self.widgets.len() {
            if self.widgets[idx].priority() == T::priority_static() {
                return self.widgets[idx].downcast_mut::<T>().unwrap();
            }
        }
        panic!("Could not find writeable area, you are dumb")
    }
    pub fn writer(&self) -> &WriteableArea {
        self.get_widget::<WriteableArea>()
    }
    pub fn writer_mut(&mut self) -> &mut WriteableArea {
        self.get_widget_mut::<WriteableArea>()
    }
    pub fn cmd(&self) -> &CommandInterpreter {
        self.get_widget()
    }
    pub fn cmd_mut(&mut self) -> &mut CommandInterpreter {
        self.get_widget_mut()
    }
    pub fn keybind(&self) -> &KeybindHandler {
        self.get_widget()
    }
    pub fn keybind_mut(&mut self) -> &mut KeybindHandler {
        self.get_widget_mut()
    }
    pub fn file_manager_mut(&mut self) -> &mut FileManager {
        self.get_widget_mut()
    }
    ///Updates the cursor position to be on the current writing area
    pub fn update_cursor(&mut self) {
        match self.mode {
            IshtarMode::Cmd => {
                self.cursor.0 = self.cmd().cursor();
                self.cursor.1 = (self.size.1 - 1) as usize;
            }
            IshtarMode::Modify | IshtarMode::Selection => {
                self.cursor = self.writer().cursor();
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
                self.cmd_mut().set("Modifying");
                self.writer_mut().enter_writing();
                let x = self.saved_cursor.0;
                let y = self.saved_cursor.1;
                self.writer_mut().set_cursor_x(x);
                self.writer_mut().set_cursor_y(y);
                self.set_priority::<WriteableArea>();
            }
            IshtarMode::Cmd => {
                self.cmd_mut().clear();

                self.set_priority::<CommandInterpreter>();
            }
            IshtarMode::Selection => {
                self.cmd_mut().set("Selection");
                self.writer_mut().enter_selection();
                self.set_priority::<WriteableArea>();
            }
        }
        self.mode = mode;
    }
    ///Saves the content of the current area at current_path + current_area_file
    pub fn save_file(&self) -> std::io::Result<()> {
        self.writer().save(&self.current_path)?;
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
        self.saved_cursor = self.writer().cursor();
    }
    pub fn exec_cmd(&mut self, cmd: &String) -> std::io::Result<ExitStatus> {
        std::process::Command::new(cmd)
            .spawn()
            .map(|mut child| child.wait())?
    }
    pub fn request_search(&mut self) {
        self.set_priority::<FileManager>();
        self.file_manager_mut().mode = ManagingMode::Searching;
        self.file_manager_mut().open();
    }
    pub fn handle_task(&mut self, task: &CmdTask) {
        match task {
            CmdTask::SetPriority(n) => self.priority = *n,
            CmdTask::CopySelection | CmdTask::CopyToSys | CmdTask::CopyToEditor => {
                let data = self.writer_mut().get_selection();
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
                self.writer_mut().goto_init_of_line();
                self.change_mode(IshtarMode::Selection);
                self.writer_mut().goto_end_of_line();
            }
            CmdTask::PasteSys | CmdTask::PasteEditor => {
                let content = if matches!(task, CmdTask::PasteEditor) {
                    self.clipboard.get_virtual().clone()
                } else {
                    self.clipboard.get()
                };
                let data = self.writer_mut().paste(&content);
                self.handle_task(&data);
            }

            CmdTask::DeleteLine => self.writer_mut().delete_line(),

            CmdTask::SavePos => self.save_position(),
            CmdTask::MoveSaved => self.set_cursor_at(self.saved_cursor.0, self.saved_cursor.1),

            CmdTask::Write(content) => {
                self.writer_mut().paste(content);
            }
            CmdTask::CreateWindow => {
                self.writer_mut().create_area();
            }
            CmdTask::DeleteWindow => {
                self.writer_mut().delete_current_area();
            }
            CmdTask::SetWindowUp => self.writer_mut().set_focus_back(),
            CmdTask::SetWindowDown => self.writer_mut().set_focus_next(),

            CmdTask::ExecCmd(cmd) => {
                let _ = self.exec_cmd(cmd);
            }
            CmdTask::ExecutePrompt(prompt) => {
                self.cmd_mut().execute_cmd(prompt);
            }
            CmdTask::MoveIOL => self.writer_mut().goto_init_of_line(),
            CmdTask::MoveEOL => self.writer_mut().goto_end_of_line(),
            CmdTask::MoveIOB => self.writer_mut().goto_init_of_file(),
            CmdTask::MoveEOB => self.writer_mut().goto_end_of_file(),
            CmdTask::MoveToLine(n) => self.writer_mut().move_y(*n as i16),
            CmdTask::MoveToRow(n) => self.writer_mut().move_x(*n as i16),
            CmdTask::EnterNormal => self.change_mode(IshtarMode::Cmd),
            CmdTask::EnterModify => self.change_mode(IshtarMode::Modify),
            CmdTask::EnterSelection => self.change_mode(IshtarMode::Selection),

            CmdTask::SaveFile => self.writer().save(&self.current_path).unwrap(),
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
            CmdTask::RequestSearchFile => self.request_search(),
            CmdTask::Exit => self.exit = true,
            CmdTask::Null | CmdTask::Continue => {}
            e => panic!("Must implement {e:?}"),
        }
    }
    fn handle_tasks(&mut self, tasks: &Vec<ConfigStatment>) {
        for task in tasks {
            match task {
                ConfigStatment::Task(t) => self.handle_task(t),
                ConfigStatment::Cmd(c) => {
                    if let Some(tasks) = self.keybind().get(c, self.mode_id()) {
                        self.handle_tasks(&tasks.clone());
                    }
                }
                _ => {}
            }
        }
    }
    ///Handles keybind input. Returns CmdTask::Null if it the caller, handler_key, must stop its
    ///execution; CmdTask::Continue otherwhite
    ///The returned message does not have relationship with the modification made.
    fn should_init_keybind(&mut self, key: KeyEvent) -> CmdTask {
        // If got modifier start listening
        if !key.modifiers.is_empty() && !self.keybind_mut().listening() {
            let old_priority = self.priority;
            self.set_priority::<KeybindHandler>();
            self.keybind_mut().start_listening(key.code, key.modifiers);
            let content = self.keybind().content();
            if let Some(tasks) = self.keybind().get(&content, self.mode_id()).cloned() {
                self.handle_tasks(&tasks);
                self.keybind_mut().stop_listening();
                self.priority = old_priority;
            }
            return CmdTask::Null;
        }
        //Dont input but listens to key
        if self.is_priority_of::<KeybindHandler>() {
            self.keybind_mut().current_mode = self.mode_id();
            match self.keybind_mut().keydown(key.code) {
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
                self.writer_mut().write_char(c);
                return;
            }
        }
        if let CmdTask::Null = self.should_init_keybind(key) {
            return;
        }
        let widget = self.find_widget_mut(self.priority);
        let task = widget.keydown(key.code);
        self.handle_task(&task);
        match self.mode {
            IshtarMode::Modify | IshtarMode::Selection => {
                self.cursor = self.writer().cursor();
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
        for i in 0..self.widgets.len() {
            if self.widgets[i].can_render() {
                self.widgets[i].renderize(area, buf);
            }
        }
    }
}
