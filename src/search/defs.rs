use crate::{
    board::Board,
    defs::MAX_PLY,
    engine::defs::{Information, SearchData, Verbosity, TT},
    movegen::{
        defs::{Move, ShortMove},
        MoveGenerator,
    },
};
use crossbeam_channel::{Receiver, Sender};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

pub use super::time::SAFEGUARD;

pub const INF: i16 = 25_000;
// pub const ASPIRATION_WINDOW: i16 = 50;
pub const CHECKMATE: i16 = 24_000;
pub const CHECKMATE_THRESHOLD: i16 = 23_900;
pub const STALEMATE: i16 = 0;
pub const DRAW: i16 = 0;
pub const CHECK_TERMINATION: usize = 0x7FF; // 2.047 nodes
pub const SEND_STATS: usize = 0x7FFFF; // 524.287 nodes
pub const MIN_TIME_STATS: u128 = 2_000; // Minimum time for sending stats
pub const MIN_TIME_CURR_MOVE: u128 = 1_000; // Minimum time for sending curr_move
pub const MAX_KILLER_MOVES: usize = 2;

pub type SearchResult = (Move, SearchTerminated);
type KillerMoves = [[ShortMove; MAX_KILLER_MOVES]; MAX_PLY as usize];

#[derive(PartialEq, Clone)]
pub struct PrincipalVariation(Vec<Move>);
impl PrincipalVariation {
    pub fn new() -> Self {
        Self { 0: Vec::new() }
    }

    pub fn first_move(&self) -> Move {
        if !self.0.is_empty() {
            self.0[0]
        } else {
            Move::new(0)
        }
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn push(&mut self, m: Move) {
        self.0.push(m);
    }

    pub fn append(&mut self, pv: &mut PrincipalVariation) {
        self.0.append(&mut pv.0);
    }
}

#[derive(PartialEq)]
// These commands can be used by the engine thread to control the search.
pub enum SearchControl {
    Start(SearchParams),
    Stop,    // Stop the search and deliver a best move.
    Abandon, // Stop the search and abandon any results.
    Quit,    // Quit the search and the engine.
    Nothing, // No-Op
}

// Ways to terminate a search.
#[derive(PartialEq, Copy, Clone)]
pub enum SearchTerminated {
    Stopped,   // Search is stopped with a best move.
    Abandoned, //Search is stopped, best move abandoned
    Quit,      // Search module (and engine) are shut down.
    Nothing,   // No command received yet.
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
    pub depth: i8,               // Maximum depth to search to
    pub move_time: u128,         // Maximum time per move to search
    pub nodes: usize,            // Maximum number of nodes to search
    pub game_time: GameTime,     // Time available for entire game
    pub search_mode: SearchMode, // Defines the mode to search in
    pub verbosity: Verbosity,    // No intermediate search stats updates
}

impl SearchParams {
    pub fn new() -> Self {
        Self {
            depth: MAX_PLY,
            move_time: 0,
            nodes: 0,
            game_time: GameTime::new(0, 0, 0, 0, None),
            search_mode: SearchMode::Nothing,
            verbosity: Verbosity::Full,
        }
    }

    pub fn is_game_time(&self) -> bool {
        self.search_mode == SearchMode::GameTime
    }
}

// The search function will put all findings collected during the running
// search into this struct.
#[derive(PartialEq)]
pub struct SearchInfo {
    start_time: Option<Instant>,     // Time the search started
    pub depth: i8,                   // Depth currently being searched
    pub seldepth: i8,                // Maximum selective depth reached
    pub nodes: usize,                // Nodes searched
    pub ply: i8,                     // Number of plies from the root
    pub killer_moves: KillerMoves,   // Killer moves (array; see "type" above)
    pub last_stats_sent: u128,       // When last stats update was sent
    pub last_curr_move_sent: u128,   // When last current move was sent
    pub allocated_time: u128,        // Allotted msecs to spend on move
    pub terminate: SearchTerminated, // Terminate flag
}

impl SearchInfo {
    pub fn new() -> Self {
        Self {
            start_time: None,
            depth: 0,
            seldepth: 0,
            nodes: 0,
            ply: 0,
            killer_moves: [[ShortMove::new(0); MAX_KILLER_MOVES]; MAX_PLY as usize],
            last_stats_sent: 0,
            last_curr_move_sent: 0,
            allocated_time: 0,
            terminate: SearchTerminated::Nothing,
        }
    }

    pub fn timer_start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn timer_elapsed(&self) -> u128 {
        if let Some(x) = self.start_time {
            x.elapsed().as_millis()
        } else {
            0
        }
    }

    pub fn interrupted(&self) -> bool {
        self.terminate != SearchTerminated::Nothing
    }
}

// After each completed depth, iterative deepening summarizes the running
// search results within this struct before sending it to the engine
// thread. The engine thread will send it to Comm, which will transform the
// information into UCI/XBoard/Console output and print it to STDOUT.
#[derive(PartialEq, Clone)]
pub struct SearchSummary {
    pub depth: i8,              // depth reached during search
    pub seldepth: i8,           // Maximum selective depth reached
    pub time: u128,             // milliseconds
    pub cp: i16,                // centipawns score
    pub mate: u8,               // mate in X moves
    pub nodes: usize,           // nodes searched
    pub nps: usize,             // nodes per second
    pub hash_full: u16,         // TT use in per mille
    pub pv: PrincipalVariation, // Principal Variation
}

impl SearchSummary {
    pub fn pv_as_string(&self) -> String {
        let mut pv = String::from("");
        for next_move in self.pv.0.iter() {
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
    pub legal_moves_total: u8,
}

impl SearchCurrentMove {
    pub fn new(curr_move: Move, curr_move_number: u8, legal_moves_total: u8) -> Self {
        Self {
            curr_move,
            curr_move_number,
            legal_moves_total,
        }
    }
}

// This struct holds search statistics. These will be sent through the
// engine thread to Comm, to be transmitted to the (G)UI.
#[derive(PartialEq, Copy, Clone)]
pub struct SearchStats {
    pub time: u128,     // Time spent searching
    pub nodes: usize,   // Number of nodes searched
    pub nps: usize,     // Speed in nodes per second
    pub hash_full: u16, // TT full in per mille
}

impl SearchStats {
    pub fn new(time: u128, nodes: usize, nps: usize, hash_full: u16) -> Self {
        Self {
            time,
            nodes,
            nps,
            hash_full,
        }
    }
}

// The search process needs references to a lot of data, such as a copy of
// the current board to make moves on, the move generator, search parameters
// (depth, time available, etc...), SearchInfo to put the results. It also
// needs references to the control receiver and report sender so it can
// receive commands from the engine and send reports back. These references
// are grouped in SearchRefs, so they don't have to be passed one by one as
// function arguments.
pub struct SearchRefs<'a> {
    pub board: &'a mut Board,
    pub mg: &'a Arc<MoveGenerator>,
    pub tt: &'a Arc<Mutex<TT<SearchData>>>,
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
    SearchStats(SearchStats),             // General search statistics.
    Ready,                                // Send when search thread is ready.
}
