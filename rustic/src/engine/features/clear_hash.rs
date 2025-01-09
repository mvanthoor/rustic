use librustic::{
    comm::defs::EngineSetOption,
    communication::feature::{Feature, UiElement},
};

pub fn new() -> Feature {
    Feature::new(
        EngineSetOption::CLEAR_HASH,
        UiElement::Button,
        None,
        None,
        None,
    )
}
