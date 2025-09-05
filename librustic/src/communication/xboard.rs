pub mod cmd_in;
pub mod cmd_out;
pub mod defs;
pub mod init;
pub mod parse;
pub mod thread_in;
pub mod thread_out;

use crate::communication::defs::{EngineOutput, EngineState};
use crate::communication::protocol::{
    Properties, Protocol, RequireGameResult, RequireStatefulMode, SupportFancyAbout,
};
use std::{sync::mpsc::Sender, thread::JoinHandle};

pub struct XBoard {
    input_thread: Option<JoinHandle<()>>,
    output_thread: Option<JoinHandle<()>>,
    output_write: Option<Sender<EngineOutput>>,
    properties: Properties,
}

impl Default for XBoard {
    fn default() -> Self {
        Self::new()
    }
}

// Public functions
impl XBoard {
    // Create a new console.
    pub fn new() -> Self {
        Self {
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
