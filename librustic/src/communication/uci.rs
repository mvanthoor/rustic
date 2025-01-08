use crate::defs::About;
use cmd_out::UciOut;
use std::{sync::mpsc::Sender, thread::JoinHandle};

pub mod cmd_in;
pub mod cmd_out;
pub mod init;
pub mod thread_in;
pub mod thread_out;

pub struct Uci {
    pub about: About,
    pub input_thread: Option<JoinHandle<()>>,
    pub output_thread: Option<JoinHandle<()>>,
    pub uci_output: Option<Sender<UciOut>>,
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
        }
    }
}
