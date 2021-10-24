use std::collections::HashSet;
use std::hash::Hash;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Tag {
    identifier: String,
}

impl Tag {
    pub fn new(identifier: &str) -> Self {
        Tag {
            identifier: identifier.into(),
        }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }
}

pub trait IntoTag: Clone {
    fn into_tag(self) -> Tag;
}

pub struct Tilemap {
    pub width: usize,
    pub height: usize,
    pub tile_width: usize,
    pub tile_height: usize,
    pub tiles: Vec<Tile>,
}

impl Tilemap {
    pub fn new(
        width: usize,
        height: usize,
        tile_width: usize,
        tile_height: usize,
        default_tags: &[impl IntoTag],
    ) -> Self {
        Self {
            width,
            height,
            tile_width,
            tile_height,
            tiles: vec![Tile::with_tags(default_tags); width * height],
        }
    }
}

#[derive(Clone)]
pub struct Tile {
    pub tags: HashSet<Tag>,
}

impl Tile {
    pub fn with_tags(tags: &[impl IntoTag]) -> Self {
        Self {
            tags: tags
                .iter()
                .cloned()
                .map(|s| s.into_tag().to_owned())
                .collect(),
        }
    }
}
