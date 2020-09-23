use std::thread::{self, JoinHandle};

pub struct Worker {
    handle: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn new() -> Self {
        Self { handle: None }
    }

    pub fn launch(&mut self) {
        let h = thread::spawn(move || {
            println!("Worker started.");
        });

        self.handle = Some(h);
    }

    pub fn get_handle(&mut self) -> Option<JoinHandle<()>> {
        self.handle.take()
    }
}
