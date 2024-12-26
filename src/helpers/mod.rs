mod vec2;
pub use vec2::Vec2;
pub mod terminal_line;
use ratatui::crossterm::terminal::size;

pub fn terminal_size() -> (u16, u16) {
    size().unwrap()
}
pub fn min_max<T: PartialOrd>(a: T, b: T) -> (T, T) {
    if a > b {
        (b, a)
    } else {
        (a, b)
    }
}
