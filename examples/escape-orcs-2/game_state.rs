use crate::orc::{create_orc, Orc};
use crate::player::{create_player, Player};
use crate::terrain::create_terrain;
use tuber::core::input::{Input, InputState};
use tuber::core::transform::Transform2D;
use tuber::ecs::ecs::EntityDefinition;
use tuber::ecs::query::accessors::W;
use tuber::ecs::system::SystemBundle;
use tuber::engine::state::{State, StateContext, StateStackRequest};
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::Graphics;
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

#[derive(Clone)]
pub(crate) enum Movement {
    Up,
    Down,
    Left,
    Right,
}

fn move_player(ecs: &mut Ecs) {
    const VELOCITY: f32 = 64.0;

    let input_state = ecs.shared_resource::<InputState>().unwrap();
    let player_direction = if input_state.is(Input::ActionDown("move_up".into())) {
        Movement::Up
    } else if input_state.is(Input::ActionDown("move_down".into())) {
        Movement::Down
    } else if input_state.is(Input::ActionDown("move_left".into())) {
        Movement::Left
    } else if input_state.is(Input::ActionDown("move_right".into())) {
        Movement::Right
    } else {
        if let Some((_, (mut player,))) = ecs.query_one::<(W<Player>,)>() {
            player.last_movement = None;
        }

        for (_, (mut orc,)) in ecs.query::<(W<Orc>,)>() {
            orc.last_movement = None;
        }
        return;
    };

    if let Some((_, (mut player, mut transform))) = ecs.query_one::<(W<Player>, W<Transform2D>)>() {
        let last = player.last_movement.replace(player_direction.clone());
        if last.is_none() {
            match player_direction {
                Movement::Up => transform.translation.1 -= VELOCITY,
                Movement::Down => transform.translation.1 += VELOCITY,
                Movement::Left => transform.translation.0 -= VELOCITY,
                Movement::Right => transform.translation.0 += VELOCITY,
            }
        }
    }

    for (_, (mut orc, mut transform)) in ecs.query::<(W<Orc>, W<Transform2D>)>() {
        let last = orc.last_movement.replace(Movement::Right);
        if last.is_none() {
            transform.translation.0 += VELOCITY
        }
    }
}
