use tuber_math::matrix::Matrix4f;
use tuber_math::quaternion::Quaternion;
use tuber_math::vector::Vector3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub translation: Vector3<f32>,
    pub angle: Vector3<f32>,
    pub rotation_center: Vector3<f32>,
    pub scale: Vector3<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LocalTransform(pub Transform);

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: (0.0, 0.0, 0.0).into(),
            angle: (0.0, 0.0, 0.0).into(),
            rotation_center: (0.0, 0.0, 0.0).into(),
            scale: (1.0, 1.0, 1.0).into(),
        }
    }
}

pub trait AsMatrix4 {
    fn as_matrix4(&self) -> Matrix4f;
}

impl AsMatrix4 for Transform {
    fn as_matrix4(&self) -> Matrix4f {
        let translate_to_rotation_center = self.rotation_center;

        Matrix4f::new_scale(&self.scale)
            * Matrix4f::new_translation(&self.translation)
            * Matrix4f::new_translation(&translate_to_rotation_center.clone())
            * Quaternion::from_euler(&self.angle).rotation_matrix()
            * Matrix4f::new_translation(&-translate_to_rotation_center)
    }
}
