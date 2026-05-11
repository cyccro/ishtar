use gapbuf::GapBuffer;
use ratatui::crossterm::terminal::size;

/// Returns the current terminal dimensions as `(columns, rows)`.
///
/// # Panics
/// Panics if the terminal size cannot be queried (e.g. not attached to a TTY).
pub fn terminal_size() -> (u16, u16) {
    size().unwrap()
}

/// Returns `(min, max)` of the two values, regardless of their order.
pub fn min_max<T: PartialOrd>(a: T, b: T) -> (T, T) {
    if a > b { (b, a) } else { (a, b) }
}

/// Returns the byte length of the UTF-8 character whose *first* byte is `b`.
///
/// Returns `1` for ASCII bytes (`< 0x80`). For multi-byte sequences the length
/// is derived from the number of leading `1` bits in the first byte.
pub fn char_size_init(mut b: u8) -> u8 {
    if b < 0x80 {
        return 1;
    }
    b &= 0b1111_0000;
    let mut n = 0u8;
    while b != 0 {
        n += 1;
        b <<= 1;
    }
    n - 1
}

/// Returns the byte length of the UTF-8 character that *ends* at `spoint` in `vec`.
///
/// Walks backwards from `spoint` to find the leading byte of the character and
/// returns its encoded length. Returns `0` if no valid leading byte is found
/// within 4 bytes (invalid UTF-8).
///
/// # Examples
/// ```rust
/// # use gapbuf::GapBuffer;
/// # use ishtarx::helpers::char_size_backwards;
/// let v: GapBuffer<u8> = vec![b'a', 0xc3, 0xa0, 0xc3, 0xa7].into(); // "aàç"
/// assert_eq!(char_size_backwards(&v, v.len() - 1), 2); // 'ç'
/// assert_eq!(char_size_backwards(&v, v.len() - 3), 2); // 'à'
/// assert_eq!(char_size_backwards(&v, 0), 1);            // 'a'
/// ```
pub fn char_size_backwards(vec: &GapBuffer<u8>, spoint: usize) -> usize {
    if vec[spoint] < 0x80 {
        return 1;
    }
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
