use crossbeam_channel::{self, Sender};
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time,
};

pub struct Worker {
    id: Arc<usize>,
    handle: Option<JoinHandle<()>>,
    tx: Option<Sender<WorkerControl>>,
}

pub enum WorkerControl {
    Nothing,
    Start,
    Stop,
    Quit,
}

impl Worker {
    pub fn new(id: usize) -> Self {
        println!("Creating worker {}", id);
        Self {
            id: Arc::new(id),
            handle: None,
            tx: None,
        }
    }

    pub fn activate(&mut self) {
        println!("Activating worker {}.", self.id);

        // Create channel for control commands
        let (control_tx, control_rx) = crossbeam_channel::unbounded::<WorkerControl>();
        let id = Arc::clone(&self.id);

        let h = thread::spawn(move || {
            let mut running = true;
            while running {
                let cmd = control_rx.try_recv().unwrap_or(WorkerControl::Nothing);
                let seconds = time::Duration::from_secs(2);

                match cmd {
                    WorkerControl::Quit => running = false,
                    _ => (),
                }

                println!("Worker {} reporting.", id);
                thread::sleep(seconds);
            }
        });

        // Store the handle and sender.
        self.handle = Some(h);
        self.tx = Some(control_tx);
    }

    pub fn send(&self, msg: WorkerControl) {
        if let Some(tx) = self.tx.clone() {
            tx.send(msg).expect("Fail.");
        }
    }

    pub fn wait_for_shutdown(&mut self) {
        println!("Shutting down worker: {}", self.id);

        if let Some(h) = self.handle.take() {
            h.join().expect("Fail.");
        }

        println!("Shutdown for worker {} completed.", self.id);
    }
}
