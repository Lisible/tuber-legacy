use crate::low_level::{mesh::Mesh, primitives::*};

/// A rectangular shape
pub struct RectangleShape {
    width: f32,
    height: f32,
}

impl RectangleShape {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }
}

impl From<RectangleShape> for Mesh {
    fn from(rectangle_shape: RectangleShape) -> Self {
        Mesh {
            vertices: vec![
                Vertex {
                    position: [0.0, 0.0, 0.0],
                    color: [1.0, 1.0, 1.0],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    position: [rectangle_shape.width, 0.0, 0.0],
                    color: [1.0, 1.0, 1.0],
                    texture_coordinates: [1.0, 0.0],
                },
                Vertex {
                    position: [0.0, rectangle_shape.height, 0.0],
                    color: [1.0, 1.0, 1.0],
                    texture_coordinates: [0.0, 1.0],
                },
                Vertex {
                    position: [rectangle_shape.width, rectangle_shape.height, 0.0],
                    color: [1.0, 1.0, 1.0],
                    texture_coordinates: [1.0, 1.0],
                },
            ],
            indices: vec![0, 2, 1, 1, 2, 3],
        }
    }
}
