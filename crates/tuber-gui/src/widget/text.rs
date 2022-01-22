use crate::widget::common::WidgetCommon;
use crate::widget::Widget;
use tuber_core::asset::AssetStore;
use tuber_core::transform::Transform2D;
use tuber_graphics::graphics::Graphics;

pub struct TextWidget {
    text: String,
    font_identifier: Option<String>,
    common: WidgetCommon,
}

impl TextWidget {
    pub fn new<S>(identifier: &str, text: S, font_identifier: Option<&str>) -> Self
    where
        S: ToString,
    {
        Self {
            text: text.to_string(),
            font_identifier: font_identifier.map(|str| str.to_string()),
            common: WidgetCommon::new(identifier),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text<S>(&mut self, text: S)
    where
        S: ToString,
    {
        self.text = text.to_string();
    }

    pub fn font_identifier(&self) -> Option<&String> {
        self.font_identifier.as_ref()
    }
}

impl Widget for TextWidget {
    fn draw(&mut self, graphics: &mut Graphics, asset_store: &mut AssetStore) {
        let transform = Transform2D::default();
        graphics.draw_text(&self.text, &self.font_identifier, &transform, asset_store);
    }

    fn common(&self) -> &WidgetCommon {
        &self.common
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_text_widget() {
        let text_widget = TextWidget::new("text_widget", "Text", None);
        assert_eq!(text_widget.text(), "Text");
        assert_eq!(text_widget.font_identifier(), None);
    }
    #[test]
    fn set_text() {
        let mut text_widget = TextWidget::new("text_widget", "Text", None);
        text_widget.set_text("Bonjour");
        assert_eq!(text_widget.text(), "Bonjour");
    }
}
