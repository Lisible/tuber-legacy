use tuber_math::matrix::*;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4f = Matrix4f::with_values([
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0
]);

pub struct Camera {
    projection_matrix: Matrix4f,
}

impl Camera {
    pub fn new_orthographic_projection(
        left: f32,
        right: f32,
        top: f32,
        bottom: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Camera {
            projection_matrix: OPENGL_TO_WGPU_MATRIX
                * Matrix4f::new_orthographic(left, right, bottom, top, near, far),
        }
    }

    pub fn new_perspective_projection(fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
        Camera {
            projection_matrix: OPENGL_TO_WGPU_MATRIX
                * Matrix4f::new_perspective(fov_y, aspect, near, far),
        }
    }

    pub fn new_perspective_projection_frustum(
        left: f32,
        right: f32,
        top: f32,
        bottom: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Camera {
            projection_matrix: OPENGL_TO_WGPU_MATRIX
                * Matrix4f::new_frustum(left, right, bottom, top, near, far),
        }
    }

    pub fn projection_matrix(&self) -> Matrix4f {
        self.projection_matrix
    }
}
