use super::{
    defs::{ErrFatal, Information},
    Engine,
};
use crate::{
    comm::{CommControl, CommReport},
    defs::Sides,
    evaluation::evaluate_position,
    misc::parse,
    search::SearchControl,
};
use std::sync::Arc;

const MOVE_NOT_ALLOWED: &str = "Move not allowed: King left in check.";
const MOVE_NOT_LEGAL: &str = "This is not a legal move.";

impl Engine {
    pub fn main_loop(&mut self) {
        // Set up a channel for incoming information.
        let (info_tx, info_rx) = crossbeam_channel::unbounded::<Information>();

        // Store the information receiver in the engine for use in other functions.
        self.info_rx = Some(info_rx);

        // Initialize Communications and Search modules.
        self.comm.init(info_tx.clone(), Arc::clone(&self.board));
        self.search.init(info_tx.clone(), Arc::clone(&self.board));

        // Update the Comm interface screen output (if any).
        self.comm.send(CommControl::Update);

        // Keep looping forever until 'quit' received.
        while !self.quit {
            let information = self.info_rx();

            match information {
                Information::Comm(cr) => self.received_comm_reports(cr),
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
    fn info_rx(&mut self) -> Information {
        match &self.info_rx {
            Some(i) => i.recv().expect(ErrFatal::CHANNEL),
            None => panic!(ErrFatal::NO_INFO_RX),
        }
    }

    fn received_comm_reports(&mut self, cr: CommReport) {
        match cr {
            // Quit Comm, Search, and then the engine itself.
            CommReport::Quit => {
                self.comm.send(CommControl::Quit);
                self.search.send(SearchControl::Quit);
                self.quit = true;
            }

            // Execute the received move.
            CommReport::Move(m) => {
                self.execute_move(m);
                self.comm.send(CommControl::Update);
            }

            // Send evaluation result upon request.
            CommReport::Evaluate => {
                let mtx_board = self.board.lock().expect(ErrFatal::LOCK);
                let eval = evaluate_position(&mtx_board);

                // Determine if white or black has last moved.
                let white = (mtx_board.us() ^ 1) == Sides::WHITE;
                let side = if white { Sides::WHITE } else { Sides::BLACK };
                std::mem::drop(mtx_board);

                self.comm.send(CommControl::Evaluation(eval, side));
                self.comm.send(CommControl::Update);
            }

            CommReport::Takeback => {
                let ok = self.board.lock().expect(ErrFatal::LOCK).unmake();
                if !ok {
                    let msg = String::from("Nothing to take back.");
                    self.comm.send(CommControl::Write(msg));
                }
                self.comm.send(CommControl::Update);
            }

            // Start or stop the search.
            CommReport::Search => self.search.send(SearchControl::Start),
            CommReport::Cancel => self.search.send(SearchControl::Stop),

            // Print the Help screen for the Comm module.
            CommReport::Help => {
                self.comm.send(CommControl::Help);
                self.comm.send(CommControl::Update);
            }

            // Ignore if Nothing reported or report is Unknown.
            CommReport::Nothing | CommReport::Unknown => (),
        }
    }
}

// These functions are executed to act on incoming information.
impl Engine {
    // Received: CommReport::Move.
    fn execute_move(&mut self, m: String) {
        // Prepare shorthand variables.
        let empty = (0usize, 0usize, 0usize);
        let potential_move = parse::algebraic_move_to_number(&m[..]).unwrap_or(empty);
        let is_pseudo_legal = self.pseudo_legal(potential_move, &self.board, &self.mg);

        // If the move is possibly legal, execute it and determine that the
        // king is not left in check.
        if let Ok(m) = is_pseudo_legal {
            let is_legal = self.board.lock().expect(ErrFatal::LOCK).make(m, &self.mg);
            if !is_legal {
                let msg = String::from(MOVE_NOT_ALLOWED);
                self.comm.send(CommControl::Write(msg));
            }
        } else {
            let msg = String::from(MOVE_NOT_LEGAL);
            self.comm.send(CommControl::Write(msg));
        }
    }
}
