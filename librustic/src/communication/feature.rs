pub trait IFeature<'a> {
    fn get_name(&self) -> &'a str;

    fn get_ui_element(&self) -> &Option<UiElement> {
        &None
    }

    fn get_default(&self) -> &Option<String> {
        &None
    }

    fn get_min(&self) -> &Option<String> {
        &None
    }

    fn get_max(&self) -> &Option<String> {
        &None
    }
}

pub struct Feature {
    name: &'static str,
    ui_element: Option<UiElement>,
    default: Option<String>,
    min: Option<String>,
    max: Option<String>,
}

impl Feature {
    pub fn new(
        name: &'static str,
        ui_element: Option<UiElement>,
        default: Option<String>,
        min: Option<String>,
        max: Option<String>,
    ) -> Self {
        Self {
            name,
            ui_element,
            default,
            min,
            max,
        }
    }
}

impl<'a> IFeature<'a> for Feature {
    fn get_name(&self) -> &'a str {
        self.name
    }

    fn get_ui_element(&self) -> &Option<UiElement> {
        &self.ui_element
    }

    fn get_default(&self) -> &Option<String> {
        &self.default
    }

    fn get_min(&self) -> &Option<String> {
        &self.min
    }

    fn get_max(&self) -> &Option<String> {
        &self.max
    }
}

pub enum UiElement {
    Spin,
    Button,
}
