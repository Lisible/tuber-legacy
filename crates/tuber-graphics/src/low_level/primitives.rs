use crate::texture::{
    MISSING_TEXTURE_IDENTIFIER, MISSING_TEXTURE_REGION, WHITE_TEXTURE_IDENTIFIER,
};
use crate::types::{Color, Size2};
use crate::{TextureMetadata, TextureRegion, DEFAULT_NORMAL_MAP_IDENTIFIER};
use std::collections::HashMap;
use std::ops::Deref;
use tuber_core::transform::Transform2D;

#[derive(Copy, Clone, Debug)]
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

/// Describes a vertex for the low-level renderer
#[derive(Copy, Clone, Debug)]
pub struct VertexDescription {
    /// The position in Normalized Device Coordinates
    pub position: (f32, f32, f32),
    /// The color of the vertex
    pub color: Color,
    /// The normalized texture coordinates of the vertex
    pub texture_coordinates: (f32, f32),
}

impl Default for VertexDescription {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0, 0.0),
            color: Color::default(),
            texture_coordinates: (0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Quad {
    pub top_left: VertexDescription,
    pub bottom_left: VertexDescription,
    pub top_right: VertexDescription,
    pub bottom_right: VertexDescription,
}

impl Quad {
    pub fn with_size(size: Size2) -> Self {
        Self {
            top_left: VertexDescription {
                position: (0.0, 0.0, 0.0),
                color: Default::default(),
                texture_coordinates: (0.0, 0.0),
            },
            bottom_left: VertexDescription {
                position: (0.0, size.height, 0.0),
                color: Default::default(),
                texture_coordinates: (0.0, 1.0),
            },
            top_right: VertexDescription {
                position: (size.width, 0.0, 0.0),
                color: Default::default(),
                texture_coordinates: (1.0, 0.0),
            },
            bottom_right: VertexDescription {
                position: (size.width, size.height, 0.0),
                color: Default::default(),
                texture_coordinates: (1.0, 1.0),
            },
        }
    }
}

impl Default for Quad {
    fn default() -> Self {
        Self {
            top_left: VertexDescription {
                position: (0.0, 0.0, 0.0),
                color: Default::default(),
                texture_coordinates: (0.0, 0.0),
            },
            bottom_left: VertexDescription {
                position: (0.0, 1.0, 0.0),
                color: Default::default(),
                texture_coordinates: (0.0, 1.0),
            },
            top_right: VertexDescription {
                position: (1.0, 0.0, 0.0),
                color: Default::default(),
                texture_coordinates: (1.0, 0.0),
            },
            bottom_right: VertexDescription {
                position: (1.0, 1.0, 0.0),
                color: Default::default(),
                texture_coordinates: (1.0, 1.0),
            },
        }
    }
}

/// Describes a quad for the low-level renderer
#[derive(Clone)]
pub struct QuadDescription {
    pub size: Size2,
    pub color: Color,
    pub material: MaterialDescription,
    pub transform: Transform2D,
}

#[derive(Clone, Debug)]
pub struct MaterialDescription {
    pub albedo_map_description: TextureDescription,
    pub normal_map_description: TextureDescription,
}

#[derive(Clone, Debug)]
pub struct TextureDescription {
    /// The identifier of the texture
    pub identifier: TextureId,
    /// The region of the texture to use
    pub texture_region: TextureRegion,
}

impl TextureDescription {
    pub fn not_found_texture_description(
        texture_metadata: &HashMap<String, TextureMetadata>,
    ) -> Self {
        Self {
            identifier: texture_metadata[MISSING_TEXTURE_IDENTIFIER].texture_id,
            texture_region: MISSING_TEXTURE_REGION,
        }
    }
    pub fn default_albedo_map_description(
        texture_metadata: &HashMap<String, TextureMetadata>,
    ) -> Self {
        Self {
            identifier: texture_metadata[WHITE_TEXTURE_IDENTIFIER].texture_id,
            texture_region: TextureRegion::one_pixel(),
        }
    }
    pub fn default_normal_map_description(
        texture_metadata: &HashMap<String, TextureMetadata>,
    ) -> Self {
        Self {
            identifier: texture_metadata[DEFAULT_NORMAL_MAP_IDENTIFIER].texture_id,
            texture_region: TextureRegion::one_pixel(),
        }
    }
}
