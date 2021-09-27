use tuber::core::input::keyboard::Key;
use tuber::core::input::{Input, InputState};
use tuber::core::transform::Transform2D;
use tuber::ecs::EntityIndex;
use tuber::engine::state::{State, StateContext, StateStackRequest};
use tuber::engine::{Engine, TuberRunner};
use tuber::engine::{EngineSettings, Result};
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::ui::Text;
use tuber::graphics::Graphics;
use tuber::graphics_wgpu::GraphicsWGPU;
use tuber::*;

fn main() -> Result<()> {
    let mut graphics = Graphics::new(Box::new(GraphicsWGPU::new()));
    graphics.set_clear_color((1.0, 1.0, 1.0));

    let mut engine = Engine::new(EngineSettings {
        graphics: Some(graphics),
        ..Default::default()
    });

    engine.push_initial_state(Box::new(GameState::new()));

    WinitTuberRunner.run(engine)
}

struct GameState {
    should_pause: bool,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            should_pause: false,
        }
    }
}

impl State for GameState {
    fn initialize(&mut self, state_context: &mut StateContext) {
        state_context.ecs.insert((
            OrthographicCamera {
                left: 0.0,
                right: 800.0,
                top: 0.0,
                bottom: 600.0,
                near: -100.0,
                far: 100.0,
            },
            Active,
            Transform2D::default(),
        ));

        state_context.ecs.insert((
            Text::new("Game state", "examples/game-state/font.json"),
            Transform2D {
                translation: (100.0, 100.0, 0),
                scale: (2.0, 2.0),
                angle: -5.0,
                ..Default::default()
            },
        ));
    }

    fn update(&mut self, state_context: &mut StateContext) {
        let input = state_context
            .ecs
            .shared_resource::<InputState>()
            .expect("Input state");

        if input.was(Input::KeyUp(Key::S)) && input.is(Input::KeyDown(Key::S)) {
            self.should_pause = true;
        }
    }

    fn stack_requests(&mut self) -> Vec<StateStackRequest> {
        if self.should_pause {
            self.should_pause = false;
            return vec![StateStackRequest::Push(Box::new(Pause::new()))];
        }

        vec![]
    }
}

struct Pause {
    should_resume: bool,
    pause_label: Option<EntityIndex>,
}

impl Pause {
    pub fn new() -> Self {
        Self {
            should_resume: false,
            pause_label: None,
        }
    }
}

impl State for Pause {
    fn initialize(&mut self, state_context: &mut StateContext) {
        state_context.ecs.insert((
            OrthographicCamera {
                left: 0.0,
                right: 800.0,
                top: 0.0,
                bottom: 600.0,
                near: -100.0,
                far: 100.0,
            },
            Active,
            Transform2D::default(),
        ));

        self.pause_label = Some(state_context.ecs.insert((
            Text::new("Pause state", "examples/game-state/font.json"),
            Transform2D {
                translation: (100.0, 100.0, 0),
                angle: 5.0,
                scale: (2.0, 2.0),
                ..Default::default()
            },
        )));
    }

    fn update(&mut self, state_context: &mut StateContext) {
        {
            let input = state_context
                .ecs
                .shared_resource::<InputState>()
                .expect("Input state");

            if input.was(Input::KeyUp(Key::S)) && input.is(Input::KeyDown(Key::S)) {
                self.should_resume = true;
            }
        }

        if self.should_resume {
            if let Some(pause_label) = self.pause_label {
                state_context.ecs.delete_by_ids(&[pause_label]);
            }

            self.pause_label = None;
        }
    }

    fn stack_requests(&mut self) -> Vec<StateStackRequest> {
        if self.should_resume {
            self.should_resume = false;
            return vec![StateStackRequest::Pop];
        }

        vec![]
    }
}
