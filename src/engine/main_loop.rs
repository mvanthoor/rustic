use crate::{
    engine::defs::{ErrFatal, Information},
    engine::Engine,
};
use std::sync::{mpsc::channel, Arc};

impl Engine {
    pub fn main_loop(&mut self) {
        // Set up a channel for incoming information.
        let (info_tx, info_rx) = channel();

        // Store the information receiver in the engine for use in other functions.
        self.info_rx = Some(info_rx);

        // Initialize Communications and Search modules.
        self.comm.init(
            info_tx.clone(),
            Arc::clone(&self.board),
            Arc::clone(&self.options),
        );
        self.search.init(
            info_tx,
            Arc::clone(&self.board),
            Arc::clone(&self.mg),
            Arc::clone(&self.tt_search),
        );

        // Keep looping forever until 'quit' received.
        while !self.quit {
            let information = &self.info_rx();

            match information {
                Information::Comm(received) => self.comm_handler(received),
                Information::Search(report) => self.search_handler(report),
            }
        }

        // Main loop has ended.
        self.comm.shutdown();
        self.search.shutdown();
    }

    // This is the main engine thread Information receiver.
    pub fn info_rx(&mut self) -> Information {
        match &self.info_rx {
            Some(i) => i.recv().expect(ErrFatal::CHANNEL),
            None => panic!("{}", ErrFatal::NO_INFO_RX),
        }
    }
}
