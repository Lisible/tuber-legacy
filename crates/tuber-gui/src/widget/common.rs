pub struct WidgetCommon {
    identifier: String,
}

impl WidgetCommon {
    pub fn new(identifier: &str) -> Self {
        Self {
            identifier: identifier.into(),
        }
    }

    pub fn identifier(&self) -> &String {
        &self.identifier
    }
}

impl Default for WidgetCommon {
    fn default() -> Self {
        Self {
            identifier: "".into(),
        }
    }
}
