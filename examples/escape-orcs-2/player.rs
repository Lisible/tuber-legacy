use crate::character::Character;
use crate::game_state::Movement;
use std::time::Instant;
use tuber::core::asset::AssetStore;
use tuber::core::transform::Transform2D;
use tuber::ecs::ecs::EntityDefinition;
use tuber::graphics::sprite::{AnimatedSprite, AnimationState, Sprite};
use tuber::graphics::texture::TextureAtlas;
use tuber_graphics::material::{Material, MaterialTexture};

pub(crate) struct Player;

pub(crate) fn create_player(asset_store: &mut AssetStore) -> impl EntityDefinition {
    let atlas = asset_store.asset::<TextureAtlas>("atlas").unwrap();

    (
        Player,
        Character {
            initial_position: (0, 0),
            animation_time: 0.0,
            movement: Movement::Idle,
        },
        Transform2D {
            translation: (0.0, 0.0, 10),
            ..Default::default()
        },
        AnimatedSprite {
            width: 64.0,
            height: 64.0,
            texture_identifier: "spritesheet".to_string(),
            animation_state: AnimationState {
                keyframes: vec![
                    atlas.texture_region("player_1").unwrap(),
                    atlas.texture_region("player_2").unwrap(),
                ],
                current_keyframe: 0,
                start_instant: Instant::now(),
                frame_duration: 500,
                flip_x: false,
            },
        },
        Sprite {
            width: 32.0,
            height: 14.0,
            offset: (14.0, 52.0, -1),
            material: Material {
                albedo_map: MaterialTexture {
                    identifier: "spritesheet".to_string(),
                    region: atlas.texture_region("shadow").unwrap(),
                },
                normal_map: None,
            },
        },
    )
}