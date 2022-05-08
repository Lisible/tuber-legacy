#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl Color {
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };

    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    pub fn from_rgb_hex(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 255f32,
            g: g as f32 / 255f32,
            b: b as f32 / 255f32,
        }
    }

    pub fn r(&self) -> f32 {
        self.r
    }
    pub fn g(&self) -> f32 {
        self.g
    }
    pub fn b(&self) -> f32 {
        self.b
    }

    pub fn to_rgb_array(&self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}
