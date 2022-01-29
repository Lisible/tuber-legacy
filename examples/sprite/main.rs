use std::time::Instant;
use tuber::core::transform::Transform2D;
use tuber::engine::state::State;
use tuber::engine::{Engine, EngineSettings, Result, TuberRunner};
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::renderable::sprite::{AnimatedSprite, Sprite};
use tuber::graphics::texture::TextureRegion;
use tuber::WinitTuberRunner;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::system::SystemBundle;
use tuber_engine::engine_context::EngineContext;
use tuber_engine::system_bundle;
use tuber_graphics::animation::AnimationState;
use tuber_graphics::material::Material;
use tuber_graphics::texture::TextureAtlas;

fn main() -> Result<()> {
    env_logger::init();
    let mut engine = Engine::new(EngineSettings {
        ..Default::default()
    });

    engine.push_initial_state(Box::new(MainState));

    WinitTuberRunner.run(engine)
}

struct MainState;
impl State for MainState {
    fn initialize(
        &mut self,
        ecs: &mut Ecs,
        system_bundles: &mut Vec<SystemBundle<EngineContext>>,
        engine_context: &mut EngineContext,
    ) {
        ecs.insert((
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

        ecs.insert((
            Transform2D {
                translation: (375.0, 275.0, 0),
                ..Default::default()
            },
            Sprite {
                width: 50.0,
                height: 50.0,
                texture_region: TextureRegion::new(0.0, 0.0, 32.0, 32.0),
                material: Material {
                    albedo_map: "sprite".to_string(),
                    normal_map: None,
                    emission_map: None,
                },
                ..Default::default()
            },
        ));

        ecs.insert((
            Transform2D {
                translation: (500.0, 275.0, 0),
                ..Default::default()
            },
            Sprite {
                width: 50.0,
                height: 50.0,
                texture_region: TextureRegion::new(0.0, 0.0, 32.0, 32.0),
                material: Material {
                    albedo_map: "sprite".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ));

        ecs.insert((
            Transform2D {
                translation: (250.0, 275.0, 0),
                ..Default::default()
            },
            Sprite {
                width: 50.0,
                height: 50.0,
                texture_region: TextureRegion::new(0.0, 0.0, 16.0, 16.0),
                material: Material {
                    albedo_map: "dfhgfhfh".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ));

        ecs.insert((
            Transform2D {
                translation: (250.0, 350.0, 0),
                ..Default::default()
            },
            Sprite {
                width: 100.0,
                height: 100.0,
                texture_region: TextureRegion::new(0.0, 0.0, 16.0, 16.0),
                material: Material {
                    albedo_map: "dfhgfhfh".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ));

        let texture_atlas = engine_context
            .asset_store
            .asset::<TextureAtlas>("atlas")
            .unwrap();
        ecs.insert((
            Transform2D {
                translation: (375.0, 350.0, 0),
                ..Default::default()
            },
            Sprite {
                width: 100.0,
                height: 100.0,
                texture_region: texture_atlas.texture_region("tree").unwrap(),
                material: Material {
                    albedo_map: "atlas_texture".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ));

        ecs.insert((
            Transform2D {
                translation: (475.0, 400.0, 0),
                ..Default::default()
            },
            Sprite {
                width: 50.0,
                height: 50.0,
                texture_region: texture_atlas.texture_region("house").unwrap(),
                material: Material {
                    albedo_map: "atlas_texture".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ));

        ecs.insert((
            Transform2D {
                translation: (0.0, 0.0, 0),
                ..Default::default()
            },
            AnimatedSprite {
                width: 100.0,
                height: 100.0,
                material: Material {
                    albedo_map: "animated_sprite".into(),
                    ..Default::default()
                },
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

        system_bundles.push(system_bundle::graphics::default_system_bundle());
    }
}
