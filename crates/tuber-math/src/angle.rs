use std::ops::{Div, Mul};

use crate::number_traits::Pi;

#[derive(Clone, Copy)]
pub struct Angle<T> {
    angle_radians: T,
}

impl<T> Angle<T>
where
    T: Copy + Pi + Mul<Output = T> + Div<Output = T> + From<i32>,
{
    pub fn new_radians(angle_radians: T) -> Self {
        Self { angle_radians }
    }

    pub fn new_degrees(angle_degrees: T) -> Self {
        Self {
            angle_radians: angle_degrees * T::pi() / T::from(180),
        }
    }

    pub fn radians(&self) -> T {
        self.angle_radians
    }

    pub fn degrees(&self) -> T {
        self.angle_radians * T::from(180) / T::pi()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_radians() {
        let angle = Angle::new_degrees(120.0);
        let radians = angle.radians();

        assert_eq!(radians as i32, (2.0 * std::f64::consts::FRAC_PI_3) as i32);
    }

    #[test]
    fn into_degrees() {
        let angle = Angle::new_radians(std::f64::consts::PI);
        let degrees = angle.degrees();

        assert_eq!(degrees as i32, 180);
    }
}
