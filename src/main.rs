mod board;
mod defines;
mod fen;
mod movegen;
mod movements;
mod print;
mod utils;

use board::Board;
use defines::*;
use movegen::*;
use movements::Movements;
use utils::*;

fn main() {
    let mut board: Board = Default::default();
    let mut movements: Movements = Default::default();
    let mut moves: MoveList = Vec::with_capacity(MAX_LEGAL_MOVES as usize);

    let test_pos: &str = "8/ppp5/P2N4/1P6/2P2pP1/8/4p2p/6R1 w - g3 0 1";
    board.initialize(test_pos);
    movements.initialize();

    engine_info();

    print::position(&board, None);

    movegen::generate(&board, WHITE, &movements, &mut moves);
    print::movelist(&moves);
    println!("...");
    movegen::generate(&board, BLACK, &movements, &mut moves);
    print::movelist(&moves);

    println!("Done.");
}
