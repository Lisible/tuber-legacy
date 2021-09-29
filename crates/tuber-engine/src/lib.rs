use state::*;
use tuber_core::input;
use tuber_core::input::{InputState, Keymap};
use tuber_core::tilemap::Tilemap;
use tuber_core::transform::Transform2D;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::query::accessors::{R, W};
use tuber_ecs::system::SystemBundle;
use tuber_graphics::camera::{Active, OrthographicCamera};
use tuber_graphics::shape::RectangleShape;
use tuber_graphics::sprite::{AnimatedSprite, Sprite};
use tuber_graphics::tilemap::TilemapRender;
use tuber_graphics::ui::{Frame, Image, NoViewTransform, Text};
use tuber_graphics::Graphics;

pub mod state;

pub struct EngineSettings {
    pub graphics: Option<Graphics>,
    pub application_title: Option<String>,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            graphics: None,
            application_title: None,
        }
    }
}

pub struct Engine {
    state_stack: StateStack,
    ecs: Ecs,
    system_bundles: Vec<SystemBundle>,
    graphics: Option<Graphics>,
    application_title: String,
}

fn create_ecs() -> Ecs {
    const KEYMAP_FILE: &'static str = "keymap.json";
    let mut ecs = Ecs::new();
    ecs.insert_shared_resource(InputState::new(
        Keymap::from_file(KEYMAP_FILE).unwrap_or(Keymap::default()),
    ));
    ecs
}

impl Engine {
    pub fn new(settings: EngineSettings) -> Engine {
        Self {
            state_stack: StateStack::new(),
            ecs: create_ecs(),
            system_bundles: vec![],
            graphics: settings.graphics,
            application_title: settings
                .application_title
                .unwrap_or("tuber Application".into()),
        }
    }

    pub fn application_title(&self) -> &str {
        &self.application_title
    }

    pub fn push_initial_state(&mut self, state: Box<dyn State>) {
        self.state_stack.push_state(
            state,
            &mut StateContext {
                ecs: &mut self.ecs,
                system_bundles: &mut self.system_bundles,
            },
        )
    }

    pub fn ignite(mut self) -> Result<()> {
        loop {
            self.step(1.0);
        }
    }

    pub fn step(&mut self, delta_time: f64) {
        let mut state_context = StateContext {
            ecs: &mut self.ecs,
            system_bundles: &mut self.system_bundles,
        };
        self.state_stack
            .update_current_state(delta_time, &mut state_context);
    }

    pub fn handle_input(&mut self, input: input::Input) {
        let mut state_context = StateContext {
            ecs: &mut self.ecs,
            system_bundles: &mut self.system_bundles,
        };
        self.state_stack.handle_input(input, &mut state_context);
    }

    pub fn on_window_resized(&mut self, width: u32, height: u32) {
        self.graphics
            .as_mut()
            .expect("No graphics")
            .on_window_resized(width, height);
    }

    pub fn graphics_mut(&mut self) -> Option<&mut Graphics> {
        self.graphics.as_mut()
    }

    pub fn state_stack_mut(&mut self) -> &mut StateStack {
        &mut self.state_stack
    }

    pub fn render(&mut self) {
        let graphics = self.graphics.as_mut().unwrap();
        let (camera_id, (camera, _, camera_transform)) = self
            .ecs
            .query_one::<(R<OrthographicCamera>, R<Active>, R<Transform2D>)>()
            .expect("There is no camera");
        graphics.update_camera(camera_id, &camera, &camera_transform);

        for (_, (tilemap, tilemap_render, transform)) in
            self.ecs
                .query::<(R<Tilemap>, R<TilemapRender>, R<Transform2D>)>()
        {
            graphics.prepare_tilemap(&tilemap, &tilemap_render, &transform);
        }

        for (_, (rectangle_shape, transform)) in
            self.ecs.query::<(R<RectangleShape>, R<Transform2D>)>()
        {
            graphics.prepare_rectangle(&rectangle_shape, &transform, true);
        }
        for (_, (sprite, transform)) in self.ecs.query::<(R<Sprite>, R<Transform2D>)>() {
            graphics.prepare_sprite(&sprite, &transform, true).unwrap();
        }
        for (_, (animated_sprite, transform)) in
            self.ecs.query::<(R<AnimatedSprite>, R<Transform2D>)>()
        {
            graphics
                .prepare_animated_sprite(&animated_sprite, &transform, true)
                .unwrap();
        }

        for (_, (mut tilemap_render,)) in self.ecs.query::<(W<TilemapRender>,)>() {
            tilemap_render.dirty = false;
        }

        for (id, (frame, transform)) in self.ecs.query::<(R<Frame>, R<Transform2D>)>() {
            let apply_view_transform = !self
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

        for (id, (text, transform)) in self.ecs.query::<(R<Text>, R<Transform2D>)>() {
            let apply_view_transform = !self
                .ecs
                .query_one_by_id::<(R<NoViewTransform>,)>(id)
                .is_some();
            graphics.prepare_text(text.text(), text.font(), &transform, apply_view_transform);
        }

        for (id, (image, transform)) in self.ecs.query::<(R<Image>, R<Transform2D>)>() {
            let apply_view_transform = !self
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
        graphics.render();
    }
}

pub struct EngineContext {
    pub delta_time: f64,
}

pub trait TuberRunner {
    fn run(&mut self, engine: Engine) -> Result<()>;
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {}
