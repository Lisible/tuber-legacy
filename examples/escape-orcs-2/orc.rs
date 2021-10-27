use crate::character::Character;
use crate::game_state::Movement;
use std::time::Instant;
use tuber::core::asset::AssetStore;
use tuber::core::transform::Transform2D;
use tuber::ecs::ecs::EntityDefinition;
use tuber::graphics::sprite::{AnimatedSprite, AnimationState};
use tuber::graphics::texture::TextureAtlas;

pub(crate) struct Orc {
    pub last_movement: Option<Movement>,
}
pub(crate) fn create_orc(asset_store: &mut AssetStore) -> impl EntityDefinition {
    let atlas = asset_store.asset::<TextureAtlas>("atlas").unwrap();

    (
        Orc {
            last_movement: None,
        },
        Character {
            initial_position: (64.0, 64.0),
            animation_time: 0.0,
            movement: Movement::Idle,
        },
        Transform2D {
            translation: (128.0, 128.0, 10),
            ..Default::default()
        },
        AnimatedSprite {
            width: 64.0,
            height: 64.0,
            texture_identifier: "spritesheet".to_string(),
            animation_state: AnimationState {
                keyframes: vec![
                    atlas.texture_region("orc_1").unwrap(),
                    atlas.texture_region("orc_2").unwrap(),
                ],
                current_keyframe: 0,
                start_instant: Instant::now(),
                frame_duration: 500,
                flip_x: false,
            },
        },
    )
}
