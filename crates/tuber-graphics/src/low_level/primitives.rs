use crate::geometry::Vertex;
use crate::types::Size2;
use std::ops::Deref;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TextureId(pub usize);

impl ToString for TextureId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Deref for TextureId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type Index = u64;

#[derive(Debug, Clone)]
pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<Index>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
        }
    }

    pub fn append_vertices(&mut self, vertices: &[Vertex]) {
        self.vertices.extend_from_slice(vertices);
    }

    pub fn append_indices(&mut self, indices: &[Index]) {
        self.indices.extend_from_slice(indices);
    }

    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    pub fn indices(&self) -> &[Index] {
        &self.indices
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices().len()
    }

    pub fn index_count(&self) -> usize {
        self.indices().len()
    }
}

#[derive(Debug, Clone)]
pub struct Quad {
    pub top_left: Vertex,
    pub bottom_left: Vertex,
    pub top_right: Vertex,
    pub bottom_right: Vertex,
}

impl Quad {
    pub fn with_size(size: Size2) -> Self {
        Self {
            top_left: Vertex {
                position: [0.0, 0.0, 0.0],
                texture_coordinates: [0.0, 0.0],
                ..Default::default()
            },
            bottom_left: Vertex {
                position: [0.0, size.height, 0.0],
                texture_coordinates: [0.0, 1.0],
                ..Default::default()
            },
            top_right: Vertex {
                position: [size.width, 0.0, 0.0],
                texture_coordinates: [1.0, 0.0],
                ..Default::default()
            },
            bottom_right: Vertex {
                position: [size.width, size.height, 0.0],
                texture_coordinates: [1.0, 1.0],
                ..Default::default()
            },
        }
    }
}

impl Default for Quad {
    fn default() -> Self {
        Self {
            top_left: Vertex {
                position: [0.0, 0.0, 0.0],
                texture_coordinates: [0.0, 0.0],
                ..Default::default()
            },
            bottom_left: Vertex {
                position: [0.0, 1.0, 0.0],
                texture_coordinates: [0.0, 1.0],
                ..Default::default()
            },
            top_right: Vertex {
                position: [1.0, 0.0, 0.0],
                texture_coordinates: [1.0, 0.0],
                ..Default::default()
            },
            bottom_right: Vertex {
                position: [1.0, 1.0, 0.0],
                texture_coordinates: [1.0, 1.0],
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Material {
    pub albedo_map_id: TextureId,
    pub normal_map_id: TextureId,
    pub emission_map_id: TextureId,
}
