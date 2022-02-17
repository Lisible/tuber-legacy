use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, Index, IndexMut, Mul, MulAssign};

use crate::number_traits::{One, Zero};
use crate::vector::Vector3;

#[derive(Clone)]
pub struct Matrix4<T = f32> {
    values: [T; 16],
}

impl<T> Debug for Matrix4<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[")?;
        for i in 0..Self::ROWS {
            write!(f, "\t")?;
            for j in 0..Self::COLS {
                write!(f, "{}, ", self.values[i * Self::COLS + j])?;
            }
            writeln!(f)?;
        }
        writeln!(f, "]")
    }
}

impl<T> Matrix4<T> {
    const COLS: usize = 4;
    const ROWS: usize = 4;

    pub fn with_values(values: [T; 16]) -> Self {
        Self { values }
    }

    #[rustfmt::skip]
    pub fn new_translation<U>(translation: Vector3<U>) -> Matrix4<U>
        where U: Copy + Zero + One {
        Matrix4 {
            values: [
                U::one(), U::zero(), U::zero(), translation.x(),
                U::zero(), U::one(), U::zero(), translation.y(),
                U::zero(), U::zero(), U::one(), translation.z(),
                U::zero(), U::zero(), U::zero(), U::one()
            ]
        }
    }

    #[rustfmt::skip]
    pub fn new_scale_uniform<U>(scale: U) -> Matrix4<U>
        where U: Copy + Zero + One {
        Self::new_scale(Vector3::new(scale, scale, scale))
    }

    #[rustfmt::skip]
    pub fn new_scale<U>(scale: Vector3<U>) -> Matrix4<U>
        where U: Copy + Zero + One {
        Matrix4 {
            values: [
                scale.x(), U::zero(), U::zero(), U::zero(),
                U::zero(), scale.y(), U::zero(), U::zero(),
                U::zero(), U::zero(), scale.z(), U::zero(),
                U::zero(), U::zero(), U::zero(), U::one(),
            ]
        }
    }
}

impl<T> Mul<Self> for Matrix4<T>
where
    T: Copy + Zero + Add<Output = T> + Mul<Output = T>,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut values = [T::zero(); 16];

        for j in 0..4 {
            for i in 0..4 {
                values[j * Self::COLS + i] = self.values[j * Self::COLS] * rhs.values[i]
                    + self.values[j * Self::COLS + 1] * rhs.values[i + Self::COLS]
                    + self.values[j * Self::COLS + 2] * rhs.values[i + Self::COLS * 2]
                    + self.values[j * Self::COLS + 3] * rhs.values[i + Self::COLS * 3];
            }
        }

        Self { values }
    }
}

impl<T> MulAssign<Self> for Matrix4<T>
where
    T: Copy + Zero + Add<Output = T> + Mul<Output = T>,
{
    fn mul_assign(&mut self, rhs: Self) {
        self.values = (self.clone() * rhs).values;
    }
}

impl<T> Index<usize> for Matrix4<T> {
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index * Self::ROWS..index * Self::ROWS + Self::COLS]
    }
}

impl<T> IndexMut<usize> for Matrix4<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.values[index * Self::ROWS..index * Self::ROWS + Self::COLS]
    }
}

pub trait Identity {
    fn identity() -> Self;
}

#[rustfmt::skip]
impl<T> Identity for Matrix4<T>
    where T: One + Zero {
    fn identity() -> Self {
        Self {
            values: [
                T::one(), T::zero(), T::zero(), T::zero(),
                T::zero(), T::one(), T::zero(), T::zero(),
                T::zero(), T::zero(), T::one(), T::zero(),
                T::zero(), T::zero(), T::zero(), T::one()
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity() {
        let m = Matrix4::<i32>::identity();

        for i in 0..4 {
            for j in 0..4 {
                if i == j {
                    assert_eq!(m[i][j], 1);
                } else {
                    assert_eq!(m[i][j], 0);
                }
            }
        }
    }

    #[test]
    fn index_mut() {
        let mut m = Matrix4::<i32>::identity();
        m[3][2] = 5;

        assert_eq!(m[3][2], 5);
    }

    #[test]
    fn index() {
        let m = Matrix4::<i32>::identity();

        assert_eq!(m[0][0], 1);
        assert_eq!(m[0][1], 0);
    }

    #[rustfmt::skip]
    #[test]
    fn mul() {
        let a = Matrix4::<i32>::with_values([
            1, 2, 3, 4,
            5, 6, 7, 8,
            9, 39, 11, 12,
            13, 14, 15, 16
        ]);
        let b = Matrix4::<i32>::with_values([
            17, 18, 19, 20,
            21, 22, 23, 24,
            25, 26, 27, 28,
            29, 30, 31, 32
        ]);

        let result = a * b;

        assert_eq!(result[0][0], 250);
        assert_eq!(result[0][1], 260);
        assert_eq!(result[0][2], 270);
        assert_eq!(result[0][3], 280);
        assert_eq!(result[1][0], 618);
        assert_eq!(result[1][1], 644);
        assert_eq!(result[1][2], 670);
        assert_eq!(result[1][3], 696);
        assert_eq!(result[2][0], 1595);
        assert_eq!(result[2][1], 1666);
        assert_eq!(result[2][2], 1737);
        assert_eq!(result[2][3], 1808);
        assert_eq!(result[3][0], 1354);
        assert_eq!(result[3][1], 1412);
        assert_eq!(result[3][2], 1470);
        assert_eq!(result[3][3], 1528);
    }

    #[rustfmt::skip]
    #[test]
    fn mul_assign() {
        let mut a = Matrix4::<i32>::with_values([
            1, 2, 3, 4,
            5, 6, 7, 8,
            9, 39, 11, 12,
            13, 14, 15, 16
        ]);
        let b = Matrix4::<i32>::with_values([
            17, 18, 19, 20,
            21, 22, 23, 24,
            25, 26, 27, 28,
            29, 30, 31, 32
        ]);

        a *= b;

        assert_eq!(a[0][0], 250);
        assert_eq!(a[0][1], 260);
        assert_eq!(a[0][2], 270);
        assert_eq!(a[0][3], 280);
        assert_eq!(a[1][0], 618);
        assert_eq!(a[1][1], 644);
        assert_eq!(a[1][2], 670);
        assert_eq!(a[1][3], 696);
        assert_eq!(a[2][0], 1595);
        assert_eq!(a[2][1], 1666);
        assert_eq!(a[2][2], 1737);
        assert_eq!(a[2][3], 1808);
        assert_eq!(a[3][0], 1354);
        assert_eq!(a[3][1], 1412);
        assert_eq!(a[3][2], 1470);
        assert_eq!(a[3][3], 1528);
    }
}
