use super::{Engine, ErrFatal, Information};
use crate::{
    comm::{CommControl, CommReport},
    misc::parse,
    search::{SearchControl, SearchReport},
};
use std::sync::Arc;

const MOVE_NOT_ALLOWED: &str = "Move not allowed: King left in check.";
const MOVE_NOT_LEGAL: &str = "This is not a legal move.";

impl Engine {
    pub fn main_loop(&mut self) {
        println!("Initializing engine...");
        // Set up a channel for incoming information.
        let (info_tx, info_rx) = crossbeam_channel::unbounded::<Information>();

        // Activate communication module.
        self.comm.activate(info_tx.clone(), Arc::clone(&self.board));

        // Activate search module.
        self.search.activate(info_tx, Arc::clone(&self.board));

        // Request Search to set up its worker threads.
        let n = self.settings.threads;
        self.search.send(SearchControl::CreateWorkers(n));

        // Wait for the workers to finish setting up. Then update Comm.
        let result = info_rx.recv().expect(ErrFatal::CHANNEL);
        if result == Information::Search(SearchReport::RequestCompleted) {
            // self.comm_tx(CommControl::Update);
        }

        // Keep looping forever until 'quit' received.
        while !self.quit {
            let information = info_rx.recv().expect(ErrFatal::CHANNEL);

            match information {
                Information::Comm(cr) => self.received_comm_reports(cr),
                Information::Search(sr) => self.received_search_reports(sr),
            }
        }

        // Main loop has ended.
        self.comm.wait_for_shutdown();
        self.search.wait_for_shutdown();
    }
}

// This block implements handling of incoming information, which will be in
// the form of either Comm or Search reports.
impl Engine {
    fn received_comm_reports(&mut self, cr: CommReport) {
        match cr {
            CommReport::Quit => {
                self.comm.send(CommControl::Quit);
                self.search.send(SearchControl::Quit);
                self.quit = true;
            }
            CommReport::Start => self.search.send(SearchControl::Start),
            CommReport::Stop => self.search.send(SearchControl::Stop),
            CommReport::Move(m) => {
                self.execute_cr_move(m);
                self.comm.send(CommControl::Update);
            }
        }
    }

    fn received_search_reports(&mut self, sr: SearchReport) {
        match sr {
            SearchReport::Finished => {
                println!("Search finished.");
                self.comm.send(CommControl::Update);
            }
            _ => (),
        }
    }
}

// If a received report requires the engine to perform a lot of
// actions, then these are collected into a function. These functions are
// implemented in this block.
impl Engine {
    // Received: CommReport::Move.
    // Make the move on the engine's board if it's legal.
    fn execute_cr_move(&mut self, m: String) {
        // Prepare shorthand variables.
        let empty = (0usize, 0usize, 0usize);
        let potential_move = parse::algebraic_move_to_number(&m[..]).unwrap_or(empty);
        let result = self.pseudo_legal(potential_move, &self.board, &self.mg);

        // If the move is possible, execute it and determine that the king
        // is not left in check.
        if let Ok(m) = result {
            let is_ok = self.board.lock().expect(ErrFatal::LOCK).make(m, &self.mg);
            if !is_ok {
                let msg = String::from(MOVE_NOT_ALLOWED);
                self.comm.send(CommControl::Write(msg));
            }
        } else {
            let msg = String::from(MOVE_NOT_LEGAL);
            self.comm.send(CommControl::Write(msg));
        }
    }
}
