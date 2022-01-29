use crate::animation::AnimationState;
use crate::graphics::RenderId;
use crate::{Material, Size2, TextureRegion};

pub struct Tilemap {
    size: Size2<usize>,
    tile_size: Size2<u32>,
    tiles: Vec<Option<Tile>>,
    material: Material,
    render_id: Option<RenderId>,
}

impl Tilemap {
    pub fn new(
        size: Size2<usize>,
        tile_size: Size2<u32>,
        material: Material,
        default_tile: Option<Tile>,
    ) -> Self {
        Self {
            size,
            tile_size,
            tiles: vec![default_tile.clone(); size.width * size.height],
            material,
            render_id: None,
        }
    }

    pub fn update_animation_state(&mut self) {
        self.tiles
            .iter_mut()
            .flat_map(|t| t.iter_mut())
            .for_each(|tile| {
                if let Tile::AnimatedTile(tile) = tile {
                    tile.animation_state.update_animation_state();
                }
            });
    }

    pub fn size(&self) -> &Size2<usize> {
        &self.size
    }

    pub fn tile_size(&self) -> &Size2<u32> {
        &self.tile_size
    }

    pub fn tiles(&self) -> &Vec<Option<Tile>> {
        &self.tiles
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: Option<Tile>) {
        assert!(x < self.size.width);
        assert!(y < self.size.height);
        self.tiles[x + y * self.size.width] = tile;
    }

    pub fn tile(&self, x: usize, y: usize) -> &Option<Tile> {
        assert!(x < self.size.width);
        assert!(y < self.size.height);
        &self.tiles[x + y * self.size.width]
    }

    pub fn material(&self) -> &Material {
        &self.material
    }

    pub fn render_id(&self) -> &Option<RenderId> {
        &self.render_id
    }

    pub(crate) fn set_render_id(&mut self, render_id: RenderId) {
        self.render_id = Some(render_id);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tile {
    StaticTile(StaticTile),
    AnimatedTile(AnimatedTile),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimatedTile {
    pub animation_state: AnimationState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticTile {
    pub texture_region: TextureRegion,
}

impl StaticTile {
    pub fn texture_region(&self) -> &TextureRegion {
        &self.texture_region
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_tile() {
        let mut tilemap = Tilemap::new(
            (10, 10).into(),
            (32, 32).into(),
            Material {
                albedo_map: "albedo_map".into(),
                ..Default::default()
            },
            None,
        );

        tilemap.set_tile(
            4,
            2,
            Some(Tile::StaticTile(StaticTile {
                texture_region: TextureRegion {
                    x: 0.0,
                    y: 0.0,
                    width: 32.0,
                    height: 32.0,
                },
            })),
        );

        assert_eq!(
            tilemap.tile(4, 2).clone().unwrap(),
            Tile::StaticTile(StaticTile {
                texture_region: TextureRegion {
                    x: 0.0,
                    y: 0.0,
                    width: 32.0,
                    height: 32.0,
                },
            })
        );
    }
}
