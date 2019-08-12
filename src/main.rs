mod board;
mod defs;
mod fen;
mod print;

use board::Board;
use defs::*;

fn main() {
    let mut board: Board = Default::default();

    println!();
    println!("{} {}, by {}", ENGINE, VERSION, AUTHOR);

    fen::read(FEN_START_POSITION, &mut board);
    print::board(&board);
    print::bitboard(board.bb_w[BB_R]);
}
