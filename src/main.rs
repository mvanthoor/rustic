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

    for i in 0..4 {
        print::bitboard(board.bb_pieces[i]);
    }
}
