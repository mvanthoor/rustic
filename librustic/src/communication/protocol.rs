use crate::communication::defs::EngineState;

pub struct Protocol;
impl Protocol {
    pub const UCI: &'static str = "uci";
    pub const XBOARD: &'static str = "xboard";
}

pub enum SupportFancyAbout {
    Yes,
    No,
}

pub enum RequireStatefulMode {
    Yes,
    No,
}

pub enum RequireGameResult {
    Yes,
    No,
}

pub struct Properties {
    protocol_name: &'static str,
    support_fancy_about: SupportFancyAbout,
    require_stateful_mode: RequireStatefulMode,
    require_game_result: RequireGameResult,
    startup_state: EngineState,
}

impl Properties {
    pub fn new(
        protocol_name: &'static str,
        support_fancy_about: SupportFancyAbout,
        require_stateful_mode: RequireStatefulMode,
        require_game_result: RequireGameResult,
        startup_state: EngineState,
    ) -> Self {
        Self {
            protocol_name,
            support_fancy_about,
            require_stateful_mode,
            require_game_result,
            startup_state,
        }
    }

    pub fn protocol_name(&self) -> &str {
        self.protocol_name
    }

    pub fn support_fancy_about(&self) -> bool {
        match self.support_fancy_about {
            SupportFancyAbout::Yes => true,
            SupportFancyAbout::No => false,
        }
    }

    pub fn require_stateful_mode(&self) -> bool {
        match self.require_stateful_mode {
            RequireStatefulMode::Yes => true,
            RequireStatefulMode::No => false,
        }
    }

    pub fn require_game_result(&self) -> bool {
        match self.require_game_result {
            RequireGameResult::Yes => true,
            RequireGameResult::No => false,
        }
    }

    pub fn startup_state(&self) -> EngineState {
        self.startup_state
    }
}
