use crate::character::Character;
use crate::game_state::Movement;
use std::time::Instant;
use tuber::core::asset::AssetStore;
use tuber::core::transform::Transform;
use tuber::ecs::ecs::EntityDefinition;
use tuber::graphics::renderable::sprite::{AnimatedSprite, Sprite};
use tuber::graphics::texture::TextureAtlas;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::{EntityIndex, Parent};
use tuber_graphics::animation::AnimationState;
use tuber_graphics::material::MaterialDescriptor;

pub(crate) struct Orc;

pub(crate) fn create_orc(ecs: &mut Ecs, asset_store: &mut AssetStore) {
    let orc_entity = ecs.insert(create_orc_entity_definition(asset_store));
    let _ = ecs.insert(create_orc_shadow_entity_definition(asset_store, orc_entity));
}

fn create_orc_entity_definition(asset_store: &mut AssetStore) -> impl EntityDefinition {
    let atlas = asset_store.asset::<TextureAtlas>("atlas").unwrap();
    (
        Orc,
        Character {
            initial_position: (4, 4),
            animation_time: 0.0,
            movement: Movement::Idle,
        },
        Transform {
            translation: (144.0, 144.0, 10.0),
            ..Default::default()
        },
        AnimatedSprite {
            width: 64.0,
            height: 64.0,
            material: MaterialDescriptor {
                albedo_map: "spritesheet".to_string(),
                normal_map: Some("normal_spritesheet".to_string()),
                emission_map: Some("emissive_spritesheet".to_string()),
            },
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

fn create_orc_shadow_entity_definition(
    asset_store: &mut AssetStore,
    orc_entity: EntityIndex,
) -> impl EntityDefinition {
    let atlas = asset_store.asset::<TextureAtlas>("atlas").unwrap();
    (
        Parent(orc_entity),
        Sprite {
            width: 48.0,
            height: 14.0,
            texture_region: atlas.texture_region("shadow").unwrap(),
            material: MaterialDescriptor {
                albedo_map: "spritesheet".to_string(),
                normal_map: Some("normal_spritesheet".to_string()),
                emission_map: Some("emissive_spritesheet".to_string()),
            },
        },
        Transform {
            translation: (6.0, 55.0, -1.0),
            ..Default::default()
        },
    )
}
