use tuber_core::asset::AssetStore;
use tuber_core::transform::Transform;
use tuber_ecs::ecs::Ecs;
use tuber_graphics::animation::AnimationState;
use tuber_graphics::material::MaterialDescriptor;
use tuber_graphics::renderable::light::PointLight;
use tuber_graphics::renderable::tilemap::{AnimatedTile, StaticTile, Tile, Tilemap};
use tuber_graphics::texture::TextureAtlas;
use tuber_graphics::types::Size2;

pub const WORLD_SIZE: Size2<usize> = Size2 {
    width: 10,
    height: 10,
};

pub const TILE_SIZE: u32 = 48;

pub fn create_lights(ecs: &mut Ecs) {
    ecs.insert((
        PointLight {
            ambient: (0.4, 0.0, 0.0).into(),
            diffuse: (0.8, 0.3, 0.0).into(),
            specular: (1.0, 0.5, 0.0).into(),
            radius: 1000.0,
        },
        Transform {
            translation: (
                (WORLD_SIZE.width as f32 / 2.0) * TILE_SIZE as f32,
                0.0,
                100.0,
            )
                .into(),
            ..Default::default()
        },
    ));
    ecs.insert((
        PointLight {
            ambient: (0.4, 0.0, 0.0).into(),
            diffuse: (0.8, 0.3, 0.0).into(),
            specular: (1.0, 0.5, 0.0).into(),
            radius: 1000.0,
        },
        Transform {
            translation: (
                (WORLD_SIZE.width as f32 / 2.0) * TILE_SIZE as f32,
                WORLD_SIZE.height as f32 * TILE_SIZE as f32,
                100.0,
            )
                .into(),
            ..Default::default()
        },
    ));
    ecs.insert((
        PointLight {
            ambient: (0.4, 0.0, 0.0).into(),
            diffuse: (0.0, 1.0, 0.0).into(),
            specular: (0.0, 1.0, 0.0).into(),
            radius: 1000.0,
        },
        Transform {
            translation: (
                (WORLD_SIZE.width as f32 / 2.0) * TILE_SIZE as f32,
                (WORLD_SIZE.height as f32 / 2.0) * TILE_SIZE as f32,
                100.0,
            )
                .into(),
            ..Default::default()
        },
    ));
}

pub fn create_tilemap(asset_store: &mut AssetStore) -> Tilemap {
    let atlas = asset_store.asset::<TextureAtlas>("atlas").unwrap();

    let mut tilemap = Tilemap::new(
        Size2::new(30, 30),
        Size2::new(TILE_SIZE, TILE_SIZE),
        MaterialDescriptor {
            albedo_map: "spritesheet".to_string(),
            normal_map: Some("normal_spritesheet".to_string()),
            emission_map: Some("emissive_spritesheet".to_string()),
        },
    );

    let _background_layer = tilemap.add_layer(Some(Tile::AnimatedTile(AnimatedTile {
        animation_state: AnimationState {
            keyframes: vec![
                atlas.texture_region("lava_1").unwrap(),
                atlas.texture_region("lava_2").unwrap(),
            ],
            ..Default::default()
        },
    })));

    let terrain_layer = tilemap.add_layer(None);
    for y in 0..5 {
        for x in 0..5 {
            tilemap.layer_mut(terrain_layer).unwrap().set_tile(
                10 + x,
                5 + y,
                Some(Tile::StaticTile(StaticTile {
                    texture_region: atlas.texture_region("stone").unwrap(),
                })),
            );
        }
    }

    tilemap
}
