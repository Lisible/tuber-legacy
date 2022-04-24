use crate::texture::TextureRegion;
use crate::GraphicsError;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use tuber_core::asset::AssetMetadata;

#[derive(Debug, Serialize, Deserialize)]
pub struct BitmapFont {
    /// Identifier of the font texture atlas
    font_atlas: Option<String>,
    /// Identifier of the font tiles texture
    font_atlas_texture: Option<String>,
    /// The region of the bitmap font on the texture tiles
    font_atlas_region: Option<TextureRegion>,
    /// The height of a line in pixels
    line_height: u32,
    /// The spacing between lines in pixels
    line_spacing: u32,
    /// The space between letters in pixels
    letter_spacing: u32,
    /// The flag specifying whether case must be taken in account when looking for a glyph
    ignore_case: bool,
    /// The glyphs assets
    glyphs: HashMap<char, BitmapGlyph>,
}

impl BitmapFont {
    pub fn font_atlas(&self) -> Option<&String> {
        self.font_atlas.as_ref()
    }

    pub fn font_atlas_texture(&self) -> Option<&String> {
        self.font_atlas_texture.as_ref()
    }

    pub fn font_atlas_region(&self) -> Option<&TextureRegion> {
        self.font_atlas_region.as_ref()
    }

    pub fn glyph(&self, character: char) -> Option<&BitmapGlyph> {
        self.glyphs.get(&character)
    }

    pub fn line_height(&self) -> u32 {
        self.line_height
    }

    pub fn line_spacing(&self) -> u32 {
        self.line_spacing
    }

    pub fn letter_spacing(&self) -> u32 {
        self.letter_spacing
    }

    pub fn ignore_case(&self) -> bool {
        self.ignore_case
    }

    pub fn from_file(path: &Path) -> Result<Self, GraphicsError> {
        Self::from_str(
            &std::fs::read_to_string(path).map_err(GraphicsError::BitmapFontFileReadError)?,
        )
    }
}

impl FromStr for BitmapFont {
    type Err = GraphicsError;

    fn from_str(json_string: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(json_string).map_err(GraphicsError::SerdeError)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BitmapGlyph {
    region: TextureRegion,
}

impl BitmapGlyph {
    pub fn region(&self) -> &TextureRegion {
        &self.region
    }
}

pub(crate) fn font_loader(asset_metadata: &AssetMetadata) -> Box<dyn Any> {
    let mut font_file_path = asset_metadata.asset_path.clone();
    font_file_path.push(&asset_metadata.metadata["font_data"]);
    Box::new(BitmapFont::from_file(&font_file_path).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_from_json() -> Result<(), GraphicsError> {
        let json = r#"
        {
            "font_atlas": "font_atlas",
            "font_atlas_texture": "font_atlas_texture",
            "font_atlas_region": {
                "x": 0,
                "y": 0,
                "width": 0,
                "height": 0
            },
            "line_height": 32,
            "line_spacing": 4,
            "ignore_case": false,
            "letter_spacing": 2,
            "glyphs": {
                "A": {
                    "region": {
                        "x": 0,
                        "y": 0,
                        "width": 32,
                        "height": 32
                    }
                },
                "D": {
                    "region": {
                        "x": 32,
                        "y": 0,
                        "width": 32,
                        "height": 32
                    }
                }
            }
        }
        "#;

        let bitmap_font = BitmapFont::from_str(json)?;
        assert_eq!(bitmap_font.font_atlas, Some("font_atlas".to_string()));
        assert_eq!(
            bitmap_font.font_atlas_texture,
            Some("font_atlas_texture".to_string())
        );
        assert_eq!(bitmap_font.line_height, 32);
        assert_eq!(bitmap_font.line_spacing, 4);
        assert_eq!(bitmap_font.letter_spacing, 2);
        assert_eq!(bitmap_font.glyphs.len(), 2);
        assert!(bitmap_font.glyphs.contains_key(&'A'));
        assert!(bitmap_font.glyphs.contains_key(&'D'));
        Ok(())
    }
}
