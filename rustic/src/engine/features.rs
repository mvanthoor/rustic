use librustic::communication::feature::{Feature, UiElement};

pub fn hash() -> Feature {
    Feature::new(
        "Hash",
        Some(UiElement::Spin),
        Some("32"),
        Some("0"),
        Some("65535"),
    )
}

pub fn clear_hash() -> Feature {
    Feature::new("Clear Hash", Some(UiElement::Button), None, None, None)
}
