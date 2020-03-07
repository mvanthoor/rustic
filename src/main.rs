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

    let test_pos: &str = "2r2r2/4k3/n3P1p1/Ppbq1b2/1P6/2B2P1B/4Q3/1R3KR1 w - b6 0 1";
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
