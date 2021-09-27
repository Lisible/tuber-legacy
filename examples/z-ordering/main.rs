use tuber::core::transform::Transform2D;
use tuber::engine::state::{State, StateContext};
use tuber::engine::{Engine, EngineSettings, Result, TuberRunner};
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::shape::RectangleShape;
use tuber::graphics::Graphics;
use tuber::graphics_wgpu::GraphicsWGPU;
use tuber::WinitTuberRunner;

fn main() -> Result<()> {
    let mut graphics = Graphics::new(Box::new(GraphicsWGPU::new()));
    graphics.set_clear_color((1.0, 1.0, 1.0));

    let mut engine = Engine::new(EngineSettings {
        graphics: Some(graphics),
        ..Default::default()
    });

    engine.push_initial_state(Box::new(MainState));

    WinitTuberRunner.run(engine)
}

struct MainState;
impl State for MainState {
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
            Transform2D {
                translation: (0.0, 0.0, 0),
                ..Default::default()
            },
            Active,
        ));

        state_context.ecs.insert((
            RectangleShape {
                width: 100.0,
                height: 100.0,
                color: (0.0, 0.0, 1.0),
            },
            Transform2D {
                translation: (100.0, 100.0, 4),
                ..Default::default()
            },
        ));

        state_context.ecs.insert((
            RectangleShape {
                width: 100.0,
                height: 100.0,
                color: (0.0, 1.0, 0.0),
            },
            Transform2D {
                translation: (75.0, 150.0, 1),
                ..Default::default()
            },
        ));

        state_context.ecs.insert((
            RectangleShape {
                width: 100.0,
                height: 100.0,
                color: (1.0, 0.0, 0.0),
            },
            Transform2D {
                translation: (150.0, 150.0, 2),
                ..Default::default()
            },
        ));

        state_context.ecs.insert((
            RectangleShape {
                width: 100.0,
                height: 100.0,
                color: (1.0, 1.0, 0.0),
            },
            Transform2D {
                translation: (100.0, 200.0, 3),
                ..Default::default()
            },
        ));

        state_context
            .system_bundles
            .push(Graphics::default_system_bundle());
    }
}
