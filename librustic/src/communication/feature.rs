pub trait IFeature {
    fn get_name();
    fn get_ui_element();
    fn get_default();
    fn get_min();
    fn get_max();
}

pub struct Feature {
    pub name: &'static str,
    pub ui_element: Option<UiElement>,
    pub default: Option<String>,
    pub min: Option<String>,
    pub max: Option<String>,
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

pub enum UiElement {
    Spin,
    Button,
}
