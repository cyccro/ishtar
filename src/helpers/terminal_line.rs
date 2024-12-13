use core::fmt;
use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    ops::{Index, IndexMut, RangeBounds},
    str::Chars,
};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct TerminalLine {
    buffer: VecDeque<char>,
}

impl TerminalLine {
    pub fn from_chars(chars: Chars) -> Self {
        Self {
            buffer: chars.collect::<VecDeque<char>>(),
        }
    }
    pub fn from_str(s: &str) -> Self {
        Self::from_chars(s.chars())
    }
    pub fn from_string(s: &String) -> Self {
        Self::from_str(s)
    }
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
        }
    }
    pub fn with_chars(buffer: VecDeque<char>) -> Self {
        Self { buffer }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn split_off(&mut self, idx: usize) -> TerminalLine {
        Self::with_chars(self.buffer.split_off(idx))
    }
    pub fn chars(&self) -> &VecDeque<char> {
        &self.buffer
    }
    pub fn chars_mut(&mut self) -> &mut VecDeque<char> {
        &mut self.buffer
    }
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    pub fn size(&self) -> usize {
        self.len() * size_of::<char>()
    }
    pub fn reserve_exact(&mut self, amount: usize) {
        self.buffer.reserve_exact(amount);
    }
    pub fn push_back(&mut self, c: char) {
        self.buffer.push_back(c);
    }
    pub fn push_front(&mut self, c: char) {
        self.buffer.push_front(c);
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
    pub fn pop_back(&mut self) -> Option<char> {
        self.buffer.pop_back()
    }
    pub fn pop_front(&mut self) -> Option<char> {
        self.buffer.pop_front()
    }
    pub fn insert(&mut self, idx: usize, c: char) {
        self.buffer.insert(idx, c);
    }
    pub fn remove(&mut self, idx: usize) -> Option<char> {
        self.buffer.remove(idx)
    }
    pub fn append_line(&mut self, line: &mut TerminalLine) {
        self.buffer.append(line.chars_mut())
    }
    pub fn clear(&mut self) {
        self.buffer.clear()
    }
    pub fn range<R: RangeBounds<usize>>(
        &self,
        r: R,
    ) -> std::collections::vec_deque::Iter<'_, char> {
        self.buffer.range(r)
    }
}
impl Display for TerminalLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Display::fmt(&self.chars().iter().collect::<String>(), f)
    }
}
impl Debug for TerminalLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.chars().iter().collect::<String>(), f)
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
impl Index<usize> for TerminalLine {
    type Output = char;
    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[index]
    }
}
impl IndexMut<usize> for TerminalLine {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buffer[index]
    }
}
