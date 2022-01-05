use crate::types::{Color, Size2};
use crate::TextureRegion;

/// Describes a vertex for the low-level renderer
pub struct VertexDescription {
    /// The position in Normalized Device Coordinates
    pub position: (f32, f32, f32),
    /// The color of the vertex
    pub color: (f32, f32, f32),
    /// The normalized texture coordinates of the vertex
    pub texture_coordinates: (f32, f32),
}

/// Describes a quad for the low-level renderer
pub struct QuadDescription {
    /// Width in Normalized Device Coordinates
    pub size: Size2,
    /// The color used for the quad's vertices
    pub color: Color,
    /// The material of the quad
    pub material: MaterialDescription,
}

#[derive(Default)]
pub struct MaterialDescription {
    pub albedo_map_description: Option<TextureDescription>,
    pub normal_map_description: Option<TextureDescription>,
}

pub struct TextureDescription {
    /// The identifier of the texture
    pub identifier: String,
    /// The region of the texture to use
    pub texture_region: TextureRegion,
}
