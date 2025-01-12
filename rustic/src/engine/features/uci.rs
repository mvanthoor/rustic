use librustic::communication::feature::{Feature, UiElement};

pub fn hash() -> Feature {
    Feature::new(
        "Hash",
        Some(UiElement::Spin),
        Some(String::from("32")),
        Some(String::from("0")),
        Some(String::from("65535")),
    )
}

pub fn clear_hash() -> Feature {
    Feature::new("Clear Hash", Some(UiElement::Button), None, None, None)
}
