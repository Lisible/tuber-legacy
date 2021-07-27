use tuber::core::input::keyboard::Key;
use tuber::core::input::Input::{KeyDown, KeyUp};
use tuber::core::input::InputState;
use tuber::core::transform::Transform2D;
use tuber::ecs::ecs::Ecs;
use tuber::ecs::query::accessors::{R, W};
use tuber::ecs::system::SystemBundle;
use tuber::engine::state::{State, StateContext};
use tuber::engine::{Engine, Result, TuberRunner};
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::shape::RectangleShape;
use tuber::graphics::Graphics;
use tuber::graphics_wgpu::GraphicsWGPU;
use tuber::WinitTuberRunner;

fn main() -> Result<()> {
    let mut engine = Engine::new();
    let graphics = Graphics::new(Box::new(GraphicsWGPU::new()));

    engine.state_stack_mut().push_state(Box::new(MainState));

    WinitTuberRunner.run(engine, graphics)
}

struct MainState;
impl State for MainState {
    fn initialize(&mut self, state_context: &mut StateContext) {
        state_context.ecs.insert((
            RectangleShape {
                width: 100.0,
                height: 100.0,
                color: (1.0, 0.0, 0.0),
            },
            Transform2D {
                translation: (400.0, 300.0, 0),
                ..Default::default()
            },
        ));

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

        let mut bundle = SystemBundle::new();
        bundle.add_system(move_camera_right_system);
        state_context.system_bundles.push(bundle);
        state_context
            .system_bundles
            .push(Graphics::default_system_bundle());
    }
}

fn move_camera_right_system(ecs: &mut Ecs) {
    let input_state = ecs.shared_resource::<InputState>().unwrap();
    let (_, (_, mut transform)) = ecs
        .query_one::<(R<OrthographicCamera>, W<Transform2D>)>()
        .unwrap();

    if input_state.is(KeyDown(Key::Z)) && input_state.is(KeyUp(Key::S)) {
        transform.translation.1 -= 10.0;
    } else if input_state.is(KeyDown(Key::S)) && input_state.is(KeyUp(Key::Z)) {
        transform.translation.1 += 10.0;
    }
    if input_state.is(KeyDown(Key::Q)) && input_state.is(KeyUp(Key::D)) {
        transform.translation.0 -= 10.0;
    } else if input_state.is(KeyDown(Key::D)) && input_state.is(KeyUp(Key::Q)) {
        transform.translation.0 += 10.0;
    }
}
