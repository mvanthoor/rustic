use super::{Engine, ErrFatal, Information};

use crate::{
    comm::{CommControl, CommReport},
    search::{SearchControl, SearchReport},
};

impl Engine {
    pub fn main_loop(&mut self) {
        // Set up channel for incoming information.
        let (info_tx, info_rx) = crossbeam_channel::unbounded::<Information>();

        // Activate comm and search modules.
        let comm_control_tx = self.comm.activate(info_tx.clone(), self.board.clone());
        let search_control_tx = self.search.activate(info_tx.clone(), self.board.clone());

        comm_control_tx
            .send(CommControl::Update)
            .expect(ErrFatal::CHANNEL_BROKEN);

        // Keep looping forever until "running" becomes false
        while self.running {
            let information = info_rx.recv().expect(ErrFatal::CHANNEL_BROKEN);

            match information {
                Information::Comm(cr) => {
                    ///// Match all incoming comm reports.
                    match cr {
                        CommReport::Quit => {
                            comm_control_tx
                                .send(CommControl::Quit)
                                .expect(ErrFatal::CHANNEL_BROKEN);

                            search_control_tx
                                .send(SearchControl::Quit)
                                .expect(ErrFatal::CHANNEL_BROKEN);

                            self.running = false;
                        }

                        CommReport::Search => search_control_tx
                            .send(SearchControl::Search)
                            .expect(ErrFatal::CHANNEL_BROKEN),

                        _ => comm_control_tx
                            .send(CommControl::Update)
                            .expect(ErrFatal::CHANNEL_BROKEN),
                    }
                }

                Information::Search(sr) => {
                    ///// Match all incoming search reports
                    match sr {
                        SearchReport::Finished => {
                            println!("Search finished.");
                            comm_control_tx
                                .send(CommControl::Update)
                                .expect(ErrFatal::CHANNEL_BROKEN)
                        }
                    }
                }
            }
        }

        self.comm.wait_for_shutdown();
        self.search.wait_for_shutdown();
    }
}
