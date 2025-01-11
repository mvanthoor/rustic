use crate::engine::Engine;
use librustic::communication::defs::Information;
use std::sync::{mpsc::channel, Arc};

impl Engine {
    pub fn main_loop(&mut self) {
        // Set up a channel for incoming information.
        let (info_tx, info_rx) = channel::<Information>();

        // Store the information receiver in the engine for use in other functions.
        self.info_rx = Some(info_rx);

        // Initialize Communications and Search modules.
        self.comm.init(
            info_tx.clone(),
            Arc::clone(&self.board),
            Arc::clone(&self.features),
        );
        self.search
            .init(info_tx, Arc::clone(&self.board), Arc::clone(&self.mg));

        // Keep looping forever until 'quit' received.
        while !self.quit {
            let incoming = match &self.info_rx {
                Some(i) => i.recv(),
                None => panic!("UCI command receiver not available"),
            };

            if let Ok(i) = incoming {
                match i {
                    Information::Command(cmd) => self.comm_handler(cmd),
                    Information::Search(report) => self.search_handler(report),
                }
            }
        }

        // Main loop has ended.
        self.comm.shutdown();
        self.search.shutdown();
    }
}
