use crate::board::Board;
use crate::defs::*;

fn set_ascii_square(bitboard: Bitboard, ascii: &mut AsciiBoard, character: char) {
    for (i, square) in ascii.iter_mut().enumerate() {
        if bitboard >> i & 1 == 1 {
            *square = character;
        }
    }
}

fn bitboards_to_ascii(board: &Board, ascii: &mut AsciiBoard) {
    let nr_of_bitboards = board.bb_w.len();
    for i in 0..nr_of_bitboards {
        match i {
            BB_K => {
                set_ascii_square(board.bb_w[BB_K], ascii, CHAR_WK);
                set_ascii_square(board.bb_b[BB_K], ascii, CHAR_BK);
            }
            BB_Q => {
                set_ascii_square(board.bb_w[BB_Q], ascii, CHAR_WQ);
                set_ascii_square(board.bb_b[BB_Q], ascii, CHAR_BQ);
            }
            BB_R => {
                set_ascii_square(board.bb_w[BB_R], ascii, CHAR_WR);
                set_ascii_square(board.bb_b[BB_R], ascii, CHAR_BR);
            }
            BB_B => {
                set_ascii_square(board.bb_w[BB_B], ascii, CHAR_WB);
                set_ascii_square(board.bb_b[BB_B], ascii, CHAR_BB);
            }
            BB_N => {
                set_ascii_square(board.bb_w[BB_N], ascii, CHAR_WN);
                set_ascii_square(board.bb_b[BB_N], ascii, CHAR_BN);
            }
            BB_P => {
                set_ascii_square(board.bb_w[BB_P], ascii, CHAR_WP);
                set_ascii_square(board.bb_b[BB_P], ascii, CHAR_BP);
            }
            _ => (),
        }
    }
}

fn to_console(ascii_board: &AsciiBoard) {
    let coordinate_alpha: &str = "ABCDEFGH";
    let mut coordinate_digit = NR_OF_FILES;

    println!();
    for current_rank in (RANK_1..=RANK_8).rev() {
        print!("{}   ", coordinate_digit);
        for current_file in FILE_A..=FILE_H {
            let square = (current_rank * NR_OF_FILES + current_file) as usize;
            print!("{} ", ascii_board[square]);
        }
        println!();
        coordinate_digit -= 1;
    }
    println!();
    print!("    ");
    for c in coordinate_alpha.chars() {
        print!("{} ", c);
    }
    println!();
}

pub fn position(board: &Board) {
    let mut ascii_board: AsciiBoard = [ASCII_EMPTY_SQUARE; 64];
    bitboards_to_ascii(board, &mut ascii_board);
    to_console(&ascii_board);
}

pub fn bitboard(bitboard: Bitboard) {
    const SQUARE_OCCUPIED: char = '1';
    let mut ascii_board: AsciiBoard = [ASCII_EMPTY_SQUARE; 64];
    set_ascii_square(bitboard, &mut ascii_board, SQUARE_OCCUPIED);
    to_console(&ascii_board);
}
