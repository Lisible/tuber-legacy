use crate::geometry::Vertex;
use crate::texture::{
    MISSING_TEXTURE_IDENTIFIER, MISSING_TEXTURE_REGION, WHITE_TEXTURE_IDENTIFIER,
};
use crate::types::Size2;
use crate::{TextureMetadata, TextureRegion, DEFAULT_NORMAL_MAP_IDENTIFIER};
use std::collections::HashMap;
use std::ops::Deref;

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
                color: Default::default(),
                texture_coordinates: [0.0, 0.0],
            },
            bottom_left: Vertex {
                position: [0.0, size.height, 0.0],
                color: Default::default(),
                texture_coordinates: [0.0, 1.0],
            },
            top_right: Vertex {
                position: [size.width, 0.0, 0.0],
                color: Default::default(),
                texture_coordinates: [1.0, 0.0],
            },
            bottom_right: Vertex {
                position: [size.width, size.height, 0.0],
                color: Default::default(),
                texture_coordinates: [1.0, 1.0],
            },
        }
    }
}

impl Default for Quad {
    fn default() -> Self {
        Self {
            top_left: Vertex {
                position: [0.0, 0.0, 0.0],
                color: Default::default(),
                texture_coordinates: [0.0, 0.0],
            },
            bottom_left: Vertex {
                position: [0.0, 1.0, 0.0],
                color: Default::default(),
                texture_coordinates: [0.0, 1.0],
            },
            top_right: Vertex {
                position: [1.0, 0.0, 0.0],
                color: Default::default(),
                texture_coordinates: [1.0, 0.0],
            },
            bottom_right: Vertex {
                position: [1.0, 1.0, 0.0],
                color: Default::default(),
                texture_coordinates: [1.0, 1.0],
            },
        }
    }
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
