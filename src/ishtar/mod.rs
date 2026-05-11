mod enums;
mod logger;
mod widget_manager;

use crate::widgets::{
    IshtarClipboard, IshtarCursor, IshtarMode, IshtarModeManager, IshtarSelectable,
};
use logger::{IshtarLogger, LogLevel};
use std::{
    env,
    error::Error,
    ops::{Deref, DerefMut},
    path::PathBuf,
    process::ExitStatus,
};
use widget_manager::WidgetManager;

use crate::{
    helpers::{terminal_size, Vec2},
    widgets::{
        file_manager::ManagingMode,
        CmdTask,
    },
};

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers},
    init,
    layout::Position,
    Frame,
};

/// Main editor state: owns all widgets, the cursor, mode manager, clipboard, and logger.
pub struct Ishtar {
    exit: bool,
    size: (u16, u16),
    current_path: PathBuf,
    logger_area: IshtarLogger,
    cursor: IshtarCursor,
    handler: WidgetManager,
    mode: IshtarModeManager,
    clipboard: IshtarClipboard,
}

impl Default for Ishtar {
    fn default() -> Self {
        let size = terminal_size();
        Self {
            size,
            exit: false,
            current_path: env::current_dir().unwrap(),
            cursor: IshtarCursor::new(),
            logger_area: IshtarLogger::new().unwrap(),
            clipboard: IshtarClipboard::new(),
            mode: IshtarModeManager::new(),
            handler: WidgetManager::new(),
        }
    }
}

impl Ishtar {
    pub fn new() -> Self {
        Self::default()
    }

    fn draw(&mut self, f: &mut Frame) {
        f.set_cursor_position(self.cursor_position());
        self.render_widgets(f);
    }

    /// Initializes the terminal and runs the main event loop until the user exits.
    pub fn run(&mut self) -> std::io::Result<()> {
        let display = self.current_path.clone();
        self.display(display.display(), LogLevel::Info);
        self.display("Initialize Process", LogLevel::Info);
        let mut terminal = init();
        terminal.show_cursor()?;
        loop {
            if self.exit {
                break;
            }
            terminal.draw(|f| self.draw(f))?;
            self.handle_event()?;
        }
        ratatui::restore();
        Ok(())
    }

    /// Moves the terminal cursor to `(x, y)`.
    #[inline]
    pub fn set_cursor_at(&mut self, x: u16, y: u16) {
        self.cursor.set_cursor(Vec2::new(x, y));
    }

    /// Returns the cursor position as a Ratatui `Position`.
    #[inline]
    fn cursor_position(&self) -> Position {
        Position::new(self.cursor.cursor().x(), self.cursor.cursor().y())
    }

    /// Saves the current cursor position for later restoration.
    pub fn save_position(&mut self) {
        self.cursor.save();
    }

    /// Returns the current mode as a `usize` index (used for keybind lookup).
    fn mode_id(&self) -> usize {
        match self.mode.current_mode() {
            IshtarMode::Cmd => 0,
            IshtarMode::Modify => 1,
            IshtarMode::Selection => 2,
        }
    }

    /// Transitions the editor to `mode`, updating the relevant widget state.
    fn change_mode(&mut self, mode: IshtarMode) {
        match mode {
            IshtarMode::Modify | IshtarMode::Selection => {
                self.handler.cmd_mut().set(&format!("{mode:?}"));
                let x = self.cursor.saved_cursor().x();
                let y = self.cursor.saved_cursor().y();
                let writer = self.handler.writer_mut();
                if matches!(mode, IshtarMode::Modify) {
                    writer.enter_writing();
                } else {
                    writer.enter_selection();
                }
                writer.set_cursor_x(x as usize);
                writer.set_cursor_y(y as usize);
            }
            IshtarMode::Cmd => {
                self.save_position();
                self.handler.cmd_mut().clear();
            }
        }
        self.mode.goto_mode(mode);
    }

    /// Saves the active buffer to `current_path / <file_name>`.
    pub fn save_file(&self) -> std::io::Result<()> {
        self.handler.writer().save(&self.current_path)?;
        Ok(())
    }

    /// Spawns `cmd` as a child process and waits for it to finish.
    pub fn exec_cmd(&mut self, cmd: &str) -> std::io::Result<ExitStatus> {
        std::process::Command::new(cmd)
            .spawn()
            .map(|mut child| child.wait())?
    }

    /// Opens the file-search widget. If `reset_dir` is `true`, resets the search root.
    pub fn request_search(&mut self, reset_dir: bool) {
        let file_manager = self.handler.file_manager_mut();
        file_manager.mode = ManagingMode::Searching;
        if reset_dir {
            file_manager.update_searcher_dir(&self.current_path);
        }
        self.handler.file_manager_mut().open();
    }

    /// Closes the file-search widget.
    pub fn stop_search(&mut self) {
        self.handler.file_manager_mut().close();
    }

    /// Executes a list of tasks in order.
    pub fn handle_tasks(&mut self, tasks: &[CmdTask]) {
        // Collect to avoid borrow issues when tasks reference self.
        let tasks: Vec<CmdTask> = tasks.to_vec();
        for task in &tasks {
            let _ = self.handle_task(task);
        }
    }

    /// Dispatches a single `CmdTask`.
    ///
    /// `Null` and `Continue` are no-ops. All other variants are handled here or
    /// delegated to the appropriate widget.
    pub fn handle_task(&mut self, task: &CmdTask) -> Result<(), Box<dyn Error + Send + Sync>> {
        if matches!(task, CmdTask::Null | CmdTask::Continue) {
            return Ok(());
        }
        self.display(format!("{task:?}"), LogLevel::Info);
        match task {
            CmdTask::SaveMode => self.mode.save_mode(),
            CmdTask::ReturnSavedMode => self.mode.goto_saved(),

            CmdTask::CopySelection | CmdTask::CopyToSys | CmdTask::CopyToEditor => {
                let Some(data) = self.handler.writer_mut().get_selection() else {
                    self.handle_task(&CmdTask::EnterModify)?;
                    return Ok(());
                };
                match task {
                    CmdTask::CopyToSys | CmdTask::CopySelection => {
                        self.clipboard.set(data)?;
                    }
                    _ => self.clipboard.set_virtual(data),
                }
                self.handle_task(&CmdTask::EnterModify)?;
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
                    self.clipboard.get()?
                };
                let data = self.handler.writer_mut().paste(&content);
                self.handle_task(&data)?;
            }

            CmdTask::DeleteLine => self.handler.writer_mut().delete_line(),

            CmdTask::SavePos => self.save_position(),
            CmdTask::MoveSaved => self.cursor.goto_saved(),

            CmdTask::Write(content) => {
                self.handler.writer_mut().paste(content);
            }
            CmdTask::CreateWindow => self.handler.writer_mut().create_area(),
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

            CmdTask::ModifyFile(f) => self.handler.writer_mut().open_file(f.into()),

            CmdTask::SaveFile => {
                if self.handler.writer().file_name().is_some() {
                    self.handler.writer().save(&self.current_path)?;
                } else {
                    self.handler
                        .cmd_mut()
                        .request_data("Give the file a name ", CmdTask::ReqSaveFile);
                }
            }
            CmdTask::SaveFileAs(msg) => {
                let writer = self.handler.writer_mut();
                writer.modify_file_name(msg);
                writer.save(&self.current_path)?;
            }

            CmdTask::Multi(tasks) => {
                for task in tasks {
                    self.handle_task(task)?;
                }
            }

            CmdTask::Log(s) => { self.display(s, LogLevel::Info); }
            CmdTask::Warn(s) => { self.display(s, LogLevel::Warn); }

            CmdTask::ReqSearchRoot => self.request_search(true),
            CmdTask::StopSearch => self.stop_search(),
            CmdTask::Exit => self.exit = true,

            // Tasks that are handled by widgets directly or not yet implemented.
            other => {
                self.display(format!("Unhandled task: {other:?}"), LogLevel::Warn);
            }
        }
        Ok(())
    }

    /// Checks whether a modifier-initiated keybind sequence should start.
    ///
    /// Returns `CmdTask::Null` if the event was consumed, `CmdTask::Continue` otherwise.
    fn should_init_keybind(&mut self, key: KeyEvent) -> CmdTask {
        if !key.modifiers.is_empty() && !self.handler.keybind().listening() {
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
        CmdTask::Continue
    }

    /// Routes a key event to the focused widget and updates cursor state.
    fn handle_key(&mut self, key: KeyEvent) {
        // Uppercase letters in Modify mode are written directly.
        if let KeyCode::Char(c) = key.code {
            if c.is_uppercase() && key.modifiers == KeyModifiers::SHIFT {
                self.handler.writer_mut().write_char(c);
                return;
            }
        }

        if let CmdTask::Null = self.should_init_keybind(key) {
            return;
        }

        let task = self.handler.writer_mut().keydown(key.code);
        let _ = self.handle_task(&task);

        // Keep cursor in sync with the text area after every keystroke.
        if matches!(
            self.mode.current_mode(),
            IshtarMode::Modify | IshtarMode::Selection
        ) {
            let (cx, cy) = self.handler.writer().cursor();
            self.cursor.set_cursor(Vec2::new(cx as u16, cy as u16));
        }
    }

    /// Reads one terminal event and dispatches it.
    pub fn handle_event(&mut self) -> std::io::Result<()> {
        if let event::Event::Key(k) = event::read()? {
            self.handle_key(k);
        }
        Ok(())
    }

    /// Renders all widgets that report `can_render() == true`.
    pub fn render_widgets(&mut self, frame: &mut Frame) {
        let area = frame.area();
        for i in 0..self.handler.widgets.len() {
            if self.handler.widgets[i].can_render() {
                self.handler.widgets[i].renderize(frame, area);
            }
        }
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
