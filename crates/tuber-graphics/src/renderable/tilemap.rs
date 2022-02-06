use crate::animation::AnimationState;
use crate::{MaterialDescriptor, Size2, TextureRegion};

pub struct Tilemap {
    size: Size2<usize>,
    tile_size: Size2<u32>,
    layers: Vec<Layer>,
    material: MaterialDescriptor,
}

impl Tilemap {
    pub fn new(size: Size2<usize>, tile_size: Size2<u32>, material: MaterialDescriptor) -> Self {
        Self {
            size,
            tile_size,
            layers: vec![],
            material,
        }
    }

    pub fn update_animation_state(&mut self) {
        self.layers_mut()
            .iter_mut()
            .map(|layer| layer.tiles_mut())
            .flat_map(|t| t.iter_mut())
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

    pub fn add_layer(&mut self, default_tile: Option<Tile>) -> usize {
        self.layers.push(Layer::new(default_tile, self.size));
        self.layers.len() - 1
    }

    pub fn layer_mut(&mut self, layer_index: usize) -> Option<&mut Layer> {
        self.layers.get_mut(layer_index)
    }

    pub fn layer(&self, layer_index: usize) -> Option<&Layer> {
        self.layers.get(layer_index)
    }

    pub fn layers(&self) -> &Vec<Layer> {
        &self.layers
    }

    pub fn layers_mut(&mut self) -> &mut Vec<Layer> {
        &mut self.layers
    }

    pub fn material(&self) -> &MaterialDescriptor {
        &self.material
    }
}

pub struct Layer {
    size: Size2<usize>,
    tiles: Vec<Option<Tile>>,
}

impl Layer {
    pub(crate) fn new(default_tile: Option<Tile>, size: Size2<usize>) -> Self {
        Self {
            size,
            tiles: vec![default_tile; size.width * size.height],
        }
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

    pub fn tiles(&self) -> &Vec<Option<Tile>> {
        &self.tiles
    }
    pub fn tiles_mut(&mut self) -> &mut Vec<Option<Tile>> {
        &mut self.tiles
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
    fn add_layer() {
        let mut tilemap = Tilemap::new(
            (10, 10).into(),
            (32, 32).into(),
            MaterialDescriptor {
                albedo_map: "albedo_map".into(),
                ..Default::default()
            },
        );

        tilemap.add_layer(None);
        tilemap.add_layer(None);

        assert_eq!(2, tilemap.layers.len());
    }

    #[test]
    fn set_tile() {
        let mut tilemap = Tilemap::new(
            (10, 10).into(),
            (32, 32).into(),
            MaterialDescriptor {
                albedo_map: "albedo_map".into(),
                ..Default::default()
            },
        );

        let layer = tilemap.add_layer(None);

        tilemap.layer_mut(layer).unwrap().set_tile(
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
            tilemap.layer(layer).unwrap().tile(4, 2).clone().unwrap(),
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
