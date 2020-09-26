// search.rs contains the engine's search routine.

mod worker;

use crate::{board::Board, engine::Information};
use crossbeam_channel::Sender;
use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};
use worker::Worker;

pub struct ErrFatal {}
impl ErrFatal {
    const CHANNEL_BROKEN: &'static str = "Channel is broken.";
    const THREAD_FAILED: &'static str = "Thread has failed.";
    const LOCK_FAILED: &'static str = "Lock failed.";
}

#[derive(PartialEq)]
pub enum SearchControl {
    Nothing,
    Workers(usize),
    Search,
    Quit,
}
#[derive(PartialEq)]
pub enum SearchReport {
    Finished,         // Search is finished.
    RequestCompleted, // Requested operation is completed.
}

pub struct Search {
    workers: Arc<Mutex<Vec<Worker>>>,
    handle_control: Option<JoinHandle<()>>,
}

impl Search {
    pub fn new() -> Self {
        Self {
            workers: Arc::new(Mutex::new(Vec::with_capacity(1))),
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

        // Create clone of the Arc holding the worker store.
        let workers = Arc::clone(&self.workers);

        // Create the control thread
        let h = thread::spawn(move || {
            // Shorthand to send RequestCompleted to the engine.
            fn request_completed(tx: &Sender<Information>) {
                let report = SearchReport::RequestCompleted;
                let information = Information::Search(report);
                tx.send(information).expect(ErrFatal::CHANNEL_BROKEN)
            }

            let mut running = true;

            // Keep running as long as no quit command is received.
            while running {
                // Read the next command.
                let control_cmd = control_rx.recv().unwrap_or(SearchControl::Nothing);

                // Process the command.
                match control_cmd {
                    // When quit is received, the thread's loop will end.
                    SearchControl::Quit => running = false,

                    // This sets up one worker per engine thread.
                    SearchControl::Workers(w) => {
                        println!("Setting up {} workers...", w);
                        let mut mtx_workers = workers.lock().expect(ErrFatal::LOCK_FAILED);

                        for i in 0..w {
                            mtx_workers.push(Worker::new(i + 1));
                        }

                        for each in mtx_workers.iter() {
                            each.call();
                        }

                        std::mem::drop(mtx_workers);

                        request_completed(&report_tx);
                    }

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
