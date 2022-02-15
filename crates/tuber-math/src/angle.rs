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

impl IntoDegrees<f32> for Angle<f32> {
    fn into_degrees(self) -> Angle<f32> {
        match self {
            Angle::Radians(_) => self,
            Angle::Degrees(angle_degrees) => {
                Self::Radians(angle_degrees * std::f32::consts::PI / 180f32)
            }
        }
    }
}

pub trait IntoRadians<T> {
    fn into_radians(self) -> Angle<T>;
}

impl IntoRadians<f32> for Angle<f32> {
    fn into_radians(self) -> Angle<f32> {
        match self {
            Angle::Radians(angle_radians) => {
                Self::Degrees(angle_radians * 180f32 / std::f32::consts::PI)
            }
            Angle::Degrees(_) => self,
        }
    }
}
