use crate::{
    board::Board,
    defs::{Side, Sides, MAX_DEPTH},
    engine::defs::Information,
    movegen::{defs::Move, MoveGenerator},
};
use crossbeam_channel::{Receiver, Sender};
use std::{sync::Arc, time::Instant};

pub const INF: i16 = 25_000;
pub const CHECKMATE: i16 = 24_000;
pub const STALEMATE: i16 = 0;
pub const DRAW: i16 = 0;
pub const CHECKPOINT: usize = 0x7FF; // 2047 nodes
pub const UPDATE_STATS: usize = 0x3FFFFF; // 4.194.303 nodes

pub type SearchResult = (Move, SearchTerminate);

#[derive(PartialEq)]
// These commands can be used by the engine thread to control the search.
pub enum SearchControl {
    Start(SearchParams),
    Stop,
    Quit,
    Nothing,
}

// Ways to terminate a search.
#[derive(PartialEq, Copy, Clone)]
pub enum SearchTerminate {
    Stop,    // Search is halted.
    Quit,    // Search module is quit completely.
    Nothing, // No command received yet.
}

// SearchMode lists how the search termination criteria will be evaluated,
// to see if the search has to be stopped.
#[derive(PartialEq, Copy, Clone)]
pub enum SearchMode {
    Depth,    // Run until requested depth is reached.
    MoveTime, // Run until 'time per move' is used up.
    Nodes,    // Run until the number of requested nodes was reached.
    GameTime, // Search determines when to quit, depending on available time.
    Infinite, // Run forever, until the 'stop' command is received.
    Nothing,  // No search mode has been defined.
}

#[derive(PartialEq, Copy, Clone)]
pub struct GameTime {
    pub wtime: u128,                // White time on the clock in milliseconds
    pub btime: u128,                // Black time on the clock in milliseconds
    pub winc: u128,                 // White time increment in milliseconds (if wtime > 0)
    pub binc: u128,                 // Black time increment in milliseconds (if btime > 0)
    pub moves_to_go: Option<usize>, // Moves to go to next time control (0 = sudden death)
}

impl GameTime {
    pub fn new(
        wtime: u128,
        btime: u128,
        winc: u128,
        binc: u128,
        moves_to_go: Option<usize>,
    ) -> Self {
        Self {
            wtime,
            btime,
            winc,
            binc,
            moves_to_go,
        }
    }
}

// This struct holds all the search parameters as set by the engine thread.
// (These parameters are either default, or provided by the user interface
// before the game starts.)
#[derive(PartialEq, Copy, Clone)]
pub struct SearchParams {
    pub search_side: Side,       // Side to start the search for
    pub depth: u8,               // Maximum depth to search to
    pub move_time: u128,         // Maximum time per move to search
    pub nodes: usize,            // Maximum number of nodes to search
    pub game_time: GameTime,     // Time available for entire game
    pub search_mode: SearchMode, // Defines the mode to search in
    pub quiet: bool,             // No intermediate search stats updates
}

impl SearchParams {
    pub fn new() -> Self {
        Self {
            search_side: Sides::WHITE,
            depth: MAX_DEPTH,
            move_time: 0,
            nodes: 0,
            game_time: GameTime::new(0, 0, 0, 0, None),
            search_mode: SearchMode::Nothing,
            quiet: false,
        }
    }
}

// The search function will put all findings collected during the running
// search into this struct.
#[derive(PartialEq)]
pub struct SearchInfo {
    pub depth: u8,
    pub seldepth: u8,
    pub start_time: Instant,
    pub best_move: Move,
    pub nodes: usize,
    pub pv: Vec<Move>,
    pub ply: u8,
    pub terminate: SearchTerminate,
}

impl SearchInfo {
    pub fn new() -> Self {
        Self {
            depth: 0,
            seldepth: 0,
            start_time: Instant::now(),
            best_move: Move::new(0),
            nodes: 0,
            pv: Vec::new(),
            ply: 0,
            terminate: SearchTerminate::Nothing,
        }
    }
}

// After each completed depth, iterative deepening summarizes the running
// search results within this struct before sending it to the engine
// thread. The engine thread will send it to Comm, which will transform the
// information into UCI/XBoard/Console output and print it to STDOUT.
#[derive(PartialEq, Clone)]
pub struct SearchSummary {
    pub depth: u8,     // depth reached during search
    pub seldepth: u8,  // Maximum selective depth reached
    pub time: u128,    // milliseconds
    pub cp: i16,       // centipawns score
    pub mate: u8,      // mate in X moves
    pub nodes: usize,  // nodes searched
    pub nps: usize,    // nodes per second
    pub pv: Vec<Move>, // Principal Variation
}

impl SearchSummary {
    pub fn pv_as_string(&self) -> String {
        let mut pv = String::from("");
        for next_move in self.pv.iter() {
            let m = format!(" {}", next_move.as_string());
            pv.push_str(&m[..]);
        }
        pv.trim().to_string()
    }
}

#[derive(PartialEq, Copy, Clone)]
// This struct holds the currently searched move, and its move number in
// the list of legal moves. This struct is sent through the engine thread
// to Comm, to be transmitted to the (G)UI.
pub struct SearchCurrentMove {
    pub curr_move: Move,
    pub curr_move_number: u8,
}

impl SearchCurrentMove {
    pub fn new(curr_move: Move, curr_move_number: u8) -> Self {
        Self {
            curr_move,
            curr_move_number,
        }
    }
}

// This struct holds search statistics. These will be sent through the
// engine thread to Comm, to be transmitted to the (G)UI.
#[derive(PartialEq, Copy, Clone)]
pub struct SearchStats {
    pub nodes: usize, // Number of nodes searched
    pub nps: usize,   // Speed in nodes per second
}

impl SearchStats {
    pub fn new(nodes: usize, nps: usize) -> Self {
        Self { nodes, nps }
    }
}

// The search process needs references to a lot of data, such as a copy of
// the current board to make moves on, the move generator, search paramters
// (depth, time available, etc...), SearchInfo to put the results. It also
// needs references to the control receiver and report sender so it can
// receive commands from the engine and send reports back. These references
// are grouped in SearchRefs, so they don't have to be passed one by one as
// function arguments.
pub struct SearchRefs<'a> {
    pub board: &'a mut Board,
    pub mg: &'a Arc<MoveGenerator>,
    pub search_params: &'a mut SearchParams,
    pub search_info: &'a mut SearchInfo,
    pub control_rx: &'a Receiver<SearchControl>,
    pub report_tx: &'a Sender<Information>,
}

// This struct holds all the reports a search can send to the engine.
#[derive(PartialEq)]
pub enum SearchReport {
    Finished(Move),                       // Search done. Contains the best move.
    SearchSummary(SearchSummary),         // Periodic intermediate results.
    SearchCurrentMove(SearchCurrentMove), // Move currently searched.
    SearchStats(SearchStats),             // General search statistics
}
