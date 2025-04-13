pub trait IFeature<'a> {
    fn get_name(&self) -> &'a str;

    fn get_ui_element(&self) -> Option<&UiElement> {
        None
    }

    fn get_default(&self) -> Option<&str> {
        None
    }

    fn get_min(&self) -> Option<&str> {
        None
    }

    fn get_max(&self) -> Option<&str> {
        None
    }
}

pub struct Feature {
    name: &'static str,
    ui_element: Option<UiElement>,
    default: Option<&'static str>,
    min: Option<&'static str>,
    max: Option<&'static str>,
}

impl Feature {
    pub fn new(
        name: &'static str,
        ui_element: Option<UiElement>,
        default: Option<&'static str>,
        min: Option<&'static str>,
        max: Option<&'static str>,
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

    fn get_ui_element(&self) -> Option<&UiElement> {
        self.ui_element.as_ref()
    }

    fn get_default(&self) -> Option<&str> {
        self.default
    }

    fn get_min(&self) -> Option<&str> {
        self.min
    }

    fn get_max(&self) -> Option<&str> {
        self.max
    }
}

pub enum UiElement {
    Spin,
    Button,
}
