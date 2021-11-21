mod geometry;
mod quad_renderer;
mod wgpu_state;

use crate::wgpu_state::WGPUState;
use std::cmp::Ordering;
use std::ops::Range;
use tuber_core::asset::AssetStore;
use tuber_core::tilemap::Tilemap;
use tuber_core::transform::Transform2D;
use tuber_ecs::EntityIndex;
use tuber_graphics::camera::OrthographicCamera;
use tuber_graphics::low_level::{LowLevelGraphicsAPI, QuadDescription};
use tuber_graphics::texture::Texture;
use tuber_graphics::tilemap::TilemapRender;
use tuber_graphics::{Color, Window, WindowSize};

#[derive(Debug)]
pub enum TuberGraphicsWGPUError {
    WGPUSurfaceError(wgpu::SurfaceError),
}

pub enum MaybeUninitialized<T> {
    Initialized(T),
    Uninitialized,
}

impl<T> MaybeUninitialized<T> {
    fn assume_initialized(&mut self) -> &mut T {
        match self {
            MaybeUninitialized::Initialized(value) => value,
            _ => panic!("Tried to use an uninitialized value"),
        }
    }
}

pub struct GraphicsWGPU {
    state: MaybeUninitialized<WGPUState>,
}
impl GraphicsWGPU {
    pub fn new() -> Self {
        Self {
            state: MaybeUninitialized::Uninitialized,
        }
    }
}

impl LowLevelGraphicsAPI for GraphicsWGPU {
    fn initialize(&mut self, window: Window, window_size: WindowSize, _asset_store: &AssetStore) {
        self.state = MaybeUninitialized::Initialized(WGPUState::new(window, window_size));
    }

    fn render(&mut self) {
        self.state.assume_initialized().render().unwrap();
    }

    fn prepare_quad(
        &mut self,
        quad_description: &QuadDescription,
        transform: &Transform2D,
        _apply_view_transform: bool,
    ) {
        self.state
            .assume_initialized()
            .prepare_quad(quad_description, transform);
    }

    fn prepare_tilemap(
        &mut self,
        _tilemap: &Tilemap,
        _tilemap_render: &TilemapRender,
        _transform: &Transform2D,
        _asset_store: &AssetStore,
    ) {
    }

    fn is_texture_in_vram(&self, _texture_identifier: &str) -> bool {
        false
    }

    fn load_texture(&mut self, _texture_data: &Texture) {}

    fn update_camera(
        &mut self,
        camera_id: EntityIndex,
        camera: &OrthographicCamera,
        transform: &Transform2D,
    ) {
        self.state
            .assume_initialized()
            .update_camera(camera_id, camera, transform);
    }

    fn set_clear_color(&mut self, _color: Color) {}

    fn on_window_resized(&mut self, size: WindowSize) {
        self.state.assume_initialized().resize(size);
    }
}

#[derive(Eq, PartialEq)]
pub struct DrawCommand {
    pub draw_command_data: DrawCommandData,
    pub z_order: i32,
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum DrawCommandData {
    QuadDrawCommand(QuadDrawCommand),
    TilemapDrawCommand(TilemapDrawCommand),
}

impl DrawCommand {
    pub fn draw_type(&self) -> DrawType {
        match self.draw_command_data {
            DrawCommandData::QuadDrawCommand(_) => DrawType::Quad,
            DrawCommandData::TilemapDrawCommand(_) => DrawType::Tilemap,
        }
    }
}

#[derive(Eq, PartialEq)]
pub struct QuadDrawCommand {
    pub draw_range: DrawRange,
    pub uniform_offset: wgpu::DynamicOffset,
    pub texture: Option<String>,
}

impl Ord for QuadDrawCommand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.texture.cmp(&other.texture)
    }
}

impl PartialOrd for QuadDrawCommand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

#[derive(Eq, PartialEq, PartialOrd)]
pub struct TilemapDrawCommand {
    pub tilemap_identifier: String,
}

impl Ord for TilemapDrawCommand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tilemap_identifier.cmp(&other.tilemap_identifier)
    }
}

impl Ord for DrawCommand {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut sort = self.z_order.cmp(&other.z_order);

        if sort == Ordering::Equal {
            sort = self.draw_command_data.cmp(&other.draw_command_data);
        }

        sort
    }
}

impl PartialOrd for DrawCommand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub enum DrawType {
    Quad,
    Tilemap,
}

#[derive(Eq, PartialEq)]
pub enum DrawRange {
    VertexIndexRange(Range<u32>),
    InstanceIndexRange(Range<u32>),
}

impl DrawRange {
    pub fn vertex_index_range(&self) -> Option<&Range<u32>> {
        match self {
            DrawRange::VertexIndexRange(range) => Some(range),
            _ => None,
        }
    }

    pub fn instance_index_range(&self) -> Option<&Range<u32>> {
        match self {
            DrawRange::InstanceIndexRange(range) => Some(range),
            _ => None,
        }
    }
}
