use std::collections::VecDeque;

use rand::{thread_rng, Rng};

use tuber::core::input::Input::ActionDown;
use tuber::core::transform::Transform;
use tuber::ecs::ecs::Ecs;
use tuber::ecs::query::accessors::{R, W};
use tuber::ecs::system::{SystemBundle, SystemResult};
use tuber::ecs::EntityIndex;
use tuber::engine::state::State;
use tuber::engine::{Engine, EngineSettings, Result, TuberRunner};
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::WinitTuberRunner;
use tuber_engine::engine_context::EngineContext;
use tuber_engine::system_bundle;
use tuber_graphics::material::MaterialDescriptor;
use tuber_graphics::renderable::sprite::Sprite;
use tuber_graphics::texture::TextureRegion;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const BODY_PART_SIZE: f32 = 64.0;
const SNAKE_SPEED: f32 = 4.0;

struct SnakeHead;

struct SnakeTail;

struct SnakeBodyPart {
    next_body_part: Option<EntityIndex>,
}

#[derive(Copy, Clone)]
struct Pivot {
    position: (f32, f32, f32),
    angle: f32,
}

struct PivotList(VecDeque<Pivot>);

struct Apple;

#[derive(Debug, Copy, Clone)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Debug)]
struct Score(u32);

fn main() -> Result<()> {
    let engine = Engine::new(EngineSettings {
        application_title: Some("Snake".into()),
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

        ecs.insert_shared_resource(PivotList(VecDeque::new()));
        ecs.insert_shared_resource(Score(0));

        spawn_snake(ecs);
        spawn_apple(ecs);

        let mut bundle = SystemBundle::new();
        bundle.add_system(move_head_system);
        bundle.add_system(move_body_parts_system);
        bundle.add_system(eat_apple_system);
        bundle.add_system(check_collision_with_body_system);
        system_bundles.push(bundle);
        system_bundles.push(system_bundle::graphics::default_system_bundle());
    }
}

fn check_collision_with_body_system(ecs: &mut Ecs, _: &mut EngineContext) -> SystemResult {
    let mut is_game_over = false;
    {
        let (head_id, (_, head_body_part, head_transform)) = ecs
            .query_one::<(R<SnakeHead>, R<SnakeBodyPart>, R<Transform>)>()
            .unwrap();
        let next_id = head_body_part.next_body_part.unwrap();
        for (body_part_id, (_, body_part_transform)) in
            ecs.query::<(R<SnakeBodyPart>, R<Transform>)>()
        {
            if head_id == body_part_id || next_id == body_part_id {
                continue;
            }

            if rectangle_intersects(
                (
                    head_transform.translation.x(),
                    head_transform.translation.y(),
                    BODY_PART_SIZE,
                    BODY_PART_SIZE,
                ),
                (
                    body_part_transform.translation.x(),
                    body_part_transform.translation.y(),
                    BODY_PART_SIZE,
                    BODY_PART_SIZE,
                ),
            ) {
                is_game_over = true;
            }
        }
    }

    if is_game_over {
        game_over(ecs);
    }

    Ok(())
}

fn move_head_system(ecs: &mut Ecs, engine_context: &mut EngineContext) -> SystemResult {
    let is_game_over = {
        let input_state = &engine_context.input_state;
        let (_, (_, mut velocity, mut transform)) = ecs
            .query_one::<(R<SnakeHead>, W<Velocity>, W<Transform>)>()
            .unwrap();

        let mut pivot_list = ecs.shared_resource_mut::<PivotList>().unwrap();
        if input_state.is(ActionDown("rotate_head_left".into())) {
            let angle_z = transform.angle.z();
            transform.angle.set_z(angle_z - 2f32.to_radians());
            pivot_list.0.push_back(Pivot {
                position: transform.translation.into(),
                angle: transform.angle.z(),
            });
        } else if input_state.is(ActionDown("rotate_head_right".into())) {
            let angle_z = transform.angle.z();
            transform.angle.set_z(angle_z + 2f32.to_radians());
            pivot_list.0.push_back(Pivot {
                position: transform.translation.into(),
                angle: transform.angle.z(),
            });
        }

        *velocity = compute_new_segment_velocity(transform.angle.z());
        *transform = compute_new_segment_position(*transform, &velocity);

        transform.translation.x() < -BODY_PART_SIZE
            || transform.translation.x() > WINDOW_WIDTH as f32
            || transform.translation.y() < -BODY_PART_SIZE
            || transform.translation.y() > WINDOW_HEIGHT as f32
    };

    if is_game_over {
        game_over(ecs);
    }

    Ok(())
}

fn game_over(ecs: &mut Ecs) {
    println!("Game Over");
    reset_score(ecs);
    respawn_snake(ecs);
}

fn reset_score(ecs: &mut Ecs) {
    let mut score = ecs.shared_resource_mut::<Score>().unwrap();
    score.0 = 0;
    println!("Score: {}", score.0);
}

fn respawn_snake(ecs: &mut Ecs) {
    ecs.delete_by_query::<(R<SnakeBodyPart>,)>();
    spawn_snake(ecs);
}

fn spawn_apple(ecs: &mut Ecs) {
    let mut rng = thread_rng();
    let _apple = ecs.insert((
        Transform {
            translation: (
                rng.gen_range(0.0..800.0 - 64.0),
                rng.gen_range(0.0..600.0 - 64.0),
                0.0,
            )
                .into(),
            ..Default::default()
        },
        Sprite {
            width: 64.0,
            height: 64.0,
            material: MaterialDescriptor {
                albedo_map: "apple_texture".into(),
                ..Default::default()
            },
            texture_region: TextureRegion {
                x: 0.0,
                y: 0.0,
                width: 32.0,
                height: 32.0,
            },
            ..Default::default()
        },
        Apple,
    ));
}

fn spawn_snake(ecs: &mut Ecs) {
    let snake_tail = ecs.insert((
        Transform {
            translation: (300.0, 300.0 + BODY_PART_SIZE, 0.0).into(),
            rotation_center: (32.0, BODY_PART_SIZE, 0.0).into(),
            ..Default::default()
        },
        Sprite {
            width: BODY_PART_SIZE,
            height: BODY_PART_SIZE,
            material: MaterialDescriptor {
                albedo_map: "snake_tail_texture".into(),
                ..Default::default()
            },
            texture_region: TextureRegion {
                x: 0.0,
                y: 0.0,
                width: 32.0,
                height: 32.0,
            },
            ..Default::default()
        },
        Velocity {
            x: 0.0,
            y: -SNAKE_SPEED,
        },
        SnakeBodyPart {
            next_body_part: None,
        },
        SnakeTail,
    ));
    let _snake_head = ecs.insert((
        Transform {
            translation: (300.0, 300.0, 0.0).into(),
            rotation_center: (BODY_PART_SIZE / 2.0, BODY_PART_SIZE, 0.0).into(),
            ..Default::default()
        },
        Sprite {
            width: BODY_PART_SIZE,
            height: BODY_PART_SIZE,
            material: MaterialDescriptor {
                albedo_map: "snake_face_texture".into(),
                ..Default::default()
            },
            texture_region: TextureRegion {
                x: 0.0,
                y: 0.0,
                width: 32.0,
                height: 32.0,
            },
            ..Default::default()
        },
        Velocity {
            x: 0.0,
            y: -SNAKE_SPEED,
        },
        SnakeHead,
        SnakeBodyPart {
            next_body_part: Some(snake_tail),
        },
    ));
}

fn move_body_parts_system(ecs: &mut Ecs, _: &mut EngineContext) -> SystemResult {
    let (head_id, _) = ecs.query_one::<(R<SnakeHead>,)>().unwrap();
    let (tail_id, _) = ecs.query_one::<(R<SnakeTail>,)>().unwrap();
    let mut pivots = ecs.shared_resource_mut::<PivotList>().unwrap();
    let mut pivots_to_delete = vec![];
    for (body_part_id, (mut transform, mut velocity)) in ecs.query::<(W<Transform>, W<Velocity>)>()
    {
        if body_part_id == head_id {
            continue;
        }

        for (pivot_index, pivot) in pivots.0.iter().enumerate() {
            if (transform.translation.x() - pivot.position.0).abs() < 0.2
                && (transform.translation.y() - pivot.position.1).abs() < 0.2
            {
                if body_part_id == tail_id {
                    pivots_to_delete.push(pivot_index);
                }
                transform.angle.set_z(pivot.angle);
            }
        }
        *velocity = compute_new_segment_velocity(transform.angle.z());
        *transform = compute_new_segment_position(*transform, &velocity);
    }

    for id_pivot_to_delete in pivots_to_delete {
        pivots.0.remove(id_pivot_to_delete);
    }

    Ok(())
}

fn eat_apple_system(ecs: &mut Ecs, _: &mut EngineContext) -> SystemResult {
    let mut grow_snake = false;
    {
        let (_, (_, head_transform, head_sprite)) = ecs
            .query_one::<(R<SnakeHead>, R<Transform>, R<Sprite>)>()
            .unwrap();
        let mut score = ecs.shared_resource_mut::<Score>().unwrap();
        let head_rectangle = (
            head_transform.translation.x(),
            head_transform.translation.z(),
            head_sprite.width,
            head_sprite.height,
        );

        let mut rng = thread_rng();
        for (_, (_, mut apple_transform, apple_sprite)) in
            ecs.query::<(R<Apple>, W<Transform>, R<Sprite>)>()
        {
            let apple_rectangle = (
                apple_transform.translation.x(),
                apple_transform.translation.z(),
                apple_sprite.width,
                apple_sprite.height,
            );

            if rectangle_intersects(head_rectangle, apple_rectangle) {
                apple_transform
                    .translation
                    .set_x(rng.gen_range(0.0..800.0 - 64.0));
                apple_transform
                    .translation
                    .set_y(rng.gen_range(0.0..600.0 - 64.0));
                score.0 += 1;
                grow_snake = true;
                println!("Score: {}", score.0)
            }
        }
    }

    if grow_snake {
        let (old_tail_id, tail_transform, tail_velocity) = {
            let (tail_id, (_, tail_transform, tail_velocity)) = ecs
                .query_one::<(R<SnakeTail>, R<Transform>, R<Velocity>)>()
                .unwrap();
            (tail_id, *tail_transform, *tail_velocity)
        };

        let new_tail_id = {
            ecs.insert((
                Transform {
                    translation: (
                        tail_transform.translation.x() - BODY_PART_SIZE / 4.0 * tail_velocity.x,
                        tail_transform.translation.y() - BODY_PART_SIZE / 4.0 * tail_velocity.y,
                        0.0,
                    )
                        .into(),
                    ..tail_transform
                },
                Sprite {
                    width: 64.0,
                    height: 64.0,
                    material: MaterialDescriptor {
                        albedo_map: "snake_tail_texture".into(),
                        ..Default::default()
                    },
                    texture_region: TextureRegion {
                        x: 0.0,
                        y: 0.0,
                        width: 32.0,
                        height: 32.0,
                    },
                    ..Default::default()
                },
                tail_velocity,
                SnakeBodyPart {
                    next_body_part: None,
                },
                SnakeTail,
            ))
        };

        {
            {
                let (_, (mut old_tail_body_part, mut sprite)) = ecs
                    .query_one_by_id::<(W<SnakeBodyPart>, W<Sprite>)>(old_tail_id)
                    .unwrap();
                old_tail_body_part.next_body_part = Some(new_tail_id);
                sprite.material = MaterialDescriptor {
                    albedo_map: "snake_body_texture".into(),
                    ..sprite.material.clone()
                };
            }
            ecs.remove_component::<SnakeTail>(old_tail_id);
        }
    }

    Ok(())
}

fn compute_new_segment_velocity(angle: f32) -> Velocity {
    Velocity {
        x: SNAKE_SPEED * angle.sin(),
        y: -SNAKE_SPEED * angle.cos(),
    }
}

fn compute_new_segment_position(transform: Transform, velocity: &Velocity) -> Transform {
    Transform {
        translation: (
            transform.translation.x() + velocity.x,
            transform.translation.y() + velocity.y,
            0.0,
        )
            .into(),
        ..transform
    }
}

fn rectangle_intersects(
    first_rectangle: (f32, f32, f32, f32),
    second_rectangle: (f32, f32, f32, f32),
) -> bool {
    return first_rectangle.0 < second_rectangle.0 + second_rectangle.2
        && first_rectangle.0 + first_rectangle.2 > second_rectangle.0
        && first_rectangle.1 < second_rectangle.1 + second_rectangle.3
        && first_rectangle.1 + first_rectangle.3 > second_rectangle.1;
}
