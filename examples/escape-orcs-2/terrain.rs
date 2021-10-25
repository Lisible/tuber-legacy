use tuber::core::tilemap::{IntoTag, Tag, Tile, Tilemap};
use tuber::ecs::ecs::EntityDefinition;
use tuber::graphics::tilemap::TilemapRender;
use tuber::proc_macros::Tag;
use tuber_core::transform::Transform2D;

#[derive(Tag, Clone)]
enum TileTags {
    Grass,
    Sand,
}

pub(crate) fn create_terrain() -> impl EntityDefinition {
    const MAP_WIDTH: usize = 100;
    const MAP_HEIGHT: usize = 100;
    const TILE_SIZE: usize = 64;

    let tilemap = Tilemap::new(
        MAP_WIDTH,
        MAP_HEIGHT,
        TILE_SIZE,
        TILE_SIZE,
        &[TileTags::Grass],
    );

    (
        tilemap,
        TilemapRender {
            identifier: "tilemap_render".to_string(),
            texture_atlas_identifier: "atlas".to_string(),
            texture_identifier: "spritesheet".to_string(),
            tile_texture_function: Box::new(tile_to_texture),
            dirty: true,
        },
        Transform2D::default(),
    )
}

fn tile_to_texture(_tile: &Tile) -> Option<&str> {
    Some("grass")
}
