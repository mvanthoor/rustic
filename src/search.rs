// search.rs contains the engine's search routine.

use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

// pub struct ErrFatal {}
// impl ErrFatal {
//     const CHANNEL_BROKEN: &'static str = "Channel is broken.";
// }

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
    pub fn activate(&mut self) -> Sender<SearchControl> {
        // Create a sender and receiver for setting up an incoming channel.
        let (in_tx, in_rx) = mpsc::channel::<SearchControl>();

        // Create the control thread
        let h = thread::spawn(move || {
            let mut control_cmd = SearchControl::NoCmd;

            // Keep running as long as no quit command is received.
            while control_cmd != SearchControl::Quit {
                // Read the next command.
                control_cmd = iterative_deepening(&in_rx);

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

fn iterative_deepening(rx: &Receiver<SearchControl>) -> SearchControl {
    let mut cmd = SearchControl::NoCmd;
    println!("Running Iterative Deepening...");

    // Do a next pass to the next depth, until either the requested depth
    // is reached, time is up, or a stop/quit command is received.
    while cmd != SearchControl::Quit {
        cmd = alpha_beta(rx);
    }

    println!("Quitting Iterative Deepening...");
    cmd
}

fn alpha_beta(rx: &Receiver<SearchControl>) -> SearchControl {
    let mut cmd = SearchControl::NoCmd;
    println!("Running Alpha/Beta...");

    // Keep searching until time is up, or stop/quit is received.
    while cmd != SearchControl::Quit {
        cmd = rx.try_recv().unwrap_or(SearchControl::NoCmd)
    }

    println!("Quitting Alpha/Beta...");
    cmd
}
