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
    print::bitboard(board.bb_special[BB_SPECIAL_W]);
    print::bitboard(board.bb_special[BB_SPECIAL_B]);
    print::bitboard(board.bb_special[BB_SPECIAL_ALL]);
}
