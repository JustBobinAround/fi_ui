use std::{
    io::{Read, Seek, SeekFrom, Write},
    ops::Range,
};

struct Gap {
    offset: usize,
    len: usize,
}

impl Gap {
    pub fn new(offset: usize, len: usize) -> Self {
        Gap { offset, len }
    }

    pub fn start(&self) -> usize {
        self.offset
    }

    pub fn end(&self) -> usize {
        self.offset + self.len
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn shift_left(&mut self, delta: usize) {
        self.offset.saturating_sub(delta);
    }

    pub fn shift_right<const BUF_END: usize>(&mut self, delta: usize) {
        if delta + self.offset + self.len < BUF_END {
            self.offset += delta;
        }
    }
}

pub struct GapBuffer<File: Read + Write + Seek> {
    file: File,
    buf: Vec<u8>,
    gap: Gap,
}

impl<File: Read + Write + Seek> GapBuffer<File> {
    const STARTING_GAP: usize = 4096;
    pub fn new(file: File) -> Self {
        Self {
            file,
            buf: Vec::with_capacity(Self::STARTING_GAP * 2),
            gap: Gap::new(Self::STARTING_GAP, Self::STARTING_GAP),
        }
    }

    pub fn iter(&self) -> std::iter::Chain<std::slice::Iter<'_, u8>, std::slice::Iter<'_, u8>> {
        self.buf[..self.gap.start()]
            .iter()
            .chain(self.buf[self.gap.end()..].iter())
    }
}

impl<File: Read + Write + Seek> std::ops::Index<usize> for GapBuffer<File> {
    type Output = u8;

    fn index(&self, idx: usize) -> &Self::Output {
        if idx < self.gap.start() {
            &self.buf[idx]
        } else {
            &self.buf[idx + self.gap.len()]
        }
    }
}
