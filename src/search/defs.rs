use crate::{
    board::Board,
    movegen::{defs::Move, MoveGenerator},
};
use crossbeam_channel::Receiver;
use std::{sync::Arc, time::Instant};

pub const INF: i16 = 25000;
pub const CHECKMATE: i16 = 24000;
pub const STALEMATE: i16 = 0;
pub const CHECKPOINT: usize = 10_000; // nodes

#[derive(PartialEq)]
// These commands can be used by the engine thread to control the search.
pub enum SearchControl {
    Start,
    Stop,
    Quit,
    Nothing,
}

// Ways to terminate a search.
#[derive(PartialEq)]
pub enum SearchTerminate {
    Stop,    // Search is halted.
    Quit,    // Search module is quit completely.
    Nothing, // No command received yet.
}

// This struct holds all the search parameters as set by the engine.
pub struct SearchParams {
    pub depth: u8,
    pub time_for_move: u128,
}

impl SearchParams {
    pub fn new(depth: u8, time_for_move: u128) -> Self {
        Self {
            depth,
            time_for_move,
        }
    }
}

// The search function will put all findings into this struct.
#[derive(PartialEq)]
pub struct SearchInfo {
    pub start_time: Instant,
    pub best_move: Move,
    pub nodes: usize,
    pub ply: u8,
    pub terminate: SearchTerminate,
}

impl SearchInfo {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            best_move: Move::new(0),
            nodes: 0,
            ply: 0,
            terminate: SearchTerminate::Nothing,
        }
    }
}

// The search process needs references to a lot of data, such as a copy of
// the current board to make moves on, the move generator, search paramters
// (depth, time available, etc...), SearchInfo to put the results, and a
// control receiver so the search can receive commands from the engine.
// These references are grouped in SearchRefs, so they don't have to be
// passed one by one as function arguments.

pub struct SearchRefs<'a> {
    pub board: &'a mut Board,
    pub mg: &'a Arc<MoveGenerator>,
    pub search_params: &'a mut SearchParams,
    pub search_info: &'a mut SearchInfo,
    pub control_rx: &'a Receiver<SearchControl>,
}
