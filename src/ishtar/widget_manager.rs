use crate::helpers::terminal_size;

use super::{
    widgets::{
        command_interpreter::CommandInterpreter, file_manager::FileManager,
        keybind_handler::KeybindHandler, writeable_area::WriteableArea, IshtarSelectable,
    },
    Ishtar,
};

///A struct made simply for managing the widgets inside ishtar
pub struct WidgetManager {
    pub widgets: Vec<Box<dyn IshtarSelectable>>,
}
impl WidgetManager {
    pub fn new() -> Self {
        let size = terminal_size();
        let configs = Ishtar::get_configs();
        let colors = std::sync::Arc::new(configs.colors);
        Self {
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
                    std::env::current_dir().unwrap(),
                    colors,
                )),
            ],
        }
    }
    fn find_widget(&self, priority: u8) -> &Box<dyn IshtarSelectable> {
        for idx in 0..self.widgets.len() {
            if self.widgets[idx].priority() == priority {
                return &self.widgets[idx];
            }
        }
        panic!("Failed to find widget with given priority");
    }
    pub fn find_widget_mut(&mut self, priority: u8) -> &mut Box<dyn IshtarSelectable> {
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
    pub fn file_manager(&self) -> &FileManager {
        self.get_widget()
    }
    pub fn file_manager_mut(&mut self) -> &mut FileManager {
        self.get_widget_mut()
    }
}
