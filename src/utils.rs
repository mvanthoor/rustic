/**
 * Utils.rs contains functions that will be useful in different parts of the program.
 * For exmple, the create_bb_ray function can be used to:
 *      - Generate masks for use in magic bitboards
 *      - Generate attack boards for use in magic bitboards
 *      - Create diagonal masks, to complement create_bb_files() an create_bb_ranks().
 * This file will also contain functions that are needed but don't fit anywhere else,
 * such as engine_info(), which prints information to the screen at startup.
 */
use crate::defines::{Bitboard, AUTHOR, ENGINE, FILE_A, FILE_H, RANK_1, RANK_8, VERSION};

/** Prints information about the engine to the screen. */
pub fn engine_info() {
    println!();
    println!("Engine: {} {}", ENGINE, VERSION);
    println!("Author: {}", AUTHOR);
}

// Piece location: (file, rank)
pub type Location = (u8, u8);

/** Direction is an enum holding all the directions the
 * pieces can move in.
 * Up, Right, Down, Left for the rook.
 * UpLeft, UpRight, DownRight, DownLeft for the bishop.
 * The queen is a combination of both.
 */
#[derive(Copy, Clone)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
    UpLeft,
    UpRight,
    DownRight,
    DownLeft,
}

/**
 * This function creates an array of 8 bitboards containing file masks.
 * The bitboard's least significant bit is on the right, so A1 is the
 * first bit (or bit 0).
 * There are 8 elements in the array, from 0 up to and including 7.
 * 0x0101_0101_0101_0101 is the hexadecimal representation of setting
 * bit 0 (A1), bit 8 (A2), bit 16 (A3), etc... for the entire A-file.
 * Then, for each file, the bits are shifted.
 * 0 shifts: masks the A file
 * 1 shifts: masks the B file
 * And so on up to and including the H-file.
*/
pub fn create_bb_files() -> [Bitboard; 8] {
    let mut bb_files: [Bitboard; 8] = [0; 8];
    for (i, file) in bb_files.iter_mut().enumerate() {
        *file = 0x0101_0101_0101_0101 << i;
    }
    bb_files
}

/**
 * This function works exacty the same as create_bb_files.
 * It set the entire first byte, or the first rank A1-H1, and then
 * it shifts upward by 8 bits, masking each of the ranks.
 */
pub fn create_bb_ranks() -> [Bitboard; 8] {
    let mut bb_ranks: [Bitboard; 8] = [0; 8];
    for (i, rank) in bb_ranks.iter_mut().enumerate() {
        *rank = 0xFF << (i * 8);
    }
    bb_ranks
}

/**
 * This function returns a (file, rank) tuple, containing
 * the file and rank a given square is on.
 */
pub fn square_on_file_rank(square: u8) -> Location {
    let file = square % 8; // square mod 8
    let rank = square / 8; // square div 8
    (file, rank)
}

/**
 * This function returns true if the given square is on the
 * given rank, and false of the square is not on the rank.
 */
pub fn square_on_rank(square: u8, rank: u8) -> bool {
    let start = (rank) * 8;
    let end = start + 7;
    (start..=end).contains(&square)
}

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
