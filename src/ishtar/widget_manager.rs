use crate::helpers::terminal_size;

use crate::widgets::{
    file_manager::FileManager,
    keybind_handler::{KeybindHandler, Keybinds},
    CommandInterpreter, IshtarSelectable, WriteableArea,
};

use super::Ishtar;

/// Editor configuration loaded at startup.
pub struct IshtarConfig {
    /// Theme color map: key name → 0xRRGGBB value.
    pub colors: std::collections::HashMap<String, u32>,
    /// Registered keybinds: sequence string → per-mode task lists.
    pub keybinds: Keybinds,
}

impl Default for IshtarConfig {
    fn default() -> Self {
        Self {
            colors: std::collections::HashMap::new(),
            keybinds: Keybinds::new(),
        }
    }
}

impl Ishtar {
    /// Loads editor configuration from `~/.config/ishtar/config.isht`.
    /// Falls back to defaults if the file is missing or cannot be parsed.
    pub fn get_configs() -> IshtarConfig {
        IshtarConfig::default()
    }
}

/// Owns and provides typed access to all editor widgets.
pub struct WidgetManager {
    /// All registered widgets, rendered and dispatched in order.
    pub widgets: Vec<Box<dyn IshtarSelectable>>,
}

impl WidgetManager {
    pub fn new() -> Self {
        let size = terminal_size();
        let configs = Ishtar::get_configs();
        let colors = std::sync::Arc::new(configs.colors);
        Self {
            widgets: vec![
                Box::new(WriteableArea::new_vertical(size.0, size.1 - 1, colors.clone())),
                Box::new(CommandInterpreter::new(colors.clone())),
                Box::new(KeybindHandler::new(configs.keybinds, colors.clone())),
                Box::new(FileManager::new_vertical(
                    true,
                    std::env::current_dir().unwrap(),
                    colors,
                )),
            ],
        }
    }

    /// Returns a reference to the widget of type `T`, or `None` if not registered.
    pub fn get_widget<T: IshtarSelectable>(&self) -> Option<&T> {
        self.widgets
            .iter()
            .find_map(|w| w.downcast_ref::<T>())
    }

    /// Returns a mutable reference to the widget of type `T`, or `None` if not registered.
    pub fn get_widget_mut<T: IshtarSelectable>(&mut self) -> Option<&mut T> {
        self.widgets
            .iter_mut()
            .find_map(|w| w.downcast_mut::<T>())
    }

    /// Convenience accessor for the text editing area.
    ///
    /// # Panics
    /// Panics if `WriteableArea` is not registered (should never happen in normal use).
    pub fn writer(&self) -> &WriteableArea {
        self.get_widget::<WriteableArea>().expect("WriteableArea not found")
    }

    /// Mutable convenience accessor for the text editing area.
    pub fn writer_mut(&mut self) -> &mut WriteableArea {
        self.get_widget_mut::<WriteableArea>().expect("WriteableArea not found")
    }

    /// Convenience accessor for the command interpreter.
    pub fn cmd(&self) -> &CommandInterpreter {
        self.get_widget::<CommandInterpreter>().expect("CommandInterpreter not found")
    }

    /// Mutable convenience accessor for the command interpreter.
    pub fn cmd_mut(&mut self) -> &mut CommandInterpreter {
        self.get_widget_mut::<CommandInterpreter>().expect("CommandInterpreter not found")
    }

    /// Convenience accessor for the keybind handler.
    pub fn keybind(&self) -> &KeybindHandler {
        self.get_widget::<KeybindHandler>().expect("KeybindHandler not found")
    }

    /// Mutable convenience accessor for the keybind handler.
    pub fn keybind_mut(&mut self) -> &mut KeybindHandler {
        self.get_widget_mut::<KeybindHandler>().expect("KeybindHandler not found")
    }

    /// Convenience accessor for the file manager.
    pub fn file_manager(&self) -> &FileManager {
        self.get_widget::<FileManager>().expect("FileManager not found")
    }

    /// Mutable convenience accessor for the file manager.
    pub fn file_manager_mut(&mut self) -> &mut FileManager {
        self.get_widget_mut::<FileManager>().expect("FileManager not found")
    }
}
