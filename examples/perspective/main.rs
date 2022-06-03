use tuber::core::transform::{LocalTransform, Transform};
use tuber::ecs::ecs::Ecs;
use tuber::ecs::system::SystemBundle;
use tuber::engine::engine_context::EngineContext;
use tuber::engine::state::State;
use tuber::engine::Engine;
use tuber::engine::EngineSettings;
use tuber::engine::TuberRunner;
use tuber::graphics::camera::{ActiveCamera, Camera};
use tuber::WinitTuberRunner;

fn main() {
    let engine = Engine::new(EngineSettings {
        initial_state: Some(Box::new(MainState { rx: 0.0, ry: 0.0 })),
        ..Default::default()
    });

    WinitTuberRunner.run(engine).unwrap();
}

struct MainState {
    rx: f32,
    ry: f32,
}

impl State for MainState {
    fn initialize(
        &mut self,
        ecs: &mut Ecs,
        _system_bundles: &mut Vec<SystemBundle<EngineContext>>,
        _engine_context: &mut EngineContext,
    ) {
        ecs.insert((
            Camera::new_perspective_projection(45f32, 800f32 / 600f32, 1f32, 10f32),
            ActiveCamera,
            Transform::default(),
            LocalTransform::default(),
        ));
    }

    fn render(&mut self, _ecs: &mut Ecs, engine_context: &mut EngineContext) {
        self.rx += 0.009;
        self.ry += 0.0035;
        engine_context
            .graphics
            .draw_cube(
                Transform {
                    translation: (0.0, 0.0, -20.0f32).into(),
                    ..Default::default()
                },
                Transform {
                    angle: (self.rx, self.ry, 0.0).into(),
                    ..Default::default()
                },
            )
            .unwrap();
    }
}
