use crate::{Material, Size2, TextureRegion};

pub struct Tilemap {
    size: Size2<usize>,
    tile_size: Size2<u32>,
    tiles: Vec<Option<Tile>>,
    material: Material,
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
            tiles: vec![default_tile.clone(); size.width() * size.height()],
            material,
        }
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
        assert!(x < self.size.width());
        assert!(y < self.size.height());
        self.tiles[x + y * self.size.width()] = tile;
    }

    pub fn tile(&self, x: usize, y: usize) -> &Option<Tile> {
        assert!(x < self.size.width());
        assert!(y < self.size.height());
        &self.tiles[x + y * self.size.width()]
    }

    pub fn material(&self) -> &Material {
        &self.material
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tile {
    texture_region: TextureRegion,
}

impl Tile {
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
            Material::new("albedo_map", None),
            None,
        );

        tilemap.set_tile(
            4,
            2,
            Some(Tile {
                texture_region: TextureRegion {
                    x: 0.0,
                    y: 0.0,
                    width: 32.0,
                    height: 32.0,
                },
            }),
        );

        assert_eq!(
            tilemap.tile(4, 2).clone().unwrap(),
            Tile {
                texture_region: TextureRegion {
                    x: 0.0,
                    y: 0.0,
                    width: 32.0,
                    height: 32.0,
                },
            }
        );
    }
}
