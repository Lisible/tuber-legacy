mod wgpu_state;

use crate::wgpu_state::WGPUState;
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
        _quad_description: &QuadDescription,
        _transform: &Transform2D,
        _apply_view_transform: bool,
    ) {
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
        _camera_id: EntityIndex,
        _camera: &OrthographicCamera,
        _transform: &Transform2D,
    ) {
    }

    fn set_clear_color(&mut self, _color: Color) {}

    fn on_window_resized(&mut self, size: WindowSize) {
        self.state.assume_initialized().resize(size);
    }
}
