// This is the search controller. It doesn't search by itself, but it
// launches one or more worker threads. The search controller can receive
// control commands from the engine, and send search results.

use std::{
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
};

pub struct ErrFatal {}
impl ErrFatal {
    const CHANNEL_BROKEN: &'static str = "Channel is broken.";
}

#[derive(PartialEq, Clone)]
pub enum SearchControl {
    NoCmd,
    Quit,
}

pub struct Search {
    handle: Option<JoinHandle<()>>,
}

impl Search {
    pub fn new() -> Self {
        Self { handle: None }
    }

    // Start the control procedure in its own thread.
    pub fn control(&mut self) -> Sender<SearchControl> {
        // Create a sender and receiver for setting up an incoming channel.
        let (in_tx, in_rx) = mpsc::channel::<SearchControl>();

        // Create the control thread
        let h = thread::spawn(move || {
            let mut control_cmd = SearchControl::NoCmd;

            // Keep running as long as no quit command is received.
            while control_cmd != SearchControl::Quit {
                // Read the next command.
                control_cmd = in_rx.recv().expect(ErrFatal::CHANNEL_BROKEN);

                // Process the command.
                match control_cmd {
                    // When quit is received, the thread's loop will end.
                    SearchControl::Quit | SearchControl::NoCmd => (),
                }
            }
        });

        // Store the thread handle.
        self.handle = Some(h);

        // Return the sender to to the caller.
        in_tx
    }

    pub fn get_handle(&mut self) -> Option<JoinHandle<()>> {
        self.handle.take()
    }
}
