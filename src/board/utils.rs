/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2022, Marcel Vanthoor
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
    defs::{Location, Pieces},
    Board,
};
use crate::{
    board::defs::Ranks,
    defs::{Side, Sides, Square},
    engine::defs::GameEndReason,
    evaluation::defs::FLIP,
    movegen::{
        defs::{MoveList, MoveType},
        MoveGenerator,
    },
};

impl Board {
    // Compute on which file and rank a given square is.
    pub fn square_on_file_rank(square: Square) -> Location {
        let file = (square % 8) as u8; // square mod 8
        let rank = (square / 8) as u8; // square div 8
        (file, rank)
    }

    // Compute if a given square is or isn't on the given rank.
    pub fn square_on_rank(square: Square, rank: Square) -> bool {
        let start = (rank) * 8;
        let end = start + 7;
        (start..=end).contains(&square)
    }

    pub const fn fourth_rank(side: Side) -> usize {
        if side == Sides::WHITE {
            Ranks::R4
        } else {
            Ranks::R5
        }
    }

    pub const fn promotion_rank(side: Side) -> usize {
        if side == Sides::WHITE {
            Ranks::R8
        } else {
            Ranks::R1
        }
    }

    pub fn is_white_square(square: Square) -> bool {
        let rank = square / 8;
        let even_square = (square & 1) == 0;
        let even_rank = (rank & 1) == 0;

        (even_rank && !even_square) || (!even_rank && even_square)
    }

    pub const fn pawn_direction(side: Side) -> i8 {
        const UP: i8 = 8;
        const DOWN: i8 = -8;

        if side == Sides::WHITE {
            UP
        } else {
            DOWN
        }
    }

    pub const fn flip(side: Side, square: Square) -> usize {
        if side == Sides::WHITE {
            FLIP[square]
        } else {
            square
        }
    }

    pub fn is_white(&self) -> bool {
        self.us() == Sides::WHITE
    }

    // This function determines if checkmate can be delivered.
    pub fn is_checkmate_possible(&self) -> bool {
        // At least one side can still deliver checkmate if one of the
        // conditions below is true.
        self.get_pieces(Pieces::PAWN, Sides::WHITE).count_ones() > 0
            || self.get_pieces(Pieces::PAWN, Sides::BLACK).count_ones() > 0
            || self.get_pieces(Pieces::QUEEN, Sides::WHITE).count_ones() > 0
            || self.get_pieces(Pieces::QUEEN, Sides::BLACK).count_ones() > 0
            || self.get_pieces(Pieces::ROOK, Sides::WHITE).count_ones() > 0
            || self.get_pieces(Pieces::ROOK, Sides::BLACK).count_ones() > 0
            || self.has_bishop_pair(Sides::WHITE)
            || self.has_bishop_pair(Sides::BLACK)
            || (self.get_pieces(Pieces::BISHOP, Sides::WHITE).count_ones() >= 1
                && self.get_pieces(Pieces::KNIGHT, Sides::WHITE).count_ones() >= 1)
            || (self.get_pieces(Pieces::BISHOP, Sides::BLACK).count_ones() >= 1
                && self.get_pieces(Pieces::KNIGHT, Sides::BLACK).count_ones() >= 1)
            || self.get_pieces(Pieces::KNIGHT, Sides::WHITE).count_ones() >= 3
            || self.get_pieces(Pieces::KNIGHT, Sides::BLACK).count_ones() >= 3
    }

    // This function determines if, and how, the game was ended.
    pub fn is_game_end(&mut self, mg: &MoveGenerator) -> GameEndReason {
        // If we don't have a legal move, we see if we are in check or not. If
        // in check, it's checkmate; if not, the result is stalemate.
        if !self.moves_available(mg) {
            // If we're in check, the opponent is attacking our king square.
            if self.we_are_in_check(mg) {
                GameEndReason::Checkmate
            } else {
                GameEndReason::Stalemate
            }
        } else {
            // If we do have legal moves, the game could still be a draw.
            match () {
                _ if self.is_draw_by_insufficient_material() => GameEndReason::Insufficient,
                _ if self.is_draw_by_fifty_move_rule() => GameEndReason::FiftyMoves,
                _ if self.is_draw_by_repetition() >= 2 => GameEndReason::ThreeFold,
                _ => GameEndReason::NotEnded,
            }
        }
    }

    // Determines if the side to move has at least one legal move.
    pub fn moves_available(&mut self, mg: &MoveGenerator) -> bool {
        let mut move_list = MoveList::new();

        // Generate pseudo-legal moves.
        mg.generate_moves(self, &mut move_list, MoveType::All);

        // We can break as soon as we find a legal move.
        for i in 0..move_list.len() {
            let m = move_list.get_move(i);
            if self.make(m, mg) {
                // Unmake the move we just made.
                self.unmake();
                // Return true, as we have at least one move.
                return true;
            }
        }

        // No legal moves available.
        false
    }

    pub fn we_are_in_check(&self, mg: &MoveGenerator) -> bool {
        mg.square_attacked(self, self.opponent(), self.king_square(self.us()))
    }
}
