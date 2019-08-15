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

    fen::read(FEN_START_POSITION, &mut board);
    print::position(&board);
    masks::create(&mut board);

    for i in 0..64 {
        print::bitboard(board.bb_mask[MASK_B][i]);
    }
}
