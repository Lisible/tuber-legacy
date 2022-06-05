use tuber::core::transform::{LocalTransform, Transform};
use tuber::ecs::ecs::Ecs;
use tuber::ecs::system::SystemBundle;
use tuber::engine::engine_context::EngineContext;
use tuber::engine::state::State;
use tuber::engine::Engine;
use tuber::engine::EngineSettings;
use tuber::engine::TuberRunner;
use tuber::graphics::camera::{ActiveCamera, Camera};
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
        ecs: &mut Ecs,
        _system_bundles: &mut Vec<SystemBundle<EngineContext>>,
        _engine_context: &mut EngineContext,
    ) {
        ecs.insert((
            Camera::new_orthographic_projection(0f32, 800f32, 0f32, 600f32, -100f32, 100f32),
            ActiveCamera,
            Transform::default(),
            LocalTransform::default(),
        ));
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
