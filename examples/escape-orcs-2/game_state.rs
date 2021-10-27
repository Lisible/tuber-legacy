use crate::character::Character;
use crate::orc::{create_orc, Orc};
use crate::player::{create_player, Player};
use crate::terrain::{create_terrain, TILE_SIZE};
use std::f32::consts::PI;
use tuber::core::input::{Input, InputState};
use tuber::core::transform::Transform2D;
use tuber::ecs::ecs::EntityDefinition;
use tuber::ecs::query::accessors::{R, W};
use tuber::ecs::system::SystemBundle;
use tuber::engine::state::{State, StateContext, StateStackRequest};
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::Graphics;
use tuber_core::DeltaTime;
use tuber_ecs::ecs::Ecs;

pub(crate) struct GameState {
    do_exit: bool,
}
impl GameState {
    pub(crate) fn new() -> Self {
        Self { do_exit: false }
    }
}

impl State for GameState {
    fn initialize(&mut self, state_context: &mut StateContext) {
        state_context.ecs.insert(create_camera());
        state_context
            .ecs
            .insert(create_player(state_context.asset_store));
        state_context
            .ecs
            .insert(create_orc(state_context.asset_store));
        state_context.ecs.insert(create_terrain());
        state_context
            .system_bundles
            .push(Graphics::default_system_bundle());

        let mut system_bundle = SystemBundle::new();
        system_bundle.add_system(move_player);
        system_bundle.add_system(update_character_position);
        state_context.system_bundles.push(system_bundle);
    }

    fn update(&mut self, state_context: &mut StateContext) {
        let input_state = state_context.ecs.shared_resource::<InputState>().unwrap();
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
        Transform2D::default(),
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Movement {
    Up,
    Down,
    Left,
    Right,
    Idle,
}

fn move_player(ecs: &mut Ecs) {
    let input_state = ecs.shared_resource::<InputState>().unwrap();
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

    if let Some((_, (_, mut character, transform))) =
        ecs.query_one::<(R<Player>, W<Character>, R<Transform2D>)>()
    {
        if character.movement == Movement::Idle {
            character.movement = player_movement;
            character.animation_time = 0.0;
            character.initial_position.0 = transform.translation.0;
            character.initial_position.1 = transform.translation.1;
        }
    }
}

fn update_character_position(ecs: &mut Ecs) {
    const ANIMATION_SPEED: f32 = 2f32;
    let delta_time = ecs.shared_resource::<DeltaTime>().unwrap().0 as f32;

    if let Some((_, (mut character, mut transform))) =
        ecs.query_one::<(W<Character>, W<Transform2D>)>()
    {
        character.animation_time += delta_time * ANIMATION_SPEED;

        if character.movement == Movement::Right {
            transform.translation.0 = character.initial_position.0
                + ease_in_out(character.animation_time) * TILE_SIZE as f32;

            if transform.translation.0 as i32
                == character.initial_position.0 as i32 + TILE_SIZE as i32
            {
                character.movement = Movement::Idle;
            }
        } else if character.movement == Movement::Left {
            transform.translation.0 = character.initial_position.0
                - ease_in_out(character.animation_time) * TILE_SIZE as f32;

            if transform.translation.0 as i32
                == character.initial_position.0 as i32 - TILE_SIZE as i32
            {
                character.movement = Movement::Idle;
            }
        } else if character.movement == Movement::Up {
            transform.translation.1 = character.initial_position.1
                - ease_in_out(character.animation_time) * TILE_SIZE as f32;

            if transform.translation.1 as i32
                == character.initial_position.1 as i32 - TILE_SIZE as i32
            {
                character.movement = Movement::Idle;
            }
        } else if character.movement == Movement::Down {
            transform.translation.1 = character.initial_position.1
                + ease_in_out(character.animation_time) * TILE_SIZE as f32;

            if transform.translation.1 as i32
                == character.initial_position.1 as i32 + TILE_SIZE as i32
            {
                character.movement = Movement::Idle;
            }
        }
    }
}

fn ease_in_out(x: f32) -> f32 {
    return -((PI * x).cos() - 1f32) / 2f32;
}
