use crate::gui::GenericWidget;
use crate::widget::common::WidgetCommon;
use std::any::Any;
use tuber_core::asset::AssetStore;
use tuber_graphics::graphics::Graphics;

pub mod common;
pub mod text;

pub trait Widget {
    fn draw(&mut self, graphics: &mut Graphics, asset_store: &mut AssetStore);
    fn common(&self) -> &WidgetCommon;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static + Widget> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<T: Widget + AsAny> GenericWidget for T {}
