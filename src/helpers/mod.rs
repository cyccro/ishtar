pub mod terminal_line;
use ratatui::crossterm::terminal::size;

pub fn terminal_size() -> (u16, u16) {
    size().unwrap()
}
pub fn percentage_of(amount: u16, percentage: u8) -> u16 {
    (amount as f32 * percentage.min(100) as f32 / 100.0) as u16
}
