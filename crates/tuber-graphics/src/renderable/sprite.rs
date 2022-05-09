use crate::low_level::mesh::Mesh;
use crate::low_level::primitives::Vertex;

pub struct Sprite {
    texture_identifier: String,
    width: f32,
    height: f32,
}

impl Sprite {
    pub fn new(texture_identifier: &str, width: f32, height: f32) -> Self {
        Self {
            texture_identifier: texture_identifier.into(),
            width,
            height,
        }
    }

    pub fn texture_identifier(&self) -> &str {
        &self.texture_identifier
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn as_mesh(&self) -> Mesh {
        Mesh {
            vertices: vec![
                Vertex {
                    position: [0.0, 0.0, 0.0],
                    color: [1.0, 1.0, 1.0],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    position: [0.0, self.height, 0.0],
                    color: [1.0, 1.0, 1.0],
                    texture_coordinates: [0.0, 1.0],
                },
                Vertex {
                    position: [self.width, 0.0, 0.0],
                    color: [1.0, 1.0, 1.0],
                    texture_coordinates: [1.0, 0.0],
                },
                Vertex {
                    position: [self.width, self.height, 0.0],
                    color: [1.0, 1.0, 1.0],
                    texture_coordinates: [1.0, 1.0],
                },
            ],
            indices: vec![0, 1, 2, 2, 1, 3],
        }
    }
}
