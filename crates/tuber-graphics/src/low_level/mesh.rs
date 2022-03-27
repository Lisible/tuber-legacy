use crate::low_level::primitives::{Index, Vertex};

pub struct Mesh {
    pub(crate) _vertices: Vec<Vertex>,
    pub(crate) _indices: Vec<Index>,
}
