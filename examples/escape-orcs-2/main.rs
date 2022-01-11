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
    let mut engine = Engine::new(EngineSettings {
        application_title: Some("Escape Orcs 2".into()),
    });

    engine.push_initial_state(Box::new(GameState::new()));

    WinitTuberRunner.run(engine)
}
