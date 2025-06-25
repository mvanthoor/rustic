use crate::{
    board::Board,
    defs::{MAX_PLY, NrOf, Sides},
    engine::defs::{Information, SearchData, TT},
    movegen::{
        defs::{Move, ShortMove},
        MoveGenerator,
    },
};
use crossbeam_channel::{Receiver, Sender};
use std::{
    sync::{Arc, Mutex, RwLock},
    time::Instant,
};

pub use super::time::OVERHEAD;

pub const INF: i16 = 25_000;
pub const ASPIRATION_WINDOW: i16 = 50;
pub const CHECKMATE: i16 = 24_000;
pub const CHECKMATE_THRESHOLD: i16 = 23_900;
pub const STALEMATE: i16 = 0;
pub const DRAW: i16 = 0;
pub const SHARP_MARGIN: i16 = 30;
pub const CHECK_TERMINATION: usize = 0x7FF;
pub const SEND_STATS: usize = 0x7FFFF;
pub const MIN_TIME_STATS: u128 = 2_000;
pub const MIN_TIME_CURR_MOVE: u128 = 1_000;
pub const MAX_KILLER_MOVES: usize = 2;
pub const NULL_MOVE_REDUCTION: i8 = 3;
pub const LMR_REDUCTION: i8 = 1;
pub const LMR_MOVE_THRESHOLD: u8 = 3;
pub const LMR_LATE_THRESHOLD: u8 = 6;
pub const LMR_LATE_REDUCTION: i8 = 2;

pub const MULTICUT_DEPTH: i8 = 4;
pub const MULTICUT_REDUCTION: i8 = 3;
pub const MULTICUT_CUTOFFS: u8 = 2;
pub const MULTICUT_MOVES: u8 = 4;

pub const RECAPTURE_EXTENSION: i8 = 1;

pub type SearchResult = (Move, SearchTerminate);
type KillerMoves = [[ShortMove; MAX_KILLER_MOVES]; MAX_PLY as usize];

#[derive(PartialEq, Clone)]
pub enum SearchControl {
    Start(SearchParams),
    Stop,
    Quit,
    Nothing,
}

#[derive(PartialEq, Copy, Clone)]
pub enum SearchTerminate {
    Stop,
    Quit,
    Nothing,
}

#[derive(PartialEq, Copy, Clone)]
pub enum SearchMode {
    Depth,
    MoveTime,
    Nodes,
    GameTime,
    Ponder,
    Infinite,
    Nothing,
}

#[derive(PartialEq, Copy, Clone)]
pub struct GameTime {
    pub wtime: u128,
    pub btime: u128,
    pub winc: u128,
    pub binc: u128,
    pub moves_to_go: Option<usize>,
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

#[derive(PartialEq, Copy, Clone)]
pub struct SearchParams {
    pub depth: i8,
    pub move_time: u128,
    pub nodes: usize,
    pub game_time: GameTime,
    pub search_mode: SearchMode,
    pub quiet: bool,
    pub sharp_margin: i16,
}

impl SearchParams {
    pub fn new() -> Self {
        Self {
            depth: MAX_PLY,
            move_time: 0,
            nodes: 0,
            game_time: GameTime::new(0, 0, 0, 0, None),
            search_mode: SearchMode::Nothing,
            quiet: false,
            sharp_margin: SHARP_MARGIN,
        }
    }

    pub fn is_game_time(&self) -> bool {
        matches!(self.search_mode, SearchMode::GameTime | SearchMode::Ponder)
    }
}

#[derive(PartialEq)]
pub struct SearchInfo {
    start_time: Option<Instant>,
    pub depth: i8,
    pub seldepth: i8,
    pub nodes: usize,
    pub ply: i8,
    pub killer_moves: KillerMoves,
    pub last_stats_sent: u128,
    pub history_heuristic: [[[u32; NrOf::SQUARES]; NrOf::PIECE_TYPES]; Sides::BOTH],
    pub counter_moves: [[[ShortMove; NrOf::SQUARES]; NrOf::PIECE_TYPES]; Sides::BOTH],
    pub last_curr_move_sent: u128,
    pub allocated_time: u128,
    pub terminate: SearchTerminate,
    pub root_analysis: Vec<RootMoveAnalysis>,
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
            history_heuristic: [[[0u32; NrOf::SQUARES]; NrOf::PIECE_TYPES]; Sides::BOTH],
            counter_moves: [[[ShortMove::new(0); NrOf::SQUARES]; NrOf::PIECE_TYPES]; Sides::BOTH],
            last_stats_sent: 0,
            last_curr_move_sent: 0,
            allocated_time: 0,
            terminate: SearchTerminate::Nothing,
            root_analysis: Vec::new(),
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
        self.terminate != SearchTerminate::Nothing
    }
}

#[derive(PartialEq, Clone)]
pub struct SearchSummary {
    pub depth: i8,
    pub seldepth: i8,
    pub time: u128,
    pub cp: i16,
    pub mate: u8,
    pub nodes: usize,
    pub nps: usize,
    pub hash_full: u16,
    pub pv: Vec<Move>,
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

#[derive(PartialEq, Copy, Clone)]
pub struct SearchStats {
    pub time: u128,
    pub nodes: usize,
    pub nps: usize,
    pub hash_full: u16,
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

#[derive(PartialEq, Clone)]
pub struct RootMoveAnalysis {
    pub mv: Move,
    pub eval: i16,
    pub good_replies: usize,
    pub reply: Option<Move>,
    pub reply_sequence: Vec<Move>,
}

pub struct SearchRefs<'a> {
    pub board: &'a mut Board,
    pub mg: &'a Arc<MoveGenerator>,
    pub tt: &'a Arc<RwLock<TT<SearchData>>>,
    pub tt_enabled: bool,
    pub search_params: &'a mut SearchParams,
    pub search_info: &'a mut SearchInfo,
    pub control_rx: &'a Receiver<SearchControl>,
    pub report_tx: &'a Sender<Information>,
}

#[derive(PartialEq)]
pub enum SearchReport {
    Finished(Move),
    SearchSummary(SearchSummary),
    SearchCurrentMove(SearchCurrentMove),
    SearchStats(SearchStats),
    InfoString(String),
}