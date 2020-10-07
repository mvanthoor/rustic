// search.rs contains the engine's search routine.

use crate::{
    board::Board,
    defs::MAX_DEPTH,
    engine::defs::{ErrFatal, Information},
};
use crossbeam_channel::Sender;
use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

pub enum SearchControl {
    Quit,
    Start,
    Stop,
}

pub struct Search {
    handle: Option<JoinHandle<()>>,
    control_tx: Option<Sender<SearchControl>>,
}

impl Search {
    pub fn new() -> Self {
        Self {
            handle: None,
            control_tx: None,
        }
    }

    pub fn init(&mut self, report_tx: Sender<Information>, board: Arc<Mutex<Board>>) {
        // Set up a channel for incoming commands
        let (control_tx, control_rx) = crossbeam_channel::unbounded::<SearchControl>();

        // Create thread-local variables.
        let _t_report_tx = report_tx.clone();
        let _t_arc_board = Arc::clone(&board);

        // Create the search thread.
        let h = thread::spawn(move || {
            let mut quit = false;
            let mut halt = true;

            while !quit {
                let cmd = control_rx.recv().expect(ErrFatal::CHANNEL);

                match cmd {
                    SearchControl::Quit => quit = true,
                    SearchControl::Start => halt = false,
                    SearchControl::Stop => halt = true,
                }

                if !halt {
                    Search::iterative_deepening();
                }
            }
        });

        // Store the thread's handle and command sender.
        self.handle = Some(h);
        self.control_tx = Some(control_tx);
    }

    // This function is used to send commands into the search thread.
    pub fn send(&self, cmd: SearchControl) {
        if let Some(tx) = &self.control_tx {
            tx.send(cmd).expect(ErrFatal::CHANNEL);
        }
    }

    pub fn wait_for_shutdown(&mut self) {
        if let Some(h) = self.handle.take() {
            h.join().expect(ErrFatal::THREAD);
        }
    }
}

impl Search {
    fn iterative_deepening() {
        let mut depth = 1;

        while depth <= MAX_DEPTH {
            Search::alpha_beta(depth);
            depth += 1;
        }
    }

    fn alpha_beta(depth: u8) {
        if depth == 0 {
            println!("done.");
            return;
        }

        println!("Depth: {}", depth);
        Search::alpha_beta(depth - 1);
    }
}
