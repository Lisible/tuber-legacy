use crate::geometry::Vertex;
use crate::primitives::{Index, Mesh};
use crate::MaterialDescriptor;

pub struct MeshDescriptor {
    vertices: Vec<Vertex>,
    indices: Vec<Index>,
    material: MaterialDescriptor,
}

impl MeshDescriptor {
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    pub fn indices(&self) -> &[Index] {
        &self.indices
    }

    pub fn material(&self) -> &MaterialDescriptor {
        &self.material
    }

    pub fn create_mesh(&self) -> Mesh {
        let mut mesh = Mesh::new();
        mesh.append_vertices(&self.vertices);
        mesh.append_indices(&self.indices);
        mesh
    }

    pub fn triangle(material: MaterialDescriptor) -> Self {
        Self {
            vertices: vec![
                Vertex {
                    position: [0.0, 1.0, 0.0],
                    normal: [0.0, 0.0, -1.0],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    position: [0.5, 0.0, 0.0],
                    normal: [0.0, 0.0, -1.0],
                    texture_coordinates: [0.5, 1.0],
                },
                Vertex {
                    position: [1.0, 1.0, 0.0],
                    normal: [0.0, 0.0, -1.0],
                    texture_coordinates: [1.0, 0.0],
                },
            ],
            indices: vec![0, 2, 1],
            material,
        }
    }
}
