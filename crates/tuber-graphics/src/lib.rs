use image::ImageError;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

pub mod camera;
pub mod color;
pub mod graphics;
pub mod low_level;
pub mod parsers;
pub mod renderable;

#[derive(Debug)]
pub enum GraphicsError {
    WGPUSurfaceError(wgpu::SurfaceError),
    TextureFileOpenError(std::io::Error),
    TextureMetadataNotFound,
    AtlasDescriptionFileOpenError(std::io::Error),
    ImageDecodeError(ImageError),
    SerdeError(serde_json::error::Error),

    BitmapFontFileReadError(std::io::Error),
    RendererUninitialized,
}

pub type GraphicsResult<T> = Result<T, GraphicsError>;

pub struct Window<'a>(pub Box<&'a dyn HasRawWindowHandle>);

unsafe impl HasRawWindowHandle for Window<'_> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0.raw_window_handle()
    }
}
