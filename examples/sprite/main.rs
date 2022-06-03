use tuber::core::transform::Transform;
use tuber::ecs::ecs::Ecs;
use tuber::ecs::system::SystemBundle;
use tuber::engine::engine_context::EngineContext;
use tuber::engine::state::State;
use tuber::engine::Engine;
use tuber::engine::EngineSettings;
use tuber::engine::TuberRunner;
use tuber::graphics::camera::Camera;
use tuber::graphics::renderable::sprite::Sprite;
use tuber::WinitTuberRunner;

fn main() {
    let engine = Engine::new(EngineSettings {
        initial_state: Some(Box::new(MainState)),
        ..Default::default()
    });

    WinitTuberRunner.run(engine).unwrap();
}

struct MainState;

impl State for MainState {
    fn initialize(
        &mut self,
        _ecs: &mut Ecs,
        _system_bundles: &mut Vec<SystemBundle<EngineContext>>,
        engine_context: &mut EngineContext,
    ) {
        engine_context
            .graphics
            .set_camera(&Camera::new_orthographic_projection(
                0f32, 800f32, 0f32, 600f32, -100f32, 100f32,
            ))
            .unwrap();
    }

    fn render(&mut self, _ecs: &mut Ecs, engine_context: &mut EngineContext) {
        engine_context
            .graphics
            .draw_sprite(
                &Sprite::new("_placeholder", 100f32, 100f32),
                Transform {
                    translation: (100.0, 100.0, 0.0).into(),
                    ..Default::default()
                },
                Transform::default()
            )
            .unwrap();
    }
}
