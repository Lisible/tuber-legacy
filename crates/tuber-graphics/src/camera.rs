use tuber_math::matrix::*;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4f = Matrix4f::with_values( [
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0
]);

pub struct OrthographicCamera {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    pub near: f32,
    pub far: f32,
}

impl OrthographicCamera {
    pub fn projection_matrix(&self) -> Matrix4f {
        OPENGL_TO_WGPU_MATRIX
            * Matrix4f::new_orthographic(
                self.left,
                self.right,
                self.bottom,
                self.top,
                self.near,
                self.far,
            )
    }
}
