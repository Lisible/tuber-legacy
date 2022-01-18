use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use tuber_core::transform::Transform2D;
use tuber_graphics::graphics::Graphics;
use tuber_graphics::renderable::shape::RectangleShape;
use tuber_graphics::types::{Color, Size2};

type WindowId = u64;

pub struct ImmediateGUI {
    windows: HashMap<WindowId, Window>,
    edition_context: Vec<WindowId>,
}

impl ImmediateGUI {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            edition_context: vec![],
        }
    }

    pub fn begin_window(&mut self, window_title: &str) {
        let window_id = self.window_id_from_window_title(window_title);
        if !self.windows.contains_key(&window_id) {
            self.windows.insert(window_id, Window::new());
        }

        self.edition_context.push(window_id);
    }

    pub fn end_window(&mut self) {
        self.edition_context.pop().unwrap();
    }

    pub fn handle_event(&mut self) {
        todo!()
    }

    pub fn render(&mut self, graphics: &mut Graphics) {
        self.windows
            .values_mut()
            .for_each(|window| window.render(graphics));
    }

    fn window_id_from_window_title(&mut self, title: &str) -> WindowId {
        let mut hasher = DefaultHasher::new();
        title.hash(&mut hasher);
        hasher.finish()
    }
}

pub struct Window {
    size: Size2,
}

impl Window {
    pub fn new() -> Self {
        Self {
            size: Size2::new(10.0, 10.0),
        }
    }
}

pub trait Renderable {
    fn render(&self, graphics: &mut Graphics);
}

impl Renderable for Window {
    fn render(&self, graphics: &mut Graphics) {
        graphics.draw_ui_rectangle(
            &RectangleShape {
                width: self.size.width,
                height: self.size.height,
                color: Color::WHITE,
            },
            &Transform2D::default(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_window() {
        let mut immediate_gui = ImmediateGUI::new();
        immediate_gui.begin_window("truc");
        immediate_gui.end_window();
        assert_eq!(immediate_gui.windows.len(), 1);
    }
}
