use crate::board::{self, Direction, Files, Ranks};
use crate::defs::{Bitboard, Square};

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
