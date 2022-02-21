use std::fmt::Display;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

pub trait One {
    fn one() -> Self;
}

impl One for i32 {
    fn one() -> Self {
        1
    }
}

impl One for f32 {
    fn one() -> Self {
        1.0
    }
}

impl One for f64 {
    fn one() -> Self {
        1.0
    }
}

pub trait Zero {
    fn zero() -> Self;
}

impl Zero for i32 {
    fn zero() -> Self {
        0
    }
}

impl Zero for f32 {
    fn zero() -> Self {
        0.0
    }
}

impl Zero for f64 {
    fn zero() -> Self {
        0.0
    }
}

pub trait Pi {
    fn pi() -> Self;
}

impl Pi for f32 {
    fn pi() -> Self {
        std::f32::consts::PI
    }
}

impl Pi for f64 {
    fn pi() -> Self {
        std::f64::consts::PI
    }
}

pub trait FloatOps:
    Sized
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Mul<Output = Self>
    + MulAssign
    + Div<Output = Self>
    + DivAssign
    + Neg<Output = Self>
{
}

impl FloatOps for f32 {}

impl FloatOps for f64 {}

pub trait Float: Display + Copy + Zero + One + Pi + FloatOps {
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn half(self) -> Self;
    fn squared(self) -> Self;
    fn sqrt(self) -> Self;
}

impl Float for f32 {
    fn sin(self) -> Self {
        self.sin()
    }

    fn cos(self) -> Self {
        self.cos()
    }
    fn half(self) -> Self {
        self * 0.5
    }

    fn squared(self) -> Self {
        self * self
    }

    fn sqrt(self) -> Self {
        self.sqrt()
    }
}

impl Float for f64 {
    fn sin(self) -> Self {
        self.sin()
    }

    fn cos(self) -> Self {
        self.cos()
    }
    fn half(self) -> Self {
        self * 0.5
    }

    fn squared(self) -> Self {
        self * self
    }

    fn sqrt(self) -> Self {
        self.sqrt()
    }
}
