use log::info;
use tuber_core::transform::{AsMatrix4, LocalTransform, Transform};
use tuber_ecs::ecs::Ecs;

use crate::camera::{ActiveCamera, Camera};
use crate::low_level::mesh::Mesh;
use crate::low_level::model::Model;
use crate::low_level::renderer::Renderer;
use crate::renderable::rectangle_shape::RectangleShape;
use crate::renderable::sprite::Sprite;
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
        info!("Initializing renderer");
        self.renderer = Some(Renderer::new(window, window_size));
    }

    /// Draws a model with the given world transform
    pub fn draw_model(
        &mut self,
        model: Model,
        world_transform: Transform,
        local_transform: Transform,
    ) -> GraphicsResult<()> {
        for mesh in model.meshes {
            self.renderer()?
                .queue_mesh(mesh, world_transform, local_transform, "_white");
        }

        Ok(())
    }

    /// Draws a rectangle shape with the given world transform
    pub fn draw_rectangle_shape(
        &mut self,
        rectangle_shape: RectangleShape,
        world_transform: Transform,
        local_transform: Transform,
    ) -> GraphicsResult<()> {
        self.renderer()?.queue_mesh(
            rectangle_shape.into(),
            world_transform,
            local_transform,
            "_white",
        );
        Ok(())
    }

    /// Draws a cube with the given world transform
    pub fn draw_cube(
        &mut self,
        world_transform: Transform,
        local_transform: Transform,
    ) -> GraphicsResult<()> {
        self.renderer()?.queue_mesh(
            Mesh::new_cube_mesh(),
            world_transform,
            local_transform,
            "_white",
        );
        Ok(())
    }

    /// Draws a sprite with the given world transform
    pub fn draw_sprite(
        &mut self,
        sprite: &Sprite,
        world_transform: Transform,
        local_transform: Transform,
    ) -> GraphicsResult<()> {
        self.renderer()?.queue_mesh(
            sprite.as_mesh(),
            world_transform,
            local_transform,
            sprite.texture_identifier(),
        );
        Ok(())
    }

    /// Renders the scene
    pub fn render_scene(&mut self, ecs: &Ecs) -> GraphicsResult<()> {
        // Use the active camera's projection matrix
        let (_, (camera, _, camera_local_transform, camera_transform)) = ecs
            .query_one::<(&Camera, &ActiveCamera, &LocalTransform, &Transform)>()
            .expect("There is no active camera in the scene");

        let view_projection_matrix = camera.projection_matrix()
            * camera_local_transform.0.as_matrix4()
            * camera_transform.as_matrix4();
        let renderer = self.renderer()?;
        renderer.set_view_projection_matrix(view_projection_matrix);
        renderer.render()
    }

    fn renderer(&mut self) -> GraphicsResult<&mut Renderer> {
        self.renderer
            .as_mut()
            .ok_or(GraphicsError::RendererUninitialized)
    }
}
