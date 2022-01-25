use crate::character::Character;
use crate::game_state::Movement;
use std::time::Instant;
use tuber::core::asset::AssetStore;
use tuber::core::transform::Transform2D;
use tuber::ecs::ecs::EntityDefinition;
use tuber::graphics::renderable::sprite::{AnimatedSprite, Sprite};
use tuber::graphics::texture::TextureAtlas;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::{EntityIndex, Parent};
use tuber_graphics::animation::AnimationState;
use tuber_graphics::material::Material;
use tuber_graphics::renderable::light::PointLight;

pub(crate) struct Player {
    pub score: u32,
}

pub(crate) fn create_player(ecs: &mut Ecs, asset_store: &mut AssetStore) {
    let player_entity = ecs.insert(create_player_entity_definition(asset_store));
    let _ = ecs.insert(create_player_shadow_entity_definition(
        asset_store,
        player_entity,
    ));

    let _ = ecs.insert((
        PointLight {
            ambient: (0.6, 0.6, 0.6).into(),
            diffuse: (1.0, 1.0, 1.0).into(),
            specular: (1.0, 1.0, 1.0).into(),
            constant: 1.0,
            linear: 0.007,
            quadratic: 0.0002,
        },
        Parent(player_entity),
        Transform2D {
            translation: (0.0, 0.0, -10),
            ..Default::default()
        },
    ));
}

fn create_player_entity_definition(asset_store: &mut AssetStore) -> impl EntityDefinition {
    let atlas = asset_store.asset::<TextureAtlas>("atlas").unwrap();

    (
        Player { score: 0 },
        Character {
            initial_position: (0, 0),
            animation_time: 0.0,
            movement: Movement::Idle,
        },
        Transform2D::default(),
        AnimatedSprite {
            width: 64.0,
            height: 64.0,
            material: Material {
                albedo_map: "spritesheet".to_string(),
                normal_map: Some("normal_spritesheet".to_string()),
            },
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

fn create_player_shadow_entity_definition(
    asset_store: &mut AssetStore,
    player_entity_index: EntityIndex,
) -> impl EntityDefinition {
    let atlas = asset_store.asset::<TextureAtlas>("atlas").unwrap();

    (
        Parent(player_entity_index),
        Sprite {
            width: 32.0,
            height: 14.0,
            texture_region: atlas.texture_region("shadow").unwrap(),
            material: Material {
                albedo_map: "spritesheet".to_string(),
                normal_map: Some("normal_spritesheet".to_string()),
            },
        },
        Transform2D {
            translation: (14.0, 52.0, -1),
            ..Default::default()
        },
    )
}
