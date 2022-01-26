use nalgebra::Matrix4;

pub struct OrthographicCamera {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    pub near: f32,
    pub far: f32,
}

impl OrthographicCamera {
    pub fn projection_matrix(&self) -> nalgebra::Matrix4<f32> {
        nalgebra::Matrix4::<f32>::new_orthographic(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        )
    }
}

pub fn mouse_coordinates_to_world_coordinates(
    mouse_coordinates: (f32, f32),
    window_size: (f32, f32),
    projection_matrix: &Matrix4<f32>,
    view_matrix: &Matrix4<f32>,
) -> (f32, f32) {
    let mouse_coordinates = nalgebra::Point3::<f32>::new(
        (mouse_coordinates.0 / window_size.0) * 2.0 - 1.0,
        -(mouse_coordinates.1 / window_size.1) * 2.0 + 1.0,
        0.0,
    );

    let mouse_coordinates = (view_matrix * projection_matrix.try_inverse().unwrap())
        .transform_point(&mouse_coordinates);

    (mouse_coordinates.x, mouse_coordinates.y)
}

pub struct Active;
