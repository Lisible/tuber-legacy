use tuber::engine::Engine;
use tuber::engine::EngineSettings;
use tuber::engine::TuberRunner;
use tuber::winit::WinitTuberRunner;

fn main() {
    let engine = Engine::new(EngineSettings {
        ..Default::default()
    });

    WinitTuberRunner.run(engine).unwrap();
}
