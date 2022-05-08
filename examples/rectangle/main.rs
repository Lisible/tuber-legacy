use tuber::core::transform::Transform;
use tuber::ecs::ecs::Ecs;
use tuber::ecs::system::SystemBundle;
use tuber::engine::engine_context::EngineContext;
use tuber::engine::state::State;
use tuber::engine::Engine;
use tuber::engine::EngineSettings;
use tuber::engine::TuberRunner;
use tuber::graphics::camera::OrthographicCamera;
use tuber::graphics::color::Color;
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
                right: 800.0,
                top: 0.0,
                bottom: 600.0,
                far: 100.0,
                near: -100.0,
            })
            .unwrap();
    }

    fn render(&mut self, _ecs: &mut Ecs, engine_context: &mut EngineContext) {
        engine_context
            .graphics
            .draw_rectangle_shape(
                RectangleShape::new(100.0, 100.0, Color::from_rgb(0.0, 0.0, 1.0)),
                Transform {
                    translation: (100.0, 100.0, 0.0).into(),
                    ..Default::default()
                },
            )
            .unwrap();
        engine_context
            .graphics
            .draw_rectangle_shape(
                RectangleShape::new(50.0, 50.0, Color::from_rgb(1.0, 0.0, 0.0)),
                Transform::default(),
            )
            .unwrap();
        engine_context
            .graphics
            .draw_rectangle_shape(
                RectangleShape::new(25.0, 25.0, Color::from_rgb(0.0, 1.0, 0.0)),
                Transform {
                    translation: (200.0, 200.0, 0.0).into(),
                    angle: (0.0, 0.0, 0.13).into(),
                    ..Default::default()
                },
            )
            .unwrap();
    }
}
