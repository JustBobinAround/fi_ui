use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub struct Bounds {
    width: usize,
    height: usize,
}

impl From<Bounds> for Vec2 {
    fn from(value: Bounds) -> Self {
        Vec2 {
            x: value.width as isize,
            y: value.height as isize,
        }
    }
}

impl Bounds {
    pub fn new(width: usize, height: usize) -> Self {
        Bounds { width, height }
    }

    pub fn width(&self) -> &usize {
        &self.width
    }

    pub fn height(&self) -> &usize {
        &self.height
    }

    pub fn into_parts(self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn is_to_the_right_of(&self, other: &Bounds) -> bool {
        self.width == other.width + 1
    }
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rect {
    top_left: Vec2,
    size: Vec2,
}

impl Rect {
    pub fn new(top_left: Vec2, size: Vec2) -> Rect {
        Rect { top_left, size }
    }

    pub fn size(&self) -> &Vec2 {
        &self.size
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub struct Vec2 {
    x: isize,
    y: isize,
}

impl Vec2 {
    pub fn new(x: isize, y: isize) -> Self {
        Vec2 { x, y }
    }

    pub fn x(&self) -> &isize {
        &self.x
    }

    pub fn set_x(&mut self, x: isize) {
        self.x = x;
    }

    pub fn y(&self) -> &isize {
        &self.y
    }

    pub fn set_y(&mut self, y: isize) {
        self.y = y;
    }

    pub fn into_parts(self) -> (isize, isize) {
        (self.x, self.y)
    }

    pub fn is_to_the_right_of(&self, other: &Vec2) -> bool {
        self.x == other.x + 1
    }

    pub fn is_within_rect(&self, r: &Rect) -> bool {
        self.x >= r.top_left.x
            && self.y >= r.top_left.y
            && self.x < r.top_left.x + r.size.x
            && self.y < r.top_left.y + r.size.y
    }

    pub fn is_within_bounds(&self, b: &Bounds) -> bool {
        self.x >= 0 && self.y >= 0 && self.x < *b.width() as isize && self.y < *b.height() as isize
    }
}
impl Ord for Vec2 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        match self.y.cmp(&other.y) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.x.cmp(&other.x),
            Ordering::Greater => Ordering::Greater,
        }
    }
    fn max(self, other: Self) -> Self {
        use std::cmp::max_by;
        max_by(self, other, Self::cmp)
    }
    fn min(self, other: Self) -> Self {
        use std::cmp::min_by;
        min_by(self, other, Self::cmp)
    }
}

impl PartialOrd for Vec2 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        match self.y.cmp(&other.y) {
            Ordering::Less => Some(Ordering::Less),
            Ordering::Equal => Some(self.x.cmp(&other.x)),
            Ordering::Greater => Some(Ordering::Greater),
        }
    }
    fn lt(&self, other: &Self) -> bool {
        use std::cmp::Ordering;
        self.partial_cmp(other).unwrap() == Ordering::Less
    }
    fn gt(&self, other: &Self) -> bool {
        use std::cmp::Ordering;
        self.partial_cmp(other).unwrap() == Ordering::Greater
    }
    fn le(&self, other: &Self) -> bool {
        use std::cmp::Ordering;
        let cmp = self.partial_cmp(other).unwrap();
        cmp == Ordering::Less || cmp == Ordering::Equal
    }
    fn ge(&self, other: &Self) -> bool {
        use std::cmp::Ordering;
        let cmp = self.partial_cmp(other).unwrap();
        cmp == Ordering::Greater || cmp == Ordering::Equal
    }
}
macro_rules! impl_op {
    (
        $trait_name_1: ident,
        $fn_name_1: ident,
        $trait_name_2: ident,
        $fn_name_2: ident,
        $op: tt
    ) => {
        impl $trait_name_1 for Vec2 {
            type Output = Vec2;
            fn $fn_name_1(mut self, rhs: Self) -> Self::Output {
                self $op rhs;
                self
            }
        }

        impl $trait_name_2 for Vec2 {
            fn $fn_name_2(&mut self, rhs: Self) {
                self.x $op rhs.x;
                self.y $op rhs.y;
            }
        }

        impl $trait_name_1 for Bounds {
            type Output = Bounds;
            fn $fn_name_1(mut self, rhs: Self) -> Self::Output {
                self $op rhs;
                self
            }
        }

        impl $trait_name_2 for Bounds {
            fn $fn_name_2(&mut self, rhs: Self) {
                self.width $op rhs.width;
                self.height $op rhs.height;
            }
        }
    };
}

impl_op!(Add, add, AddAssign, add_assign, +=);
impl_op!(Sub, sub, SubAssign, sub_assign, -=);
impl_op!(Mul, mul, MulAssign, mul_assign, *=);
impl_op!(Div, div, DivAssign, div_assign, /=);
