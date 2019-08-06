mod board;
mod defs;
mod fen;

use board::Board;
use defs::*;
use fen::*;

fn main() {
    println!();
    println!("{} {}, by {}", ENGINE, VERSION, AUTHOR);
    let mut board: Board = Default::default();

    fen_read(FEN_START_POSITION, &mut board);
    board.print();
}
