use crate::orc::create_orc;
use crate::player;
use crate::player::create_player;
use tuber::core::transform::Transform2D;
use tuber::ecs::ecs::EntityDefinition;
use tuber::engine::state::{State, StateContext};
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::Graphics;

pub(crate) struct GameState;
impl State for GameState {
    fn initialize(&mut self, state_context: &mut StateContext) {
        state_context.ecs.insert(create_camera());
        state_context
            .ecs
            .insert(create_player(state_context.asset_store));
        state_context
            .ecs
            .insert(create_orc(state_context.asset_store));
        state_context
            .system_bundles
            .push(Graphics::default_system_bundle());
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
