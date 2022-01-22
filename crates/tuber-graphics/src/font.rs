use crate::{BitmapFont, TextureData};
use std::str::FromStr;

pub(crate) const DEFAULT_FONT_IDENTIFIER: &'static str = "_default_font";

pub(crate) fn create_default_bitmap_font_texture() -> TextureData {
    let bytes = include_bytes!("../textures/default_font.png");
    let image = image::load_from_memory(bytes).unwrap();
    let image = image.as_rgba8().unwrap();

    TextureData {
        identifier: DEFAULT_FONT_IDENTIFIER.to_string(),
        size: (128, 32),
        bytes: image.to_vec(),
        srgb: true,
    }
}

pub(crate) fn default_bitmap_font() -> BitmapFont {
    BitmapFont::from_str(include_str!("../fonts/default_font.json")).unwrap()
}
