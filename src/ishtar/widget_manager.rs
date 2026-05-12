use crate::helpers::terminal_size;

use crate::widgets::{
    CommandInterpreter, FocusSide, IshtarSelectable, WriteableArea,
    file_manager::FileManager,
    keybind_handler::{KeybindHandler, Keybinds},
};

use super::Ishtar;

/// Owns and provides typed access to all editor widgets.
pub struct WidgetManager {
    /// All registered widgets, rendered and dispatched in order.
    pub widgets: Vec<Box<dyn IshtarSelectable>>,
    /// Index of the currently focused widget.
    pub focused: usize,
}

impl WidgetManager {
    pub fn new() -> Self {
        let size = terminal_size();
        Self {
            widgets: vec![
                Box::new(WriteableArea::new_vertical(size.0, size.1 - 1)),
                Box::new(CommandInterpreter::new(size.0, size.1)),
                Box::new(KeybindHandler::new(Keybinds::new())),
                Box::new(FileManager::new(true, std::env::current_dir().unwrap())),
            ],
            focused: 0,
        }
    }

    /// Returns a mutable reference to the currently focused widget.
    pub fn focused_mut(&mut self) -> &mut Box<dyn IshtarSelectable> {
        &mut self.widgets[self.focused]
    }

    /// Advance focus to the next widget (wraps around).
    pub fn focus_next(&mut self) {
        let len = self.widgets.len();
        self.focused = (self.focused + 1) % len;
    }

    /// Move focus to the previous widget (wraps around).
    pub fn focus_previous(&mut self) {
        let len = self.widgets.len();
        self.focused = if self.focused == 0 {
            len - 1
        } else {
            self.focused - 1
        };
    }

    /// Set focus to a specific widget index.
    pub fn set_focus(&mut self, index: usize) {
        if index < self.widgets.len() {
            self.focused = index;
        }
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

    /// Returns a reference to the widget of type `T`, or `None` if not registered.
    pub fn get_widget<T: IshtarSelectable>(&self) -> Option<&T> {
        self.widgets.iter().find_map(|w| w.downcast_ref::<T>())
    }

    /// Returns a mutable reference to the widget of type `T`, or `None` if not registered.
    pub fn get_widget_mut<T: IshtarSelectable>(&mut self) -> Option<&mut T> {
        self.widgets.iter_mut().find_map(|w| w.downcast_mut::<T>())
    }

    /// Convenience accessor for the text editing area.
    ///
    /// # Panics
    /// Panics if `WriteableArea` is not registered (should never happen in normal use).
    pub fn writer(&self) -> &WriteableArea {
        self.get_widget::<WriteableArea>()
            .expect("WriteableArea not found")
    }

    /// Mutable convenience accessor for the text editing area.
    pub fn writer_mut(&mut self) -> &mut WriteableArea {
        self.get_widget_mut::<WriteableArea>()
            .expect("WriteableArea not found")
    }

    pub fn set_focus_to<T: IshtarSelectable>(&mut self) {
        let id = self
            .widgets
            .iter()
            .position(|p| p.downcast_ref::<T>().is_some())
            .unwrap_or(2);
        self.focused = id; //3 == command interpreter
    }

    /// Convenience accessor for the command interpreter.
    pub fn cmd(&self) -> &CommandInterpreter {
        self.get_widget::<CommandInterpreter>()
            .expect("CommandInterpreter not found")
    }

    /// Mutable convenience accessor for the command interpreter.
    pub fn cmd_mut(&mut self) -> &mut CommandInterpreter {
        self.get_widget_mut::<CommandInterpreter>()
            .expect("CommandInterpreter not found")
    }

    /// Convenience accessor for the keybind handler.
    pub fn keybind(&self) -> &KeybindHandler {
        self.get_widget::<KeybindHandler>()
            .expect("KeybindHandler not found")
    }

    /// Mutable convenience accessor for the keybind handler.
    pub fn keybind_mut(&mut self) -> &mut KeybindHandler {
        self.get_widget_mut::<KeybindHandler>()
            .expect("KeybindHandler not found")
    }

    /// Convenience accessor for the file manager.
    pub fn file_manager(&self) -> &FileManager {
        self.get_widget::<FileManager>()
            .expect("FileManager not found")
    }

    /// Mutable convenience accessor for the file manager.
    pub fn file_manager_mut(&mut self) -> &mut FileManager {
        self.get_widget_mut::<FileManager>()
            .expect("FileManager not found")
    }
}
