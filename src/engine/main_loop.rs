use super::{
    defs::{ErrFatal, Information},
    Engine,
};
use crate::{
    comm::{CommControl, CommReport},
    evaluation::evaluate_position,
    misc::parse,
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
        self.search.init();

        // Keep looping forever until 'quit' received.
        while !self.quit {
            let information = self.info_rx();

            match information {
                Information::Comm(cr) => self.received_comm_reports(cr),
            }
        }

        // Main loop has ended.
        self.comm.wait_for_shutdown();
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
            CommReport::InitCompleted => self.comm.send(CommControl::Update),

            CommReport::Quit => {
                self.comm.send(CommControl::Quit);
                self.quit = true;
            }

            CommReport::Move(m) => {
                self.execute_move(m);
                self.comm.send(CommControl::Update);
            }

            CommReport::Evaluate => {
                let evaluation = evaluate_position(&self.board.lock().expect(ErrFatal::LOCK));
                let msg = format!("Evaluation: {}", evaluation);
                self.comm.send(CommControl::Write(msg));
                self.comm.send(CommControl::Update);
            }

            _ => (),
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
        let result = self.pseudo_legal(potential_move, &self.board, &self.mg);

        // If the move is possibly legal, execute it and determine that the
        // king is not left in check.
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
