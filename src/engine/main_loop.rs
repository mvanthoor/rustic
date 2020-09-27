use super::{ControlTx, Engine, ErrFatal, Information};

use crate::{
    comm::{CommControl, CommReport},
    misc::parse,
    search::{SearchControl, SearchReport},
};

impl Engine {
    pub fn main_loop(&mut self) {
        // Set up a channel for incoming information.
        let (info_tx, info_rx) = crossbeam_channel::unbounded::<Information>();

        // Activate comm and search modules and give them info senders.
        let comm_control_tx = self.comm.activate(info_tx.clone(), self.board.clone());

        // info_tx can be moved instead of cloned because it is no longer
        // used after this statement..
        let search_control_tx = self.search.activate(info_tx, self.board.clone());

        // Store the provided control command senders for Comm and Search.
        self.ctrl_tx = ControlTx::new(Some(comm_control_tx), Some(search_control_tx));

        // Request Search to set up its worker threads.
        self.search_tx(SearchControl::Workers(self.settings.threads as usize));

        // Wait for the workers to finish setting up. Then update Comm.
        let result = info_rx.recv().expect(ErrFatal::CHANNEL_BROKEN);
        if result == Information::Search(SearchReport::RequestCompleted) {
            self.comm_tx(CommControl::Update);
        }

        // Keep looping forever until "running" becomes false.
        while self.running {
            let information = info_rx.recv().expect(ErrFatal::CHANNEL_BROKEN);

            match information {
                Information::Comm(cr) => self.received_comm_reports(cr),
                Information::Search(sr) => self.received_search_reports(sr),
            }
        }

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
                self.comm_tx(CommControl::Quit);
                self.search_tx(SearchControl::Quit);
                self.running = false;
            }
            CommReport::Search => self.search_tx(SearchControl::Search),
            CommReport::Move(m) => {
                self.execute_cr_move(m);
                self.comm_tx(CommControl::Update);
            }
        }
    }

    fn received_search_reports(&mut self, sr: SearchReport) {
        match sr {
            SearchReport::Finished => {
                println!("Search finished.");
                self.comm_tx(CommControl::Update);
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
            let is_ok = self
                .board
                .lock()
                .expect(ErrFatal::BOARD_LOCK)
                .make(m, &self.mg);
            if !is_ok {
                let msg = String::from("Not allowed: Move leaves king in check.");
                self.comm_tx(CommControl::Write(msg));
            }
        } else {
            let msg = String::from("Illegal move.");
            self.comm_tx(CommControl::Write(msg));
        }
    }
}
