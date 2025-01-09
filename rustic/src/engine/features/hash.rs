use librustic::{
    comm::defs::{EngineOptionDefaults, EngineSetOption},
    communication::feature::{Feature, UiElement},
};

pub fn new() -> Feature {
    Feature::new(
        EngineSetOption::HASH,
        UiElement::Spin,
        Some(EngineOptionDefaults::HASH_DEFAULT.to_string()),
        Some(EngineOptionDefaults::HASH_MIN.to_string()),
        Some(EngineOptionDefaults::max_hash().to_string()),
    )
}
