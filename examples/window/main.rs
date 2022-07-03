use tuber::engine::state::State;
use tuber::engine::Result;
use tuber::engine::TuberRunner;
use tuber::engine::{Engine, EngineSettings};
use tuber::WinitTuberRunner;

fn main() -> Result<()> {
    env_logger::init();
    let engine = Engine::new(EngineSettings {
        application_title: None,
        initial_state: Some(Box::new(MainState)),
    });

    WinitTuberRunner.run(engine)
}

struct MainState;
impl State for MainState {}
