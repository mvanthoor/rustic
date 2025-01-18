use crate::engine::Engine;
use librustic::{basetypes::error::ErrFatal, communication::defs::EngineInput};
use std::sync::{mpsc::channel, Arc};

impl Engine {
    pub fn main_loop(&mut self) {
        // Set up a channel for incoming information.
        let (info_tx, info_rx) = channel::<EngineInput>();

        // Store the information receiver in the engine for use in other functions.
        self.info_rx = Some(info_rx);

        // Initialize Communications and Search modules.
        self.comm.init(info_tx.clone(), Arc::clone(&self.features));
        self.search
            .init(info_tx, Arc::clone(&self.board), Arc::clone(&self.mg));

        // Keep looping forever until 'quit' received.
        while !self.quit {
            let incoming = match &self.info_rx {
                Some(i) => i.recv(),
                None => panic!("{}", ErrFatal::NO_INFO_RX),
            };

            if let Ok(i) = incoming {
                match i {
                    EngineInput::Uci(cmd) => self.uci_handler(cmd),
                    EngineInput::XBoard(cmd) => (),
                    EngineInput::Search(report) => self.search_handler(report),
                }
            }
        }

        // Main loop has ended.
        self.comm.shutdown();
        self.search.shutdown();
    }
}
