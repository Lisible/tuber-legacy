use std::time::Instant;
use tuber::core::transform::Transform2D;
use tuber::engine::state::{State, StateContext};
use tuber::engine::{Engine, EngineSettings, Result, TuberRunner};
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::sprite::{AnimatedSprite, AnimationState, Sprite};
use tuber::graphics::texture::{TextureRegion, TextureSource};
use tuber::graphics::Graphics;
use tuber::graphics_wgpu::GraphicsWGPU;
use tuber::WinitTuberRunner;

fn main() -> Result<()> {
    let mut engine = Engine::new(EngineSettings {
        graphics: Some(Graphics::new(Box::new(GraphicsWGPU::new()))),
        ..Default::default()
    });

    engine.push_initial_state(Box::new(MainState));

    WinitTuberRunner.run(engine)
}

struct MainState;
impl State for MainState {
    fn initialize(&mut self, state_context: &mut StateContext) {
        state_context.ecs.insert((
            OrthographicCamera {
                left: 0.0,
                right: 800.0,
                top: 0.0,
                bottom: 600.0,
                near: -100.0,
                far: 100.0,
            },
            Transform2D {
                translation: (0.0, 0.0, 0),
                ..Default::default()
            },
            Active,
        ));

        state_context.ecs.insert((
            Transform2D {
                translation: (375.0, 275.0, 0),
                ..Default::default()
            },
            Sprite {
                width: 50.0,
                height: 50.0,
                texture: "examples/sprite/sprite.png".into(),
            },
        ));

        state_context.ecs.insert((
            Transform2D {
                translation: (500.0, 275.0, 0),
                ..Default::default()
            },
            Sprite {
                width: 50.0,
                height: 50.0,
                texture: "examples/sprite/sprite2.png".into(),
            },
        ));

        state_context.ecs.insert((
            Transform2D {
                translation: (250.0, 275.0, 0),
                ..Default::default()
            },
            Sprite {
                width: 50.0,
                height: 50.0,
                texture: "fqgqgqgpng".into(),
            },
        ));

        state_context.ecs.insert((
            Transform2D {
                translation: (250.0, 350.0, 0),
                ..Default::default()
            },
            Sprite {
                width: 100.0,
                height: 100.0,
                texture: TextureSource::TextureRegion(
                    "mkgskgsmlgk".into(),
                    TextureRegion::new(0.0, 0.0, 16.0, 16.0),
                ),
            },
        ));

        state_context.ecs.insert((
            Transform2D {
                translation: (375.0, 350.0, 0),
                ..Default::default()
            },
            Sprite {
                width: 100.0,
                height: 100.0,
                texture: TextureSource::TextureAtlas(
                    "examples/sprite/texture-atlas.json".into(),
                    "tree".into(),
                ),
            },
        ));

        state_context.ecs.insert((
            Transform2D {
                translation: (475.0, 400.0, 0),
                ..Default::default()
            },
            Sprite {
                width: 50.0,
                height: 50.0,
                texture: TextureSource::TextureAtlas(
                    "examples/sprite/texture-atlas.json".into(),
                    "house".into(),
                ),
            },
        ));

        state_context.ecs.insert((
            Transform2D {
                translation: (0.0, 0.0, 0),
                ..Default::default()
            },
            AnimatedSprite {
                width: 100.0,
                height: 100.0,
                texture: TextureSource::WholeTexture("examples/sprite/animated_sprite.png".into()),

                animation_state: AnimationState {
                    keyframes: vec![
                        TextureRegion::new(0.0, 0.0, 16.0, 16.0),
                        TextureRegion::new(16.0, 0.0, 16.0, 16.0),
                        TextureRegion::new(32.0, 0.0, 16.0, 16.0),
                        TextureRegion::new(48.0, 0.0, 16.0, 16.0),
                        TextureRegion::new(64.0, 0.0, 16.0, 16.0),
                        TextureRegion::new(80.0, 0.0, 16.0, 16.0),
                    ],
                    current_keyframe: 0,
                    start_instant: Instant::now(),
                    frame_duration: 100,
                    flip_x: true,
                },
            },
        ));

        state_context
            .system_bundles
            .push(Graphics::default_system_bundle());
    }
}
