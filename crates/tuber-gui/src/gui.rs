use crate::widget::{AsAny, Widget};
use tuber_core::asset::AssetStore;
use tuber_graphics::graphics::Graphics;

pub struct GUI {
    root: Root,
}
impl GUI {
    pub fn new() -> Self {
        Self { root: Root::new() }
    }

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

pub trait GenericWidget: Widget + AsAny {}
pub struct Root {
    widgets: Vec<Box<dyn GenericWidget>>,
}

impl Root {
    pub fn new() -> Self {
        Self { widgets: vec![] }
    }

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_root() {
        let root = Root::new();
        assert_eq!(root.widgets().len(), 0)
    }

    #[test]
    fn add_widget() {
        use crate::widget::text::TextWidget;

        let mut root = Root::new();
        assert_eq!(root.widgets().len(), 0);
        root.add_widget(Box::new(TextWidget::new("Text", "font")));
        assert_eq!(root.widgets().len(), 1);
    }
}
