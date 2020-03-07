mod board;
mod defines;
mod fen;
mod movegenerator;
mod print;
mod utils;

use board::Board;
use defines::*;
use movegenerator::gen::{MoveList, MAX_LEGAL_MOVES};
use movegenerator::init::Movements;
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

    movegenerator::gen::generate(&board, WHITE, &movements, &mut moves);
    print::movelist(&moves);
    println!("...");
    movegenerator::gen::generate(&board, BLACK, &movements, &mut moves);
    print::movelist(&moves);

    println!("Done.");
}
