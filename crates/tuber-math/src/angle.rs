use std::ops::{Div, Mul};

use crate::number_traits::Pi;

#[derive(Clone, Copy)]
pub enum Angle<T> {
    Radians(T),
    Degrees(T),
}

impl<T> Angle<T>
where
    T: Copy,
{
    pub fn value(&self) -> T {
        match self {
            Angle::Radians(value) => *value,
            Angle::Degrees(value) => *value,
        }
    }
}

pub trait IntoDegrees<T> {
    fn into_degrees(self) -> Angle<T>;
}

impl<T> IntoDegrees<T> for Angle<T>
where
    T: Copy + Pi + Mul<Output = T> + Div<Output = T> + From<i32>,
{
    fn into_degrees(self) -> Angle<T> {
        match self {
            Angle::Radians(angle_radians) => Self::Degrees(angle_radians * T::from(180) / T::pi()),
            Angle::Degrees(_) => self,
        }
    }
}

pub trait IntoRadians<T> {
    fn into_radians(self) -> Angle<T>;
}

impl<T> IntoRadians<T> for Angle<T>
where
    T: Copy + Pi + Mul<Output = T> + Div<Output = T> + From<i32>,
{
    fn into_radians(self) -> Angle<T> {
        match self {
            Angle::Radians(_) => self,
            Angle::Degrees(angle_degrees) => Self::Radians(angle_degrees * T::pi() / T::from(180)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_radians() {
        let angle = Angle::Degrees(120.0);
        let radians = angle.into_radians().value();

        assert_eq!(radians as i32, (2.0 * std::f64::consts::FRAC_PI_3) as i32);
    }

    #[test]
    fn into_degrees() {
        let angle = Angle::Radians(std::f64::consts::PI);
        let degrees = angle.into_degrees().value();

        assert_eq!(degrees as i32, 180);
    }
}
