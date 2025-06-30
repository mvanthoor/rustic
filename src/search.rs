/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2024, Marcel Vanthoor
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
    SearchTerminate, ThreadId, ThreadLocalData, TimeStats,
};
use std::{
    sync::{Arc, Mutex, RwLock, atomic::{AtomicBool, Ordering}},
    thread::{self, JoinHandle},
    time::Instant,
};

// Thread-safe termination flag
static SEARCH_TERMINATED: AtomicBool = AtomicBool::new(false);

pub struct Search {
    handle: Option<JoinHandle<()>>,
    control_tx: Option<Sender<SearchControl>>,
    thread_id: ThreadId,
}

impl Search {
    pub fn new(thread_id: ThreadId) -> Self {
        Self {
            handle: None,
            control_tx: None,
            thread_id,
        }
    }

    pub fn init(
        &mut self,
        report_tx: Sender<Information>,
        board: Arc<Mutex<Board>>,
        mg: Arc<MoveGenerator>,
        tt: Arc<RwLock<TT<SearchData>>>,
        tt_enabled: bool,
        time_stats: Arc<Mutex<TimeStats>>,
    ) {
        // Set up a channel for incoming commands
        let (control_tx, control_rx) = crossbeam_channel::unbounded::<SearchControl>();

        // Create thread-local variables
        let t_report_tx = report_tx;
        let thread_id = self.thread_id;

        // Create the search thread
        let h = thread::spawn(move || {
            // Create thread-local variables
            let arc_board = Arc::clone(&board);
            let arc_mg = Arc::clone(&mg);
            let arc_tt = Arc::clone(&tt);
            let arc_time_stats = Arc::clone(&time_stats);
            let mut search_params = SearchParams::new();

            // Create thread-local data structures
            let mut thread_local_data = ThreadLocalData::new(thread_id);
            let mut quit = false;
            let mut halt = true;

            // As long as the search isn't quit, keep this thread alive
            while !quit {
                // Wait for the next incoming command from the engine
                let cmd = control_rx.recv().expect(ErrFatal::CHANNEL);

                // And react accordingly
                match cmd {
                    SearchControl::Start(sp) => {
                        search_params = sp;
                        halt = false; // This will start the search
                        SEARCH_TERMINATED.store(false, Ordering::Relaxed);
                    }
                    SearchControl::Stop => {
                        halt = true;
                        SEARCH_TERMINATED.store(true, Ordering::Relaxed);
                    }
                    SearchControl::Quit => {
                        quit = true;
                        SEARCH_TERMINATED.store(true, Ordering::Relaxed);
                    }
                    SearchControl::Nothing => (),
                }

                // Search isn't halted and not going to quit
                if !halt && !quit {
                    // Copy the current board to be used in this thread
                    let mtx_board = arc_board.lock().expect(ErrFatal::LOCK);
                    let mut board = mtx_board.clone();
                    std::mem::drop(mtx_board);

                    // Create a place to put search information
                    let mut search_info = SearchInfo::new();
                    
                    // Get the persistent time statistics
                    let mut time_stats_guard = arc_time_stats.lock().expect(ErrFatal::LOCK);
                    search_info.time_stats = time_stats_guard.clone();
                    std::mem::drop(time_stats_guard);

                    // Create references to all needed information and structures
                    let mut search_refs = SearchRefs {
                        board: &mut board,
                        mg: &arc_mg,
                        tt: &arc_tt,
                        tt_enabled,
                        search_params: &mut search_params,
                        search_info: &mut search_info,
                        control_rx: &control_rx,
                        report_tx: &t_report_tx,
                        thread_local_data: &mut thread_local_data,
                    };

                    // Start the search using Iterative Deepening
                    let (best_move, terminate) = Search::iterative_deepening(&mut search_refs);

                    // Update the persistent time statistics
                    let mut time_stats_guard = arc_time_stats.lock().expect(ErrFatal::LOCK);
                    *time_stats_guard = search_info.time_stats.clone();
                    std::mem::drop(time_stats_guard);

                    // Inform the engine that the search has finished
                    let information = Information::Search(SearchReport::Finished(best_move));
                    t_report_tx.send(information).expect(ErrFatal::CHANNEL);

                    // If the search was finished due to a Stop or Quit
                    // command then either halt or quit the search
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

        // Store the thread's handle and command sender
        self.handle = Some(h);
        self.control_tx = Some(control_tx);
    }

    // This function is used to send commands into the search thread
    pub fn send(&self, cmd: SearchControl) {
        if let Some(tx) = &self.control_tx {
            tx.send(cmd).expect(ErrFatal::CHANNEL);
        }
    }

    // After sending the quit command, the engine calls this function to
    // wait for the search to shut down
    pub fn wait_for_shutdown(&mut self) {
        if let Some(h) = self.handle.take() {
            h.join().expect(ErrFatal::THREAD);
        }
    }
}

pub struct SearchManager {
    workers: Vec<Search>,
    thread_count: usize,
    search_start_time: Option<Instant>,
    time_stats: TimeStats,
}

impl SearchManager {
    pub fn new(threads: usize) -> Self {
        let mut workers = Vec::with_capacity(threads);
        for i in 0..threads {
            workers.push(Search::new(i as ThreadId));
        }

        Self { 
            workers,
            thread_count: threads,
            search_start_time: None,
            time_stats: TimeStats::new(),
        }
    }

    pub fn init(
        &mut self,
        report_tx: Sender<Information>,
        board: Arc<Mutex<Board>>,
        mg: Arc<MoveGenerator>,
        tt: Arc<RwLock<TT<SearchData>>>,
        tt_enabled: bool,
    ) {
        let time_stats = Arc::new(Mutex::new(self.time_stats.clone()));
        for w in self.workers.iter_mut() {
            w.init(
                report_tx.clone(),
                Arc::clone(&board),
                Arc::clone(&mg),
                Arc::clone(&tt),
                tt_enabled,
                Arc::clone(&time_stats),
            );
        }
    }

    pub fn send(&self, cmd: SearchControl) {
        for w in self.workers.iter() {
            let c = cmd.clone();
            w.send(c);
        }
    }

    pub fn wait_for_shutdown(&mut self) {
        for w in self.workers.iter_mut() {
            w.wait_for_shutdown();
        }
    }

    pub fn start_search(&mut self) {
        self.search_start_time = Some(Instant::now());
        SEARCH_TERMINATED.store(false, Ordering::Relaxed);
    }

    pub fn stop_search(&self) {
        SEARCH_TERMINATED.store(true, Ordering::Relaxed);
    }

    pub fn is_terminated(&self) -> bool {
        SEARCH_TERMINATED.load(Ordering::Relaxed)
    }

    pub fn thread_count(&self) -> usize {
        self.thread_count
    }

    pub fn get_time_stats(&self) -> TimeStats {
        self.time_stats.clone()
    }

    pub fn update_time_stats(&mut self, new_stats: TimeStats) {
        self.time_stats = new_stats;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        board::Board,
        engine::defs::{SearchData, TT},
        movegen::MoveGenerator,
        search::defs::{SearchControl, SearchInfo, SearchParams, SearchRefs, ThreadLocalData},
    };
    use crossbeam_channel::unbounded;
    use std::sync::{Arc, RwLock};

    #[test]
    fn test_search_manager_creation() {
        let manager = SearchManager::new(4);
        assert_eq!(manager.thread_count(), 4);
        assert_eq!(manager.workers.len(), 4);
    }

    #[test]
    fn test_thread_local_data() {
        let mut tld = ThreadLocalData::new(1);
        assert_eq!(tld.thread_id, 1);
        assert_eq!(tld.nodes_searched, 0);
        assert!(tld.best_move_found.is_none());

        tld.start_search();
        assert!(tld.search_start_time.is_some());
        assert_eq!(tld.nodes_searched, 0);
        assert_eq!(tld.search_depth, 0);

        tld.increment_nodes();
        assert_eq!(tld.nodes_searched, 1);

        let test_move = crate::movegen::defs::Move::new(0x1234);
        tld.update_best_move(test_move);
        assert!(tld.best_move_found.is_some());
        assert_eq!(tld.best_move_found.as_ref().map(|m| m.get_move()), Some(0x1234));
    }

    #[test]
    fn test_search_termination_flag() {
        // Test that the global termination flag works correctly
        assert!(!SearchManager::new(1).is_terminated());
        
        let mut manager = SearchManager::new(1);
        manager.stop_search();
        assert!(manager.is_terminated());
        
        manager.start_search();
        assert!(!manager.is_terminated());
    }

    #[test]
    fn test_thread_safety() {
        // Test that multiple threads can be created and managed safely
        let mut manager = SearchManager::new(2);
        let (info_tx, _info_rx) = unbounded::<Information>();
        let mut board = Board::new();
        board.fen_read(None).unwrap(); // Set to standard starting position
        let board = Arc::new(Mutex::new(board));
        let mg = Arc::new(MoveGenerator::new());
        let tt = Arc::new(RwLock::new(TT::<SearchData>::new(32)));

        manager.init(
            info_tx,
            Arc::clone(&board),
            Arc::clone(&mg),
            Arc::clone(&tt),
            true,
        );

        // Test that we can send commands to all threads
        let search_params = SearchParams::new();
        manager.send(SearchControl::Start(search_params));
        
        // Test that we can stop all threads
        manager.send(SearchControl::Stop);
        
        // Test that we can quit all threads
        manager.send(SearchControl::Quit);
        manager.wait_for_shutdown();
    }

    #[test]
    fn test_tt_batching() {
        // Test that TT batching works correctly
        let mut tld = ThreadLocalData::new(0);
        let _tt = Arc::new(RwLock::new(TT::<SearchData>::new(32)));
        
        // Add some test data to the batch
        let test_key = 0x1234567890ABCDEF;
        let test_data = SearchData::create(5, 0, crate::engine::defs::HashFlag::Exact, 100, crate::movegen::defs::ShortMove::new(0));
        
        tld.tt_batch.add(test_key, test_data);
        assert_eq!(tld.tt_batch.len(), 1);
        assert!(!tld.tt_batch.is_full());
        
        // Fill the batch
        for _ in 0..15 {
            tld.tt_batch.add(test_key, test_data);
        }
        assert!(tld.tt_batch.is_full());
        
        // Test clearing
        tld.tt_batch.clear();
        assert_eq!(tld.tt_batch.len(), 0);
    }

    #[test]
    fn test_search_refs_with_thread_local_data() {
        let mut board = Board::new();
        let mg = Arc::new(MoveGenerator::new());
        let tt: Arc<RwLock<TT<SearchData>>> = Arc::new(RwLock::new(TT::new(32)));
        let (_control_tx, control_rx) = unbounded::<SearchControl>();
        let (report_tx, _report_rx) = unbounded::<Information>();
        let mut search_params = SearchParams::new();
        let mut search_info = SearchInfo::new();
        let mut thread_local_data = ThreadLocalData::new(0);

        let refs = SearchRefs {
            board: &mut board,
            mg: &mg,
            tt: &tt,
            tt_enabled: true,
            search_params: &mut search_params,
            search_info: &mut search_info,
            control_rx: &control_rx,
            report_tx: &report_tx,
            thread_local_data: &mut thread_local_data,
        };

        // Test that the refs structure is properly constructed
        assert_eq!(refs.thread_local_data.thread_id, 0);
        assert_eq!(refs.tt_enabled, true);
    }
}