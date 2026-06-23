use std::collections::{BTreeMap, BTreeSet};

pub static EMPTY_SELECTION_TREE: SelectionTree = SelectionTree {
    tree: BTreeSet::new(),
    translation_offset: 0,
};

#[derive(Debug, Eq)]
pub enum SelectionBound {
    Start(usize),
    End(usize),
}

impl PartialEq for SelectionBound {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Start(a) => match other {
                Self::Start(b) => a == b,
                _ => false,
            },
            Self::End(a) => match other {
                Self::End(b) => a == b,
                _ => false,
            },
        }
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl SelectionBound {
    pub fn bound_val(&self) -> &usize {
        match self {
            SelectionBound::Start(n) => n,
            SelectionBound::End(n) => n,
        }
    }

    pub fn is_start(&self) -> bool {
        match self {
            SelectionBound::Start(_) => true,
            SelectionBound::End(_) => false,
        }
    }

    pub fn is_end(&self) -> bool {
        !self.is_start()
    }

    pub fn translate(self, delta: isize) -> Self {
        match self {
            SelectionBound::Start(n) => SelectionBound::Start(n.saturating_add_signed(delta)),
            SelectionBound::End(n) => SelectionBound::End(n.saturating_add_signed(delta)),
        }
    }
}

impl PartialOrd for SelectionBound {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.bound_val().cmp(&other.bound_val()))
    }
}

impl Ord for SelectionBound {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bound_val().cmp(&other.bound_val())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SelectionTree {
    tree: BTreeSet<SelectionBound>,
    translation_offset: isize,
}

impl SelectionTree {
    pub const fn new() -> Self {
        SelectionTree {
            tree: BTreeSet::new(),
            translation_offset: 0isize,
        }
    }

    pub fn main_cursor(&self) -> usize {
        self.tree
            .first()
            .map(|first| {
                let a = first
                    .bound_val()
                    .saturating_add_signed(self.translation_offset);
                a
            })
            .unwrap_or(0)
    }

    pub fn insert(&mut self, start_n: usize, end_n: usize) {
        let end_n = end_n + 1;
        let start = SelectionBound::Start(start_n);
        let end = SelectionBound::End(end_n);

        let mut tail = self.tree.split_off(&start);
        let mut to_keep = tail.split_off(&end);

        // dbg!(&tail);
        // dbg!(&to_keep);

        match tail.first() {
            Some(SelectionBound::Start(_)) | None => {
                self.tree.insert(start);
            }
            _ => {}
        }

        match to_keep.first() {
            Some(SelectionBound::Start(n)) => {
                if *n == end_n {
                    to_keep.pop_first();
                } else {
                    self.tree.insert(end);
                }
            }
            None => {
                self.tree.insert(end);
            }
            _ => {}
        }

        self.tree.append(&mut to_keep);
    }

    pub fn is_main_cursor(&self, offset: usize) -> bool {
        self.main_cursor() == offset
    }

    pub fn contains(&self, start: usize) -> bool {
        self.tree.contains(&SelectionBound::Start(
            0usize.saturating_add_signed(self.translation_offset - start as isize),
        ))
    }

    pub fn translate(&mut self, delta: isize) {
        self.translation_offset += delta;
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_insert_1() {
        let mut tree_a = SelectionTree::new();
        tree_a.insert(0, 5);
        tree_a.insert(8, 10);
        tree_a.insert(12, 15);
        tree_a.insert(17, 20);
        tree_a.insert(7, 16);

        let mut tree_b = SelectionTree::new();
        tree_b.insert(0, 5);
        tree_b.insert(7, 16);
        tree_b.insert(17, 20);

        assert_eq!(tree_a, tree_b);
    }

    #[test]
    fn test_insert_2() {
        let mut tree_a = SelectionTree::new();
        tree_a.insert(0, 5);
        tree_a.insert(8, 10);
        tree_a.insert(12, 15);
        tree_a.insert(17, 20);
        tree_a.insert(7, 17);

        let mut tree_b = SelectionTree::new();
        tree_b.insert(0, 5);
        tree_b.insert(7, 20);

        assert_eq!(tree_a, tree_b);
    }

    #[test]
    fn test_insert_3() {
        let mut tree_a = SelectionTree::new();

        tree_a.insert(0, 5);
        tree_a.insert(8, 10);
        tree_a.insert(12, 15);
        tree_a.insert(17, 20);
        tree_a.insert(10, 18);

        let mut tree_b = SelectionTree::new();
        tree_b.insert(0, 5);
        tree_b.insert(8, 20);

        assert_eq!(tree_a, tree_b);
    }

    #[test]
    fn test_insert_4() {
        let mut tree_a = SelectionTree::new();

        tree_a.insert(0, 5);
        tree_a.insert(8, 10);
        tree_a.insert(12, 15);
        tree_a.insert(17, 20);
        tree_a.insert(10, 17);

        let mut tree_b = SelectionTree::new();
        tree_b.insert(0, 5);
        tree_b.insert(8, 20);

        assert_eq!(tree_a, tree_b);
    }

    #[test]
    fn test_insert_5() {
        let mut tree_a = SelectionTree::new();

        tree_a.insert(0, 0);
        tree_a.insert(1, 1);

        let mut tree_b = SelectionTree::new();
        tree_b.insert(0, 5);

        assert_eq!(tree_a, tree_b);
    }
}
