use crate::game_state::Movement;
use crate::item::Item;
use std::time::Instant;
use tuber::core::asset::AssetStore;
use tuber::core::transform::Transform2D;
use tuber::ecs::ecs::EntityDefinition;
use tuber::graphics::sprite::{AnimatedSprite, AnimationState};
use tuber::graphics::texture::TextureAtlas;

pub(crate) struct Player {
    pub last_movement: Option<Movement>,
    pub item: Option<Item>,
    pub score: u32,
}

pub(crate) fn create_player(asset_store: &mut AssetStore) -> impl EntityDefinition {
    let atlas = asset_store.asset::<TextureAtlas>("atlas").unwrap();

    (
        Player {
            last_movement: None,
            item: None,
            score: 0,
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
    )
}
