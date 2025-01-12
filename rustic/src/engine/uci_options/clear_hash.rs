use librustic::communication::feature::{Feature, UiElement};

const NAME: &str = "Clear Hash";

pub fn new() -> Feature {
    Feature::new(NAME, Some(UiElement::Button), None, None, None)
}
