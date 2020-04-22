use crate::defs::{Bitboard, Square, FILE_A, FILE_H, RANK_1, RANK_8};
use crate::{board, board::Direction};

// TODO: Update comment
pub fn create_bb_ray(bb_in: Bitboard, square: Square, direction: Direction) -> Bitboard {
    let mut file = board::square_on_file_rank(square).0 as usize;
    let mut rank = board::square_on_file_rank(square).1 as usize;
    let mut bb_square = 1u64 << square;
    let mut bb_ray = 0;
    let mut done = false;
    while !done {
        done = true;
        match direction {
            Direction::Up => {
                if rank != RANK_8 {
                    bb_square <<= 8;
                    bb_ray |= bb_square;
                    rank += 1;
                    done = (bb_square & bb_in) > 0;
                }
            }
            Direction::Right => {
                if file != FILE_H {
                    bb_square <<= 1;
                    bb_ray |= bb_square;
                    file += 1;
                    done = (bb_square & bb_in) > 0;
                }
            }
            Direction::Down => {
                if rank != RANK_1 {
                    bb_square >>= 8;
                    bb_ray |= bb_square;
                    rank -= 1;
                    done = (bb_square & bb_in) > 0;
                }
            }
            Direction::Left => {
                if file != FILE_A {
                    bb_square >>= 1;
                    bb_ray |= bb_square;
                    file -= 1;
                    done = (bb_square & bb_in) > 0;
                }
            }
            Direction::UpLeft => {
                if (rank != RANK_8) && (file != FILE_A) {
                    bb_square <<= 7;
                    bb_ray |= bb_square;
                    rank += 1;
                    file -= 1;
                    done = (bb_square & bb_in) > 0;
                }
            }
            Direction::UpRight => {
                if (rank != RANK_8) && (file != FILE_H) {
                    bb_square <<= 9;
                    bb_ray |= bb_square;
                    rank += 1;
                    file += 1;
                    done = (bb_square & bb_in) > 0;
                }
            }
            Direction::DownRight => {
                if (rank != RANK_1) && (file != FILE_H) {
                    bb_square >>= 7;
                    bb_ray |= bb_square;
                    rank -= 1;
                    file += 1;
                    done = (bb_square & bb_in) > 0;
                }
            }
            Direction::DownLeft => {
                if (rank != RANK_1) && (file != FILE_A) {
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
