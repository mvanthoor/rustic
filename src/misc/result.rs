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

// This file implements functions to enable either the search function or
// the engine to determine if the game is drawn. This can be a FIDE-rule
// draw such as the 50-move rule, or a technical draw due to insufficient
// material to deliver mate.

use crate::{
    board::{defs::Pieces, Board},
    defs::{Sides, MAX_MOVE_RULE},
    movegen::{
        defs::{MoveList, MoveType},
        MoveGenerator,
    },
    search::defs::SearchRefs,
};

pub enum GameResult {
    Checkmate,
    Stalemate,
    Insufficient,
    FiftyMoves,
    ThreeFold,
    Running,
}

// Returns true if the position should be evaluated as a draw.
pub fn is_draw(refs: &SearchRefs) -> bool {
    is_insufficient_material(refs.board)
        || is_repetition(refs.board) > 0
        || is_fifty_move_rule(refs.board)
}

// Checks the 50-move rule.
pub fn is_fifty_move_rule(board: &Board) -> bool {
    board.game_state.halfmove_clock >= MAX_MOVE_RULE
}

// Detects position repetitions in the game's history.
pub fn is_repetition(board: &Board) -> u8 {
    let mut count = 0;
    let mut stop = false;
    let mut i = (board.history.len() - 1) as i16;

    // Search the history list.
    while i >= 0 && !stop {
        let historic = board.history.get_ref(i as usize);

        // If the historic zobrist key is equal to the one of the board
        // passed into the function, then we found a repetition.
        if historic.zobrist_key == board.game_state.zobrist_key {
            count += 1;
        }

        // If the historic HMC is 0, it indicates that this position
        // was created by a capture or pawn move. We don't have to
        // search further back, because before this, we can't ever
        // repeat. After all, the capture or pawn move can't be
        // reverted or repeated.
        stop = historic.halfmove_clock == 0;

        // Search backwards.
        i -= 1;
    }
    count
}

#[rustfmt::skip]
// Returns true if there is insufficient matrial to deliver mate.
pub fn is_insufficient_material(board: &Board) -> bool {
    // It's not a draw if: ...there are still pawns.
    let w_p = board.get_pieces(Pieces::PAWN, Sides::WHITE).count_ones() > 0;     
    let b_p = board.get_pieces(Pieces::PAWN, Sides::BLACK).count_ones() > 0;        
    // ...there's a major piece on the board.
    let w_q = board.get_pieces(Pieces::QUEEN, Sides::WHITE).count_ones() > 0;
    let b_q = board.get_pieces(Pieces::QUEEN, Sides::BLACK).count_ones() > 0;
    let w_r = board.get_pieces(Pieces::ROOK, Sides::WHITE).count_ones() > 0;
    let b_r = board.get_pieces(Pieces::ROOK, Sides::BLACK).count_ones() > 0;
    // ...or two bishops for one side.
    // FIXME : Bishops must be on squares of different color
    let w_b = board.get_pieces(Pieces::BISHOP, Sides::WHITE).count_ones() > 1;
    let b_b = board.get_pieces(Pieces::BISHOP, Sides::BLACK).count_ones() > 1;
    // ... or a bishop+knight for at least one side.
    let w_bn =
        board.get_pieces(Pieces::BISHOP, Sides::WHITE).count_ones() > 0 &&
        board.get_pieces(Pieces::KNIGHT, Sides::WHITE).count_ones() > 0;
    let b_bn =
        board.get_pieces(Pieces::BISHOP, Sides::BLACK).count_ones() > 0 &&
        board.get_pieces(Pieces::KNIGHT, Sides::BLACK).count_ones() > 0;
     
    // If one of the conditions above is true, we still have enough
    // material for checkmate, so insufficient_material returns false.
    !(w_p || b_p || w_q || b_q || w_r || b_r || w_b || b_b ||  w_bn || b_bn)
}

// This function determines if, and how, the game was ended.
pub fn game_over(board: &mut Board, mg: &MoveGenerator) -> GameResult {
    let mut move_list = MoveList::new();
    let mut result = GameResult::Running;
    let mut has_move = false;

    // Generate all the pseudo-legal moves.
    mg.generate_moves(board, &mut move_list, MoveType::All);

    // As soon as we find a legal move, the game may not yet be over.
    for i in 0..move_list.len() {
        let m = move_list.get_move(i);
        if board.make(m, mg) {
            has_move = true;
            break;
        }
    }

    // If we don't have a legal move, we see if we are in check or not. If
    // in check, it's checkmate; if not, the result is stalemate.
    if !has_move {
        // If we're in check, the opponent is attacking our king square.
        if mg.square_attacked(board, board.opponent(), board.king_square(board.us())) {
            result = GameResult::Checkmate;
        } else {
            result = GameResult::Stalemate;
        }
    } else {
        // If we do have legal moves, the game could still be a draw.
        if is_insufficient_material(board) {
            result = GameResult::Insufficient;
        } else if is_fifty_move_rule(board) {
            result = GameResult::FiftyMoves;
        } else if is_repetition(board) >= 2 {
            result = GameResult::ThreeFold
        }
    }

    result
}