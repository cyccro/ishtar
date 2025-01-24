use core::fmt;
use std::{
    fmt::{Debug, Display},
    ops::RangeBounds,
};

use gapbuf::GapBuffer;

use super::char_size_backwards;

// A wrapper to make easier the manipulation in Ishtar Buffer Lines
// It used to be a wrapper to VecDeque<char> but since sizeof(char) == 4 in rust, it's better
// using u8 instead.
/// Wrapper for managing lines efficiently
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct TerminalLine {
    buffer: GapBuffer<u8>,
}

impl TerminalLine {
    pub fn from_str(s: &str) -> Self {
        let mut buffer = GapBuffer::with_capacity(s.len());
        for c in s.as_bytes() {
            buffer.push_back(*c);
        }
        Self { buffer }
    }
    pub fn new() -> Self {
        Self {
            buffer: GapBuffer::new(),
        }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: GapBuffer::with_capacity(capacity),
        }
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    #[inline]
    pub fn bytes(&self) -> &GapBuffer<u8> {
        &self.buffer
    }
    #[inline]
    pub fn bytes_mut(&mut self) -> &mut GapBuffer<u8> {
        &mut self.buffer
    }
    #[inline]
    pub fn get(&self, idx: usize) -> Option<&u8> {
        self.buffer.get(idx)
    }
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    #[inline]
    pub fn size(&self) -> usize {
        self.len() * size_of::<char>()
    }
    #[inline]
    pub fn reserve(&mut self, amount: usize) {
        self.buffer.reserve(amount);
    }
    #[inline]
    pub fn reserve_exact(&mut self, amount: usize) {
        self.buffer.reserve_exact(amount);
    }
    #[inline]
    pub fn drain<R: RangeBounds<usize>>(&mut self, r: R) -> gapbuf::Drain<'_, u8> {
        self.buffer.drain(r)
    }
    pub fn split_off(&mut self, idx: usize) -> TerminalLine {
        self.buffer.set_gap(idx);
        let mut target = TerminalLine::new();
        while self.buffer.len() - self.buffer.gap() > 0 {
            target
                .buffer
                .push_back(self.buffer.remove(self.buffer.gap()));
        }
        target
    }
    pub fn insert(&mut self, idx: usize, c: char) {
        let b = &mut [0; 4];
        let buf = c.encode_utf8(b).as_bytes();

        if idx == self.len() {
            self.buffer.reserve_exact(c.len_utf8());
        }
        self.buffer.set_gap(idx);
        self.buffer.insert_many(idx, buf.iter().cloned());
    }
    ///Removes the char at the given idx, backwards if true and returns the char removed and its
    ///size
    pub fn remove(&mut self, idx: usize, backwards: bool) -> Option<(char, usize)> {
        if let Some(c) = self.buffer.get(idx) {
            if c.is_ascii() {
                self.buffer.set_gap(idx);
                Some((self.buffer.remove(idx) as char, 1))
            } else {
                let size = char_size_backwards(&self.buffer, idx);
                let mut byte: u32 = 0;
                if backwards {
                    for i in 0..size - 1 {
                        byte |= self.buffer.remove(idx - i) as u32;
                        byte <<= 8;
                    }
                    byte |= self.buffer.remove(idx - size + 1) as u32;
                } else {
                    let size = size + 1;
                    for i in 0..size - 1 {
                        byte |= self.buffer.remove(idx + i) as u32;
                        byte <<= 8;
                    }
                    byte |= self.buffer.remove(idx + size - 1) as u32;
                }
                Some((char::from_u32(byte).unwrap(), size))
            }
        } else {
            None
        }
    }
    #[inline]
    pub fn push_back(&mut self, c: char) {
        self.insert(self.len(), c);
    }
    #[inline]
    pub fn push_front(&mut self, c: char) {
        self.insert(0, c);
    }
    pub fn push_str_back(&mut self, s: &str) {
        if self.len() + s.len() > self.capacity() {
            self.reserve_exact(s.len());
        }
        for c in s.chars() {
            self.push_back(c)
        }
    }
    pub fn push_str_front(&mut self, s: &str) {
        if self.len() + s.len() > self.capacity() {
            self.reserve_exact(s.len());
        }
        for c in s.chars() {
            self.push_front(c);
        }
    }
    #[inline]
    pub fn pop_back(&mut self) -> Option<(char, usize)> {
        self.remove(self.len() - 1, true)
    }
    pub fn pop_front(&mut self) -> Option<(char, usize)> {
        if let Some(b) = self.get(0) {
            if b.is_ascii() {
                self.remove(0, false)
            } else {
                let mut b = *b;
                let mut n = 0;
                while b != 0 {
                    b <<= 0;
                    n += 1;
                }
                self.remove(n - 1, false)
            }
        } else {
            None
        }
    }
    pub fn append_line(&mut self, line: TerminalLine) {
        if self.buffer.capacity() < self.buffer.len() + line.len() {
            self.buffer.reserve(line.len());
        }
        self.buffer
            .insert_many(self.buffer.len(), line.buffer.into_iter());
    }
    #[inline]
    pub fn clear(&mut self) {
        self.buffer.clear()
    }
    #[inline]
    pub fn range<R: RangeBounds<usize>>(&self, r: R) -> gapbuf::Range<'_, u8> {
        self.buffer.range(r)
    }
    #[inline]
    pub fn gap(&self) -> usize {
        self.buffer.gap()
    }
}
impl Display for TerminalLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vec = {
            let mut vec = Vec::with_capacity(self.buffer.len());
            for i in 0..self.buffer.len() {
                vec.push(*self.buffer.get(i).unwrap())
            }
            String::from_utf8(vec).unwrap()
        };
        fmt::Display::fmt(&vec, f)
    }
}
impl Debug for TerminalLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.bytes(), f)
    }
}
impl Clone for TerminalLine {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
        }
    }
}
impl Default for TerminalLine {
    fn default() -> Self {
        Self::new()
    }
}
