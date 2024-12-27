pub mod defs;
mod display;
mod draw;
mod fen;
mod gamestate;
mod history;
mod init;
mod moving;
mod playmove;
mod support;
mod utils;
mod zobrist;

use self::{defs::Pieces, gamestate::GameState, history::History, zobrist::ZobristRandoms};
use crate::defs::{Bitboard, NrOf, Piece, Sides, EMPTY};
use std::sync::Arc;

// This file implements the engine's board representation; it is bit-board
// based, with the least significant bit being A1.
#[derive(Clone)]
pub struct Board {
    pub bb_pieces: [[Bitboard; NrOf::PIECE_TYPES]; Sides::BOTH],
    pub bb_side: [Bitboard; Sides::BOTH],
    pub game_state: GameState,
    pub history: History,
    pub piece_list: [Piece; NrOf::SQUARES],
    zobrist_randoms: Arc<ZobristRandoms>,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

// Public functions for use by other modules.
impl Board {
    // Creates a new board with either the provided FEN, or the starting position.
    pub fn new() -> Self {
        Self {
            bb_pieces: [[EMPTY; NrOf::PIECE_TYPES]; Sides::BOTH],
            bb_side: [EMPTY; Sides::BOTH],
            game_state: GameState::new(),
            history: History::new(),
            piece_list: [Pieces::NONE; NrOf::SQUARES],
            zobrist_randoms: Arc::new(ZobristRandoms::new()),
        }
    }
}
