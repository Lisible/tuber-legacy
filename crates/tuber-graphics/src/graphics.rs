use tuber_core::transform::Transform;

use crate::camera::OrthographicCamera;
use crate::low_level::renderer::Renderer;
use crate::renderable::rectangle_shape::RectangleShape;
use crate::GraphicsError;
use crate::GraphicsResult;
use crate::Window;

#[derive(Default)]
pub struct Graphics {
    renderer: Option<Renderer>,
}

impl Graphics {
    /// Initializes the renderer
    pub fn initialize(&mut self, window: Window, window_size: (u32, u32)) {
        self.renderer = Some(Renderer::new(window, window_size));
    }

    /// Draws a rectangle shape
    pub fn draw_rectangle_shape(
        &mut self,
        rectangle_shape: RectangleShape,
        world_transform: Transform,
    ) -> GraphicsResult<()> {
        self.renderer()?
            .queue_mesh(rectangle_shape.into(), world_transform, "_white");
        Ok(())
    }

    /// Set the camera used for rendering
    pub fn set_camera(&mut self, camera: &OrthographicCamera) -> GraphicsResult<()> {
        self.renderer()?
            .set_view_projection_matrix(camera.projection_matrix());
        Ok(())
    }

    /// Renders the scene
    pub fn render_scene(&mut self) -> GraphicsResult<()> {
        self.renderer()?.render()
    }

    pub fn renderer(&mut self) -> GraphicsResult<&mut Renderer> {
        self.renderer
            .as_mut()
            .ok_or(GraphicsError::RendererUninitialized)
    }
}
