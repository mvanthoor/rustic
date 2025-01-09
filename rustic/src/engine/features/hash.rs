use librustic::communication::feature::{Feature, UiElement};

const NAME: &str = "Hash";
const DEFAULT: &str = "32";
const MIN: &str = "0";
const MAX: &str = "65535";

pub fn new() -> Feature {
    Feature::new(
        NAME,
        UiElement::Spin,
        Some(String::from(DEFAULT)),
        Some(String::from(MIN)),
        Some(String::from(MAX)),
    )
}
