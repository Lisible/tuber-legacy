use crate::character::Character;
use crate::orc::{create_orc, Orc};
use crate::player::{create_player, Player};
use crate::terrain::TILE_SIZE;
use rand::prelude::ThreadRng;
use rand::Rng;
use std::f32::consts::PI;
use tuber::core::input::keyboard::Key;
use tuber::core::input::Input;
use tuber::core::transform::Transform2D;
use tuber::core::DeltaTime;
use tuber::ecs::ecs::Ecs;
use tuber::ecs::ecs::EntityDefinition;
use tuber::ecs::query::accessors::{R, W};
use tuber::ecs::system::SystemBundle;
use tuber::engine::engine_context::EngineContext;
use tuber::engine::state::{State, StateStackRequest};
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::g_buffer::GBufferComponent;
use tuber::graphics::tilemap::{Tile, Tilemap};
use tuber_graphics::material::Material;
use tuber_graphics::texture::TextureRegion;
use tuber_graphics::tilemap::TileDescription;

pub(crate) struct GameState {
    tilemap: Tilemap,
    do_exit: bool,
}
impl GameState {
    pub(crate) fn new() -> Self {
        let mut tilemap = Tilemap::new(
            10,
            10,
            Material {
                albedo_map: "spritesheet".to_string(),
                normal_map: None,
            },
            None,
        );

        let tile = Tile::Tile(TileDescription {
            texture_region: TextureRegion {
                x: 0.0,
                y: 0.0,
                width: 32.0,
                height: 32.0,
            },
        });
        tilemap.set_tile(0, 0, Some(tile));

        Self {
            tilemap,
            do_exit: false,
        }
    }
}

struct RandomNumberGenerator(ThreadRng);

impl State for GameState {
    fn initialize(
        &mut self,
        ecs: &mut Ecs,
        system_bundles: &mut Vec<SystemBundle<EngineContext>>,
        engine_context: &mut EngineContext,
    ) {
        ecs.insert_shared_resource(RandomNumberGenerator(rand::thread_rng()));

        ecs.insert(create_camera());
        ecs.insert(create_player(&mut engine_context.asset_store));
        ecs.insert(create_orc(&mut engine_context.asset_store));
        system_bundles.push(tuber::engine::system_bundle::graphics::default_system_bundle());

        let mut system_bundle = SystemBundle::<EngineContext>::new();
        system_bundle.add_system(move_player);
        system_bundle.add_system(update_character_position);
        system_bundle.add_system(update_camera_position);
        system_bundle.add_system(switch_rendered_g_buffer_component);
        system_bundles.push(system_bundle);
    }

    fn update(&mut self, _ecs: &mut Ecs, engine_context: &mut EngineContext) {
        let input_state = &engine_context.input_state;
        if input_state.is(Input::ActionDown("exit_game".into())) {
            self.do_exit = true;
        }
    }

    fn stack_requests(&mut self) -> Vec<StateStackRequest> {
        if self.do_exit {
            return vec![StateStackRequest::Pop];
        }

        vec![]
    }
}

fn switch_rendered_g_buffer_component(_ecs: &mut Ecs, engine_context: &mut EngineContext) {
    let input_state = &engine_context.input_state;
    if input_state.is(Input::KeyDown(Key::F1)) {
        println!("Switch to albedo GBuffer component");
        engine_context
            .graphics
            .as_mut()
            .unwrap()
            .set_rendered_g_buffer_component(GBufferComponent::Albedo);
    } else if input_state.is(Input::KeyDown(Key::F2)) {
        println!("Switch to normal GBuffer component");
        engine_context
            .graphics
            .as_mut()
            .unwrap()
            .set_rendered_g_buffer_component(GBufferComponent::Normal);
    }
}

fn create_camera() -> impl EntityDefinition {
    (
        OrthographicCamera {
            left: 0.0,
            right: 800.0,
            top: 0.0,
            bottom: 600.0,
            near: -100.0,
            far: 100.0,
        },
        Active,
        Transform2D {
            translation: (-368.0, -268.0, 0),
            ..Default::default()
        },
    )
}

pub(crate) fn update_camera_position(ecs: &mut Ecs, _: &mut EngineContext) {
    let (_, (_, player_transform)) = ecs.query_one::<(R<Player>, R<Transform2D>)>().unwrap();
    let (_, (_, mut camera_transform)) = ecs
        .query_one::<(R<OrthographicCamera>, W<Transform2D>)>()
        .unwrap();

    camera_transform.translation.0 = player_transform.translation.0 - 368f32;
    camera_transform.translation.1 = player_transform.translation.1 - 268f32;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Movement {
    Up,
    Down,
    Left,
    Right,
    Idle,
}

fn move_player(ecs: &mut Ecs, engine_context: &mut EngineContext) {
    let player_movement = {
        let input_state = &engine_context.input_state;
        let player_movement = if input_state.is(Input::ActionDown("move_up".into())) {
            Movement::Up
        } else if input_state.is(Input::ActionDown("move_down".into())) {
            Movement::Down
        } else if input_state.is(Input::ActionDown("move_left".into())) {
            Movement::Left
        } else if input_state.is(Input::ActionDown("move_right".into())) {
            Movement::Right
        } else {
            return;
        };
        player_movement
    };

    move_orcs(ecs);

    if let Some((_, (_, mut character, transform))) =
        ecs.query_one::<(R<Player>, W<Character>, R<Transform2D>)>()
    {
        if character.movement == Movement::Idle {
            character.movement = player_movement;
            character.animation_time = 0.0;
            character.initial_position.0 = transform.translation.0 as i32 / TILE_SIZE as i32;
            character.initial_position.1 = transform.translation.1 as i32 / TILE_SIZE as i32;
        }
    }
}

fn move_orcs(ecs: &mut Ecs) {
    let rng = &mut ecs
        .shared_resource_mut::<RandomNumberGenerator>()
        .unwrap()
        .0;

    const MOVEMENTS: [Movement; 4] = [
        Movement::Up,
        Movement::Down,
        Movement::Left,
        Movement::Right,
    ];

    for (_, (_, mut character, transform)) in ecs.query::<(R<Orc>, W<Character>, R<Transform2D>)>()
    {
        if character.movement == Movement::Idle {
            character.movement = MOVEMENTS[rng.gen_range(0..4)];
            character.animation_time = 0.0;
            character.initial_position.0 = transform.translation.0 as i32 / TILE_SIZE as i32;
            character.initial_position.1 = transform.translation.1 as i32 / TILE_SIZE as i32;
        }
    }
}

fn update_character_position(ecs: &mut Ecs, _: &mut EngineContext) {
    const ANIMATION_SPEED: f32 = 2f32;
    let delta_time = ecs.shared_resource::<DeltaTime>().unwrap().0 as f32;

    for (_, (mut character, mut transform)) in ecs.query::<(W<Character>, W<Transform2D>)>() {
        let target_position =
            compute_target_position(character.initial_position, character.movement);
        character.animation_time += delta_time * ANIMATION_SPEED;
        let delta_translation = ease_in_out(character.animation_time) * TILE_SIZE as f32;

        transform.translation = match character.movement {
            Movement::Up => (
                transform.translation.0,
                character.initial_position.1 as f32 * TILE_SIZE as f32 - delta_translation,
                transform.translation.2,
            ),
            Movement::Down => (
                transform.translation.0,
                character.initial_position.1 as f32 * TILE_SIZE as f32 + delta_translation,
                transform.translation.2,
            ),
            Movement::Left => (
                character.initial_position.0 as f32 * TILE_SIZE as f32 - delta_translation,
                transform.translation.1,
                transform.translation.2,
            ),
            Movement::Right => (
                character.initial_position.0 as f32 * TILE_SIZE as f32 + delta_translation,
                transform.translation.1,
                transform.translation.2,
            ),
            _ => transform.translation,
        };

        if character.animation_time >= 1f32 && character.movement != Movement::Idle {
            transform.translation.0 = target_position.0 as f32 * TILE_SIZE as f32;
            transform.translation.1 = target_position.1 as f32 * TILE_SIZE as f32;
            character.movement = Movement::Idle;
        }
    }
}

fn compute_target_position(initial_position: (i32, i32), movement: Movement) -> (i32, i32) {
    match movement {
        Movement::Up => (initial_position.0, initial_position.1 - 1),
        Movement::Down => (initial_position.0, initial_position.1 + 1),
        Movement::Left => (initial_position.0 - 1, initial_position.1),
        Movement::Right => (initial_position.0 + 1, initial_position.1),
        _ => initial_position,
    }
}

fn ease_in_out(x: f32) -> f32 {
    return -((PI * x).cos() - 1f32) / 2f32;
}
