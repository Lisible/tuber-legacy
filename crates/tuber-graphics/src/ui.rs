use crate::texture::TextureRegion;
use crate::Color;

pub struct Image {
    pub width: f32,
    pub height: f32,
    pub texture_region: TextureRegion,
    pub texture_identifier: String,
}

pub struct Frame {
    pub width: f32,
    pub height: f32,
    pub color: Color,
}

pub struct Text {
    text: String,
    font: String,
}

impl Text {
    pub fn new(text: &str, font: &str) -> Self {
        Self {
            text: text.into(),
            font: font.into(),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn font(&self) -> &str {
        &self.font
    }
    pub fn set_font(&mut self, font: &str) {
        self.font = font.to_string();
    }
}

pub struct NoViewTransform;
