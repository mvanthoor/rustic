use librustic::communication::uci::uci_option::{UciOption, UiElement};

const NAME: &str = "Clear Hash";

pub fn new() -> UciOption {
    UciOption::new(NAME, UiElement::Button, None, None, None)
}
