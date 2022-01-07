use crate::primitives::TextureId;
use crate::texture::TextureUsage::{Albedo, Normal};
use crate::GraphicsError;
use crate::GraphicsError::{ImageDecodeError, TextureFileOpenError};
use nalgebra::Vector4;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;
use tuber_core::asset::AssetMetadata;

pub type TextureSize = (u32, u32);

pub struct TextureData {
    pub identifier: String,
    pub size: TextureSize,
    pub bytes: Vec<u8>,
    pub srgb: bool,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct TextureRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl TextureRegion {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn one_pixel() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }
    }

    pub fn normalize(self, texture_width: u32, texture_height: u32) -> Self {
        let texture_width = texture_width as f32;
        let texture_height = texture_height as f32;
        Self {
            x: self.x / texture_width,
            y: self.y / texture_height,
            width: self.width / texture_width,
            height: self.height / texture_height,
        }
    }

    pub fn flip_x(self) -> Self {
        Self {
            x: self.x + self.width,
            y: self.y,
            width: -self.width,
            height: self.height,
        }
    }
}

impl From<TextureRegion> for Vector4<f32> {
    fn from(region: TextureRegion) -> Self {
        Vector4::new(region.x, region.y, region.width, region.height)
    }
}

pub struct TextureMetadata {
    pub texture_id: TextureId,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize)]
pub struct TextureAtlas {
    pub textures: HashMap<String, TextureRegion>,
}

impl TextureAtlas {
    pub fn texture_region(&self, texture_name: &str) -> Option<TextureRegion> {
        self.textures.get(texture_name).cloned()
    }
}

pub(crate) fn texture_loader(asset_metadata: &AssetMetadata) -> Box<dyn Any> {
    use image::io::Reader as ImageReader;
    let mut file_path = asset_metadata.asset_path.clone();
    file_path.push(asset_metadata.metadata.get("texture_data").unwrap());
    let image = ImageReader::open(file_path)
        .map_err(|e| TextureFileOpenError(e))
        .unwrap()
        .decode()
        .map_err(|e| ImageDecodeError(e))
        .unwrap();
    let image = image.as_rgba8().unwrap();

    let usage: TextureUsage = asset_metadata
        .metadata
        .get("usage")
        .cloned()
        .unwrap_or("albedo".to_string())
        .parse()
        .unwrap();

    let srgb = usage == TextureUsage::Albedo;
    Box::new(TextureData {
        identifier: asset_metadata.identifier.clone(),
        size: image.dimensions(),
        bytes: image.to_vec(),
        srgb,
    })
}
pub(crate) fn texture_atlas_loader(asset_metadata: &AssetMetadata) -> Box<dyn Any> {
    let mut texture_atlas_path = asset_metadata.asset_path.clone();
    texture_atlas_path.push(
        asset_metadata
            .metadata
            .get("texture_atlas_description")
            .unwrap(),
    );
    let atlas_description_file = File::open(texture_atlas_path)
        .map_err(|e| GraphicsError::AtlasDescriptionFileOpenError(e))
        .unwrap();
    let reader = BufReader::new(atlas_description_file);
    let texture_atlas: TextureAtlas = serde_json::from_reader(reader)
        .map_err(|e| GraphicsError::SerdeError(e))
        .unwrap();

    Box::new(texture_atlas)
}

#[derive(Eq, PartialEq)]
pub enum TextureUsage {
    Albedo,
    Normal,
}

impl FromStr for TextureUsage {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "normal" => Normal,
            _ => Albedo,
        })
    }
}

pub const WHITE_TEXTURE_IDENTIFIER: &'static str = "_white";
pub const WHITE_TEXTURE_SIZE: (f32, f32) = (1.0, 1.0);
pub(crate) fn create_white_texture() -> TextureData {
    TextureData {
        identifier: WHITE_TEXTURE_IDENTIFIER.into(),
        size: (1, 1),
        bytes: vec![0xFF, 0xFF, 0xFF, 0xFF],
        srgb: true,
    }
}

pub const MISSING_TEXTURE_IDENTIFIER: &'static str = "_placeholder";
pub const MISSING_TEXTURE_REGION: TextureRegion = TextureRegion {
    x: 0.0,
    y: 0.0,
    width: 32.0,
    height: 32.0,
};
pub(crate) fn create_placeholder_texture() -> TextureData {
    let bytes = include_bytes!("../textures/default_texture.png");
    let image = image::load_from_memory(bytes).unwrap();
    let image = image.as_rgba8().unwrap();

    TextureData {
        identifier: MISSING_TEXTURE_IDENTIFIER.into(),
        size: image.dimensions(),
        bytes: image.to_vec(),
        srgb: true,
    }
}

pub const DEFAULT_NORMAL_MAP_IDENTIFIER: &'static str = "_normal_map";
pub(crate) fn create_normal_map_texture() -> TextureData {
    TextureData {
        identifier: DEFAULT_NORMAL_MAP_IDENTIFIER.into(),
        size: (1, 1),
        bytes: vec![0x80, 0x80, 0xFF, 0xFF],
        srgb: false,
    }
}
