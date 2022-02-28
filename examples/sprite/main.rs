use std::time::Instant;

use tuber::core::transform::Transform;
use tuber::ecs::ecs::Ecs;
use tuber::ecs::system::SystemBundle;
use tuber::engine::engine_context::EngineContext;
use tuber::engine::state::State;
use tuber::engine::system_bundle;
use tuber::engine::{Engine, EngineSettings, Result, TuberRunner};
use tuber::graphics::animation::AnimationState;
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::material::MaterialDescriptor;
use tuber::graphics::renderable::sprite::{AnimatedSprite, Sprite};
use tuber::graphics::texture::TextureAtlas;
use tuber::graphics::texture::TextureRegion;
use tuber::WinitTuberRunner;

fn main() -> Result<()> {
    env_logger::init();
    let engine = Engine::new(EngineSettings {
        application_title: Some("Sprite".into()),
        initial_state: Some(Box::new(MainState)),
    });

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
            Transform {
                translation: (0.0, 0.0, 0.0).into(),
                ..Default::default()
            },
            Active,
        ));

        ecs.insert((
            Transform {
                translation: (375.0, 275.0, 0.0).into(),
                ..Default::default()
            },
            Sprite {
                width: 50.0,
                height: 50.0,
                texture_region: TextureRegion::new(0.0, 0.0, 32.0, 32.0),
                material: MaterialDescriptor {
                    albedo_map: "sprite".to_string(),
                    normal_map: None,
                    emission_map: None,
                },
                ..Default::default()
            },
        ));

        ecs.insert((
            Transform {
                translation: (500.0, 275.0, 0.0).into(),
                ..Default::default()
            },
            Sprite {
                width: 50.0,
                height: 50.0,
                texture_region: TextureRegion::new(0.0, 0.0, 32.0, 32.0),
                material: MaterialDescriptor {
                    albedo_map: "sprite".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ));

        ecs.insert((
            Transform {
                translation: (250.0, 275.0, 0.0).into(),
                ..Default::default()
            },
            Sprite {
                width: 50.0,
                height: 50.0,
                texture_region: TextureRegion::new(0.0, 0.0, 16.0, 16.0),
                material: MaterialDescriptor {
                    albedo_map: "dfhgfhfh".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ));

        ecs.insert((
            Transform {
                translation: (250.0, 350.0, 0.0).into(),
                ..Default::default()
            },
            Sprite {
                width: 100.0,
                height: 100.0,
                texture_region: TextureRegion::new(0.0, 0.0, 16.0, 16.0),
                material: MaterialDescriptor {
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
            Transform {
                translation: (375.0, 350.0, 0.0).into(),
                ..Default::default()
            },
            Sprite {
                width: 100.0,
                height: 100.0,
                texture_region: texture_atlas.texture_region("tree").unwrap(),
                material: MaterialDescriptor {
                    albedo_map: "atlas_texture".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ));

        ecs.insert((
            Transform {
                translation: (475.0, 400.0, 0.0).into(),
                ..Default::default()
            },
            Sprite {
                width: 50.0,
                height: 50.0,
                texture_region: texture_atlas.texture_region("house").unwrap(),
                material: MaterialDescriptor {
                    albedo_map: "atlas_texture".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ));

        ecs.insert((
            Transform {
                translation: (0.0, 0.0, 0.0).into(),
                ..Default::default()
            },
            AnimatedSprite {
                width: 100.0,
                height: 100.0,
                material: MaterialDescriptor {
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
