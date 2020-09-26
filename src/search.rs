// search.rs contains the engine's search routine.

use crate::{board::Board, engine::Information};
use crossbeam_channel::Sender;
use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

pub struct ErrFatal {}
impl ErrFatal {
    const CHANNEL_BROKEN: &'static str = "Channel is broken.";
    const THREAD_FAILED: &'static str = "Thread has failed.";
}

#[derive(PartialEq, Clone)]
pub enum SearchControl {
    Nothing,
    Search,
    Quit,
}

pub enum SearchReport {
    Finished,
}

pub struct Search {
    handle_control: Option<JoinHandle<()>>,
}

impl Search {
    pub fn new() -> Self {
        Self {
            handle_control: None,
        }
    }

    // Start the control procedure in its own thread.
    pub fn activate(
        &mut self,
        report_tx: Sender<Information>,
        _board: Arc<Mutex<Board>>,
    ) -> Sender<SearchControl> {
        println!("Starting Search Control thread.");

        // Create a sender and receiver for setting up an incoming channel.
        let (control_tx, control_rx) = crossbeam_channel::unbounded::<SearchControl>();

        // Create the control thread
        let h = thread::spawn(move || {
            let mut running = true;

            // Keep running as long as no quit command is received.
            while running {
                // Read the next command.
                let control_cmd = control_rx.recv().unwrap_or(SearchControl::Nothing);

                // Process the command.
                match control_cmd {
                    // When quit is received, the thread's loop will end.
                    SearchControl::Quit => running = false,
                    SearchControl::Search => {
                        let report = SearchReport::Finished;
                        let information = Information::Search(report);
                        report_tx.send(information).expect(ErrFatal::CHANNEL_BROKEN)
                    }
                    _ => (),
                }
            }

            println!("Quitting Search Control thread.");
        });

        // Store the thread handle.
        self.handle_control = Some(h);

        // Return the sender to to the caller.
        control_tx
    }

    // After the engine issues 'quit' to the control thread, it calls this
    // function and then sits here waiting for the search to shut down.
    pub fn wait_for_shutdown(&mut self) {
        println!("Waiting for Search to shut down...");

        if let Some(h) = self.handle_control.take() {
            h.join().expect(ErrFatal::THREAD_FAILED);
        }

        println!("Search shutdown completed.");
    }
}
