use tuber::core::transform::Transform;
use tuber::ecs::ecs::Ecs;
use tuber::ecs::system::SystemBundle;
use tuber::engine::engine_context::EngineContext;
use tuber::engine::state::State;
use tuber::engine::Engine;
use tuber::engine::EngineSettings;
use tuber::engine::TuberRunner;
use tuber::graphics::camera::Camera;
use tuber::graphics::parsers::obj_parser::ObjParser;
use tuber::graphics::parsers::ModelParser;
use tuber::WinitTuberRunner;

fn main() {
    env_logger::init();
    let engine = Engine::new(EngineSettings {
        initial_state: Some(Box::new(MainState {
            angle_y: 0f32,
            angle_x: 0f32,
        })),
        ..Default::default()
    });

    WinitTuberRunner.run(engine).unwrap();
}

struct MainState {
    angle_y: f32,
    angle_x: f32,
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
                45f32,
                800f32 / 600f32,
                1.0,
                100.0,
            ))
            .unwrap();
    }

    fn render(&mut self, _ecs: &mut Ecs, engine_context: &mut EngineContext) {
        self.angle_y += 0.01;
        self.angle_x += 0.04;
        let model = ObjParser::parse_model(include_str!("./model.obj")).unwrap();

        engine_context
            .graphics
            .draw_model(
                model,
                Transform {
                    angle: (self.angle_x, self.angle_y / 2.0, 0.0).into(),
                    translation: (0.0, 0.0, -10.0).into(),
                    ..Default::default()
                },
                Transform::default(),
            )
            .unwrap();
    }
}
