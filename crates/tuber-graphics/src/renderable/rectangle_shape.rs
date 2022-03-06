/// A rectangular shape
pub struct RectangleShape {
    width: f32,
    height: f32,
}

impl RectangleShape {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }
}
