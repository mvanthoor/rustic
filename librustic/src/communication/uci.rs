use crate::{
    communication::defs::EngineState,
    communication::protocol::{
        Properties, Protocol, RequireGameResult, RequireStatefulMode, SupportFancyAbout,
    },
    communication::uci::cmd_out::UciOut,
    defs::About,
};
use std::{sync::mpsc::Sender, thread::JoinHandle};

pub mod cmd_in;
pub mod cmd_out;
pub mod init;
pub mod output;
pub mod parse;
pub mod thread_in;
pub mod thread_out;

pub struct Uci {
    pub about: About,
    pub input_thread: Option<JoinHandle<()>>,
    pub output_thread: Option<JoinHandle<()>>,
    pub uci_output: Option<Sender<UciOut>>,
    pub properties: Properties,
}

impl Default for Uci {
    fn default() -> Self {
        Self::new(About::default())
    }
}

// Public functions
impl Uci {
    // Create a new console.
    pub fn new(about: About) -> Self {
        Self {
            about,
            input_thread: None,
            output_thread: None,
            uci_output: None,
            properties: Properties::new(
                Protocol::UCI,
                SupportFancyAbout::Yes,
                RequireStatefulMode::No,
                RequireGameResult::No,
                EngineState::Waiting,
            ),
        }
    }
}
