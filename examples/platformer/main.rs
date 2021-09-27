use tuber::core::input::keyboard::Key;
use tuber::core::input::{Input, InputState};
use tuber::core::transform::Transform2D;
use tuber::ecs::ecs::Ecs;
use tuber::ecs::query::accessors::{R, W};
use tuber::ecs::system::{SystemBundle, SystemResult};
use tuber::engine::state::{State, StateContext};
use tuber::engine::{Engine, EngineSettings, Result, TuberRunner};
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::shape::RectangleShape;
use tuber::graphics::Graphics;
use tuber::graphics_wgpu::GraphicsWGPU;
use tuber::physics::{Collidable, CollisionShape, Physics, RigidBody2D, StaticBody2D};
use tuber::WinitTuberRunner;

fn main() -> Result<()> {
    let mut engine = Engine::new(EngineSettings {
        graphics: Some(Graphics::new(Box::new(GraphicsWGPU::new()))),
        ..Default::default()
    });

    engine.push_initial_state(Box::new(MainState));

    WinitTuberRunner.run(engine)
}

pub struct MainState;
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
                width: 50.0,
                height: 100.0,
                color: (1.0, 0.0, 0.0),
            },
            Transform2D {
                translation: (100.0, 100.0, 0),
                ..Default::default()
            },
            RigidBody2D::default(),
            Collidable {
                shapes: vec![CollisionShape::from_rectangle(0.0, 0.0, 50.0, 100.0)],
                ..Default::default()
            },
        ));

        state_context.ecs.insert((
            RectangleShape {
                width: 800.0,
                height: 50.0,
                color: (0.0, 1.0, 0.0),
            },
            Transform2D {
                translation: (0.0, 550.0, 0),
                ..Default::default()
            },
            StaticBody2D,
            Collidable {
                shapes: vec![CollisionShape::from_rectangle(0.0, 0.0, 800.0, 50.0)],
                ..Default::default()
            },
        ));
        state_context.ecs.insert((
            RectangleShape {
                width: 100.0,
                height: 50.0,
                color: (0.0, 1.0, 0.0),
            },
            Transform2D {
                translation: (350.0, 499.0, 0),
                ..Default::default()
            },
            StaticBody2D,
            Collidable {
                shapes: vec![CollisionShape::from_rectangle(0.0, 0.0, 100.0, 50.0)],
                ..Default::default()
            },
        ));

        state_context.ecs.insert((
            RectangleShape {
                width: 300.0,
                height: 50.0,
                color: (0.0, 1.0, 0.0),
            },
            Transform2D {
                translation: (200.0, 200.0, 0),
                angle: 15.0,
                ..Default::default()
            },
            StaticBody2D,
            Collidable {
                shapes: vec![CollisionShape::from_rectangle(0.0, 0.0, 300.0, 50.0)],
                ..Default::default()
            },
        ));

        state_context
            .ecs
            .insert_shared_resource(Physics::new((0.0, 1.0)));

        state_context
            .system_bundles
            .push(Physics::default_system_bundle());
        state_context
            .system_bundles
            .push(Graphics::default_system_bundle());
        let mut bundle = SystemBundle::new();
        bundle.add_system(jump_system);
        bundle.add_system(move_system);
        state_context.system_bundles.push(bundle);
    }
}

fn move_system(ecs: &mut Ecs) -> SystemResult {
    let input = ecs.shared_resource::<InputState>().unwrap();
    let (_, (mut rigid_body, transform)) =
        ecs.query_one::<(W<RigidBody2D>, R<Transform2D>)>().unwrap();
    let (_, (mut camera_transform, _)) = ecs
        .query_one::<(W<Transform2D>, R<OrthographicCamera>)>()
        .unwrap();
    camera_transform.translation = (
        transform.translation.0 - 400.0,
        transform.translation.1 - 300.0,
        0,
    );
    if input.is(Input::KeyDown(Key::Q)) {
        rigid_body.acceleration.x = -5.0;

        if rigid_body.velocity.x <= -10.0 {
            rigid_body.velocity.x = -10.0;
        }
    } else if input.is(Input::KeyDown(Key::D)) {
        rigid_body.acceleration.x = 5.0;

        if rigid_body.velocity.x >= 10.0 {
            rigid_body.velocity.x = 10.0;
        }
    } else {
        rigid_body.acceleration.x = 0.0;
        if rigid_body.velocity.x > 0.0 {
            if rigid_body.velocity.y.abs() < 0.5 {
                rigid_body.velocity.x -= 1.0;
            } else {
                rigid_body.velocity.x -= 0.01;
            }
            if rigid_body.velocity.x < 0.0 {
                rigid_body.velocity.x = 0.0;
            }
        } else if rigid_body.velocity.x < 0.0 {
            if rigid_body.velocity.y.abs() < 0.5 {
                rigid_body.velocity.x += 1.0;
            } else {
                rigid_body.velocity.x += 0.01;
            }
            if rigid_body.velocity.x > 0.0 {
                rigid_body.velocity.x = 0.0;
            }
        }
    }

    Ok(())
}

fn jump_system(ecs: &mut Ecs) -> SystemResult {
    let input = ecs.shared_resource::<InputState>().unwrap();
    let (_, (mut rigid_body,)) = ecs.query_one::<(W<RigidBody2D>,)>().unwrap();
    if input.is(Input::KeyDown(Key::Z)) {
        if rigid_body.velocity.y.abs() == 0.0 {
            rigid_body.acceleration.y = -40.0;
        }
    }

    Ok(())
}
