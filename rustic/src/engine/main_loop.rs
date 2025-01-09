use crate::engine::Engine;
use librustic::{
    basetypes::error::ErrFatal, comm::defs::Information, communication::uci::cmd_in::UciIn,
    search::defs::SearchReport,
};
use std::sync::{mpsc::channel, Arc};

impl Engine {
    pub fn main_loop(&mut self) {
        // Set up a channel for incoming information.
        let (info_tx, info_rx) = channel::<UciIn>();
        let (search_tx, search_rx) = channel::<SearchReport>();

        // Store the information receiver in the engine for use in other functions.
        self.info_rx = Some(info_rx);
        self.search_rx = Some(search_rx);

        // Initialize Communications and Search modules.
        self.comm.init(
            info_tx.clone(),
            Arc::clone(&self.board),
            Arc::clone(&self.options),
        );
        self.search
            .init(search_tx, Arc::clone(&self.board), Arc::clone(&self.mg));

        // Keep looping forever until 'quit' received.
        while !self.quit {
            match &self.info_rx {
                Some(uci) => {
                    let cmd = uci.try_recv();
                    if let Ok(cmd) = cmd {
                        self.comm_handler(cmd);
                    }
                }
                None => panic!("UCI command receiver not available"),
            }

            match &self.search_rx {
                Some(search) => {
                    let report = search.try_recv();
                    if let Ok(report) = report {
                        self.search_handler(report);
                    }
                }
                None => panic!("UCI command receiver not available"),
            }
        }

        // Main loop has ended.
        self.comm.shutdown();
        self.search.shutdown();
    }
}
