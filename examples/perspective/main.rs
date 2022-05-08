use tuber::core::transform::Transform;
use tuber::ecs::ecs::Ecs;
use tuber::ecs::system::SystemBundle;
use tuber::engine::engine_context::EngineContext;
use tuber::engine::state::State;
use tuber::engine::Engine;
use tuber::engine::EngineSettings;
use tuber::engine::TuberRunner;
use tuber::graphics::camera::Camera;
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
        _ecs: &mut Ecs,
        _system_bundles: &mut Vec<SystemBundle<EngineContext>>,
        engine_context: &mut EngineContext,
    ) {
        engine_context
            .graphics
            .set_camera(&Camera::new_perspective_projection(
                -0.8f32, 0.8f32, -0.45f32, 0.45f32, 0.1f32, 100f32,
            ))
            .unwrap();
    }

    fn render(&mut self, _ecs: &mut Ecs, engine_context: &mut EngineContext) {
        self.rx += 0.009;
        self.ry += 0.0035;
        engine_context
            .graphics
            .draw_cube(Transform {
                translation: (0.0, 0.0, -2.0f32).into(),
                angle: (self.rx, self.ry, 0.0).into(),
                ..Default::default()
            })
            .unwrap();
    }
}
