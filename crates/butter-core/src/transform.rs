use butter_math::{quaternion::Quaternion, vector::Vector3};

pub struct Transform {
    _scale: Vector3,
    _translation: Vector3,
    _rotation: Quaternion,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            _scale: Vector3::new(1.0, 1.0, 1.0),
            _translation: Vector3::new(0.0, 0.0, 0.0),
            _rotation: Quaternion::new(1.0, Vector3::new(0.0, 0.0, 0.0)),
        }
    }
}
