use crate::polygon_mode::PolygonMode;
use crate::primitives::TextureId;
use crate::types::{Color, WindowSize};
use crate::{GBufferComponent, OrthographicCamera, QuadDescription, Size2, TextureData, Window};
use tuber_core::asset::AssetStore;
use tuber_core::transform::Transform2D;
use tuber_ecs::EntityIndex;

/// The low level API
pub trait LowLevelGraphicsAPI {
    /// Initializes the API for a given window
    fn initialize(&mut self, window: Window, window_size: WindowSize, asset_store: &AssetStore);

    fn pre_draw_quads(&mut self, size: Size2<u32>, quads: &[QuadDescription]) -> QuadDescription;
    fn draw_quads(&mut self, quads: &[QuadDescription]);

    fn is_texture_in_vram(&self, texture_id: TextureId) -> bool;
    fn load_texture_in_vram(&mut self, texture_data: &TextureData) -> TextureId;

    /// Updates the view/projection matrix
    fn update_camera(
        &mut self,
        camera_id: EntityIndex,
        camera: &OrthographicCamera,
        transform: &Transform2D,
    );

    fn set_clear_color(&mut self, color: Color);
    fn set_rendered_g_buffer_component(&mut self, g_buffer_component: GBufferComponent);
    fn set_polygon_mode(&mut self, polygon_mode: PolygonMode);
    fn on_window_resized(&mut self, size: WindowSize);
}
