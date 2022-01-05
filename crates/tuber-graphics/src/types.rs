pub type WindowSize = (u32, u32);

pub struct Size2<T = f32> {
    width: T,
    height: T,
}

impl<T: Copy> Size2<T> {
    pub fn new(width: T, height: T) -> Self {
        Size2 { width, height }
    }

    pub fn width(&self) -> T {
        self.width
    }

    pub fn height(&self) -> T {
        self.height
    }
}

#[derive(Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
    };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn r<T: From<ColorComponent>>(&self) -> T {
        ColorComponent(self.r).into()
    }

    pub fn g<T: From<ColorComponent>>(&self) -> T {
        ColorComponent(self.g).into()
    }

    pub fn b<T: From<ColorComponent>>(&self) -> T {
        ColorComponent(self.b).into()
    }
}

impl From<(f32, f32, f32)> for Color {
    fn from((r, g, b): (f32, f32, f32)) -> Self {
        Self {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
        }
    }
}

impl From<Color> for (f32, f32, f32) {
    fn from(color: Color) -> Self {
        (
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
        )
    }
}

pub struct ColorComponent(u8);
impl From<ColorComponent> for f32 {
    fn from(component: ColorComponent) -> Self {
        component.0 as f32 / 255.0
    }
}

impl From<ColorComponent> for f64 {
    fn from(component: ColorComponent) -> Self {
        component.0 as f64 / 255.0
    }
}
