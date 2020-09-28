// search.rs contains the engine's search routine.

mod worker;

use crate::{
    board::Board,
    engine::{ErrFatal, Information},
};
use crossbeam_channel::Sender;
use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};
use worker::{Worker, WorkerControl};

#[derive(PartialEq)]
pub enum SearchControl {
    Nothing,
    CreateWorkers(usize),
    Start,
    Stop,
    Quit,
}
#[derive(PartialEq)]
pub enum SearchReport {
    Finished,         // Search is finished.
    RequestCompleted, // Requested operation is completed.
}

pub struct Search {
    worker_pool: Arc<Mutex<Vec<Worker>>>,      // Pool of workers
    control_handle: Option<JoinHandle<()>>,    // Thread handle to control search.
    control_tx: Option<Sender<SearchControl>>, // Sender for SearchControl Commands
}

impl Search {
    pub fn new() -> Self {
        Self {
            worker_pool: Arc::new(Mutex::new(Vec::with_capacity(1))),
            control_handle: None,
            control_tx: None,
        }
    }

    // Start the control procedure in its own thread.
    pub fn activate(&mut self, report_tx: Sender<Information>, _board: Arc<Mutex<Board>>) {
        println!("Starting Search Control thread.");

        // Create a sender and receiver for setting up an incoming channel.
        let (control_tx, control_rx) = crossbeam_channel::unbounded::<SearchControl>();

        // Clone Arc to worker pool for use in thread.
        let t_worker_pool = Arc::clone(&self.worker_pool);

        // Create the control thread
        let h = thread::spawn(move || {
            let mut quit = false;

            // Keep running as long as no quit command is received.
            while !quit {
                // Read the next command.
                let control_cmd = control_rx.recv().unwrap_or(SearchControl::Nothing);

                // Process the command.
                match control_cmd {
                    // This sets up one worker per engine thread.
                    SearchControl::CreateWorkers(n) => {
                        Search::create_workers(&t_worker_pool, n);
                        Search::request_completed(&report_tx);
                    }

                    SearchControl::Start => {
                        for worker in t_worker_pool.lock().expect(ErrFatal::LOCK).iter() {
                            worker.send(WorkerControl::Start);
                        }
                    }

                    SearchControl::Stop => {
                        for worker in t_worker_pool.lock().expect(ErrFatal::LOCK).iter() {
                            worker.send(WorkerControl::Stop);
                        }
                    }

                    // When quit is received, the thread's loop will end.
                    SearchControl::Quit => {
                        Search::shutdown_workers(&t_worker_pool);
                        quit = true;
                    }

                    _ => (),
                }
            }

            println!("Quitting Search Control thread.");
        });

        // Store the thread handle and sender.
        self.control_handle = Some(h);
        self.control_tx = Some(control_tx);
    }
}

// Public functions for controlling the search.
impl Search {
    // This function can send messages into the control thread.
    pub fn send(&self, msg: SearchControl) {
        if let Some(tx) = self.control_tx.clone() {
            tx.send(msg).expect(ErrFatal::CHANNEL);
        }
    }

    // After the engine issues 'quit' to the control thread, it calls this
    // function and then sits here waiting for the search to shut down.
    pub fn wait_for_shutdown(&mut self) {
        println!("Waiting for Search shutdown...");

        if let Some(h) = self.control_handle.take() {
            h.join().expect(ErrFatal::THREAD);
        }

        println!("Search shutdown completed.");
    }
}

// Private functions
impl Search {
    // Creates 'n' number of workers.
    fn create_workers(worker_pool: &Arc<Mutex<Vec<Worker>>>, n: usize) {
        println!("Setting up {} workers.", n);

        let mut mtx_workers = worker_pool.lock().expect(ErrFatal::LOCK);

        for i in 0..n {
            let mut worker = Worker::new(i + 1);
            worker.activate();
            mtx_workers.push(worker);
        }
    }

    // Sends a shutdown command to all workers and waits for them to quit.
    fn shutdown_workers(worker_pool: &Arc<Mutex<Vec<Worker>>>) {
        let mut mtx_workers = worker_pool.lock().expect(ErrFatal::LOCK);

        println!("Send quit to workers.");

        for worker in mtx_workers.iter() {
            worker.send(WorkerControl::Quit);
        }

        println!("Waiting for worker shutdown...");

        for worker in mtx_workers.iter_mut() {
            worker.wait_for_shutdown();
        }

        println!("Worker shutdown completed.");
    }

    // Shorthand to send RequestCompleted to the engine.
    fn request_completed(tx: &Sender<Information>) {
        let report = SearchReport::RequestCompleted;
        let information = Information::Search(report);
        tx.send(information).expect(ErrFatal::CHANNEL)
    }
}
