use crate::low_level::primitives::{Index, Vertex};

#[derive(Default)]
pub struct Mesh {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<Index>,
}

impl Mesh {
    pub fn new_cube_mesh() -> Self {
        Mesh {
            vertices: vec![
                Vertex {
                    position: [1f32, 1f32, -1f32],
                    color: [1f32, 0f32, 0f32],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    position: [1f32, -1f32, -1f32],
                    color: [0f32, 1f32, 0f32],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    position: [1f32, 1f32, 1f32],
                    color: [0f32, 0f32, 1f32],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    position: [1f32, -1f32, 1f32],
                    color: [1f32, 0f32, 1f32],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    position: [-1f32, 1f32, -1f32],
                    color: [0f32, 1f32, 1f32],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    position: [-1f32, -1f32, -1f32],
                    color: [1f32, 1f32, 0f32],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    position: [-1f32, 1f32, 1f32],
                    color: [1f32, 1f32, 1f32],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    position: [-1f32, -1f32, 1f32],
                    color: [0f32, 0f32, 0f32],
                    texture_coordinates: [0.0, 0.0],
                },
            ],
            indices: vec![
                4, 2, 0, 2, 7, 3, 6, 5, 7, 1, 7, 5, 0, 3, 1, 4, 1, 5, 4, 6, 2, 2, 6, 7, 6, 4, 5, 1,
                3, 7, 0, 2, 3, 4, 0, 1,
            ],
        }
    }
}
