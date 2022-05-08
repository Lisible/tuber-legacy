use crate::low_level::primitives::{Index, Vertex};

#[derive(Default)]
pub struct Mesh {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<Index>,
}
