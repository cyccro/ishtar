use downcast_rs::{impl_downcast, Downcast};
use isht::CmdTask;
use ratatui::{buffer::Buffer, crossterm::event::KeyCode, layout::Rect, widgets::Widget};

pub trait IshtarSelectable: Downcast {
    fn priority_static() -> u8
    where
        Self: Sized;
    fn priority(&self) -> u8;
    fn keydown(&mut self, key: KeyCode) -> CmdTask;
    fn can_render(&self) -> bool {
        true
    }
    fn renderize(&mut self, area: Rect, buf: &mut Buffer);
}
impl_downcast!(IshtarSelectable);

pub mod clipboard;
pub mod command_interpreter;
pub mod file_manager;
pub mod keybind_handler;
pub mod text_area;
pub mod writeable_area;
