use crate::texture::{
    MISSING_TEXTURE_IDENTIFIER, MISSING_TEXTURE_REGION, WHITE_TEXTURE_IDENTIFIER,
};
use crate::types::{Color, Size2};
use crate::{TextureRegion, DEFAULT_NORMAL_MAP_IDENTIFIER};
use tuber_core::transform::Transform2D;

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
    pub transform: Transform2D,
}

#[derive(Default)]
pub struct MaterialDescription {
    pub albedo_map_description: Option<TextureDescription>,
    pub normal_map_description: Option<TextureDescription>,
}

#[derive(Clone)]
pub struct TextureDescription {
    /// The identifier of the texture
    pub identifier: String,
    /// The region of the texture to use
    pub texture_region: TextureRegion,
}

impl TextureDescription {
    pub fn not_found_texture_description() -> Self {
        Self {
            identifier: MISSING_TEXTURE_IDENTIFIER.into(),
            texture_region: MISSING_TEXTURE_REGION,
        }
    }
    pub fn default_albedo_map_description() -> Self {
        Self {
            identifier: WHITE_TEXTURE_IDENTIFIER.into(),
            texture_region: TextureRegion::one_pixel(),
        }
    }
    pub fn default_normal_map_description() -> Self {
        Self {
            identifier: DEFAULT_NORMAL_MAP_IDENTIFIER.into(),
            texture_region: TextureRegion::one_pixel(),
        }
    }
}
