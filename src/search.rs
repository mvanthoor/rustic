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

// search.rs contains the engine's search routine.

mod alpha_beta;
pub mod defs;
mod iter_deep;
mod qsearch;
mod sorting;
mod time;
mod utils;

use crate::{
    board::Board,
    engine::defs::{ErrFatal, Information},
    engine::defs::{SearchData, TT},
    movegen::MoveGenerator,
};
use crossbeam_channel::Sender;
use defs::{
    SearchControl, SearchInfo, SearchParams, SearchRefs, SearchReport, SearchSummary,
    SearchTerminate,
};
use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

pub struct Search {
    handle: Option<JoinHandle<()>>,
    control_tx: Option<Sender<SearchControl>>,
}

impl Search {
    pub fn new() -> Self {
        Self {
            handle: None,
            control_tx: None,
        }
    }

    pub fn init(
        &mut self,
        report_tx: Sender<Information>, // Used to send information to engine.
        board: Arc<Mutex<Board>>,       // Arc pointer to engine's board.
        mg: Arc<MoveGenerator>,         // Arc pointer to engine's move generator.
        tt: Arc<Mutex<TT<SearchData>>>,
        tt_enabled: bool,
    ) {
        // Set up a channel for incoming commands
        let (control_tx, control_rx) = crossbeam_channel::unbounded::<SearchControl>();

        // Create thread-local variables.
        let t_report_tx = report_tx;

        // Create the search thread.
        let h = thread::spawn(move || {
            // Create thread-local variables.
            let arc_board = Arc::clone(&board);
            let arc_mg = Arc::clone(&mg);
            let arc_tt = Arc::clone(&tt);
            let mut search_params = SearchParams::new();

            let mut quit = false;
            let mut halt = true;

            // As long as the search isn't quit, keep this thread alive.
            while !quit {
                // Wait for the next incoming command from the engine.
                let cmd = control_rx.recv().expect(ErrFatal::CHANNEL);

                // And react accordingly.
                match cmd {
                    SearchControl::Start(sp) => {
                        search_params = sp;
                        halt = false; // This will start the search.
                    }
                    SearchControl::Stop => halt = true,
                    SearchControl::Quit => quit = true,
                    SearchControl::Nothing => (),
                }

                // Search isn't halted and not going to quit.
                if !halt && !quit {
                    // Copy the current board to be used in this thread.
                    let mtx_board = arc_board.lock().expect(ErrFatal::LOCK);
                    let mut board = mtx_board.clone();
                    std::mem::drop(mtx_board);

                    // Create a place to put search information
                    let mut search_info = SearchInfo::new();

                    // Create references to all needed information and structures.
                    let mut search_refs = SearchRefs {
                        board: &mut board,
                        mg: &arc_mg,
                        tt: &arc_tt,
                        tt_enabled,
                        search_params: &mut search_params,
                        search_info: &mut search_info,
                        control_rx: &control_rx,
                        report_tx: &t_report_tx,
                    };

                    // Start the search using Iterative Deepening.
                    let (best_move, terminate) = Search::iterative_deepening(&mut search_refs);

                    // Inform the engine that the search has finished.
                    let information = Information::Search(SearchReport::Finished(best_move));
                    t_report_tx.send(information).expect(ErrFatal::CHANNEL);

                    // If the search was finished due to a Stop or Quit
                    // command then either halt or quit the search.
                    match terminate {
                        SearchTerminate::Stop => {
                            halt = true;
                        }
                        SearchTerminate::Quit => {
                            halt = true;
                            quit = true;
                        }
                        SearchTerminate::Nothing => (),
                    }
                }
            }
        });

        // Store the thread's handle and command sender.
        self.handle = Some(h);
        self.control_tx = Some(control_tx);
    }

    // This function is used to send commands into the search thread.
    pub fn send(&self, cmd: SearchControl) {
        if let Some(tx) = &self.control_tx {
            tx.send(cmd).expect(ErrFatal::CHANNEL);
        }
    }

    // After sending the quit command, the engine calls this function to
    // wait for the search to shut down.
    pub fn wait_for_shutdown(&mut self) {
        if let Some(h) = self.handle.take() {
            h.join().expect(ErrFatal::THREAD);
        }
    }
}
