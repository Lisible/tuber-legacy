use tuber::core::transform::Transform;
use tuber::ecs::ecs::Ecs;
use tuber::ecs::system::SystemBundle;
use tuber::engine::engine_context::EngineContext;
use tuber::engine::state::State;
use tuber::engine::Engine;
use tuber::engine::EngineSettings;
use tuber::engine::TuberRunner;
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
    fn render(&mut self, _ecs: &mut Ecs, engine_context: &mut EngineContext) { 
       engine_context
            .graphics
            .draw_rectangle_shape(RectangleShape::new(1.0, 1.0))
            .unwrap();
    }
}
