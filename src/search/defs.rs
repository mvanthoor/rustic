use crate::{
    board::Board,
    movegen::{defs::Move, MoveGenerator},
};
use crossbeam_channel::Receiver;
use std::sync::Arc;

pub const INF: i16 = 25000;
pub const CHECKMATE: i16 = 24000;
pub const STALEMATE: i16 = 0;
pub const CHECKPOINT: usize = 20_000; // nodes

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
}

impl SearchParams {
    pub fn new(depth: u8) -> Self {
        Self { depth }
    }
}

// The search function will put all findings into this struct.
#[derive(PartialEq)]
pub struct SearchInfo {
    pub best_move: Move,
    pub termination: SearchTerminate,
    pub nodes: usize,
    pub ply: u8,
}

impl SearchInfo {
    pub fn new() -> Self {
        Self {
            best_move: Move::new(0),
            termination: SearchTerminate::Nothing,
            nodes: 0,
            ply: 0,
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
