use tuber::core::input::keyboard::Key;
use tuber::core::input::Input::{KeyDown, KeyUp};
use tuber::core::input::InputState;
use tuber::core::transform::Transform2D;
use tuber::ecs::ecs::Ecs;
use tuber::ecs::query::accessors::{R, W};
use tuber::ecs::system::SystemBundle;
use tuber::engine::state::{State, StateContext};
use tuber::engine::{Engine, TuberRunner};
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::ui::Text;
use tuber::graphics::Graphics;
use tuber::graphics_wgpu::GraphicsWGPU;
use tuber::WinitTuberRunner;

fn main() -> tuber::engine::Result<()> {
    let mut engine = Engine::new();
    let mut runner = WinitTuberRunner;
    let mut graphics = Graphics::new(Box::new(GraphicsWGPU::new()));
    graphics.set_clear_color((1.0, 1.0, 1.0));
    engine.state_stack_mut().push_state(Box::new(MainState));
    runner.run(engine, graphics)
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
                translation: (0.0, 0.0),
                ..Default::default()
            },
            Active,
        ));

        state_context.ecs.insert((
            Text::new(
                "Hello World\nThis is a second line",
                "examples/bitmap-font/font.json",
            ),
            Transform2D {
                translation: (100.0, 100.0),
                angle: 0.0,
                ..Default::default()
            },
        ));

        state_context
            .system_bundles
            .push(Graphics::default_system_bundle());

        let mut bundle = SystemBundle::new();
        bundle.add_system(move_camera_system);
        state_context.system_bundles.push(bundle);
    }
}

fn move_camera_system(ecs: &mut Ecs) {
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
    if input_state.is(KeyDown(Key::E)) && input_state.is(KeyUp(Key::D)) {
        transform.scale.0 += 1.0;
        transform.scale.1 += 1.0;
    } else if input_state.is(KeyDown(Key::A)) && input_state.is(KeyUp(Key::Q)) {
        transform.scale.0 -= 1.0;
        transform.scale.1 -= 1.0;
    }
}
