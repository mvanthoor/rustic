mod board;
mod defs;
mod fen;
mod masks;
mod print;

use board::Board;
use defs::*;

fn main() {
    let mut board: Board = Default::default();

    println!();
    println!("{} {}, by {}", ENGINE, VERSION, AUTHOR);

    board.create_start_position();
    print::position(&board);
    print::bitboard(board.bb_w[BB_P]);
    print::bitboard(board.bb_b[BB_P]);
    print::bitboard(board.bb_pieces[BB_PIECES_PAWNS]);

    for i in 0..8 {
        print::bitboard(board.bb_files[i]);
    }
}
