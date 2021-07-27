use tuber::core::transform::Transform2D;
use tuber::engine::state::{State, StateContext};
use tuber::engine::{Engine, Result, TuberRunner};
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::shape::RectangleShape;
use tuber::graphics::Graphics;
use tuber::graphics_wgpu::GraphicsWGPU;
use tuber::physics::{Collidable, CollisionShape, Physics, RigidBody2D, StaticBody2D};
use tuber::WinitTuberRunner;

struct MouseControlled;

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

        state_context.ecs.insert((
            RectangleShape {
                width: 100.0,
                height: 100.0,
                color: (1.0, 0.0, 0.0),
            },
            Transform2D {
                translation: (200.0, 200.0, 0),
                ..Default::default()
            },
            StaticBody2D,
            Collidable {
                shapes: vec![CollisionShape::from_rectangle(0.0, 0.0, 100.0, 100.0)],
                ..Default::default()
            },
        ));

        state_context.ecs.insert((
            MouseControlled,
            RectangleShape {
                width: 100.0,
                height: 100.0,
                color: (1.0, 0.0, 0.0),
            },
            Transform2D {
                translation: (200.0, 0.0, 0),
                ..Default::default()
            },
            RigidBody2D::default(),
            Collidable {
                shapes: vec![CollisionShape::from_rectangle(0.0, 0.0, 100.0, 100.0)],
                ..Default::default()
            },
        ));

        let physics = Physics::new((0.0, 1.0));
        state_context.ecs.insert_shared_resource(physics);

        state_context
            .system_bundles
            .push(Physics::default_system_bundle());
        state_context
            .system_bundles
            .push(Graphics::default_system_bundle());
    }
}
