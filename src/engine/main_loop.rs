/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2021, Marcel Vanthoor
https://rustic-chess.org/

Rustic is written in the Rust programming language. It is an original
work, not derived from any engine that came before it. However, it does
use a lot of concepts which are well-known and are in use by most if not
all classical alpha/beta-based chess engines.

Rustic is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License version 3 as published by
the Free Software Foundation.

Rustic is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
for more details.

You should have received a copy of the GNU General Public License along
with this program.  If not, see <http://www.gnu.org/licenses/>.
======================================================================= */

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
            None => panic!("{}", ErrFatal::NO_INFO_RX),
        }
    }
}
