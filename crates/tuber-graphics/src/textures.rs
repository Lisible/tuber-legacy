use std::any::Any;

use tuber_core::asset::AssetMetadata;

use crate::GraphicsError;
use crate::wgpu::*;

pub struct Texture {
    inner: WGPUTexture,
}

impl Texture {
    pub fn new(wgpu_texture: WGPUTexture) -> Self {
        Self {
            inner: wgpu_texture
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TextureSize {
    width: u32,
    height: u32,
}

impl TextureSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl From<(u32, u32)> for TextureSize {
    fn from((width, height): (u32, u32)) -> Self {
        Self {
            width,
            height,
        }
    }
}

impl From<TextureSize> for WGPUExtent3d {
    fn from(texture_size: TextureSize) -> Self {
        WGPUExtent3d {
            width: texture_size.width,
            height: texture_size.height,
            depth_or_array_layers: 1,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TextureFormat {
    Rgba8UnormSrgb
}

pub type TextureUsages = WGPUTextureUsages;

impl From<TextureFormat> for WGPUTextureFormat {
    fn from(texture_format: TextureFormat) -> Self {
        match texture_format {
            TextureFormat::Rgba8UnormSrgb => WGPUTextureFormat::Rgba8UnormSrgb
        }
    }
}

pub(crate) struct TextureAsset {
    identifier: String,
    size: TextureSize,
    bytes: Vec<u8>,
}

pub(crate) fn texture_loader(asset_metadata: &AssetMetadata) -> Box<dyn Any> {
    use image::io::Reader as ImageReader;
    let mut file_path = asset_metadata.asset_path.clone();
    file_path.push(asset_metadata.metadata.get("texture_data").unwrap());
    let image = ImageReader::open(file_path)
        .map_err(|e| GraphicsError::TextureFileOpenError(e))
        .unwrap()
        .decode()
        .map_err(|e| GraphicsError::ImageDecodeError(e))
        .unwrap();
    let image = image.as_rgba8().unwrap();

    Box::new(TextureAsset {
        identifier: asset_metadata.identifier.clone(),
        size: image.dimensions().into(),
        bytes: image.to_vec(),
    })
}