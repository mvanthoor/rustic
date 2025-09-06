pub mod cmd_in;
pub mod cmd_out;
mod defs;
mod init;
mod parse;
mod print;
mod thread_in;
mod thread_out;

use crate::communication::defs::{EngineOutput, EngineState};
use crate::communication::protocol::{
    Properties, Protocol, RequireGameResult, RequireStatefulMode, SupportFancyAbout,
};
use std::{sync::mpsc::Sender, thread::JoinHandle};

pub struct XBoard {
    properties: Properties,
    input_thread: Option<JoinHandle<()>>,
    output_thread: Option<JoinHandle<()>>,
    output_write: Option<Sender<EngineOutput>>,
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
            properties: Properties::new(
                Protocol::XBOARD,
                SupportFancyAbout::No,
                RequireStatefulMode::Yes,
                RequireGameResult::Yes,
                EngineState::Waiting,
            ),
            input_thread: None,
            output_thread: None,
            output_write: None,
        }
    }
}
