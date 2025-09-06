pub mod cmd_in;
pub mod cmd_out;
mod init;
mod parse;
mod print;
mod thread_in;
mod thread_out;

use crate::communication::{
    defs::{EngineInfo, EngineOutput, EngineState},
    feature::Feature,
    protocol::{Properties, Protocol, RequireGameResult, RequireStatefulMode, SupportFancyAbout},
};
use std::{
    sync::{Arc, mpsc::Sender},
    thread::JoinHandle,
};

pub struct Uci {
    engine_info: EngineInfo,
    features: Arc<Vec<Feature>>,
    properties: Properties,
    input_thread: Option<JoinHandle<()>>,
    output_thread: Option<JoinHandle<()>>,
    output_write: Option<Sender<EngineOutput>>,
}

impl Uci {
    // Create a new console.
    pub fn new(engine_info: EngineInfo, features: Arc<Vec<Feature>>) -> Self {
        Self {
            engine_info,
            features,
            properties: Properties::new(
                Protocol::UCI,
                SupportFancyAbout::Yes,
                RequireStatefulMode::No,
                RequireGameResult::No,
                EngineState::UciNotUsed,
            ),
            input_thread: None,
            output_thread: None,
            output_write: None,
        }
    }
}
