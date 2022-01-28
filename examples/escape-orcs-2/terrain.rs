use tuber_core::asset::AssetStore;
use tuber_graphics::animation::AnimationState;
use tuber_graphics::material::Material;
use tuber_graphics::renderable::tilemap::{AnimatedTile, StaticTile, Tile, Tilemap};
use tuber_graphics::texture::TextureAtlas;
use tuber_graphics::types::Size2;

pub const WORLD_SIZE: Size2<usize> = Size2 {
    width: 10,
    height: 10,
};

pub const TILE_SIZE: u32 = 48;

pub fn create_tilemap(asset_store: &mut AssetStore) -> Tilemap {
    let atlas = asset_store.asset::<TextureAtlas>("atlas").unwrap();

    let mut tilemap = Tilemap::new(
        Size2::new(10, 10),
        Size2::new(TILE_SIZE, TILE_SIZE),
        Material {
            albedo_map: "spritesheet".to_string(),
            normal_map: Some("normal_spritesheet".to_string()),
            emission_map: Some("emissive_spritesheet".to_string()),
        },
        Some(Tile::StaticTile(StaticTile {
            texture_region: atlas.texture_region("stone").unwrap(),
        })),
    );

    for y in 0..WORLD_SIZE.height {
        tilemap.set_tile(
            0,
            y,
            Some(Tile::AnimatedTile(AnimatedTile {
                animation_state: AnimationState {
                    keyframes: vec![
                        atlas.texture_region("cliff_west_1").unwrap(),
                        atlas.texture_region("cliff_west_2").unwrap(),
                    ],
                    ..Default::default()
                },
            })),
        );
        tilemap.set_tile(
            WORLD_SIZE.width - 1,
            y,
            Some(Tile::AnimatedTile(AnimatedTile {
                animation_state: AnimationState {
                    keyframes: vec![
                        atlas.texture_region("cliff_east_1").unwrap(),
                        atlas.texture_region("cliff_east_2").unwrap(),
                    ],
                    ..Default::default()
                },
            })),
        );
    }

    for x in 0..WORLD_SIZE.width {
        tilemap.set_tile(
            x,
            0,
            Some(Tile::AnimatedTile(AnimatedTile {
                animation_state: AnimationState {
                    keyframes: vec![
                        atlas.texture_region("cliff_north_1").unwrap(),
                        atlas.texture_region("cliff_north_2").unwrap(),
                    ],
                    ..Default::default()
                },
            })),
        );
        tilemap.set_tile(
            x,
            WORLD_SIZE.height - 1,
            Some(Tile::AnimatedTile(AnimatedTile {
                animation_state: AnimationState {
                    keyframes: vec![
                        atlas.texture_region("cliff_south_1").unwrap(),
                        atlas.texture_region("cliff_south_2").unwrap(),
                    ],
                    ..Default::default()
                },
            })),
        );
    }

    tilemap.set_tile(
        0,
        0,
        Some(Tile::AnimatedTile(AnimatedTile {
            animation_state: AnimationState {
                keyframes: vec![
                    atlas.texture_region("corner_north_west_1").unwrap(),
                    atlas.texture_region("corner_north_west_2").unwrap(),
                ],
                ..Default::default()
            },
        })),
    );
    tilemap.set_tile(
        WORLD_SIZE.width - 1,
        0,
        Some(Tile::AnimatedTile(AnimatedTile {
            animation_state: AnimationState {
                keyframes: vec![
                    atlas.texture_region("corner_north_east_1").unwrap(),
                    atlas.texture_region("corner_north_east_2").unwrap(),
                ],
                ..Default::default()
            },
        })),
    );
    tilemap.set_tile(
        0,
        WORLD_SIZE.height - 1,
        Some(Tile::AnimatedTile(AnimatedTile {
            animation_state: AnimationState {
                keyframes: vec![
                    atlas.texture_region("corner_south_west_1").unwrap(),
                    atlas.texture_region("corner_south_west_2").unwrap(),
                ],
                ..Default::default()
            },
        })),
    );
    tilemap.set_tile(
        WORLD_SIZE.width - 1,
        WORLD_SIZE.height - 1,
        Some(Tile::AnimatedTile(AnimatedTile {
            animation_state: AnimationState {
                keyframes: vec![
                    atlas.texture_region("corner_south_east_1").unwrap(),
                    atlas.texture_region("corner_south_east_2").unwrap(),
                ],
                ..Default::default()
            },
        })),
    );

    tilemap
}
