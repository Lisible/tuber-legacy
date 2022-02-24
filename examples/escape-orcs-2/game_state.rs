use std::f32::consts::PI;

use rand::prelude::ThreadRng;
use rand::Rng;

use tuber::core::input::keyboard::Key;
use tuber::core::input::Input;
use tuber::core::transform::Transform;
use tuber::core::DeltaTime;
use tuber::ecs::ecs::Ecs;
use tuber::ecs::ecs::EntityDefinition;
use tuber::ecs::query::accessors::{R, W};
use tuber::ecs::system::SystemBundle;
use tuber::engine::engine_context::EngineContext;
use tuber::engine::state::{State, StateStackRequest};
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::g_buffer::GBufferComponent;
use tuber_core::transform::IntoMatrix4;
use tuber_graphics::camera::world_region;
use tuber_graphics::low_level::polygon_mode::PolygonMode;
use tuber_graphics::renderable::tilemap::Tilemap;
use tuber_gui::widget::text::TextWidget;

use crate::character::Character;
use crate::orc::{create_orc, Orc};
use crate::player::{create_player, Player};
use crate::terrain::{create_lights, create_tilemap, TILE_SIZE};

pub(crate) struct GameState {
    do_exit: bool,
    tilemap: Option<Tilemap>,
}

impl GameState {
    pub(crate) fn new() -> Self {
        Self {
            do_exit: false,
            tilemap: None,
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
        engine_context
            .graphics
            .as_mut()
            .unwrap()
            .set_ambient_light((0.3, 0.3, 0.3).into());
        self.tilemap = Some(create_tilemap(&mut engine_context.asset_store));

        create_lights(ecs);
        ecs.insert_shared_resource(RandomNumberGenerator(rand::thread_rng()));

        ecs.insert(create_camera());
        create_player(ecs, &mut engine_context.asset_store);
        create_orc(ecs, &mut engine_context.asset_store);
        system_bundles.push(tuber::engine::system_bundle::graphics::default_system_bundle());

        let mut system_bundle = SystemBundle::<EngineContext>::new();
        system_bundle.add_system(move_player);
        system_bundle.add_system(update_score_label);
        system_bundle.add_system(update_character_position);
        system_bundle.add_system(update_camera_position);
        system_bundle.add_system(switch_rendered_g_buffer_component);
        system_bundle.add_system(switch_polygon_mode);
        system_bundles.push(system_bundle);

        engine_context
            .gui
            .root()
            .add_widget(Box::new(TextWidget::new("score_text", "Score 0", None)));
    }

    fn update(&mut self, _ecs: &mut Ecs, engine_context: &mut EngineContext) {
        let input_state = &engine_context.input_state;
        if input_state.is(Input::ActionDown("exit_game".into())) {
            self.do_exit = true;
        }

        self.tilemap.as_mut().unwrap().update_animation_state();
    }

    fn render(&mut self, ecs: &mut Ecs, engine_context: &mut EngineContext) {
        let (_, (camera, transform)) = ecs
            .query_one::<(R<OrthographicCamera>, R<Transform>)>()
            .unwrap();

        let world_region = world_region(&camera.projection_matrix(), &transform.into_matrix4());

        engine_context.graphics.as_mut().unwrap().draw_tilemap(
            &mut engine_context.asset_store,
            self.tilemap.as_mut().unwrap(),
            Transform {
                translation: (0.0, 0.0, -4.0).into(),
                ..Default::default()
            }
            .into_matrix4(),
            world_region,
        );
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
        engine_context
            .graphics
            .as_mut()
            .unwrap()
            .set_rendered_g_buffer_component(GBufferComponent::Albedo);
    } else if input_state.is(Input::KeyDown(Key::F2)) {
        engine_context
            .graphics
            .as_mut()
            .unwrap()
            .set_rendered_g_buffer_component(GBufferComponent::Normal);
    }
}

fn switch_polygon_mode(_ecs: &mut Ecs, engine_context: &mut EngineContext) {
    let input_state = &engine_context.input_state;
    if input_state.is(Input::KeyDown(Key::F3)) {
        engine_context
            .graphics
            .as_mut()
            .unwrap()
            .set_polygon_mode(PolygonMode::Fill);
    } else if input_state.is(Input::KeyDown(Key::F4)) {
        engine_context
            .graphics
            .as_mut()
            .unwrap()
            .set_polygon_mode(PolygonMode::Line);
    }
}

fn update_score_label(ecs: &mut Ecs, engine_context: &mut EngineContext) {
    let (_, (player,)) = ecs.query_one::<(R<Player>,)>().unwrap();
    let text_widget = engine_context
        .gui
        .root()
        .find_mut::<TextWidget>("score_text")
        .unwrap();
    text_widget.set_text(format!("Score {}", player.score));
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
        Transform {
            translation: (-368.0, -268.0, 0.0).into(),
            ..Default::default()
        },
    )
}

pub(crate) fn update_camera_position(ecs: &mut Ecs, _: &mut EngineContext) {
    let (_, (_, player_transform)) = ecs.query_one::<(R<Player>, R<Transform>)>().unwrap();
    let (_, (_, mut camera_transform)) = ecs
        .query_one::<(R<OrthographicCamera>, W<Transform>)>()
        .unwrap();

    camera_transform.translation.x = player_transform.translation.x - 368f32;
    camera_transform.translation.y = player_transform.translation.y - 268f32;
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

    if let Some((_, (mut player, mut character, transform))) =
        ecs.query_one::<(W<Player>, W<Character>, R<Transform>)>()
    {
        if character.movement == Movement::Idle {
            player.score += 1;
            character.movement = player_movement;
            character.animation_time = 0.0;
            character.initial_position.0 = transform.translation.x as i32 / TILE_SIZE as i32;
            character.initial_position.1 = transform.translation.y as i32 / TILE_SIZE as i32;
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

    for (_, (_, mut character, transform)) in ecs.query::<(R<Orc>, W<Character>, R<Transform>)>() {
        if character.movement == Movement::Idle {
            character.movement = MOVEMENTS[rng.gen_range(0..4)];
            character.animation_time = 0.0;
            character.initial_position.0 = transform.translation.x as i32 / TILE_SIZE as i32;
            character.initial_position.1 = transform.translation.y as i32 / TILE_SIZE as i32;
        }
    }
}

fn update_character_position(ecs: &mut Ecs, _: &mut EngineContext) {
    const ANIMATION_SPEED: f32 = 2f32;
    let delta_time = ecs.shared_resource::<DeltaTime>().unwrap().0 as f32;

    for (_, (mut character, mut transform)) in ecs.query::<(W<Character>, W<Transform>)>() {
        let target_position =
            compute_target_position(character.initial_position, character.movement);
        character.animation_time += delta_time * ANIMATION_SPEED;
        let delta_translation = ease_in_out(character.animation_time) * TILE_SIZE as f32;

        transform.translation = match character.movement {
            Movement::Up => (
                transform.translation.x,
                character.initial_position.1 as f32 * TILE_SIZE as f32 - delta_translation,
                transform.translation.z,
            )
                .into(),
            Movement::Down => (
                transform.translation.x,
                character.initial_position.1 as f32 * TILE_SIZE as f32 + delta_translation,
                transform.translation.z,
            )
                .into(),
            Movement::Left => (
                character.initial_position.0 as f32 * TILE_SIZE as f32 - delta_translation,
                transform.translation.y,
                transform.translation.z,
            )
                .into(),
            Movement::Right => (
                character.initial_position.0 as f32 * TILE_SIZE as f32 + delta_translation,
                transform.translation.y,
                transform.translation.z,
            )
                .into(),
            _ => transform.translation,
        };

        if character.animation_time >= 1f32 && character.movement != Movement::Idle {
            transform.translation.x = target_position.0 as f32 * TILE_SIZE as f32;
            transform.translation.y = target_position.1 as f32 * TILE_SIZE as f32;
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
