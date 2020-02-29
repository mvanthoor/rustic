mod board;
mod defines;
mod fen;
mod magics;
mod movegen;
mod print;
mod utils;

use board::Board;
use defines::*;
use magics::Magics;
use movegen::*;
use utils::*;

fn main() {
    let mut board: Board = Default::default();
    let mut magics: Magics = Default::default();
    let mut moves: MoveList = Vec::with_capacity(MAX_LEGAL_MOVES as usize);

    let test_pos: &str = "8/ppp5/P2N4/1P6/2P2pP1/8/4p2p/6R1 w - g3 0 1";
    board.initialize(test_pos);
    magics.initialize();

    engine_info();
    print::position(&board, None);

    movegen::generate(&board, WHITE, &magics, &mut moves);
    print::movelist(&moves);
    movegen::generate(&board, BLACK, &magics, &mut moves);
    print::movelist(&moves);

    print::bitboard(0x0101_0101_0101_0101, None);

    println!("Done.");
}
