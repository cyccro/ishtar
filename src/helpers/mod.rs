mod vec2;
use gapbuf::GapBuffer;
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
///Gets the size of the char by seeing its first byte considering it as a utf8 byte
pub fn char_size_init(mut b: u8) -> u8 {
    if b < 0x80 {
        1
    } else {
        b &= 0b1111_0000;
        let mut n = 0;
        while b != 0 {
            n += 1;
            b <<= 1;
        }
        n - 1
    }
}
///Gets the size of the char at the given start point backwards
///Example
///```rust
///let v = vec![b'a', 0xc3, 0xa0, 0xc3, 0xa7]; //'aàç'
///char_size_backwards(&v, v.len()-1); //2
///char_size_backwards(&v, v.len()-3); //2
///char_size_backwards(&v, 1); // 0; Invalid Byte
///char_size_backwards(&v, 0); //1
///```
pub fn char_size_backwards(vec: &GapBuffer<u8>, spoint: usize) -> usize {
    if vec[spoint] < 0x80 {
        1
    } else {
        let mut bcount = 0;
        while bcount < 4 {
            let mut c = vec[spoint - bcount] & 0b1111_0000;
            if c > 0b1000_0000 {
                let mut n = 0;
                while c != 0 {
                    n += 1;
                    c <<= 1;
                }
                return n - 1;
            }
            bcount += 1;
        }
        0
    }
}
