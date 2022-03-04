use tuber::engine::state::State;
use tuber::engine::Engine;
use tuber::engine::EngineSettings;
use tuber::engine::TuberRunner;
use tuber::WinitTuberRunner;

fn main() {
    let engine = Engine::new(EngineSettings {
        initial_state: Some(Box::new(MainState)),
        ..Default::default()
    });

    WinitTuberRunner.run(engine).unwrap();
}

struct MainState;
impl State for MainState {}
