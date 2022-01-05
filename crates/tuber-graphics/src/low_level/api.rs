use crate::types::{Color, WindowSize};
use crate::{GBufferComponent, OrthographicCamera, QuadDescription, TextureData, Window};
use tuber_core::asset::AssetStore;
use tuber_core::transform::Transform2D;
use tuber_ecs::EntityIndex;

/// The low level API
pub trait LowLevelGraphicsAPI {
    /// Initializes the API for a given window
    fn initialize(&mut self, window: Window, window_size: WindowSize, asset_store: &AssetStore);
    /// Renders
    fn render(&mut self);

    /// Prepares the render of a quad
    fn prepare_quad(
        &mut self,
        quad_description: &QuadDescription,
        transform: &Transform2D,
        apply_view_transform: bool,
    );

    fn is_texture_in_vram(&self, texture_identifier: &str) -> bool;
    fn load_texture_in_vram(&mut self, texture_data: &TextureData);

    /// Updates the view/projection matrix
    fn update_camera(
        &mut self,
        camera_id: EntityIndex,
        camera: &OrthographicCamera,
        transform: &Transform2D,
    );

    fn set_clear_color(&mut self, color: Color);
    fn set_rendered_g_buffer_component(&mut self, g_buffer_component: GBufferComponent);
    fn on_window_resized(&mut self, size: WindowSize);
}
