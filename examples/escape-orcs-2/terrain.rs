use std::collections::HashSet;
use tuber::core::tilemap::{IntoTag, Tag, Tile, Tilemap};
use tuber::ecs::ecs::EntityDefinition;
use tuber::graphics::tilemap::TilemapRender;
use tuber::proc_macros::Tag;
use tuber_core::transform::Transform2D;

#[derive(Tag, Clone)]
enum TileTags {
    Stone,
    CliffNorth,
    CliffEast,
    CliffSouth,
    CliffWest,
    CornerNorthWest,
    CornerNorthEast,
    CornerSouthEast,
    CornerSouthWest,
    Walkable,
}

pub(crate) fn create_terrain() -> impl EntityDefinition {
    const MAP_WIDTH: usize = 12;
    const MAP_HEIGHT: usize = 10;
    const TILE_SIZE: usize = 64;

    let mut tilemap = Tilemap::new(
        MAP_WIDTH,
        MAP_HEIGHT,
        TILE_SIZE,
        TILE_SIZE,
        &[TileTags::Stone, TileTags::Walkable],
    );

    tilemap.tiles[0].tags.remove(&TileTags::Stone.into_tag());
    tilemap.tiles[0]
        .tags
        .insert(TileTags::CornerNorthWest.into_tag());
    tilemap.tiles[MAP_WIDTH - 1]
        .tags
        .remove(&TileTags::Stone.into_tag());
    tilemap.tiles[MAP_WIDTH - 1]
        .tags
        .insert(TileTags::CornerNorthEast.into_tag());
    tilemap.tiles[MAP_WIDTH - 1 + (MAP_HEIGHT - 1) * MAP_WIDTH]
        .tags
        .remove(&TileTags::Stone.into_tag());
    tilemap.tiles[MAP_WIDTH - 1 + (MAP_HEIGHT - 1) * MAP_WIDTH]
        .tags
        .insert(TileTags::CornerSouthEast.into_tag());
    tilemap.tiles[(MAP_HEIGHT - 1) * MAP_WIDTH]
        .tags
        .remove(&TileTags::Stone.into_tag());
    tilemap.tiles[(MAP_HEIGHT - 1) * MAP_WIDTH]
        .tags
        .insert(TileTags::CornerSouthWest.into_tag());

    for i in 1..MAP_WIDTH - 1 {
        tilemap.tiles[i].tags.remove(&TileTags::Stone.into_tag());
        tilemap.tiles[i]
            .tags
            .insert(TileTags::CliffNorth.into_tag());
        tilemap.tiles[i + (MAP_HEIGHT - 1) * MAP_WIDTH]
            .tags
            .remove(&TileTags::Stone.into_tag());
        tilemap.tiles[i + (MAP_HEIGHT - 1) * MAP_WIDTH]
            .tags
            .insert(TileTags::CliffSouth.into_tag());
    }

    for i in 1..MAP_HEIGHT - 1 {
        tilemap.tiles[i * MAP_WIDTH]
            .tags
            .remove(&TileTags::Stone.into_tag());
        tilemap.tiles[i * MAP_WIDTH]
            .tags
            .insert(TileTags::CliffWest.into_tag());
        tilemap.tiles[MAP_WIDTH - 1 + i * MAP_WIDTH]
            .tags
            .remove(&TileTags::Stone.into_tag());
        tilemap.tiles[MAP_WIDTH - 1 + i * MAP_WIDTH]
            .tags
            .insert(TileTags::CliffEast.into_tag());
    }

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

fn tile_to_texture(tile: &Tile) -> Option<&str> {
    if tile.tags.contains(&TileTags::Stone.into_tag()) {
        return Some("stone");
    } else if tile.tags.contains(&TileTags::CornerNorthWest.into_tag()) {
        return Some("corner_north_west_1");
    } else if tile.tags.contains(&TileTags::CornerNorthEast.into_tag()) {
        return Some("corner_north_east_1");
    } else if tile.tags.contains(&TileTags::CornerSouthEast.into_tag()) {
        return Some("corner_south_east_1");
    } else if tile.tags.contains(&TileTags::CornerSouthWest.into_tag()) {
        return Some("corner_south_west_1");
    } else if tile.tags.contains(&TileTags::CliffNorth.into_tag()) {
        return Some("cliff_north_1");
    } else if tile.tags.contains(&TileTags::CliffSouth.into_tag()) {
        return Some("cliff_south_1");
    } else if tile.tags.contains(&TileTags::CliffEast.into_tag()) {
        return Some("cliff_east_1");
    } else if tile.tags.contains(&TileTags::CliffWest.into_tag()) {
        return Some("cliff_west_1");
    }

    None
}
