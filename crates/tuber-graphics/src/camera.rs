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

pub fn world_region(projection_matrix: &Matrix4<f32>, view_matrix: &Matrix4<f32>) -> WorldRegion {
    let min_coordinates = nalgebra::Point3::<f32>::new(-1.0, -1.0, 0.0);
    let max_coordinates = nalgebra::Point3::<f32>::new(1.0, 1.0, 0.0);

    let view_inv_proj = view_matrix * projection_matrix.try_inverse().unwrap();

    let min_coordinates = view_inv_proj.transform_point(&min_coordinates);
    let max_coordinates = view_inv_proj.transform_point(&max_coordinates);

    WorldRegion::new(
        min_coordinates.x,
        max_coordinates.x,
        max_coordinates.y,
        min_coordinates.y,
    )
}

#[derive(Debug)]
pub struct WorldRegion {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

impl WorldRegion {
    pub fn new(min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    pub fn min_x(&self) -> f32 {
        self.min_x
    }

    pub fn max_x(&self) -> f32 {
        self.max_x
    }

    pub fn min_y(&self) -> f32 {
        self.min_y
    }

    pub fn max_y(&self) -> f32 {
        self.max_y
    }

    pub fn is_in_region(&self, x: f32, y: f32, tolerance_x: f32, tolerance_y: f32) -> bool {
        x >= self.min_x - tolerance_x
            && x <= self.max_x + tolerance_x
            && y >= self.min_y - tolerance_y
            && y <= self.max_y + tolerance_y
    }
}

pub struct Active;
