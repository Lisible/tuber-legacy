use crate::game_state::GameState;
use tuber::engine::{Engine, EngineSettings};
use tuber::engine::{Result, TuberRunner};
use tuber::WinitTuberRunner;

mod character;
mod game_state;
mod orc;
mod player;
mod terrain;

fn main() -> Result<()> {
    let engine = Engine::new(EngineSettings {
        application_title: Some("Escape Orcs 2".into()),
        initial_state: Some(Box::new(GameState::new())),
    });

    WinitTuberRunner.run(engine)
}
