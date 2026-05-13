use std::ops::{Deref, DerefMut};

use ratatui::widgets::Widget;

use crate::helpers::terminal_size;

use crate::widgets::{
    CommandInterpreter, FocusSide, IshtarSelectable, WriteableArea, file_manager::FileManager,
    keybind_handler::KeybindHandler,
};

/// Owns and provides typed access to all editor widgets.
pub struct WidgetManager {
    writeable: WriteableArea,
    command_interpreter: CommandInterpreter,
    keybinds: KeybindHandler,
    /// All registered widgets, rendered and dispatched in order.
    widgets: Vec<Box<dyn IshtarSelectable>>,
    /// Index of the currently focused widget.
    focused: usize,
}

impl WidgetManager {
    pub const WRITEABLE_INDEX: usize = 0;
    pub const COMMAND_INTERPRETER_INDEX: usize = 1;
    pub const KEYBINDS_INDEX: usize = 2;
    pub fn new() -> Self {
        let size = terminal_size();
        Self {
            writeable: WriteableArea::new_vertical(size.0, size.1 - 1),
            command_interpreter: CommandInterpreter::new(size.0, size.1),
            keybinds: KeybindHandler::new(),
            widgets: vec![],
            focused: 0,
        }
    }

    /// Advance focus to the next widget (wraps around).
    pub fn focus_next(&mut self) {
        let len = self.widgets.len();
        self.focused = (self.focused + 1) % len;
    }

    /// Move focus to the previous widget (wraps around).
    pub fn focus_previous(&mut self) {
        self.focused = if self.focused == 0 {
            self.widgets.len() - 1
        } else {
            self.focused - 1
        };
    }

    /// Set focus to a specific widget index.
    pub fn set_focus(&mut self, index: usize) {
        self.focused = index;
    }

    /// Move focus in a spatial direction based on widget bounding rects.
    pub fn focus_direction(&mut self, side: FocusSide) {
        let Some(current) = self.widgets[self.focused].area() else {
            return;
        };
        let cx = (current.x + current.width / 2) as f32;
        let cy = (current.y + current.height / 2) as f32;

        let mut best = None;
        let mut best_dist = f32::MAX;

        for (i, w) in self.widgets.iter().enumerate() {
            if i == self.focused {
                continue;
            }
            let Some(rect) = w.area() else {
                continue;
            };

            let tx = (rect.x + rect.width / 2) as f32;
            let ty = (rect.y + rect.height / 2) as f32;
            let dx = tx - cx;
            let dy = ty - cy;

            let dist = match side {
                FocusSide::Right => {
                    if dx <= 0.0 {
                        continue;
                    }
                    dx + dy.abs()
                }
                FocusSide::Left => {
                    if dx >= 0.0 {
                        continue;
                    }
                    -dx + dy.abs()
                }
                FocusSide::Down => {
                    if dy <= 0.0 {
                        continue;
                    }
                    dy + dx.abs()
                }
                FocusSide::Up => {
                    if dy >= 0.0 {
                        continue;
                    }
                    -dy + dx.abs()
                }
            };

            if dist < best_dist {
                best_dist = dist;
                best = Some(i);
            }
        }

        if let Some(idx) = best {
            self.focused = idx;
        }
    }

    /// Convenience accessor for the text editing area.
    ///
    /// # Panics
    /// Panics if `WriteableArea` is not registered (should never happen in normal use).
    pub fn writer(&self) -> &WriteableArea {
        &self.writeable
    }

    /// Mutable convenience accessor for the text editing area.
    pub fn writer_mut(&mut self) -> &mut WriteableArea {
        &mut self.writeable
    }
    /// Convenience accessor for the command interpreter.
    pub fn cmd(&self) -> &CommandInterpreter {
        &self.command_interpreter
    }

    /// Mutable convenience accessor for the command interpreter.
    pub fn cmd_mut(&mut self) -> &mut CommandInterpreter {
        &mut self.command_interpreter
    }

    /// Convenience accessor for the keybind handler.
    pub fn keybind(&self) -> &KeybindHandler {
        &self.keybinds
    }

    /// Mutable convenience accessor for the keybind handler.
    pub fn keybind_mut(&mut self) -> &mut KeybindHandler {
        &mut self.keybinds
    }
}

impl IshtarSelectable for WidgetManager {
    fn cursor(&self) -> Option<(usize, usize)> {
        (**self).cursor()
    }

    fn keydown(&mut self, key: ratatui::crossterm::event::KeyCode) -> crate::widgets::CmdTask {
        match self.focused {
            Self::WRITEABLE_INDEX => self.writeable.keydown(key),
            Self::COMMAND_INTERPRETER_INDEX => self.command_interpreter.keydown(key),
            Self::KEYBINDS_INDEX => self.keybinds.keydown(key),
            _ => self.widgets[self.focused - 3].keydown(key),
        }
    }
    fn can_render(&self) -> bool {
        true
    }
    fn keydown(&mut self, key: ratatui::crossterm::event::KeyCode) -> crate::widgets::CmdTask {
        match self.focused {
            Self::WRITEABLE_INDEX => self.writeable.keydown(key),
            Self::COMMAND_INTERPRETER_INDEX => self.command_interpreter.keydown(key),
            Self::KEYBINDS_INDEX => self.keybinds.keydown(key),
            _ => self.widgets[self.focused - 3].keydown(key),
        }
    }
    fn can_render(&self) -> bool {
        true
    }
    fn renderize(&self, f: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        if self.writeable.can_render() {
            self.writeable.render(area, f.buffer_mut());
        }
        if self.command_interpreter.can_render() {
            self.command_interpreter.render(area, f.buffer_mut());
        }
        if self.keybinds.can_render() {
            self.keybinds.render(area, f.buffer_mut());
        }
        for widget in &self.widgets {
            if widget.can_render() {
                widget.renderize(f, area);
            }
        }
    }
}

impl Deref for WidgetManager {
    type Target = dyn IshtarSelectable;
    fn deref(&self) -> &Self::Target {
        match self.focused {
            Self::WRITEABLE_INDEX => &self.writeable,
            Self::COMMAND_INTERPRETER_INDEX => &self.command_interpreter,
            Self::KEYBINDS_INDEX => &self.keybinds,
            n => &*self.widgets[n],
        }
    }
}
impl DerefMut for WidgetManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.focused {
            Self::WRITEABLE_INDEX => &mut self.writeable,
            Self::COMMAND_INTERPRETER_INDEX => &mut self.command_interpreter,
            Self::KEYBINDS_INDEX => &mut self.keybinds,
            n => &mut *self.widgets[n],
        }
    }
}
