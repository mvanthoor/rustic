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

use super::MoveGenerator;
use crate::{
    board::{
        defs::{Direction, Files, Location, Ranks, BB_FILES, BB_RANKS, BB_SQUARES},
        Board,
    },
    defs::{Bitboard, Square},
};

pub type BlockerBoards = Vec<Bitboard>;
pub type AttackBoards = Vec<Bitboard>;

impl MoveGenerator {
    // Explanation of rook mask, step by step. Get the location of square the
    // rook is on, as a (file, rank) tuple. Create the bitboards for files, ranks,
    // and the rook's square. Get the bitboards of the file and rank the rook is on.
    // Create a bitboard for the edges of the board, but do NOT include an edge if
    // the rook is actually on it. (Otherwise all bits would be unset.) Create the
    // rook's mask by combining its file and rank bitboards. For the final result:
    // exclude the edge squares and rook's square from the mask.
    pub fn rook_mask(square: Square) -> Bitboard {
        let location = Board::square_on_file_rank(square);
        let bb_rook_square = BB_SQUARES[square];
        let bb_edges = MoveGenerator::edges_without_piece(location);
        let bb_mask = BB_FILES[location.0 as usize] | BB_RANKS[location.1 as usize];

        bb_mask & !bb_edges & !bb_rook_square
    }

    // bishop_mask() works a bit differently compared to rook_mask(),
    // but in the end it does the same thing: create a mask for a sliding piece.
    // First, a bitboard containing all the edges (if the piece is not on the edge).
    // Starting at the given square, the function generates four rays, one for eeach
    // diagonal direction, on an empty board. As a final result, the four rays are
    // combined, to generate all bishop moves from that square, on an empty board.
    // Then the edges are clipped off, as they are not needed in the mask.
    pub fn bishop_mask(square: Square) -> Bitboard {
        let location = Board::square_on_file_rank(square);
        let bb_edges = MoveGenerator::edges_without_piece(location);
        let bb_up_left = MoveGenerator::bb_ray(0, square, Direction::UpLeft);
        let bb_up_right = MoveGenerator::bb_ray(0, square, Direction::UpRight);
        let bb_down_right = MoveGenerator::bb_ray(0, square, Direction::DownRight);
        let bb_down_left = MoveGenerator::bb_ray(0, square, Direction::DownLeft);

        (bb_up_left | bb_up_right | bb_down_right | bb_down_left) & !bb_edges
    }

    // This function creates a bitboard holding all the edges of the board, as
    // needed to clip the board edges off the rook and bishop masks. To prevent
    // clipping the entire ray if the piece itself is on an edge, the edge(s)
    // containing the piece are excluded.
    fn edges_without_piece(location: Location) -> Bitboard {
        let bb_piece_file = BB_FILES[location.0 as usize];
        let bb_piece_rank = BB_RANKS[location.1 as usize];

        (BB_FILES[Files::A] & !bb_piece_file)
            | (BB_FILES[Files::H] & !bb_piece_file)
            | (BB_RANKS[Ranks::R1] & !bb_piece_rank)
            | (BB_RANKS[Ranks::R8] & !bb_piece_rank)
    }

    // This function takes a square, and all the blocker boards belonging to that
    // squre. Then it'll iterate through those blocker boards, and generate the
    // attack board belonging to that blocker board. The 'length' parameter is the
    // length of the given array of blocker boards.
    pub fn rook_attack_boards(square: Square, blockers: &[Bitboard]) -> AttackBoards {
        let mut bb_attack_boards: AttackBoards = Vec::new();

        for b in blockers.iter() {
            let bb_attacks = MoveGenerator::bb_ray(*b, square, Direction::Up)
                | MoveGenerator::bb_ray(*b, square, Direction::Right)
                | MoveGenerator::bb_ray(*b, square, Direction::Down)
                | MoveGenerator::bb_ray(*b, square, Direction::Left);
            bb_attack_boards.push(bb_attacks);
        }

        bb_attack_boards
    }

    // Same as the function above, but for the bishop.
    pub fn bishop_attack_boards(square: Square, blockers: &[Bitboard]) -> AttackBoards {
        let mut bb_attack_boards: AttackBoards = Vec::new();

        for b in blockers.iter() {
            let bb_attacks = MoveGenerator::bb_ray(*b, square, Direction::UpLeft)
                | MoveGenerator::bb_ray(*b, square, Direction::UpRight)
                | MoveGenerator::bb_ray(*b, square, Direction::DownRight)
                | MoveGenerator::bb_ray(*b, square, Direction::DownLeft);
            bb_attack_boards.push(bb_attacks);
        }

        bb_attack_boards
    }

    // blocker_boards() takes a piece mask. This is a bitboard in which all
    // the bits are set for a square where a slider can move to, without the edges.
    // (As generated by the functions in the mask.rs file.) blocker_boards()
    // generates all possible permutations for the given mask, using the Carry
    // Rippler method. See the given link, or http://rustic-chess.org for more
    // information.

    pub fn blocker_boards(mask: Bitboard) -> BlockerBoards {
        let d: Bitboard = mask;
        let mut bb_blocker_boards: BlockerBoards = Vec::new();
        let mut n: Bitboard = 0;

        // Carry-Rippler
        // https://www.chessprogramming.org/Traversing_Subsets_of_a_Set
        loop {
            bb_blocker_boards.push(n);
            n = n.wrapping_sub(d) & d;
            if n == 0 {
                break;
            }
        }

        bb_blocker_boards
    }

    // This is a long function, but fortunately it's easy to understand. It creates
    // a ray for a sliding piece, in one of 8 directions: up, left, right, down,
    // upleft, upright, downleft, downright. (Some programs call it N, E, S, W, NW,
    // NE, SE, SW.) The function starts at the given square, in a given direction,
    // and then it keeps iterating in that direction until it either hits a piece,
    // or the edge of the board. Therefore, in each call, only one of the eight
    // blocks of this function wille be executed.
    pub fn bb_ray(bb_in: Bitboard, square: Square, direction: Direction) -> Bitboard {
        let mut file = Board::square_on_file_rank(square).0 as usize;
        let mut rank = Board::square_on_file_rank(square).1 as usize;
        let mut bb_square = BB_SQUARES[square];
        let mut bb_ray = 0;
        let mut done = false;
        while !done {
            done = true;
            match direction {
                Direction::Up => {
                    if rank != Ranks::R8 {
                        bb_square <<= 8;
                        bb_ray |= bb_square;
                        rank += 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::Right => {
                    if file != Files::H {
                        bb_square <<= 1;
                        bb_ray |= bb_square;
                        file += 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::Down => {
                    if rank != Ranks::R1 {
                        bb_square >>= 8;
                        bb_ray |= bb_square;
                        rank -= 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::Left => {
                    if file != Files::A {
                        bb_square >>= 1;
                        bb_ray |= bb_square;
                        file -= 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::UpLeft => {
                    if (rank != Ranks::R8) && (file != Files::A) {
                        bb_square <<= 7;
                        bb_ray |= bb_square;
                        rank += 1;
                        file -= 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::UpRight => {
                    if (rank != Ranks::R8) && (file != Files::H) {
                        bb_square <<= 9;
                        bb_ray |= bb_square;
                        rank += 1;
                        file += 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::DownRight => {
                    if (rank != Ranks::R1) && (file != Files::H) {
                        bb_square >>= 7;
                        bb_ray |= bb_square;
                        rank -= 1;
                        file += 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
                Direction::DownLeft => {
                    if (rank != Ranks::R1) && (file != Files::A) {
                        bb_square >>= 9;
                        bb_ray |= bb_square;
                        rank -= 1;
                        file -= 1;
                        done = (bb_square & bb_in) > 0;
                    }
                }
            };
        }
        bb_ray
    }
}
