use crate::renderer::renderer::Renderer;
use crate::Window;

pub struct Graphics {
    renderer: Option<Renderer>,
}

impl Graphics {
    pub fn new() -> Self {
        Self { renderer: None }
    }

    pub fn initialize(&mut self, window: Window, window_size: (u32, u32)) {
        self.renderer = Some(Renderer::new(window, window_size));
    }

    pub fn render_scene(&mut self) {
        // FIXME handle errors appropriately
        self.renderer.as_mut().unwrap().render();
    }
}
