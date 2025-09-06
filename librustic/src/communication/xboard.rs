pub mod cmd_in;
pub mod cmd_out;
mod defs;
mod init;
mod parse;
mod print;
mod thread_in;
mod thread_out;

use crate::communication::defs::{EngineInfo, EngineOutput, EngineState};
use crate::communication::feature::Feature;
use crate::communication::protocol::{
    Properties, Protocol, RequireGameResult, RequireStatefulMode, SupportFancyAbout,
};
use std::sync::Arc;
use std::{sync::mpsc::Sender, thread::JoinHandle};

pub struct XBoard {
    engine_info: EngineInfo,
    features: Arc<Vec<Feature>>,
    properties: Properties,
    input_thread: Option<JoinHandle<()>>,
    output_thread: Option<JoinHandle<()>>,
    output_write: Option<Sender<EngineOutput>>,
}

// Public functions
impl XBoard {
    // Create a new console.
    pub fn new(engine_info: EngineInfo, features: Arc<Vec<Feature>>) -> Self {
        Self {
            engine_info,
            features,
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
