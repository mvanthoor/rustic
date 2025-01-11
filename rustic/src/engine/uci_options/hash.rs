use librustic::communication::uci::uci_option::{UciOption, UiElement};

const NAME: &str = "Hash";
const DEFAULT: &str = "32";
const MIN: &str = "0";
const MAX: &str = "65535";

pub fn new() -> UciOption {
    UciOption::new(
        NAME,
        UiElement::Spin,
        Some(String::from(DEFAULT)),
        Some(String::from(MIN)),
        Some(String::from(MAX)),
    )
}
