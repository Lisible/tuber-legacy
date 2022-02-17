use std::fmt::{Debug, Display, Formatter};
use std::ops::{Index, IndexMut};

use crate::number_traits::{Float, One, Zero};
use crate::vector::Vector3;

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
}
