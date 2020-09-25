use super::{Engine, ErrFatal};

use crate::{
    comm::{CommControl, CommReport},
    search::SearchControl,
};
use crossbeam_channel;

impl Engine {
    pub fn main_loop(&mut self) {
        let (comm_report_tx, comm_report_rx) = crossbeam_channel::unbounded::<CommReport>();

        let comm_control_tx = self.comm.activate(comm_report_tx, self.board.clone());
        comm_control_tx
            .send(CommControl::Update)
            .expect(ErrFatal::CHANNEL_BROKEN);

        // Activate the engine's search module.
        let search_tx = self.search.activate();

        // Keep reading incoming commands until "Quit" is received.
        let mut comm_report = CommReport::Nothing;

        // Keep looping until quit is received.
        while comm_report != CommReport::Quit {
            comm_report = comm_report_rx.recv().expect(ErrFatal::CHANNEL_BROKEN);

            match comm_report {
                CommReport::Quit => {
                    comm_control_tx
                        .send(CommControl::Quit)
                        .expect(ErrFatal::CHANNEL_BROKEN);

                    search_tx
                        .send(SearchControl::Quit)
                        .expect(ErrFatal::CHANNEL_BROKEN)
                }

                CommReport::Search => search_tx
                    .send(SearchControl::Search)
                    .expect(ErrFatal::CHANNEL_BROKEN),

                _ => comm_control_tx
                    .send(CommControl::Update)
                    .expect(ErrFatal::CHANNEL_BROKEN),
            }
        }

        self.comm.wait_for_shutdown();

        // The main loop has ended. Wait for the search to quit.
        if let Some(h) = self.search.get_handle() {
            h.join().expect(ErrFatal::FAIL_QUIT_SEARCH)
        }
    }
}
