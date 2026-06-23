use std::{
    collections::VecDeque,
    io::Read,
    ops::{Index, IndexMut},
};

pub static EMPTY_BUF: GapBuffer = GapBuffer {
    l_buf: VecDeque::new(),
    r_buf: VecDeque::new(),
};

#[derive(Clone, Default, Debug)]
pub struct GapBuffer {
    l_buf: VecDeque<char>,
    r_buf: VecDeque<char>,
}

impl GapBuffer {
    pub const fn new() -> Self {
        GapBuffer {
            l_buf: VecDeque::new(),
            r_buf: VecDeque::new(),
        }
    }

    fn shift_left(&mut self, offset: usize) {
        let mut dx = 0;
        while let Some(c) = self.r_buf.pop_front()
            && dx < offset
        {
            self.l_buf.push_back(c);
            dx += 1;
        }
    }

    fn shift_right(&mut self, offset: usize) {
        let mut dx = 0;
        while let Some(c) = self.l_buf.pop_back()
            && dx < offset
        {
            self.r_buf.push_front(c);
            dx += 1;
        }
    }

    fn shift(&mut self, offset: isize) {
        if offset < 0 {
            self.shift_left(offset.abs() as usize);
        } else {
            self.shift_right(offset as usize);
        }
    }

    fn current_offset(&self) -> usize {
        self.l_buf.len()
    }

    pub fn insert(&mut self, idx: usize, c: char) {
        let offset = self.current_offset() as isize - idx as isize;
        self.shift(offset);
        self.r_buf.push_front(c);
    }

    pub fn get(&self, idx: usize) -> Option<&char> {
        if idx < self.current_offset() {
            self.l_buf.get(idx)
        } else {
            self.r_buf.get(idx - self.current_offset())
        }
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut char> {
        if idx < self.current_offset() {
            self.l_buf.get_mut(idx)
        } else {
            self.r_buf.get_mut(idx - self.current_offset())
        }
    }

    pub fn capacity(&self) -> usize {
        self.l_buf.capacity() + self.r_buf.capacity()
    }

    pub fn len(&self) -> usize {
        self.l_buf.len() + self.r_buf.len()
    }

    pub fn iter<'a>(&'a self) -> GapBufferIter<'a> {
        GapBufferIter {
            buff: self,
            idx: 0,
            end: self.len(),
        }
    }

    pub fn iter_from<'a>(&'a self, range: std::ops::RangeFrom<usize>) -> GapBufferIter<'a> {
        GapBufferIter {
            buff: self,
            idx: range.start,
            end: self.len(),
        }
    }

    pub fn iter_to<'a>(&'a self, range: std::ops::RangeTo<usize>) -> GapBufferIter<'a> {
        GapBufferIter {
            buff: self,
            idx: 0,
            end: range.end,
        }
    }
}

impl From<String> for GapBuffer {
    fn from(value: String) -> Self {
        GapBuffer {
            l_buf: VecDeque::new(),
            r_buf: value.chars().collect(),
        }
    }
}

impl Index<usize> for GapBuffer {
    type Output = char;

    fn index<'a>(&'a self, index: usize) -> &'a char {
        self.get(index).unwrap()
    }
}

impl IndexMut<usize> for GapBuffer {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut char {
        self.get_mut(index).unwrap()
    }
}

// #[derive(Clone)]
// pub struct GapByteBufferIter<'a> {
//     buff: &'a GapCharBuffer,
//     idx: usize,
//     end: usize,
// }

// impl<'a> Iterator for GapByteBufferIter<'a> {
//     type Item = &'a char;

//     fn next(&mut self) -> Option<&'a char> {
//         if self.idx >= self.end {
//             return None;
//         }
//         let next = self.buff.get(self.idx);
//         if next.is_some() {
//             self.idx += 1;
//         }
//         next
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         let len = self.buff.len();
//         (len, Some(len))
//     }
// }

// impl<'a> DoubleEndedIterator for GapByteBufferIter<'a> {
//     fn next_back(&mut self) -> Option<<Self as Iterator>::Item> {
//         if self.idx >= self.end {
//             return None;
//         }
//         self.end -= 1;
//         self.buff.get(self.end)
//     }
// }

#[derive(Clone)]
pub struct GapBufferIter<'a> {
    buff: &'a GapBuffer,
    idx: usize,
    end: usize,
}

impl<'a> Iterator for GapBufferIter<'a> {
    type Item = &'a char;

    fn next(&mut self) -> Option<&'a char> {
        if self.idx >= self.end {
            return None;
        }
        let next = self.buff.get(self.idx);
        if next.is_some() {
            self.idx += 1;
        }
        next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.buff.len();
        (len, Some(len))
    }
}

impl<'a> DoubleEndedIterator for GapBufferIter<'a> {
    fn next_back(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.idx >= self.end {
            return None;
        }
        self.end -= 1;
        self.buff.get(self.end)
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_insert() {
        let mut test: GapBuffer = GapBuffer::new();

        for x in 0..100u8 {
            if x % 2 == 0 {
                test.insert((x / 2) as usize, x as char);
            }
        }
        dbg!(&test);
        assert!(
            test.len() == 50,
            "After even insertions, buffer length is {}",
            test.len()
        );
        for x in 0..100u8 {
            if x % 2 == 0 {
                assert!(
                    test[(x / 2) as usize] == x as char,
                    "insertion failed at {}",
                    x / 2
                );
            }
        }

        for (j, i) in test.iter_from(0..).enumerate() {
            assert!(*i as u8 % 2 == 0, "insertion failed at {}", j);
        }

        //Test insertion in the middle.
        for x in 0..100u8 {
            if x % 2 == 1 {
                test.insert(x as usize, x as char);
            }
        }
        assert!(
            test.len() == 100,
            "After odd insertions, buffer length is {}",
            test.len()
        );
    }

    #[test]
    fn test_index() {
        //Test indexing.
        let mut test: GapBuffer = GapBuffer::new();

        for x in 0..100u8 {
            test.insert(x as usize, x as char);
        }

        for x in 0..100u8 {
            assert!(test[x as usize] == x as char, "Index {} failed", x);
        }
    }

    #[test]
    fn test_remove() {
        //Test removal.

        let mut test1: GapBuffer = GapBuffer::new();

        for x in 0..10u8 {
            test1.insert(x as usize, x as char);
        }

        // for x in 0..10 {
        //     assert!(
        //         test1.remove(0) == Some(x),
        //         "Remove failed at {} (forward)",
        //         x
        //     );
        // }
    }
}
