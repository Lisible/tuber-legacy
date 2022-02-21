use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use crate::number_traits::Float;

pub type Vector3<T = f32> = Vector<T, 3>;
pub type Vector4<T = f32> = Vector<T, 4>;

#[derive(Debug, Clone)]
pub struct Vector<T, const DIM: usize> {
    values: [T; DIM],
}

impl<T, const DIM: usize> Vector<T, DIM>
where
    T: Float,
{
    pub fn norm(&self) -> T {
        let mut norm = T::zero();
        for value in self.values {
            norm += value.squared();
        }
        norm.sqrt()
    }

    pub fn normalize(&mut self) {
        let norm = self.norm();
        for value in &mut self.values {
            *value /= norm;
        }
    }

    pub fn normalized(&self) -> Self {
        let mut normalized = self.clone();
        normalized.normalize();
        normalized
    }
}

impl<T> Vector<T, 3>
where
    T: Copy,
{
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { values: [x, y, z] }
    }

    pub fn x(&self) -> T {
        self.values[0]
    }
    pub fn set_x(&mut self, value: T) {
        self.values[0] = value;
    }

    pub fn y(&self) -> T {
        self.values[1]
    }
    pub fn set_y(&mut self, value: T) {
        self.values[1] = value;
    }

    pub fn z(&self) -> T {
        self.values[2]
    }
    pub fn set_z(&mut self, value: T) {
        self.values[2] = value;
    }
}

impl<T, const DIM: usize> Display for Vector<T, DIM>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for i in 0..DIM {
            write!(f, "{}", self.values[i])?;
            if i != DIM - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")
    }
}

impl<T, const DIM: usize> Add for Vector<T, DIM>
where
    T: Copy + Add<Output = T>,
{
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        for i in 0..DIM {
            self.values[i] = self.values[i] + rhs.values[i];
        }
        self
    }
}

impl<T, const DIM: usize> AddAssign for Vector<T, DIM>
where
    T: Copy + AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..DIM {
            self.values[i] += rhs.values[i];
        }
    }
}

impl<T, const DIM: usize> Sub for Vector<T, DIM>
where
    T: Copy + Sub<Output = T>,
{
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        for i in 0..DIM {
            self.values[i] = self.values[i] - rhs.values[i];
        }
        self
    }
}

impl<T, const DIM: usize> SubAssign for Vector<T, DIM>
where
    T: Copy + SubAssign,
{
    fn sub_assign(&mut self, rhs: Self) {
        for i in 0..DIM {
            self.values[i] -= rhs.values[i];
        }
    }
}

impl<T, const DIM: usize> Mul<T> for Vector<T, DIM>
where
    T: Copy + Mul<Output = T>,
{
    type Output = Self;

    fn mul(mut self, rhs: T) -> Self::Output {
        for i in 0..DIM {
            self.values[i] = self.values[i] * rhs;
        }
        self
    }
}

impl<T, const DIM: usize> MulAssign<T> for Vector<T, DIM>
where
    T: Copy + MulAssign,
{
    fn mul_assign(&mut self, rhs: T) {
        for i in 0..DIM {
            self.values[i] *= rhs;
        }
    }
}

impl<T, const DIM: usize> Div<T> for Vector<T, DIM>
where
    T: Copy + Div<Output = T>,
{
    type Output = Self;

    fn div(mut self, rhs: T) -> Self::Output {
        for i in 0..DIM {
            self.values[i] = self.values[i] / rhs;
        }
        self
    }
}

impl<T, const DIM: usize> DivAssign<T> for Vector<T, DIM>
where
    T: Copy + DivAssign,
{
    fn div_assign(&mut self, rhs: T) {
        for i in 0..DIM {
            self.values[i] /= rhs;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector3_new() {
        let v = Vector3::new(1, 2, 3);

        assert_eq!(v.x(), 1);
        assert_eq!(v.y(), 2);
        assert_eq!(v.z(), 3);
    }

    #[test]
    fn add() {
        let a = Vector3::new(1, 2, 3);
        let b = Vector3::new(4, 5, 6);

        let result = a + b;

        assert_eq!(result.x(), 5);
        assert_eq!(result.y(), 7);
        assert_eq!(result.z(), 9);
    }

    #[test]
    fn add_assign() {
        let mut a = Vector3::new(1, 2, 3);
        let b = Vector3::new(4, 5, 6);

        a += b;

        assert_eq!(a.x(), 5);
        assert_eq!(a.y(), 7);
        assert_eq!(a.z(), 9);
    }

    #[test]
    fn sub() {
        let a = Vector3::new(1, 2, 3);
        let b = Vector3::new(4, 3, 2);

        let result = a - b;

        assert_eq!(result.x(), -3);
        assert_eq!(result.y(), -1);
        assert_eq!(result.z(), 1);
    }

    #[test]
    fn sub_assign() {
        let mut a = Vector3::new(1, 2, 3);
        let b = Vector3::new(4, 3, 2);

        a -= b;

        assert_eq!(a.x(), -3);
        assert_eq!(a.y(), -1);
        assert_eq!(a.z(), 1);
    }

    #[test]
    fn mul_scalar() {
        let a = Vector3::new(1, 2, 3);
        let b = 5;

        let result = a * b;

        assert_eq!(result.x(), 5);
        assert_eq!(result.y(), 10);
        assert_eq!(result.z(), 15);
    }

    #[test]
    fn display() {
        let result = format!("{}", Vector3::new(1, 2, 3));
        assert_eq!("(1, 2, 3)", &result);
    }

    #[test]
    fn norm() {
        let vector = Vector3::new(1.0, 2.0, 3.0);
        assert_float_absolute_eq!(vector.norm(), 3.74, 0.01);
    }

    #[test]
    fn normalize() {
        let mut vector = Vector3::new(1.0, 2.0, 3.0);

        vector.normalize();

        assert_float_absolute_eq!(vector.x(), 0.26, 0.01);
        assert_float_absolute_eq!(vector.y(), 0.53, 0.01);
        assert_float_absolute_eq!(vector.z(), 0.80, 0.01);
    }

    #[test]
    fn normalized() {
        let vector = Vector3::new(1.0, 2.0, 3.0);

        let normalized = vector.normalized();

        assert_float_absolute_eq!(normalized.x(), 0.26, 0.01);
        assert_float_absolute_eq!(normalized.y(), 0.53, 0.01);
        assert_float_absolute_eq!(normalized.z(), 0.80, 0.01);
    }
}
