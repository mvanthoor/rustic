use crate::basetypes::error::ErrFatal;
use crate::communication::defs::EngineOutput;
use crate::communication::feature::Feature;
use crate::communication::xboard::XBoard;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::thread;

use super::cmd_out::XBoardOut;

impl XBoard {
    pub fn output_thread(&mut self, features: Arc<Vec<Feature>>) {
        // Create an incoming channel where the information is received.
        let (transmitter_for_engine, received_from_engine) = channel::<EngineOutput>();

        // Create the output thread. This will print information sent by
        // the engine thread to stdio. The information will either end up
        // on screen, or be captured by a GUI depending on how the engine
        // is being run.
        let thread = thread::spawn(move || {
            loop {
                // The receiver will block the thread until information is received.
                let print_to_stdio = received_from_engine.recv().expect(ErrFatal::CHANNEL);

                // EngineOutput could also contain output destined for
                // other protocols than UCI. We are only interested in the
                // UCI-part in this thread.
                if let EngineOutput::XBoard(cmd) = print_to_stdio {
                    match cmd {
                        XBoardOut::Quit => break, // This will shut down the input thread.
                    }
                }
            }
        });

        // Store the thread's handle and output transmitter for use by the
        // engine thread.
        self.output_thread = Some(thread);
        self.output_write = Some(transmitter_for_engine);
    }
}
