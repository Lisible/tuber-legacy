use crate::low_level::primitives::{Index, Vertex};

pub struct Mesh {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<Index>,
}
