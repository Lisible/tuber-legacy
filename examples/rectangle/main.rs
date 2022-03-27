use tuber::ecs::ecs::Ecs;
use tuber::ecs::system::SystemBundle;
use tuber::engine::engine_context::EngineContext;
use tuber::engine::state::State;
use tuber::engine::Engine;
use tuber::engine::EngineSettings;
use tuber::engine::TuberRunner;
use tuber::graphics::camera::OrthographicCamera;
use tuber::graphics::renderable::rectangle_shape::RectangleShape;
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
            .set_camera(&OrthographicCamera {
                left: 0.0,
                right: 1.0,
                top: 0.0,
                bottom: 1.0,
                far: 100.0,
                near: -100.0,
            })
            .unwrap();
    }

    fn render(&mut self, _ecs: &mut Ecs, engine_context: &mut EngineContext) {
        engine_context
            .graphics
            .draw_rectangle_shape(RectangleShape::new(1.0, 1.0))
            .unwrap();
    }
}
