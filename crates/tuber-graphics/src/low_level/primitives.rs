use tuber_math::vector::{Vector2f, Vector3f};

pub struct Vertex {
    pub(crate) position: Vector3f,
    pub(crate) color: Vector3f,
    pub(crate) texture_coordinates: Vector2f,
}

pub type Index = u16;
