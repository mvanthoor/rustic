use crate::board::Board;
use crate::defs::*;

fn put_piece_onto_ascii(bitboard: Bitboard, ascii: &mut AsciiBoard, piece: char) {
    for (i, square) in ascii.iter_mut().enumerate() {
        if bitboard >> i & 1 == 1 {
            *square = piece;
        }
    }
}

fn bitboards_to_ascii(board: &Board, ascii: &mut AsciiBoard) {
    let nr_of_bitboards = board.bb_w.len();
    for i in 0..nr_of_bitboards {
        match i {
            BB_K => {
                put_piece_onto_ascii(board.bb_w[BB_K], ascii, CHAR_WK);
                put_piece_onto_ascii(board.bb_b[BB_K], ascii, CHAR_BK);
            }
            BB_Q => {
                put_piece_onto_ascii(board.bb_w[BB_Q], ascii, CHAR_WQ);
                put_piece_onto_ascii(board.bb_b[BB_Q], ascii, CHAR_BQ);
            }
            BB_R => {
                put_piece_onto_ascii(board.bb_w[BB_R], ascii, CHAR_WR);
                put_piece_onto_ascii(board.bb_b[BB_R], ascii, CHAR_BR);
            }
            BB_B => {
                put_piece_onto_ascii(board.bb_w[BB_B], ascii, CHAR_WB);
                put_piece_onto_ascii(board.bb_b[BB_B], ascii, CHAR_BB);
            }
            BB_N => {
                put_piece_onto_ascii(board.bb_w[BB_N], ascii, CHAR_WN);
                put_piece_onto_ascii(board.bb_b[BB_N], ascii, CHAR_BN);
            }
            BB_P => {
                put_piece_onto_ascii(board.bb_w[BB_P], ascii, CHAR_WP);
                put_piece_onto_ascii(board.bb_b[BB_P], ascii, CHAR_BP);
            }
            _ => (),
        }
    }
}

pub fn board(board: &Board) {
    let coordinate_alpha: &str = "ABCDEFGH";
    let mut coordinate_digit = 8;
    let mut ascii_board: AsciiBoard = [ASCII_EMPTY_SQUARE; 64];

    bitboards_to_ascii(board, &mut ascii_board);
    println!();
    for r in (RANK_1..=RANK_8).rev() {
        print!("{}   ", coordinate_digit);
        for f in FILE_A..=FILE_H {
            let square = (r * 8 + f) as usize;
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

pub fn bitboard(bitboard: Bitboard) {
    let coordinate_alpha: &str = "ABCDEFGH";
    let mut coordinate_digit = 8;

    println!();
    println!("Bitboard");
    for r in (RANK_1..=RANK_8).rev() {
        print!("{}   ", coordinate_digit);
        for f in FILE_A..=FILE_H {
            let square = (r * 8 + f) as usize;
            let value = bitboard >> square & 1 == 1;
            if value {
                print!("{} ", '1');
            } else {
                print!("{} ", ASCII_EMPTY_SQUARE);
            }
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
