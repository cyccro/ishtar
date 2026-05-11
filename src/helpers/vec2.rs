use std::ops::{Add, Div, Mul, Sub};

/// A 2-D vector with `u16` components, used for positions and sizes throughout the editor.
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
pub struct Vec2(u16, u16);

impl Vec2 {
    /// Creates a new vector from `(x, y)` components.
    pub fn new(x: u16, y: u16) -> Self {
        Self(x, y)
    }

    /// Returns the horizontal component.
    pub fn x(&self) -> u16 {
        self.0
    }

    /// Returns the vertical component.
    pub fn y(&self) -> u16 {
        self.1
    }

    /// Returns a mutable reference to the horizontal component.
    pub fn x_mut(&mut self) -> &mut u16 {
        &mut self.0
    }

    /// Returns a mutable reference to the vertical component.
    pub fn y_mut(&mut self) -> &mut u16 {
        &mut self.1
    }

    /// Returns `x² * y²` (squared magnitude, not the standard dot-product form).
    #[inline]
    pub fn len_squared(&self) -> u16 {
        self.x().pow(2) * self.y().pow(2)
    }

    /// Returns the Euclidean length of the vector.
    pub fn len(&self) -> f32 {
        (self.len_squared() as f32).sqrt()
    }

    /// Returns the Euclidean distance between `self` and `other`.
    pub fn distance(&self, other: &Vec2) -> f32 {
        (self - other).len()
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x() + rhs.x(), self.y() + rhs.y())
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x() - rhs.x(), self.y() - rhs.y())
    }
}

impl Mul for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x() * rhs.x(), self.y() * rhs.y())
    }
}

impl Div for Vec2 {
    type Output = Vec2;
    fn div(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x() / rhs.x(), self.y() / rhs.y())
    }
}

// Reference variants so callers can use `&v1 + &v2` without moving.
impl Add for &Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x() + rhs.x(), self.y() + rhs.y())
    }
}

impl Sub for &Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x() - rhs.x(), self.y() - rhs.y())
    }
}

impl Mul for &Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x() * rhs.x(), self.y() * rhs.y())
    }
}

impl Div for &Vec2 {
    type Output = Vec2;
    fn div(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x() / rhs.x(), self.y() / rhs.y())
    }
}
