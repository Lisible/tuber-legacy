mod composition;
mod g_buffer;
mod geometry;
mod quad_renderer;
mod texture;
mod wgpu_state;

use crate::wgpu_state::WGPUState;
use tuber_core::asset::AssetStore;
use tuber_core::transform::Transform2D;
use tuber_ecs::EntityIndex;
use tuber_graphics::camera::OrthographicCamera;
use tuber_graphics::g_buffer::GBufferComponent;
use tuber_graphics::low_level::polygon_mode::PolygonMode;
use tuber_graphics::low_level::primitives::TextureId;
use tuber_graphics::low_level::{api::LowLevelGraphicsAPI, primitives::QuadDescription};
use tuber_graphics::texture::TextureData;
use tuber_graphics::types::{Size2, WindowSize};
use tuber_graphics::{types::Color, Window};

#[derive(Debug)]
pub enum TuberGraphicsWGPUError {
    WGPUSurfaceError(wgpu::SurfaceError),
}

pub enum MaybeUninitialized<T> {
    Initialized(T),
    Uninitialized,
}

impl<T> MaybeUninitialized<T> {
    fn assume_initialized(&self) -> &T {
        match self {
            MaybeUninitialized::Initialized(value) => value,
            _ => panic!("Tried to use an uninitialized value"),
        }
    }

    fn assume_initialized_mut(&mut self) -> &mut T {
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

    fn create_transparent_quad(&mut self, size: Size2) -> QuadDescription {
        self.state
            .assume_initialized_mut()
            .create_transparent_quad(size)
    }

    fn pre_draw_quads(&mut self, destination_quad: &QuadDescription, quads: &[QuadDescription]) {
        self.state
            .assume_initialized_mut()
            .pre_draw_quads(destination_quad, quads);
    }

    fn draw_quads(&mut self, quads: &[QuadDescription]) {
        self.state.assume_initialized_mut().draw_quads(quads);
    }

    fn is_texture_in_vram(&self, texture_id: TextureId) -> bool {
        self.state
            .assume_initialized()
            .is_texture_in_vram(texture_id)
    }

    fn load_texture_in_vram(&mut self, texture_data: &TextureData) -> TextureId {
        self.state
            .assume_initialized_mut()
            .load_texture_in_vram(texture_data)
    }

    fn update_camera(
        &mut self,
        camera_id: EntityIndex,
        camera: &OrthographicCamera,
        transform: &Transform2D,
    ) {
        self.state
            .assume_initialized_mut()
            .update_camera(camera_id, camera, transform);
    }

    fn set_clear_color(&mut self, color: Color) {
        self.state.assume_initialized_mut().set_clear_color(color);
    }

    fn set_rendered_g_buffer_component(&mut self, g_buffer_component: GBufferComponent) {
        self.state
            .assume_initialized_mut()
            .set_rendered_g_buffer_component(g_buffer_component);
    }

    fn set_polygon_mode(&mut self, polygon_mode: PolygonMode) {
        self.state
            .assume_initialized_mut()
            .set_polygon_mode(polygon_mode);
    }

    fn on_window_resized(&mut self, size: WindowSize) {
        self.state.assume_initialized_mut().resize(size);
    }
}
