use crate::defines::*;

// Piece location: (file, rank)
pub type Location = (u8, u8);

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

pub fn create_bb_files() -> [Bitboard; 8] {
    // 0x0101_0101_0101_0101 is bits set for A1, A2...
    let mut bb_files: [Bitboard; 8] = [0; 8];
    for (i, file) in bb_files.iter_mut().enumerate() {
        *file = 0x0101_0101_0101_0101 << i;
    }
    bb_files
}

pub fn create_bb_ranks() -> [Bitboard; 8] {
    // 0xFF is all bits set for Rank 1; entire first byte of u64.
    let mut bb_ranks: [Bitboard; 8] = [0; 8];
    for (i, rank) in bb_ranks.iter_mut().enumerate() {
        *rank = 0xFF << (i * 8);
    }
    bb_ranks
}

pub fn square_on_file_rank(square: u8) -> Location {
    let file = square % 8; // square mod 8
    let rank = square / 8; // square div 8
    (file, rank)
}

pub fn square_on_rank(square: u8, rank: u8) -> bool {
    let start = (rank) * 8;
    let end = start + 7;
    (start..=end).contains(&square)
}

// TODO: Test ray block with piece, for every direction
pub fn create_bb_ray(bb_in: Bitboard, square: u8, direction: Direction) -> Bitboard {
    let mut file: i8 = square_on_file_rank(square).0 as i8;
    let mut rank: i8 = square_on_file_rank(square).1 as i8;
    let mut bb_square = 1u64 << square;
    let mut bitboard = 0;
    let mut done = false;
    while !done {
        done = true;
        match direction {
            Direction::Up => {
                if rank != (RANK_8 as i8) {
                    bb_square <<= 8;
                    bitboard ^= bb_square;
                    rank += 1;
                    done = (rank > RANK_8 as i8) || (bb_square & bb_in) > 0;
                }
            }
            Direction::Right => {
                if file != (FILE_H as i8) {
                    bb_square <<= 1;
                    bitboard ^= bb_square;
                    file += 1;
                    done = (file > FILE_H as i8) || (bb_square & bb_in) > 0;
                }
            }
            Direction::Down => {
                if rank != (RANK_1 as i8) {
                    bb_square >>= 8;
                    bitboard ^= bb_square;
                    rank -= 1;
                    done = (rank < RANK_1 as i8) || (bb_square & bb_in) > 0;
                }
            }
            Direction::Left => {
                if file != (FILE_A as i8) {
                    bb_square >>= 1;
                    bitboard ^= bb_square;
                    file -= 1;
                    done = (file < FILE_A as i8) || (bb_square & bb_in) > 0;
                }
            }
            #[rustfmt::skip]
            Direction::UpLeft => {
                if rank != (RANK_8 as i8) && file != (FILE_A as i8) {
                    bb_square <<= 7;
                    bitboard ^= bb_square;
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
                    bitboard ^= bb_square;
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
                    bitboard ^= bb_square;
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
                    bitboard ^= bb_square;
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
    bitboard
}
