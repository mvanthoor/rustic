use crate::engine::ErrFatal;
use crossbeam_channel::{self, Sender};
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time,
};

pub enum WorkerControl {
    Nothing,
    Start,
    Stop,
    Quit,
}

pub struct Worker {
    id: Arc<usize>,                            // Worker Id
    control_handle: Option<JoinHandle<()>>,    // Thread handle
    control_tx: Option<Sender<WorkerControl>>, // Sender for WorkerControl commands.
}

impl Worker {
    pub fn new(id: usize) -> Self {
        println!("Creating worker {}.", id);
        Self {
            id: Arc::new(id),
            control_handle: None,
            control_tx: None,
        }
    }

    pub fn activate(&mut self) {
        println!("Activating worker {}.", self.id);

        // Create channel for control commands
        let (control_tx, control_rx) = crossbeam_channel::unbounded::<WorkerControl>();

        // Create local variables for use in thread.
        let id = Arc::clone(&self.id);

        let h = thread::spawn(move || {
            let mut quit = false; // Quits and shuts down the thread.
            let mut halt = true; // Thread stops running, but doesn't quit.

            while !quit {
                let seconds = time::Duration::from_secs(2);

                let cmd = if !halt {
                    control_rx.try_recv().unwrap_or(WorkerControl::Nothing)
                } else {
                    control_rx.recv().unwrap_or(WorkerControl::Nothing)
                };

                match cmd {
                    WorkerControl::Quit => quit = true,
                    WorkerControl::Start => halt = false,
                    WorkerControl::Stop => halt = true,
                    _ => (),
                }

                if !halt {
                    println!("Worker {} reporting.", id);
                    thread::sleep(seconds);
                }
            }
        });

        // Store the handle and sender.
        self.control_handle = Some(h);
        self.control_tx = Some(control_tx);
    }

    // Used to send commands into the Worker thread.
    pub fn send(&self, msg: WorkerControl) {
        if let Some(tx) = self.control_tx.clone() {
            tx.send(msg).expect(ErrFatal::CHANNEL);
        }
    }

    pub fn wait_for_shutdown(&mut self) {
        println!("Shutting down worker: {}", self.id);

        if let Some(h) = self.control_handle.take() {
            h.join().expect(ErrFatal::CHANNEL);
        }

        println!("Shutdown for worker {} completed.", self.id);
    }
}
