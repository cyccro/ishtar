use downcast_rs::{impl_downcast, Downcast};
use isht::CmdTask;
use ratatui::{crossterm::event::KeyCode, layout::Rect, Frame};

pub trait IshtarSelectable: Downcast {
    fn priority_static() -> u8
    where
        Self: Sized;
    fn priority(&self) -> u8;
    fn keydown(&mut self, key: KeyCode) -> CmdTask;
    fn can_render(&self) -> bool {
        true
    }
    fn renderize(&self, f: &mut Frame, area: Rect);
}
impl_downcast!(IshtarSelectable);

pub mod clipboard;
pub mod command_interpreter;
pub mod file_manager;
pub mod keybind_handler;
pub mod popup;
pub mod text_area;
pub mod writeable_area;
