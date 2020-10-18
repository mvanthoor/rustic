use super::{
    defs::{ErrFatal, Information},
    Engine,
};
use crate::comm::CommControl;
use std::sync::Arc;

impl Engine {
    pub fn main_loop(&mut self) {
        // Set up a channel for incoming information.
        let (info_tx, info_rx) = crossbeam_channel::unbounded::<Information>();

        // Store the information receiver in the engine for use in other functions.
        self.info_rx = Some(info_rx);

        // Initialize Communications and Search modules.
        self.comm.init(info_tx.clone(), Arc::clone(&self.board));
        self.search.init(
            info_tx.clone(),
            Arc::clone(&self.board),
            Arc::clone(&self.mg),
        );

        // Update the Comm interface screen output (if any).
        self.comm.send(CommControl::Update);

        // Keep looping forever until 'quit' received.
        while !self.quit {
            let information = &self.info_rx();

            match information {
                Information::Comm(cr) => self.comm_reports(cr),
                Information::Search(sr) => self.search_reports(sr),
            }
        }

        // Main loop has ended.
        self.comm.wait_for_shutdown();
        self.search.wait_for_shutdown();
    }

    // This is the main engine thread Information receiver.
    fn info_rx(&mut self) -> Information {
        match &self.info_rx {
            Some(i) => i.recv().expect(ErrFatal::CHANNEL),
            None => panic!(ErrFatal::NO_INFO_RX),
        }
    }
}
