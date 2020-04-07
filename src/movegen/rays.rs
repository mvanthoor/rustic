use crate::defs::{Bitboard, FILE_A, FILE_H, RANK_1, RANK_8};
use crate::{board, board::Direction};

/**
 * create_bb_ray() is a long function, but it's actually quite simple.
 * It creates rays from the given square in any direction.
 *
 * It takes the following parameters:
 * bb_in: Incoming bitboard
 * square: Start ray from this square (but don't include the square)
 * direction: the direction of the ray.
 *
 * Basically, the function starts on the given square and then keeps looping,
 * only executing one of the eight match blocks; the one matching the given
 * direction. Thus, in one loop, it will only use one match block.
 * If starting on square 28 (E4) in direction UP, it will iterate over the
 * Direction::Up block until done.
 *
 * The ray ends, either when the edge of the board is reached, or it hits a piece.
 * (bb_square & bb_in > 0). A piece/square in bb_in will be *included* in the ray,
 * because the ray can 'see' it.
 */
pub fn create_bb_ray(bb_in: Bitboard, square: u8, direction: Direction) -> Bitboard {
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
            #[rustfmt::skip]
            Direction::UpLeft => {
                if (rank != RANK_8) && (file != FILE_A) {
                    bb_square <<= 7;
                    bb_ray |= bb_square;
                    rank += 1;
                    file -= 1;
                    done = (bb_square & bb_in) > 0;
                }
            },
            #[rustfmt::skip]
            Direction::UpRight => {
                if (rank != RANK_8) && (file != FILE_H) {
                    bb_square <<= 9;
                    bb_ray |= bb_square;
                    rank += 1;
                    file += 1;
                    done = (bb_square & bb_in) > 0;
                }
            },
            #[rustfmt::skip]
            Direction::DownRight => {
                if (rank != RANK_1) && (file != FILE_H) {
                    bb_square >>= 7;
                    bb_ray |= bb_square;
                    rank -= 1;
                    file += 1;
                    done = (bb_square & bb_in) > 0;
                }
            },
            #[rustfmt::skip]
            Direction::DownLeft => {
                if (rank != RANK_1) && (file != FILE_A) {
                    bb_square >>= 9;
                    bb_ray |= bb_square;
                    rank -= 1;
                    file -= 1;
                    done = (bb_square & bb_in) > 0;
                }
            },
        };
    }
    bb_ray
}
