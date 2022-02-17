use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

pub type Vector3<T = f32> = Vector<T, 3>;
pub type Vector4<T = f32> = Vector<T, 4>;

#[derive(Debug)]
pub struct Vector<T, const DIM: usize> {
    values: [T; DIM],
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

    pub fn y(&self) -> T {
        self.values[1]
    }

    pub fn z(&self) -> T {
        self.values[2]
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
    fn vector_add() {
        let a = Vector3::new(1, 2, 3);
        let b = Vector3::new(4, 5, 6);

        let result = a + b;

        assert_eq!(result.x(), 5);
        assert_eq!(result.y(), 7);
        assert_eq!(result.z(), 9);
    }

    #[test]
    fn vector_add_assign() {
        let mut a = Vector3::new(1, 2, 3);
        let b = Vector3::new(4, 5, 6);

        a += b;

        assert_eq!(a.x(), 5);
        assert_eq!(a.y(), 7);
        assert_eq!(a.z(), 9);
    }

    #[test]
    fn vector_sub() {
        let a = Vector3::new(1, 2, 3);
        let b = Vector3::new(4, 3, 2);

        let result = a - b;

        assert_eq!(result.x(), -3);
        assert_eq!(result.y(), -1);
        assert_eq!(result.z(), 1);
    }

    #[test]
    fn vector_sub_assign() {
        let mut a = Vector3::new(1, 2, 3);
        let b = Vector3::new(4, 3, 2);

        a -= b;

        assert_eq!(a.x(), -3);
        assert_eq!(a.y(), -1);
        assert_eq!(a.z(), 1);
    }

    #[test]
    fn vector_mul_scalar() {
        let a = Vector3::new(1, 2, 3);
        let b = 5;

        let result = a * b;

        assert_eq!(result.x(), 5);
        assert_eq!(result.y(), 10);
        assert_eq!(result.z(), 15);
    }

    #[test]
    fn vector_display() {
        let result = format!("{}", Vector3::new(1, 2, 3));
        assert_eq!("(1, 2, 3)", &result);
    }
}
