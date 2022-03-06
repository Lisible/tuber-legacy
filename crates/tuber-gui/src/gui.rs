use tuber_core::asset::AssetStore;
use tuber_graphics::graphics::Graphics;

use crate::widget::{AsAny, Widget};

#[derive(Default)]
pub struct GUI {
    root: Root,
}

impl GUI {
    pub fn root(&mut self) -> &mut Root {
        &mut self.root
    }

    pub fn render(&mut self, graphics: &mut Graphics, asset_store: &mut AssetStore) {
        self.root
            .widgets_mut()
            .iter_mut()
            .for_each(|widget| widget.draw(graphics, asset_store));
    }
}

impl Default for GUI {
    fn default() -> Self {
        Self::new()
    }
}

pub trait GenericWidget: Widget + AsAny {}

#[derive(Default)]
pub struct Root {
    widgets: Vec<Box<dyn GenericWidget>>,
}

impl Root {
    pub fn add_widget(&mut self, widget: Box<dyn GenericWidget>) {
        self.widgets.push(widget);
    }

    pub fn find<T: 'static>(&self, identifier: &str) -> Option<&T> {
        self.widgets
            .iter()
            .find(|widget| widget.common().identifier() == identifier)
            .map(|widget| widget.as_any().downcast_ref::<T>().unwrap())
    }

    pub fn find_mut<T: 'static>(&mut self, identifier: &str) -> Option<&mut T> {
        self.widgets
            .iter_mut()
            .find(|widget| widget.common().identifier() == identifier)
            .map(|widget| widget.as_any_mut().downcast_mut::<T>().unwrap())
    }

    pub fn widgets(&self) -> &[Box<dyn GenericWidget>] {
        self.widgets.as_slice()
    }

    pub fn widgets_mut(&mut self) -> &mut [Box<dyn GenericWidget>] {
        self.widgets.as_mut_slice()
    }
}

impl Default for Root {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default_root() {
        let root = Root::default();
        assert_eq!(root.widgets().len(), 0)
    }

    #[test]
    fn add_widget() {
        use crate::widget::text::TextWidget;

        let mut root = Root::default();
        assert_eq!(root.widgets().len(), 0);
        root.add_widget(Box::new(TextWidget::new("text_widget", "Text", None)));
        assert_eq!(root.widgets().len(), 1);
    }
}
