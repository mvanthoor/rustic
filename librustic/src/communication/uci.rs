pub mod cmd_in;
pub mod cmd_out;
pub mod defs;
pub mod init;
pub mod parse;
pub mod print;
pub mod thread_in;
pub mod thread_out;

use crate::{
    communication::defs::{EngineOutput, EngineState},
    communication::protocol::{
        Properties, Protocol, RequireGameResult, RequireStatefulMode, SupportFancyAbout,
    },
    defs::About,
};
use std::{sync::mpsc::Sender, thread::JoinHandle};

pub struct Uci {
    pub about: About,
    pub input_thread: Option<JoinHandle<()>>,
    pub output_thread: Option<JoinHandle<()>>,
    pub output_write: Option<Sender<EngineOutput>>,
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
            output_write: None,
            properties: Properties::new(
                Protocol::UCI,
                SupportFancyAbout::Yes,
                RequireStatefulMode::No,
                RequireGameResult::No,
                EngineState::UciNotUsed,
            ),
        }
    }
}
