use tuber::core::input::keyboard::Key;
use tuber::core::input::Input;
use tuber::core::transform::Transform;
use tuber::ecs::ecs::Ecs;
use tuber::ecs::query::accessors::{R, W};
use tuber::ecs::system::{SystemBundle, SystemResult};
use tuber::engine::engine_context::EngineContext;
use tuber::engine::state::State;
use tuber::engine::system_bundle;
use tuber::engine::{Engine, EngineSettings, Result, TuberRunner};
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::renderable::shape::RectangleShape;
use tuber::graphics::types::Color;
use tuber::WinitTuberRunner;

const BALL_COUNT: usize = 10;
const PADDLE_WIDTH: f32 = 20.0;
const PADDLE_HEIGHT: f32 = 100.0;
const BALL_SIZE: f32 = 10.0;
const LEFT_PADDLE_INITIAL_POSITION: (f32, f32, f32) = (50.0, 250.0, 0.0);
const RIGHT_PADDLE_INITIAL_POSITION: (f32, f32, f32) = (730.0, 250.0, 0.0);
const BALL_INITIAL_POSITION: (f32, f32, f32) = (395.0, 295.0, 0.0);

struct Ball;

struct Paddle;

struct Player;

struct Velocity {
    x: f32,
    y: f32,
}

fn main() -> Result<()> {
    let engine = Engine::new(EngineSettings {
        application_title: Some("Tennis".into()),
        initial_state: Some(Box::new(MainState)),
    });

    WinitTuberRunner.run(engine)
}

struct MainState;

impl State for MainState {
    fn initialize(
        &mut self,
        ecs: &mut Ecs,
        system_bundles: &mut Vec<SystemBundle<EngineContext>>,
        _engine_context: &mut EngineContext,
    ) {
        ecs.insert((
            OrthographicCamera {
                left: 0.0,
                right: 800.0,
                top: 0.0,
                bottom: 600.0,
                near: -100.0,
                far: 100.0,
            },
            Transform {
                translation: (0.0, 0.0, 0.0).into(),
                ..Default::default()
            },
            Active,
        ));

        let _left_paddle = ecs.insert((
            RectangleShape {
                width: PADDLE_WIDTH,
                height: PADDLE_HEIGHT,
                color: Color::WHITE,
            },
            Transform {
                translation: LEFT_PADDLE_INITIAL_POSITION.into(),
                ..Default::default()
            },
            Paddle,
            Player,
        ));

        let _right_paddle = ecs.insert((
            RectangleShape {
                width: PADDLE_WIDTH,
                height: PADDLE_HEIGHT,
                color: Color::WHITE,
            },
            Transform {
                translation: RIGHT_PADDLE_INITIAL_POSITION.into(),
                ..Default::default()
            },
            Paddle,
        ));

        use rand::Rng;
        let mut rng = rand::thread_rng();
        for _ in 0..BALL_COUNT {
            let _ball = ecs.insert((
                RectangleShape {
                    width: BALL_SIZE,
                    height: BALL_SIZE,
                    color: (
                        rng.gen_range(0.0..=1.0),
                        rng.gen_range(0.0..=1.0),
                        rng.gen_range(0.0..=1.0),
                    )
                        .into(),
                },
                Velocity {
                    x: rng.gen_range(-10.0..=-5.0),
                    y: rng.gen_range(-10.0..=5.0),
                },
                Transform {
                    translation: BALL_INITIAL_POSITION.into(),
                    rotation_center: (BALL_SIZE / 2.0, BALL_SIZE / 2.0, 0.0).into(),
                    ..Default::default()
                },
                Ball,
            ));
        }

        let mut bundle = SystemBundle::new();
        bundle.add_system(move_ball_system);
        bundle.add_system(move_paddle_system);
        bundle.add_system(collision_system);
        system_bundles.push(system_bundle::graphics::default_system_bundle());
        system_bundles.push(bundle);
    }
}

fn move_paddle_system(ecs: &mut Ecs, engine_context: &mut EngineContext) -> SystemResult {
    let input_state = &engine_context.input_state;
    for (_id, (mut transform, _)) in ecs.query::<(W<Transform>, R<Player>)>() {
        if input_state.is(Input::KeyDown(Key::Z)) {
            transform.translation.y -= 5.0;
        } else if input_state.is(Input::KeyDown(Key::S)) {
            transform.translation.y += 5.0;
        }
    }

    Ok(())
}

fn move_ball_system(ecs: &mut Ecs, _: &mut EngineContext) -> SystemResult {
    for (_id, (rectangle_shape, mut transform, mut velocity)) in
        ecs.query::<(R<RectangleShape>, W<Transform>, W<Velocity>)>()
    {
        if (transform.translation.x + rectangle_shape.width >= 800.0)
            || (transform.translation.x <= 0.0)
        {
            velocity.x = -velocity.x;
        }

        if (transform.translation.y + rectangle_shape.height >= 600.0)
            || (transform.translation.y <= 0.0)
        {
            velocity.y = -velocity.y;
        }

        transform.translation.x += velocity.x;
        transform.translation.y += velocity.y;
        transform.angle.z += 1.0;
    }

    Ok(())
}

fn collision_system(ecs: &mut Ecs, _: &mut EngineContext) -> SystemResult {
    {
        for (_paddle_id, (paddle_transform, paddle_shape, _)) in
            ecs.query::<(R<Transform>, R<RectangleShape>, R<Paddle>)>()
        {
            let paddle_position = paddle_transform.translation;
            for (_ball_id, (mut ball_transform, mut velocity, _)) in
                ecs.query::<(W<Transform>, W<Velocity>, R<Ball>)>()
            {
                let ball_position = ball_transform.translation;

                if !ball_is_close_to_paddle(
                    ball_position.into(),
                    BALL_SIZE,
                    paddle_transform.translation.into(),
                    PADDLE_WIDTH,
                    PADDLE_HEIGHT,
                ) {
                    continue;
                }

                if ball_position.x < paddle_position.x + paddle_shape.width
                    && ball_position.x + BALL_SIZE > paddle_position.x
                    && ball_position.y > paddle_position.y
                    && ball_position.y + BALL_SIZE < paddle_position.y + paddle_shape.height
                {
                    ball_transform.translation.x += if velocity.x >= 0.0 {
                        -(ball_position.x + BALL_SIZE - paddle_position.x)
                    } else {
                        paddle_position.x + paddle_shape.width - ball_position.x
                    };
                    velocity.x = -velocity.x;
                }

                if ball_position.y < paddle_position.y + paddle_shape.height
                    && ball_position.y + BALL_SIZE > paddle_position.y
                    && ball_position.x > paddle_position.x
                    && ball_position.x + BALL_SIZE < paddle_position.x + paddle_shape.width
                {
                    ball_transform.translation.y += if velocity.y >= 0.0 {
                        -(ball_position.y + BALL_SIZE - paddle_position.y)
                    } else {
                        paddle_position.y + paddle_shape.height - ball_position.y
                    };
                    velocity.y = -velocity.y;
                }
            }
        }
    }

    Ok(())
}

fn ball_is_close_to_paddle(
    ball_position: (f32, f32, f32),
    ball_size: f32,
    paddle_position: (f32, f32, f32),
    paddle_width: f32,
    paddle_height: f32,
) -> bool {
    ball_position.0 + ball_size > paddle_position.0 - ball_size
        && ball_position.0 < paddle_position.0 + paddle_width + ball_size
        && ball_position.1 + ball_size > paddle_position.1 - ball_size
        && ball_position.1 < paddle_position.1 + paddle_height + ball_size
}
