use crate::game_state::Movement;

pub(crate) struct Character {
    pub initial_position: (f32, f32),
    pub animation_time: f32,
    pub movement: Movement,
}
