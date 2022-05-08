use tuber_core::transform::Transform;

use crate::camera::Camera;
use crate::low_level::mesh::Mesh;
use crate::low_level::primitives::Vertex;
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

    pub fn draw_cube(&mut self, world_transform: Transform) -> GraphicsResult<()> {
        let cube_mesh = Mesh {
            vertices: vec![
                Vertex {
                    position: [-1f32, 1f32, 1f32],
                    color: [1f32, 0f32, 0f32],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    // green
                    position: [-1f32, -1f32, 1f32],
                    color: [0f32, 1f32, 0f32],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    position: [1f32, 1f32, 1f32],
                    color: [0f32, 0f32, 1f32],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    // purple
                    position: [1f32, -1f32, 1f32],
                    color: [1f32, 0f32, 1f32],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    position: [-1f32, 1f32, -1f32],
                    color: [0f32, 1f32, 1f32],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    // yellow
                    position: [-1f32, -1f32, -1f32],
                    color: [1f32, 1f32, 0f32],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    position: [1f32, 1f32, -1f32],
                    color: [1f32, 1f32, 1f32],
                    texture_coordinates: [0.0, 0.0],
                },
                Vertex {
                    // black
                    position: [1f32, -1f32, -1f32],
                    color: [0f32, 0f32, 0f32],
                    texture_coordinates: [0.0, 0.0],
                },
            ],
            indices: vec![
                0, 2, 3, 0, 3, 1, 2, 6, 7, 2, 7, 3, 6, 4, 5, 6, 5, 7, 4, 0, 1, 4, 1, 5, 0, 4, 6, 0,
                6, 2, 1, 7, 5, 1, 3, 7,
            ],
        };

        self.renderer()?
            .queue_mesh(cube_mesh, world_transform, "_white");
        Ok(())
    }

    /// Set the camera used for rendering
    pub fn set_camera(&mut self, camera: &Camera) -> GraphicsResult<()> {
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
