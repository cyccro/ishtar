use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec2(u16, u16);

impl Vec2 {
    pub fn new(x: u16, y: u16) -> Self {
        Self(x, y)
    }
    pub fn x(&self) -> u16 {
        self.0
    }
    pub fn y(&self) -> u16 {
        self.1
    }
    pub fn x_mut(&mut self) -> &mut u16 {
        &mut self.0
    }
    pub fn y_mut(&mut self) -> &mut u16 {
        &mut self.1
    }
    #[inline]
    pub fn len_squared(&self) -> u16 {
        self.x().pow(2) * self.y().pow(2)
    }
    pub fn len(&self) -> f32 {
        (self.len_squared() as f32).powf(0.5)
    }
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
