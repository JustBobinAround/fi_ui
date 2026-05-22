use std::{
    io::{Read, Seek, SeekFrom, Write},
    marker::PhantomData,
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
        self.offset = self.offset.saturating_sub(delta);
    }

    pub fn shift_right<const BUF_END: usize>(&mut self, delta: usize) {
        if delta + self.offset + self.len < BUF_END {
            self.offset += delta;
        }
    }
}

pub struct GapBuffer<'a, File: Read + Write + Seek> {
    file: &'a mut File,
    buf: Vec<u8>,
    gap: Gap,
}

impl<'a, File: Read + Write + Seek> GapBuffer<'a, File> {
    const STARTING_GAP: usize = 4096;
    pub fn new(file: &'a mut File) -> Result<Self, std::io::Error> {
        let mut buf = Vec::new();
        let init_file_size = file.read_to_end(&mut buf)?;
        let gap = Gap::new(init_file_size, Self::STARTING_GAP);
        buf.reserve(gap.len());

        Ok(Self { file, buf, gap })
    }

    pub fn iter(&self) -> std::iter::Chain<std::slice::Iter<'_, u8>, std::slice::Iter<'_, u8>> {
        self.buf[..self.gap.start()]
            .iter()
            .chain(self.buf[self.gap.end()..].iter())
    }

    pub fn as_slices(&self, range: std::ops::Range<usize>) -> (&[u8], &[u8]) {
        let gap_len = self.gap.len();

        let mut left = &self.buf[0..0];
        let mut right = &self.buf[0..0];

        todo!()
    }
}

impl<'a, File: Read + Write + Seek> std::ops::Index<usize> for GapBuffer<'a, File> {
    type Output = u8;

    fn index(&self, idx: usize) -> &Self::Output {
        if idx < self.gap.start() {
            &self.buf[idx]
        } else {
            &self.buf[idx + self.gap.len()]
        }
    }
}
