use crate::definitions::{Bitboard, FILE_A, FILE_H, RANK_1, RANK_8};
use crate::utils::{square_on_file_rank, Direction};

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
 * The ray ends, when bb_square either passes over the edge of the board, or hits
 * a set bit (piece) in bb_in. A piece/square in bb_in will be *included* in the ray,
 * because the ray can 'see' it.
 */
pub fn create_bb_ray(bb_in: Bitboard, square: u8, direction: Direction) -> Bitboard {
    let mut file = square_on_file_rank(square).0 as i8;
    let mut rank = square_on_file_rank(square).1 as i8;
    let mut bb_square = 1u64 << square;
    let mut bb_ray = 0;
    let mut done = false;
    while !done {
        done = true;
        match direction {
            Direction::Up => {
                if rank != (RANK_8 as i8) {
                    bb_square <<= 8;
                    bb_ray |= bb_square;
                    rank += 1;
                    done = (rank > RANK_8 as i8) || (bb_square & bb_in) > 0;
                }
            }
            Direction::Right => {
                if file != (FILE_H as i8) {
                    bb_square <<= 1;
                    bb_ray |= bb_square;
                    file += 1;
                    done = (file > FILE_H as i8) || (bb_square & bb_in) > 0;
                }
            }
            Direction::Down => {
                if rank != (RANK_1 as i8) {
                    bb_square >>= 8;
                    bb_ray |= bb_square;
                    rank -= 1;
                    done = (rank < RANK_1 as i8) || (bb_square & bb_in) > 0;
                }
            }
            Direction::Left => {
                if file != (FILE_A as i8) {
                    bb_square >>= 1;
                    bb_ray |= bb_square;
                    file -= 1;
                    done = (file < FILE_A as i8) || (bb_square & bb_in) > 0;
                }
            }
            #[rustfmt::skip]
            Direction::UpLeft => {
                if rank != (RANK_8 as i8) && file != (FILE_A as i8) {
                    bb_square <<= 7;
                    bb_ray |= bb_square;
                    rank += 1;
                    file -= 1;
                    done = (rank > RANK_8 as i8)
                        || (file < FILE_A as i8)
                        || (bb_square & bb_in) > 0;
                }
            },
            #[rustfmt::skip]
            Direction::UpRight => {
                if rank != (RANK_8 as i8) && file != (FILE_H as i8) {
                    bb_square <<= 9;
                    bb_ray |= bb_square;
                    rank += 1;
                    file += 1;
                    done = (rank > RANK_8 as i8)
                        || (file > FILE_H as i8)
                        || (bb_square & bb_in) > 0;
                }
            },
            #[rustfmt::skip]
            Direction::DownRight => {
                if rank != (RANK_1 as i8) && file != (FILE_H as i8) {
                    bb_square >>= 7;
                    bb_ray |= bb_square;
                    rank -= 1;
                    file += 1;
                    done = (rank < RANK_1 as i8)
                        || (file > FILE_H as i8)
                        || (bb_square & bb_in) > 0;
                }
            },
            #[rustfmt::skip]
            Direction::DownLeft => {
                if rank != (RANK_1 as i8) && file != (FILE_A as i8) {
                    bb_square >>= 9;
                    bb_ray |= bb_square;
                    rank -= 1;
                    file -= 1;
                    done =
                        (rank < RANK_1 as i8)
                        || (file < FILE_A as i8)
                        || (bb_square & bb_in) > 0;
                }
            },
        };
    }
    bb_ray
}
