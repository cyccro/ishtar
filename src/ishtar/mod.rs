mod logger;
mod tasks;
mod widget_manager;

use crate::widgets::{
    IshtarClipboard, IshtarCursor, IshtarMode, IshtarModeManager, IshtarSelectable,
};
use logger::{IshtarLogger, LogLevel};
use std::{
    env,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};
use widget_manager::WidgetManager;

use crate::{
    helpers::Vec2,
    plugins::{PluginCmd, PluginManager},
    widgets::CmdTask,
};

use ratatui::{
    Frame,
    crossterm::event::{self, KeyCode, KeyEvent},
    init,
    layout::Position,
};

/// Main editor state: owns all widgets, the cursor, mode manager, clipboard, logger, and plugins.
pub struct Ishtar {
    exit: bool,
    current_path: PathBuf,
    logger_area: IshtarLogger,
    cursor: IshtarCursor,
    widgets_manager: WidgetManager,
    mode: IshtarModeManager,
    clipboard: IshtarClipboard,
    plugin_manager: PluginManager,
    /// Accumulated key sequence for plugin keybinds (space leader).
    plugin_seq: Option<Vec<String>>,
}

impl Default for Ishtar {
    fn default() -> Self {
        let pm = PluginManager::new().unwrap();
        Self {
            exit: false,
            current_path: env::current_dir().unwrap(),
            cursor: IshtarCursor::new(),
            logger_area: IshtarLogger::new().unwrap(),
            clipboard: IshtarClipboard::new(),
            mode: IshtarModeManager::new(),
            widgets_manager: WidgetManager::new(),
            plugin_manager: pm,
            plugin_seq: None,
        }
    }
}

impl Ishtar {
    pub fn new() -> Self {
        let mut out = Self::default();
        let errors = out.plugin_manager.load_plgins(Path::new("./plugin"));
        for (error, path) in errors {
            out.logger_area
                .queue(&format!("When reading path {path:?}, received: {error:?}"));
        }
        out.logger_area.flush(LogLevel::Error);

        out
    }
    /// Returns the cursor position as a Ratatui `Position`.
    #[inline]
    fn cursor_position(&self) -> Position {
        Position::new(self.cursor.cursor().x(), self.cursor.cursor().y())
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

    /// Saves the current cursor position for later restoration.
    pub fn save_position(&mut self) {
        self.cursor.save();
    }

    /// Transitions the editor to `mode`, updating the relevant widget state.
    fn change_mode(&mut self, mode: IshtarMode) {
        match mode {
            IshtarMode::Modify | IshtarMode::Selection => {
                self.widgets_manager.cmd_mut().set(&format!("{mode:?}"));
                let x = self.cursor.saved_cursor().x();
                let y = self.cursor.saved_cursor().y();
                let writer = self.widgets_manager.writer_mut();
                if matches!(mode, IshtarMode::Modify) {
                    writer.enter_writing();
                } else {
                    writer.enter_selection();
                }
                writer.set_cursor_x(x as usize);
                writer.set_cursor_y(y as usize);
                self.widgets_manager
                    .set_focus(WidgetManager::WRITEABLE_INDEX);
            }
            IshtarMode::Cmd => {
                self.save_position();
                self.widgets_manager.cmd_mut().clear();
                self.widgets_manager
                    .set_focus(WidgetManager::COMMAND_INTERPRETER_INDEX);
            }
        }
        self.mode.goto_mode(mode);
    }

    /// Checks whether a modifier-initiated keybind sequence should start.
    ///
    /// Returns `CmdTask::Null` if the event was consumed, `CmdTask::Continue` otherwise.
    fn should_init_keybind(&mut self, key: KeyEvent) -> CmdTask {
        if !key.modifiers.is_empty() && !self.widgets_manager.keybind().listening() {
            self.widgets_manager
                .keybind_mut()
                .start_listening(key.code, key.modifiers);
            let content = self.widgets_manager.keybind().content();
            if let Some(task) = self
                .widgets_manager
                .keybind()
                .get(&content, self.mode.current_mode())
            {
                self.handle_task(&task.clone()).unwrap();

                self.widgets_manager.keybind_mut().stop_listening();
            }
            return CmdTask::Null;
        }
        CmdTask::Continue
    }

    /// Checks for plugin keybind sequences (space leader).
    /// Returns `true` if the event was consumed.
    fn handle_plugin_keybind(&mut self, key: KeyEvent) -> bool {
        // Don't capture in insert mode so typing still works.
        if matches!(self.mode.current_mode(), IshtarMode::Modify) {
            return false;
        }

        if let Some(ref mut seq) = self.plugin_seq {
            let s = Self::key_to_string(key.code);
            seq.push(s);
            let joined = seq.join("+");
            if let Some(cmds) = self.plugin_manager.dispatch(&joined) {
                self.plugin_seq = None;
                for pc in cmds {
                    if let Some(task) = Self::plugin_cmd_to_task(pc) {
                        let _ = self.handle_task(&task);
                    }
                }
                return true;
            }
            if !self.plugin_manager.has_prefix(&joined) {
                self.plugin_seq = None; // dead sequence
            }
            return true;
        }

        // Only start a sequence on space (leader key).
        if key.code == KeyCode::Char(' ') && key.modifiers.is_empty() {
            let s = Self::key_to_string(key.code);
            if self.plugin_manager.has_prefix(&s) {
                self.plugin_seq = Some(vec![s]);
                return true;
            }
        }
        false
    }

    /// Routes a key event to the focused widget and updates cursor state.
    fn handle_key(&mut self, key: KeyEvent) {
        // Plugin keybind sequences take priority.
        if self.handle_plugin_keybind(key) {
            return;
        }

        if let CmdTask::Null = self.should_init_keybind(key) {
            return;
        }

        let task = self.widgets_manager.keydown(key.code);
        let _ = self.handle_task(&task);

        self.sync_cursor();
    }

    /// Convert a `KeyCode` to the string used in plugin keybind patterns.
    fn key_to_string(kc: KeyCode) -> String {
        match kc {
            KeyCode::Char(' ') => "space".into(),
            KeyCode::Char(c) => c.to_string(),
            other => format!("{other:?}"),
        }
    }

    /// Convert a `PluginCmd` to `CmdTask` for execution.
    fn plugin_cmd_to_task(pc: PluginCmd) -> Option<CmdTask> {
        match pc {
            PluginCmd::Null => None,
            PluginCmd::Exit => Some(CmdTask::Exit),
            PluginCmd::Write(s) => Some(CmdTask::Write(s)),
        }
    }

    /// Synchronises the terminal cursor position from the focused widget.
    fn sync_cursor(&mut self) {
        if let Some((cx, cy)) = self.widgets_manager.cursor() {
            self.display("Corno", LogLevel::Info);
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
        self.widgets_manager.renderize(frame, area);
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
