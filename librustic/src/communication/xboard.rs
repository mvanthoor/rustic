pub mod cmd_in;
pub mod cmd_out;
pub mod init;
pub mod thread_in;
pub mod thread_out;

use crate::communication::defs::{EngineOutput, EngineState};
use crate::communication::protocol::{
    Properties, Protocol, RequireGameResult, RequireStatefulMode, SupportFancyAbout,
};
use crate::defs::About;
use std::{sync::mpsc::Sender, thread::JoinHandle};

pub struct XBoard {
    pub about: About,
    pub input_thread: Option<JoinHandle<()>>,
    pub output_thread: Option<JoinHandle<()>>,
    pub output_write: Option<Sender<EngineOutput>>,
    pub properties: Properties,
}

impl Default for XBoard {
    fn default() -> Self {
        Self::new(About::default())
    }
}

// Public functions
impl XBoard {
    // Create a new console.
    pub fn new(about: About) -> Self {
        Self {
            about,
            input_thread: None,
            output_thread: None,
            output_write: None,
            properties: Properties::new(
                Protocol::XBOARD,
                SupportFancyAbout::No,
                RequireStatefulMode::Yes,
                RequireGameResult::Yes,
                EngineState::Waiting,
            ),
        }
    }
}
