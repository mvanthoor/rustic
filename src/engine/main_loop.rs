use super::{Engine, ErrFatal};

use crate::{comm::Incoming, search::SearchControl};

impl Engine {
    pub fn main_loop(&mut self) {
        // Activate the engine's search module.
        let search_tx = self.search.activate();

        // Keep reading incoming commands until "Quit" is received.
        let mut comm_cmd = Incoming::NoCmd;
        while comm_cmd != Incoming::Quit {
            self.comm.print_before_read(self.board.clone());
            comm_cmd = self.comm.read();

            match comm_cmd {
                Incoming::Quit => search_tx
                    .send(SearchControl::Quit)
                    .expect(ErrFatal::CHANNEL_BROKEN),
                Incoming::Search => search_tx
                    .send(SearchControl::Search)
                    .expect(ErrFatal::CHANNEL_BROKEN),
                _ => (),
            }
        }

        // The main loop has ended. Wait for the search to quit.
        if let Some(h) = self.search.get_handle() {
            h.join().expect(ErrFatal::FAIL_QUIT_SEARCH)
        }
    }
}
