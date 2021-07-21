use state::*;
use tuber_core::input;
use tuber_core::tilemap::Tilemap;
use tuber_core::transform::Transform2D;
use tuber_ecs::query::accessors::{R, W};
use tuber_graphics::camera::{Active, OrthographicCamera};
use tuber_graphics::shape::RectangleShape;
use tuber_graphics::sprite::{AnimatedSprite, Sprite};
use tuber_graphics::tilemap::TilemapRender;
use tuber_graphics::ui::{Frame, Image, NoViewTransform, Text};
use tuber_graphics::Graphics;

pub mod state;

pub struct Engine {
    state_stack: StateStack,
    graphics: Option<Graphics>,
}

impl Engine {
    pub fn new() -> Engine {
        Self {
            state_stack: StateStack::new(),
            graphics: None,
        }
    }

    pub fn ignite(mut self) -> Result<()> {
        loop {
            self.step(1.0);
        }
    }

    pub fn step(&mut self, delta_time: f64) {
        self.state_stack.update_current_state(delta_time);
    }

    pub fn handle_input(&mut self, input: input::Input) {
        self.state_stack.handle_input(input);
    }

    pub fn on_window_resized(&mut self, width: u32, height: u32) {
        self.graphics
            .as_mut()
            .expect("No graphics")
            .on_window_resized(width, height);
    }

    pub fn set_graphics(&mut self, graphics: Graphics) {
        self.graphics = Some(graphics);
    }

    pub fn graphics_mut(&mut self) -> Option<&mut Graphics> {
        self.graphics.as_mut()
    }

    pub fn state_stack_mut(&mut self) -> &mut StateStack {
        &mut self.state_stack
    }

    pub fn render(&mut self) {
        let graphics = self.graphics.as_mut().unwrap();
        for state_context in self.state_stack.state_contexts_mut() {
            let (camera_id, (camera, _, camera_transform)) = state_context
                .ecs
                .query_one::<(R<OrthographicCamera>, R<Active>, R<Transform2D>)>()
                .expect("There is no camera");
            graphics.update_camera(camera_id, &camera, &camera_transform);

            for (_, (tilemap, tilemap_render, transform)) in
                state_context
                    .ecs
                    .query::<(R<Tilemap>, R<TilemapRender>, R<Transform2D>)>()
            {
                graphics.prepare_tilemap(&tilemap, &tilemap_render, &transform);
            }

            for (_, (rectangle_shape, transform)) in state_context
                .ecs
                .query::<(R<RectangleShape>, R<Transform2D>)>()
            {
                graphics.prepare_rectangle(&rectangle_shape, &transform, true);
            }
            for (_, (sprite, transform)) in state_context.ecs.query::<(R<Sprite>, R<Transform2D>)>()
            {
                graphics.prepare_sprite(&sprite, &transform, true).unwrap();
            }
            for (_, (animated_sprite, transform)) in state_context
                .ecs
                .query::<(R<AnimatedSprite>, R<Transform2D>)>()
            {
                graphics
                    .prepare_animated_sprite(&animated_sprite, &transform, true)
                    .unwrap();
            }

            for (_, (mut tilemap_render,)) in state_context.ecs.query::<(W<TilemapRender>,)>() {
                tilemap_render.dirty = false;
            }

            for (id, (frame, transform)) in state_context.ecs.query::<(R<Frame>, R<Transform2D>)>()
            {
                let apply_view_transform = !state_context
                    .ecs
                    .query_one_by_id::<(R<NoViewTransform>,)>(id)
                    .is_some();
                graphics.prepare_rectangle(
                    &RectangleShape {
                        width: frame.width,
                        height: frame.height,
                        color: frame.color,
                    },
                    &transform,
                    apply_view_transform,
                );
            }

            for (id, (text, transform)) in state_context.ecs.query::<(R<Text>, R<Transform2D>)>() {
                let apply_view_transform = !state_context
                    .ecs
                    .query_one_by_id::<(R<NoViewTransform>,)>(id)
                    .is_some();
                graphics.prepare_text(text.text(), text.font(), &transform, apply_view_transform);
            }

            for (id, (image, transform)) in state_context.ecs.query::<(R<Image>, R<Transform2D>)>()
            {
                let apply_view_transform = !state_context
                    .ecs
                    .query_one_by_id::<(R<NoViewTransform>,)>(id)
                    .is_some();
                let sprite = Sprite {
                    width: image.width,
                    height: image.height,
                    texture: image.texture.clone(),
                };

                graphics
                    .prepare_sprite(&sprite, &transform, apply_view_transform)
                    .unwrap();
            }
        }

        graphics.render();
    }
}

pub struct EngineContext {
    pub delta_time: f64,
}

pub trait TuberRunner {
    fn run(&mut self, engine: Engine, graphics: Graphics) -> Result<()>;
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {}
