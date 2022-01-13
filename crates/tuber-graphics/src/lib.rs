use image::ImageError;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use crate::bitmap_font::BitmapFont;
use crate::camera::{Active, OrthographicCamera};
use crate::g_buffer::GBufferComponent;
use crate::immediate_gui::ImmediateGUI;
use crate::low_level::*;
use crate::material::Material;
use crate::polygon_mode::PolygonMode;
use crate::primitives::{MaterialDescription, QuadDescription, TextureDescription};
use crate::renderable::shape::RectangleShape;
use crate::renderable::sprite::{AnimatedSprite, Sprite};
use crate::renderable::tilemap::{Tile, Tilemap};
use crate::texture::{
    texture_atlas_loader, texture_loader, TextureAtlas, TextureData, TextureMetadata,
    TextureRegion, DEFAULT_NORMAL_MAP_IDENTIFIER,
};
use crate::types::{Color, Size2, WindowSize};
use low_level::wgpu_state::WGPUState;

pub mod animation;
pub mod bitmap_font;
pub mod camera;
pub mod g_buffer;
pub mod graphics;
pub mod immediate_gui;
pub mod low_level;
pub mod material;
pub mod renderable;
pub mod texture;
pub mod types;

#[derive(Debug)]
pub enum GraphicsError {
    WGPUSurfaceError(wgpu::SurfaceError),
    TextureFileOpenError(std::io::Error),
    TextureMetadataNotFound,
    AtlasDescriptionFileOpenError(std::io::Error),
    ImageDecodeError(ImageError),
    SerdeError(serde_json::error::Error),
    BitmapFontFileReadError(std::io::Error),
}

pub struct Window<'a>(pub Box<&'a dyn HasRawWindowHandle>);
unsafe impl HasRawWindowHandle for Window<'_> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0.raw_window_handle()
    }
}

#[derive(Copy, Clone)]
pub struct RenderId(pub usize);
