use tuber::engine::state::State;
use tuber::engine::{Engine, EngineSettings};
use tuber::engine::{Result, TuberRunner};
use tuber::graphics::Graphics;
use tuber::graphics_wgpu::GraphicsWGPU;
use tuber::WinitTuberRunner;

fn main() -> Result<()> {
    let mut engine = Engine::new(EngineSettings {
        graphics: Some(Graphics::new(Box::new(GraphicsWGPU::new()))),
        application_title: Some("Escape Orcs 2".into()),
    });

    engine.push_initial_state(Box::new(MainState));

    WinitTuberRunner.run(engine)
}

struct MainState;
impl State for MainState {}
